package com.crazyms.auth.account.web.dto;

import com.crazyms.auth.account.domain.Account;
import java.util.UUID;

public record AccountResponse(UUID id, String email, String role) {

  public static AccountResponse from(Account account) {
    return new AccountResponse(account.id(), account.email(), account.role().name());
  }
}
