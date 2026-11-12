//! Drives the real router against real Postgres 18 and a mock auth serving the test JWKS.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use api::config::AUDIENCE;
use api::interfaces::http::router;
use api::jobs::heartbeat::Heartbeat;
use api::state::AppState;
use application::TicketService;
use axum::Router;
use axum::body::Body;
use axum::http::{HeaderMap, Request, StatusCode, header};
use http_body_util::BodyExt;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use jwks::JwksVerifier;
use postgres::PostgresTicketRepository;
use serde_json::{Value, json};
use sqlx::PgPool;
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, ImageExt};
use testcontainers_modules::postgres::Postgres;
use time::OffsetDateTime;
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::matchers::path;
use wiremock::{Mock, MockServer, ResponseTemplate};

const ISSUER: &str = "http://127.0.0.1:8080";
const TRACE: &str = "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01";

struct Response {
  status: StatusCode,
  headers: HeaderMap,
  body: Value,
}

struct TestApi {
  _db: ContainerAsync<Postgres>,
  _auth: MockServer,
  app: Router,
  pool: PgPool,
  heartbeat: Arc<Heartbeat>,
}

impl TestApi {
  async fn start() -> Self {
    let db = Postgres::default()
      .with_db_name("tickets")
      .with_tag("18-alpine")
      .with_copy_to(
        "/docker-entrypoint-initdb.d/01-roles.sql",
        include_bytes!("../../db/init/01-roles.sql").to_vec(),
      )
      .with_copy_to("/run/secrets/TICKETS_DB_MIGRATOR_PASSWORD", b"migrator-test".to_vec())
      .with_copy_to("/run/secrets/TICKETS_DB_APP_PASSWORD", b"app-test".to_vec())
      .start()
      .await
      .expect("start postgres");
    let port = db.get_host_port_ipv4(5432).await.expect("postgres port");
    let url = |user: &str, password: &str| format!("postgres://{user}:{password}@127.0.0.1:{port}/tickets");
    let migrator = PgPool::connect(&url("tickets_migrator", "migrator-test"))
      .await
      .expect("migrator");
    postgres::migrate(&migrator).await.expect("migrate");
    let pool = PgPool::connect(&url("tickets_app", "app-test")).await.expect("app");

    let auth = MockServer::start().await;
    let jwks = include_str!("../../adapters/jwks/tests/keys/jwks-a.json");
    Mock::given(path("/oauth2/jwks"))
      .respond_with(ResponseTemplate::new(200).set_body_raw(jwks, "application/json"))
      .mount(&auth)
      .await;
    let jwks_url = format!("{}/oauth2/jwks", auth.uri());
    let ten_minutes = Duration::from_secs(600);
    let verifier = JwksVerifier::new(
      reqwest::Client::new(),
      jwks_url,
      ISSUER,
      AUDIENCE,
      ten_minutes,
      ten_minutes,
    );
    let heartbeat = Arc::new(Heartbeat::new(OffsetDateTime::now_utc()));
    let state = AppState {
      tickets: Arc::new(TicketService::new(PostgresTicketRepository::new(pool.clone()))),
      verifier: Arc::new(verifier),
      heartbeat: Arc::clone(&heartbeat),
    };
    Self {
      _db: db,
      _auth: auth,
      app: router(state),
      pool,
      heartbeat,
    }
  }

  async fn send(&self, method: &str, uri: &str, caller: Option<&Caller>, body: Option<Value>) -> Response {
    let mut request = Request::builder().method(method).uri(uri).header("traceparent", TRACE);
    if let Some(caller) = caller {
      request = request.header(header::AUTHORIZATION, format!("Bearer {}", caller.token));
    }
    let request = match body {
      Some(body) => request
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string())),
      None => request.body(Body::empty()),
    };
    let response = self.app.clone().oneshot(request.unwrap()).await.unwrap();
    let (status, headers) = (response.status(), response.headers().clone());
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    Response {
      status,
      headers,
      body: serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    }
  }
}

/// A logged-in user with a real ES256 access token, as auth would issue it.
struct Caller {
  id: Uuid,
  token: String,
}

impl Caller {
  fn new(role: &str) -> Self {
    let id = Uuid::now_v7();
    let exp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 600;
    let claims = json!({
      "sub": id.to_string(), "iss": ISSUER, "aud": [AUDIENCE], "exp": exp, "client_id": "gateway", "roles": [role],
    });
    let mut header = Header::new(Algorithm::ES256);
    header.kid = Some("a".to_owned());
    header.typ = Some("at+jwt".to_owned());
    let key = EncodingKey::from_ec_pem(include_bytes!("../../adapters/jwks/tests/keys/a.pem")).unwrap();
    Self {
      id,
      token: encode(&header, &claims, &key).unwrap(),
    }
  }
}

fn printer_on_fire() -> Value {
  json!({"title": "Printer on fire", "body": "Please help."})
}

fn is_problem(response: &Response) -> bool {
  response
    .headers
    .get(header::CONTENT_TYPE)
    .is_some_and(|value| value == "application/problem+json")
}

#[tokio::test]
async fn a_customer_opens_lists_and_reads_their_ticket() {
  let api = TestApi::start().await;
  let alice = Caller::new("customer");

  let created = api
    .send("POST", "/api/tickets", Some(&alice), Some(printer_on_fire()))
    .await;
  let id = created.body["id"].as_str().unwrap().to_owned();
  let list = api.send("GET", "/api/tickets", Some(&alice), None).await;
  let read = api.send("GET", &format!("/api/tickets/{id}"), Some(&alice), None).await;

  assert_eq!(created.status, StatusCode::CREATED);
  assert_eq!(created.headers[header::LOCATION], format!("/api/tickets/{id}"));
  assert_eq!(created.body["status"], "open");
  assert_eq!(created.body["requester_id"], alice.id.to_string());
  assert_eq!(list.body["items"].as_array().unwrap().len(), 1);
  assert_eq!(list.body["next_before"], Value::Null);
  assert!(
    list.body["items"][0].get("body").is_none(),
    "lists carry no ticket body"
  );
  assert_eq!(read.body["title"], "Printer on fire");
}

