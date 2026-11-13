package com.crazyms.notifications.notification.domain.port;

import com.crazyms.notifications.notification.domain.Notification;

/** Port: hands one notification to the mail server. */
public interface Mailer {

  void send(Notification notification, String to);
}
