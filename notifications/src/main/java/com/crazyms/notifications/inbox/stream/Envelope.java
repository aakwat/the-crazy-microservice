package com.crazyms.notifications.inbox.stream;

import com.crazyms.notifications.notification.domain.TicketEvent;

/** What one envelope says and its trace; traceparent is null when the producer sent none. */
record Envelope(TicketEvent event, String traceparent) {}