#[tokio::test]
async fn others_get_404_and_only_agents_change_tickets() {
  let api = TestApi::start().await;
  let (alice, bob, agent) = (Caller::new("customer"), Caller::new("customer"), Caller::new("agent"));
  let created = api
    .send("POST", "/api/tickets", Some(&alice), Some(printer_on_fire()))
    .await;
  let uri = format!("/api/tickets/{}", created.body["id"].as_str().unwrap());

  let by_bob = api.send("GET", &uri, Some(&bob), None).await;
  let by_alice = api
    .send("PATCH", &uri, Some(&alice), Some(json!({"status": "in_progress"})))
    .await;
  let assigned = api
    .send(
      "PATCH",
      &uri,
      Some(&agent),
      Some(json!({"status": "in_progress", "assignee_id": agent.id})),
    )
    .await;
  let skipped = api
    .send("PATCH", &uri, Some(&agent), Some(json!({"status": "closed"})))
    .await;
  let unassigned = api
    .send("PATCH", &uri, Some(&agent), Some(json!({"assignee_id": null})))
    .await;

  assert_eq!(by_bob.status, StatusCode::NOT_FOUND);
  assert!(is_problem(&by_bob));
  assert_eq!(by_alice.status, StatusCode::FORBIDDEN);
  assert_eq!(assigned.status, StatusCode::OK);
  assert_eq!(assigned.body["assignee_id"], agent.id.to_string());
  assert_eq!(skipped.status, StatusCode::UNPROCESSABLE_ENTITY);
  assert_eq!(unassigned.body["assignee_id"], Value::Null);
  assert_eq!(unassigned.body["status"], "in_progress");
}

#[tokio::test]
async fn bad_requests_get_rfc_9457_problems() {
  let api = TestApi::start().await;
  let alice = Caller::new("customer");
  let mass_assignment = json!({"title": "t", "body": "b", "requester_id": Uuid::now_v7()});
  let oversized = json!({"title": "t", "body": "x".repeat(70 * 1024)});

  let cases = [
    (
      "unknown field",
      api
        .send("POST", "/api/tickets", Some(&alice), Some(mass_assignment))
        .await,
      StatusCode::BAD_REQUEST,
    ),
    (
      "blank title",
      api
        .send(
          "POST",
          "/api/tickets",
          Some(&alice),
          Some(json!({"title": " ", "body": "b"})),
        )
        .await,
      StatusCode::UNPROCESSABLE_ENTITY,
    ),
    (
      "oversized body",
      api.send("POST", "/api/tickets", Some(&alice), Some(oversized)).await,
      StatusCode::PAYLOAD_TOO_LARGE,
    ),
    (
      "malformed id",
      api.send("GET", "/api/tickets/not-a-uuid", Some(&alice), None).await,
      StatusCode::NOT_FOUND,
    ),
    (
      "unknown path",
      api.send("GET", "/api/nope", Some(&alice), None).await,
      StatusCode::NOT_FOUND,
    ),
    (
      "wrong method",
      api.send("DELETE", "/api/tickets", Some(&alice), None).await,
      StatusCode::METHOD_NOT_ALLOWED,
    ),
  ];

  for (name, response, status) in cases {
    assert_eq!(response.status, status, "{name}");
    assert!(is_problem(&response), "{name}: not a problem+json response");
  }
}

#[tokio::test]
async fn callers_without_a_valid_token_get_the_same_generic_401() {
  let api = TestApi::start().await;
  let forged = Caller {
    id: Uuid::now_v7(),
    token: "not.a.jwt".to_owned(),
  };

  for (name, caller) in [("no token", None), ("garbage token", Some(&forged))] {
    let response = api.send("GET", "/api/tickets", caller, None).await;
    assert_eq!(response.status, StatusCode::UNAUTHORIZED, "{name}");
    assert_eq!(response.headers[header::WWW_AUTHENTICATE], "Bearer", "{name}");
    assert_eq!(
      response.body["detail"], "log in or send a valid access token",
      "{name}: the reason must not leak"
    );
  }
}

#[tokio::test]
async fn writes_carry_the_gateways_traceparent_into_the_outbox() {
  let api = TestApi::start().await;

  api
    .send(
      "POST",
      "/api/tickets",
      Some(&Caller::new("customer")),
      Some(printer_on_fire()),
    )
    .await;

  let traceparent: Option<String> = sqlx::query_scalar("SELECT traceparent FROM outbox")
    .fetch_one(&api.pool)
    .await
    .unwrap();
  assert_eq!(traceparent.as_deref(), Some(TRACE));
}

#[tokio::test]
async fn health_turns_red_when_the_outbox_relay_falls_silent() {
  let api = TestApi::start().await;
  let now = OffsetDateTime::now_utc();

  let alive = api.send("GET", "/healthz", None, None).await;
  api.heartbeat.beat(now - time::Duration::minutes(2));
  let silent = api.send("GET", "/healthz", None, None).await;

  assert_eq!(alive.status, StatusCode::NO_CONTENT);
  assert_eq!(silent.status, StatusCode::SERVICE_UNAVAILABLE);
}
