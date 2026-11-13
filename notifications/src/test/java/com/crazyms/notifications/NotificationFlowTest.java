package com.crazyms.notifications;

import static org.assertj.core.api.Assertions.assertThat;
import static org.awaitility.Awaitility.await;

import java.time.Duration;
import java.util.UUID;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.context.annotation.Import;

/** Events in on Redis, emails out through Mailpit, addresses from the auth stub. */
// Auto-configuration conditions read these before the containers' values exist; those then win.
@SpringBootTest(
    properties = {
      "NOTIFICATIONS_SMTP_HOST=localhost",
      "NOTIFICATIONS_AUTH_CLIENT_SECRET=unused",
      "NOTIFICATIONS_AUTH_TOKEN_URL=http://localhost/unused"
    })
@Import(TestcontainersConfiguration.class)
class NotificationFlowTest {

  private static final Duration WAIT = Duration.ofSeconds(30);

  @Autowired TestBus bus;
  @Autowired TestMailbox mailbox;
  @Autowired AuthStub auth;

  @BeforeEach
  void busIsReady() {
    bus.awaitConsumerGroup();
  }

  @Test
  void aNewTicketEmailsItsRequesterOnceEvenWhenTheEventArrivesTwice() throws Exception {
    var requester = account();
    var envelope = created(UUID.randomUUID(), requester.id());

    bus.publish(envelope);
    bus.publish(envelope);
    // Published after the duplicate, so once its email is in, the duplicate has been handled.
    var marker = account();
    bus.publish(created(UUID.randomUUID(), marker.id()));

    await().atMost(WAIT).until(() -> mailbox.subjectsTo(marker.email()).size() == 1);
    assertThat(mailbox.subjectsTo(requester.email())).containsExactly("We received your ticket");
  }

  @Test
  void aChangeEmailsTheRequesterAndTheAssignee() {
    var requester = account();
    var assignee = account();

    bus.publish(changed(UUID.randomUUID(), requester.id(), assignee.id()));

    await()
        .atMost(WAIT)
        .untilAsserted(() -> {
          assertThat(mailbox.subjectsTo(requester.email()))
              .containsExactly("Your ticket is now in progress");
          assertThat(mailbox.subjectsTo(assignee.email()))
              .containsExactly("A ticket assigned to you is now in progress");
        });
  }

  @Test
  void authBeingDownDelaysTheEmailInsteadOfLosingIt() throws Exception {
    var requester = account();
    auth.failLookupsFor(requester.id());

    bus.publish(created(UUID.randomUUID(), requester.id()));
    Thread.sleep(3_000);
    assertThat(mailbox.subjectsTo(requester.email())).isEmpty();
    auth.recover(requester.id());

    await().atMost(WAIT).until(() -> mailbox.subjectsTo(requester.email()).size() == 1);
  }

  @Test
  void aMessageThatKeepsFailingIsDeadLetteredAfterFiveDeliveries() throws Exception {
    var requester = account();
    auth.failLookupsFor(requester.id());
    var eventId = UUID.randomUUID();

    bus.publish(created(eventId, requester.id()));

    await()
        .atMost(Duration.ofSeconds(60))
        .until(() ->
            bus.deadLetters().stream()
                .anyMatch(dead ->
                    dead.getOrDefault("envelope", "").contains(eventId.toString())
                        && dead.get("reason").equals("too many deliveries")));
    assertThat(mailbox.subjectsTo(requester.email())).isEmpty();
  }

  @Test
  void anUnreadableEnvelopeIsDeadLetteredAtOnce() {
    var garbage = "not json " + UUID.randomUUID();

    bus.publish(garbage);

    await()
        .atMost(WAIT)
        .until(() ->
            bus.deadLetters().stream().anyMatch(dead -> garbage.equals(dead.get("envelope"))));
  }

  private record Account(UUID id, String email) {}

  private Account account() {
    var account = new Account(UUID.randomUUID(), UUID.randomUUID() + "@example.test");
    auth.addAccount(account.id(), account.email());
    return account;
  }

  private static String created(UUID eventId, UUID requester) {
    return envelope(eventId, "ticket.created", "open", requester, null);
  }

  private static String changed(UUID eventId, UUID requester, UUID assignee) {
    return envelope(eventId, "ticket.changed", "in_progress", requester, assignee);
  }

  // The contract shape (contracts/events/tickets.events.schema.json), as the tickets relay sends
  // it.
  private static String envelope(
      UUID eventId, String type, String status, UUID requester, UUID assignee) {
    var ticket = UUID.randomUUID();
    var assigneeJson = assignee == null ? "null" : "\"" + assignee + "\"";
    return """
    {"id":"%s","type":"%s","version":1,"occurred_at":"2026-11-12T09:00:00Z",\
    "subject_id":"%s","traceparent":null,"payload":{"ticket_id":"%s","status":"%s",\
    "requester_id":"%s","assignee_id":%s}}\
    """
        .formatted(eventId, type, ticket, ticket, status, requester, assigneeJson);
  }
}
