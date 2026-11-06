package com.crazyms.auth.account.config;

import jakarta.validation.constraints.NotBlank;
import org.springframework.boot.context.properties.ConfigurationProperties;
import org.springframework.validation.annotation.Validated;

@Validated
@ConfigurationProperties("auth.account")
public record AccountProperties(@NotBlank String defaultRole) {}
