package com.crazyms.notifications.inbox.domain;

import com.crazyms.notifications.notification.domain.TicketEvent;

/** One stream entry: its id, how often it was handed out, the raw envelope and what it says. */
public record InboxMessage(String id, long deliveries, String envelope, TicketEvent event) {}
