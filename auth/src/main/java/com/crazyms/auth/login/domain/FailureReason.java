package com.crazyms.auth.login.domain;

import java.util.Locale;

public enum FailureReason {
  BAD_PASSWORD,
  UNKNOWN_IDENTIFIER,
  ACCOUNT_LOCKED;

  public String code() {
    return name().toLowerCase(Locale.ROOT);
  }
}
