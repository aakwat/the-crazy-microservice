package com.crazyms.auth.account.domain;

import java.util.Objects;
import java.util.regex.Pattern;

/** A role name chosen by the deploying project (e.g. "agent"), not by this service. */
public record Role(String name) {

  private static final Pattern NAME = Pattern.compile("[a-z][a-z0-9_-]{0,31}");

  public Role {
    Objects.requireNonNull(name);
    if (!NAME.matcher(name).matches()) {
      throw new IllegalArgumentException("role must match " + NAME + ": " + name);
    }
  }
}
