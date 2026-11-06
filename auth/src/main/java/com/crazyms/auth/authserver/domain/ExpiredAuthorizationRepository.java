package com.crazyms.auth.authserver.domain;

import java.time.Instant;

/** Port: removes authorizations whose every token has expired; SAS itself never deletes them. */
public interface ExpiredAuthorizationRepository {

  /** Returns how many authorizations were removed. */
  int deleteExpiredBefore(Instant cutoff);
}
