package com.crazyms.auth.account.domain;

import static org.assertj.core.api.Assertions.assertThatCode;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.ValueSource;

class RoleTest {

  @ParameterizedTest
  @ValueSource(strings = {"customer", "agent", "support-lead", "tier_2", "a"})
  void acceptsLowercaseNamesProjectsWouldUse(String name) {
    assertThatCode(() -> new Role(name)).doesNotThrowAnyException();
  }

  // Role names go straight into signed tokens, so anything odd is rejected.
  @ParameterizedTest
  @ValueSource(
      strings = {
        "",
        "Agent",
        "ROLE_admin",
        "1st",
        "-x",
        "has space",
        "a\"b",
        "abcdefghijklmnopqrstuvwxyzabcdefg"
      })
  void rejectsNamesThatAreNotSimpleLowercaseIdentifiers(String name) {
    assertThatThrownBy(() -> new Role(name)).isInstanceOf(IllegalArgumentException.class);
  }
}
