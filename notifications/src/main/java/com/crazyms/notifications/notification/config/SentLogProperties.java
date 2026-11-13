package com.crazyms.notifications.notification.config;

import jakarta.validation.constraints.NotNull;
import java.time.Duration;
import org.springframework.boot.context.properties.ConfigurationProperties;
import org.springframework.validation.annotation.Validated;

@Validated
@ConfigurationProperties("notifications.sent-log")
public record SentLogProperties(@NotNull Duration retention) {}
