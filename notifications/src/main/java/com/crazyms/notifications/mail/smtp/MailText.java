package com.crazyms.notifications.mail.smtp;

import com.crazyms.notifications.notification.domain.Notification;

/** Subject and body per reason; ids and status only, since events never carry ticket text. */
record MailText(String subject, String body) {

  static MailText of(Notification n) {
    var ticket = n.ticketId();
    var status = n.status().replace('_', ' ');
    return switch (n.reason()) {
      case TICKET_OPENED ->
          new MailText("We received your ticket", "Your ticket " + ticket + " is open.");
      case TICKET_CHANGED ->
          new MailText(
              "Your ticket is now " + status, "Your ticket " + ticket + " is now " + status + ".");
      case ASSIGNED_TICKET_CHANGED ->
          new MailText(
              "A ticket assigned to you is now " + status,
              "Ticket " + ticket + ", assigned to you, is now " + status + ".");
    };
  }
}
