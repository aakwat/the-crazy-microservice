package com.crazyms.notifications.notification.domain;

import static java.util.Objects.requireNonNull;

import java.util.List;
import java.util.UUID;

/** A ticket event as notifying needs it; assigneeId is null when unassigned. */
public record TicketEvent(
    UUID id,
    TicketEventType type,
    UUID ticketId,
    String reference,
    String status,
    UUID requesterId,
    UUID assigneeId) {

  public TicketEvent {
    requireNonNull(id, "id");
    requireNonNull(type, "type");
    requireNonNull(ticketId, "ticketId");
    requireNonNull(reference, "reference");
    requireNonNull(status, "status");
    requireNonNull(requesterId, "requesterId");
  }

  /** A new ticket tells its requester; a change also tells the assignee if someone else. */
  public List<Notification> notifications() {
    if (type == TicketEventType.CREATED) {
      return List.of(notify(requesterId, NotificationReason.TICKET_OPENED));
    }
    var requester = notify(requesterId, NotificationReason.TICKET_CHANGED);
    if (assigneeId == null || assigneeId.equals(requesterId)) {
      return List.of(requester);
    }
    return List.of(requester, notify(assigneeId, NotificationReason.ASSIGNED_TICKET_CHANGED));
  }

  private Notification notify(UUID recipientId, NotificationReason reason) {
    return new Notification(id, recipientId, reference, status, reason);
  }
}
