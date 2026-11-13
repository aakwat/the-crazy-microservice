package com.crazyms.notifications.notification.domain;

import java.util.UUID;

/** One email owed to one account for one event; (eventId, recipientId) is what dedupes it. */
public record Notification(
    UUID eventId, UUID recipientId, String reference, String status, NotificationReason reason) {}
