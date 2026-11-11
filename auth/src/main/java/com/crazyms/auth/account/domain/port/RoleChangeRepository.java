package com.crazyms.auth.account.domain.port;

import com.crazyms.auth.account.domain.RoleChange;
import java.time.Instant;

/** Port: the append-only audit of role changes. */
public interface RoleChangeRepository {

  void insert(RoleChange change);

  /** Returns how many rows the retention sweep removed. */
  int deleteOlderThan(Instant cutoff);
}
