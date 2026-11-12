package com.crazyms.auth.account.web.dto;

import com.crazyms.auth.account.domain.Account;
import java.util.UUID;

/** Only what a mailer needs; the role stays inside auth (GDPR minimisation). */
public record AccountEmailResponse(UUID id, String email) {

  public static AccountEmailResponse from(Account account) {
    return new AccountEmailResponse(account.id(), account.email());
  }
}
