package com.crazyms.notifications.account.config;

import jakarta.validation.constraints.NotBlank;
import org.springframework.boot.context.properties.ConfigurationProperties;
import org.springframework.validation.annotation.Validated;

/** Where auth answers on the private network; tokens come from spring.security.oauth2.client. */
@Validated
@ConfigurationProperties("notifications.auth")
public record AuthProperties(@NotBlank String url) {}
