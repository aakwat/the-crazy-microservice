package com.crazyms.notifications.notification.domain;

import java.util.Arrays;
import java.util.Optional;

public enum TicketEventType {
  CREATED("ticket.created"),
  CHANGED("ticket.changed");

  private final String wireName;

  TicketEventType(String wireName) {
    this.wireName = wireName;
  }

  public static Optional<TicketEventType> fromWire(String name) {
    return Arrays.stream(values()).filter(t -> t.wireName.equals(name)).findFirst();
  }
}
