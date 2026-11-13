package com.crazyms.notifications.inbox.config;

import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotNull;
import jakarta.validation.constraints.Positive;
import java.time.Duration;
import org.springframework.boot.context.properties.ConfigurationProperties;
import org.springframework.validation.annotation.Validated;

/** The event bus login, and how long a handed-out message may sit before another try. */
@Validated
@ConfigurationProperties("notifications.inbox")
public record InboxProperties(
    @NotBlank String host,
    @Positive int port,
    @NotBlank String user,
    @NotBlank String password,
    @NotNull Duration stuckAfter) {}
