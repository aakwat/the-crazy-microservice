//! Runs against a real Redis 8 and checks entries against the event contract.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeSet;
use std::time::Duration;

use domain::{EventPublisher, OutboxEvent};
use redis::AsyncCommands;
use redis::streams::StreamRangeReply;
use serde_json::Value;
use streams::{BusAddress, ENVELOPE_FIELD, RedisEventPublisher};
use testcontainers::core::IntoContainerPort;
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, ImageExt};
use testcontainers_modules::redis::{REDIS_PORT, Redis};
use time::macros::datetime;
use uuid::Uuid;

const SCHEMA: &str = include_str!("../../../../contracts/events/tickets.events.schema.json");
const TIMEOUT: Duration = Duration::from_secs(2);
const PASSWORD: &str = "test-password";

async fn start_redis(host_port: Option<u16>) -> (ContainerAsync<Redis>, u16) {
  let image = Redis::default()
    .with_tag("8.10.3-alpine")
    .with_cmd(["redis-server", "--requirepass", PASSWORD]);
  let container = match host_port {
    Some(port) => image.with_mapped_port(port, REDIS_PORT.tcp()).start().await,
    None => image.start().await,
  }
  .expect("start redis");
  let port = container.get_host_port_ipv4(REDIS_PORT).await.expect("redis port");
  (container, port)
}

fn address(port: u16) -> BusAddress {
  BusAddress {
    host: "127.0.0.1".to_owned(),
    port,
    user: "default".to_owned(),
    password: PASSWORD.to_owned(),
  }
}

fn event(status: &str) -> OutboxEvent {
  let ticket = Uuid::now_v7();
  OutboxEvent {
    id: Uuid::now_v7(),
    stream: "tickets.events".to_owned(),
    event_type: "ticket.changed".to_owned(),
    version: 1,
    subject_id: ticket,
    traceparent: Some("00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01".to_owned()),
    payload: format!(
      r#"{{"ticket_id":"{ticket}","reference":"T-1","status":"{status}","requester_id":"{}","assignee_id":null}}"#,
      Uuid::now_v7()
    ),
    occurred_at: datetime!(2026-11-12 09:00:00.123456 UTC),
  }
}

async fn envelopes(port: u16) -> Vec<Value> {
  let mut redis = redis::Client::open(format!("redis://default:{PASSWORD}@127.0.0.1:{port}"))
    .unwrap()
    .get_multiplexed_async_connection()
    .await
    .unwrap();
  let reply: StreamRangeReply = redis.xrange_all("tickets.events").await.unwrap();
  reply
    .ids
    .iter()
    .map(|entry| serde_json::from_str(&entry.get::<String>(ENVELOPE_FIELD).unwrap()).unwrap())
    .collect()
}

fn keys(object: &Value) -> BTreeSet<String> {
  object.as_object().unwrap().keys().cloned().collect()
}

#[tokio::test]
async fn events_land_in_order_in_the_contract_envelope() {
  let (_redis, port) = start_redis(None).await;
  let publisher = RedisEventPublisher::new(address(port), TIMEOUT).unwrap();
  let (first, second) = (event("open"), event("closed"));

  publisher.publish(&first).await.unwrap();
  publisher.publish(&second).await.unwrap();

  let envelopes = envelopes(port).await;
  let schema: Value = serde_json::from_str(SCHEMA).unwrap();
  assert_eq!(envelopes.len(), 2);
  assert_eq!(keys(&envelopes[0]), keys(&schema["properties"]));
  assert_eq!(
    keys(&envelopes[0]["payload"]),
    keys(&schema["properties"]["payload"]["properties"])
  );
  assert_eq!(envelopes[0]["id"], first.id.to_string());
  assert_eq!(envelopes[0]["occurred_at"], "2026-11-12T09:00:00.123456Z");
  assert_eq!(envelopes[0]["payload"]["status"], "open");
  assert_eq!(envelopes[1]["id"], second.id.to_string());
}

#[tokio::test]
async fn a_bus_that_comes_up_late_takes_the_next_publish() {
  let port = std::net::TcpListener::bind("127.0.0.1:0")
    .unwrap()
    .local_addr()
    .unwrap()
    .port();
  let publisher = RedisEventPublisher::new(address(port), TIMEOUT).expect("no connection needed to start");

  assert!(publisher.publish(&event("open")).await.is_err());

  let (_redis, _) = start_redis(Some(port)).await;
  let mut delivered = false;
  for _ in 0..5 {
    if publisher.publish(&event("open")).await.is_ok() {
      delivered = true;
      break;
    }
    tokio::time::sleep(Duration::from_millis(500)).await;
  }
  assert!(delivered, "publisher never reconnected");
  assert_eq!(envelopes(port).await.len(), 1);
}
