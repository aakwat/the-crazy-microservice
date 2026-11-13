package com.crazyms.notifications.mail.config;

import jakarta.validation.constraints.Email;
import jakarta.validation.constraints.NotBlank;
import org.springframework.boot.context.properties.ConfigurationProperties;
import org.springframework.validation.annotation.Validated;

/** The sender address; the server itself is spring.mail. */
@Validated
@ConfigurationProperties("notifications.mail")
public record MailProperties(@NotBlank @Email String from) {}
