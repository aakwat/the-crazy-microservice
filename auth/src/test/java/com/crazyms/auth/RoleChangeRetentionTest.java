package com.crazyms.auth;

import static org.assertj.core.api.Assertions.assertThat;

import com.crazyms.auth.account.domain.Role;
import com.crazyms.auth.account.domain.RoleChange;
import com.crazyms.auth.account.domain.port.RoleChangeRepository;
import com.crazyms.auth.account.service.AccountService;
import com.crazyms.auth.account.service.RoleChangeRetention;
import java.time.Clock;
import java.time.Duration;
import java.util.UUID;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.jdbc.core.simple.JdbcClient;

class RoleChangeRetentionTest extends AuthTestSupport {

  @Autowired RoleChangeRetention retention;

  @Autowired RoleChangeRepository roleChanges;

  @Autowired AccountService accounts;

  @Autowired JdbcClient jdbc;

  @Autowired Clock clock;

  @Test
  void purgesOnlyRoleChangesOlderThanTheRetention() {
    var actor = accounts.register(uniqueEmail(), "correct horse battery").id();
    var expired = accounts.register(uniqueEmail(), "correct horse battery").id();
    var recent = accounts.register(uniqueEmail(), "correct horse battery").id();
    var now = clock.instant();
    roleChanges.insert(
        new RoleChange(
            expired,
            actor,
            new Role("customer"),
            new Role("agent"),
            now.minus(Duration.ofDays(366))));
    roleChanges.insert(
        new RoleChange(
            recent,
            actor,
            new Role("customer"),
            new Role("agent"),
            now.minus(Duration.ofDays(364))));

    assertThat(retention.purgeExpired()).isPositive();

    assertThat(count(expired)).isZero();
    assertThat(count(recent)).isOne();
  }

  private long count(UUID accountId) {
    return jdbc.sql("SELECT COUNT(*) FROM account_role_change WHERE account_id = :id")
        .param("id", accountId)
        .query(Long.class)
        .single();
  }
}
