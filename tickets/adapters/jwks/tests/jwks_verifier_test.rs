//! Signs real ES256 tokens with fixed test keys and serves their JWKS from a mock auth.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use domain::{AccessTokenVerifier, TokenError};
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use jwks::JwksVerifier;
use serde_json::{Value, json};
use uuid::Uuid;
use wiremock::matchers::path;
use wiremock::{Mock, MockServer, ResponseTemplate};

const ISSUER: &str = "http://127.0.0.1:8080";
const AUDIENCE: &str = "crazy-ms";
const JWKS_A: &str = include_str!("keys/jwks-a.json");
const JWKS_AB: &str = include_str!("keys/jwks-ab.json");
const LONG: Duration = Duration::from_secs(600);

/// Turns a valid token into one the verifier must refuse.
type Mutation = fn(&mut Token);

struct Token {
  kid: &'static str,
  signing_key: &'static str,
  typ: &'static str,
  claims: Value,
}

impl Token {
  fn access(kid: &'static str) -> Self {
    let exp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 600;
    let claims = json!({
      "sub": Uuid::now_v7().to_string(), "iss": ISSUER, "aud": [AUDIENCE], "exp": exp,
      "client_id": "gateway", "roles": ["agent"],
    });
    Self {
      kid,
      signing_key: kid,
      typ: "at+jwt",
      claims,
    }
  }

  fn sign(&self) -> String {
    let pem = match self.signing_key {
      "a" => include_bytes!("keys/a.pem").as_slice(),
      _ => include_bytes!("keys/b.pem").as_slice(),
    };
    let mut header = Header::new(Algorithm::ES256);
    header.kid = Some(self.kid.to_owned());
    header.typ = Some(self.typ.to_owned());
    encode(&header, &self.claims, &EncodingKey::from_ec_pem(pem).unwrap()).unwrap()
  }
}

/// A mock auth answering `/oauth2/jwks` with each response in turn; the last one repeats.
async fn auth(responses: &[(u16, &str)]) -> MockServer {
  let server = MockServer::start().await;
  for (i, (status, body)) in responses.iter().enumerate() {
    let mock = Mock::given(path("/oauth2/jwks"))
      .respond_with(ResponseTemplate::new(*status).set_body_raw((*body).to_owned(), "application/json"));
    let mock = if i + 1 < responses.len() {
      mock.up_to_n_times(1)
    } else {
      mock
    };
    mock.mount(&server).await;
  }
  server
}

fn verifier(auth: &MockServer, max_key_age: Duration, min_refetch_interval: Duration) -> JwksVerifier {
  let url = format!("{}/oauth2/jwks", auth.uri());
  JwksVerifier::new(
    reqwest::Client::new(),
    url,
    ISSUER,
    AUDIENCE,
    max_key_age,
    min_refetch_interval,
  )
}

async fn fetches(auth: &MockServer) -> usize {
  auth.received_requests().await.map_or(0, |requests| requests.len())
}

#[tokio::test]
async fn a_valid_access_token_gives_its_subject_and_roles() {
  let auth = auth(&[(200, JWKS_A)]).await;
  let token = Token::access("a");

  let principal = verifier(&auth, LONG, LONG).verify(&token.sign()).await.unwrap();

  assert_eq!(principal.subject().to_string(), token.claims["sub"]);
  assert!(principal.is_agent());
}

#[tokio::test]
async fn both_rfc_9068_typ_spellings_are_accepted() {
  let auth = auth(&[(200, JWKS_A)]).await;
  let mut token = Token::access("a");
  token.typ = "application/at+jwt";

  assert!(verifier(&auth, LONG, LONG).verify(&token.sign()).await.is_ok());
}

#[tokio::test]
async fn tokens_that_are_not_our_access_tokens_are_refused() {
  let auth = auth(&[(200, JWKS_A)]).await;
  let verifier = verifier(&auth, LONG, LONG);
  let mutations: [(&str, Mutation); 7] = [
    ("ID token shape", |t| t.typ = "JWT"),
    ("other audience", |t| t.claims["aud"] = json!(["someone-else"])),
    ("other issuer", |t| t.claims["iss"] = json!("http://evil.test")),
    ("expired", |t| t.claims["exp"] = json!(1)),
    ("non-account subject", |t| t.claims["sub"] = json!("jobs")),
    ("no client_id", |t| {
      _ = t.claims.as_object_mut().unwrap().remove("client_id");
    }),
    ("signed with another key", |t| t.signing_key = "b"),
  ];

  for (name, mutate) in mutations {
    let mut token = Token::access("a");
    mutate(&mut token);
    let result = verifier.verify(&token.sign()).await;
    assert!(matches!(result, Err(TokenError::Invalid(_))), "{name}: got {result:?}");
  }
  let garbage = verifier.verify("not.a.jwt").await;
  assert!(
    matches!(garbage, Err(TokenError::Invalid(_))),
    "garbage: got {garbage:?}"
  );
}

#[tokio::test]
async fn keys_are_cached_while_fresh() {
  let auth = auth(&[(200, JWKS_A)]).await;
  let verifier = verifier(&auth, LONG, LONG);

  verifier.verify(&Token::access("a").sign()).await.unwrap();
  verifier.verify(&Token::access("a").sign()).await.unwrap();

  assert_eq!(fetches(&auth).await, 1);
}

#[tokio::test]
async fn unknown_kids_cannot_flood_auth() {
  let auth = auth(&[(200, JWKS_A)]).await;
  let verifier = verifier(&auth, LONG, LONG);
  verifier.verify(&Token::access("a").sign()).await.unwrap();

  for _ in 0..5 {
    let result = verifier.verify(&Token::access("b").sign()).await;
    assert!(matches!(result, Err(TokenError::Invalid(_))), "got {result:?}");
  }

  assert_eq!(fetches(&auth).await, 1);
}

#[tokio::test]
async fn an_unknown_kid_picks_up_a_rotated_key() {
  let auth = auth(&[(200, JWKS_A), (200, JWKS_AB)]).await;
  let verifier = verifier(&auth, LONG, Duration::ZERO);

  verifier.verify(&Token::access("a").sign()).await.unwrap();
  let rotated = verifier.verify(&Token::access("b").sign()).await;

  assert!(
    rotated.is_ok(),
    "a key added by rotation must be picked up: {rotated:?}"
  );
}

#[tokio::test]
async fn a_stale_key_is_refreshed_and_survives_a_short_auth_outage() {
  let auth = auth(&[(200, JWKS_A), (503, "")]).await;
  let verifier = verifier(&auth, Duration::from_millis(100), Duration::ZERO);
  verifier.verify(&Token::access("a").sign()).await.unwrap();

  tokio::time::sleep(Duration::from_millis(120)).await;
  let during_outage = verifier.verify(&Token::access("a").sign()).await;

  assert_eq!(fetches(&auth).await, 2, "a stale key must trigger a refresh");
  assert!(
    during_outage.is_ok(),
    "a recent key must survive a short outage: {during_outage:?}"
  );
}

#[tokio::test]
async fn auth_down_with_no_usable_key_is_unavailable() {
  for jwks in [(503, ""), (200, r#"{"keys": []}"#)] {
    let auth = auth(&[jwks]).await;

    let result = verifier(&auth, LONG, LONG).verify(&Token::access("a").sign()).await;

    assert!(
      matches!(result, Err(TokenError::Unavailable(_))),
      "{jwks:?}: got {result:?}"
    );
  }
}
