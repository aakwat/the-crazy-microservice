package com.crazyms.auth.account.config;

import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotEmpty;
import jakarta.validation.constraints.NotNull;
import java.time.Duration;
import java.util.List;
import org.springframework.boot.context.properties.ConfigurationProperties;
import org.springframework.validation.annotation.Validated;

@Validated
@ConfigurationProperties("auth.account")
public record AccountProperties(
    @NotBlank String defaultRole,
    @NotEmpty List<String> roles,
    @NotNull Duration roleChangeRetention) {}
