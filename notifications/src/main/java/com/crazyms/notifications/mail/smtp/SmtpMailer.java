package com.crazyms.notifications.mail.smtp;

import com.crazyms.notifications.mail.config.MailProperties;
import com.crazyms.notifications.notification.domain.Notification;
import com.crazyms.notifications.notification.domain.port.Mailer;
import org.springframework.mail.SimpleMailMessage;
import org.springframework.mail.javamail.JavaMailSender;
import org.springframework.stereotype.Component;

@Component
class SmtpMailer implements Mailer {

  private final JavaMailSender sender;
  private final MailProperties props;

  SmtpMailer(JavaMailSender sender, MailProperties props) {
    this.sender = sender;
    this.props = props;
  }

  @Override
  public void send(Notification notification, String to) {
    var text = MailText.of(notification);
    var message = new SimpleMailMessage();
    message.setFrom(props.from());
    message.setTo(to);
    message.setSubject(text.subject());
    message.setText(text.body());
    sender.send(message);
  }
}
