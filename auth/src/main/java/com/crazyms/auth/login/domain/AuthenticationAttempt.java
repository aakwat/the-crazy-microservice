package com.crazyms.auth.login.domain;

import java.time.Instant;
import java.util.Objects;
import java.util.UUID;

/** One login outcome; a null accountId (no matching account) is the spraying signal. */
public record AuthenticationAttempt(
    String submittedIdentifier,
    UUID accountId,
    FailureReason failureReason,
    ClientInfo client,
    Instant attemptedAt) {

  private static final int MAX_IDENTIFIER = 320;

  public AuthenticationAttempt {
    submittedIdentifier = submittedIdentifier == null ? "" : submittedIdentifier;
    if (submittedIdentifier.length() > MAX_IDENTIFIER) {
      submittedIdentifier = submittedIdentifier.substring(0, MAX_IDENTIFIER);
    }
    Objects.requireNonNull(client);
    Objects.requireNonNull(attemptedAt);
  }

  public static AuthenticationAttempt succeeded(
      String identifier, UUID accountId, ClientInfo client, Instant at) {
    return new AuthenticationAttempt(
        identifier, Objects.requireNonNull(accountId), null, client, at);
  }

  public static AuthenticationAttempt failed(
      String identifier, UUID accountId, FailureReason reason, ClientInfo client, Instant at) {
    return new AuthenticationAttempt(
        identifier, accountId, Objects.requireNonNull(reason), client, at);
  }

  public boolean successful() {
    return failureReason == null;
  }
}
