package com.crazyms.auth.login.config;

import jakarta.validation.Valid;
import jakarta.validation.constraints.Min;
import jakarta.validation.constraints.NotNull;
import java.time.Duration;
import org.springframework.boot.context.properties.ConfigurationProperties;
import org.springframework.validation.annotation.Validated;

@Validated
@ConfigurationProperties("auth.login")
public record LoginProperties(@NotNull @Valid Lockout lockout, @NotNull Duration attemptRetention) {

  public record Lockout(@Min(1) int maxFailures, @NotNull Duration duration) {}
}
