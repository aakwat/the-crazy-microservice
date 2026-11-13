package com.crazyms.notifications.notification.domain;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.tuple;

import java.util.UUID;
import org.junit.jupiter.api.Test;

class TicketEventTest {

  private final UUID requester = UUID.randomUUID();
  private final UUID assignee = UUID.randomUUID();

  @Test
  void aNewTicketTellsOnlyItsRequester() {
    var notifications = event(TicketEventType.CREATED, null).notifications();

    assertThat(notifications)
        .extracting(Notification::recipientId, Notification::reason)
        .containsExactly(tuple(requester, NotificationReason.TICKET_OPENED));
  }

  @Test
  void aChangeTellsTheRequesterAndTheAssignee() {
    var notifications = event(TicketEventType.CHANGED, assignee).notifications();

    assertThat(notifications)
        .extracting(Notification::recipientId, Notification::reason)
        .containsExactly(
            tuple(requester, NotificationReason.TICKET_CHANGED),
            tuple(assignee, NotificationReason.ASSIGNED_TICKET_CHANGED));
  }

  @Test
  void aRequesterWhoIsAlsoTheAssigneeGetsOneEmail() {
    assertThat(event(TicketEventType.CHANGED, requester).notifications()).hasSize(1);
    assertThat(event(TicketEventType.CHANGED, null).notifications()).hasSize(1);
  }

  private TicketEvent event(TicketEventType type, UUID assigneeId) {
    return new TicketEvent(
        UUID.randomUUID(), type, UUID.randomUUID(), "T-1", "in_progress", requester, assigneeId);
  }
}
