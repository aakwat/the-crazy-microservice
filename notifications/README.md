# notifications

Emails the requester and the assignee when a ticket changes, in Java (Spring Boot), with its own Postgres.

## Job

- Reads `tickets.events` as the consumer group `notifications`; delivery is at least once.
- Sends each email once: `sent_notification (event_id, recipient_id)` dedupes redeliveries.
- Gets addresses from auth's `GET /internal/accounts/{id}` per send; never stores or logs one.
- A message that fails 5 times goes to `tickets.events.dlq`; an unreadable one goes there at once.
- `/actuator/health/liveness` turns DOWN when the inbox worker stops finishing rounds.

## Layout

| Package        | Holds                                                         |
| -------------- | ------------------------------------------------------------- |
| `notification` | the recipients rule, the use case, the sent log and its sweep |
| `inbox`        | the Redis Streams consumer, its policy, worker and liveness   |
| `account`      | the auth lookup (client credentials, timeouts)                |
| `mail`         | the SMTP sender and the message text                          |

## Run alone

```fish
./dev-secrets.sh
docker compose up -d --build --wait
```

Needs auth (`NOTIFICATIONS_AUTH_URL`, client `notifications`) and the event bus. Emails land in Mailpit:
http://127.0.0.1:8025.

## Test

```fish
mvn clean verify
```

Starts real Postgres, Redis and Mailpit with Testcontainers, so it needs a container socket.
