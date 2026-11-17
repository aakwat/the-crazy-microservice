package com.crazyms.notifications.inbox.domain;

import com.crazyms.notifications.notification.domain.TicketEvent;

/** One stream entry: its id, deliveries, the raw envelope, what it says and its nullable trace. */
public record InboxMessage(
    String id, long deliveries, String envelope, TicketEvent event, String traceparent) {}
