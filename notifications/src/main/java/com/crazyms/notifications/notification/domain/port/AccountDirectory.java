package com.crazyms.notifications.notification.domain.port;

import java.util.Optional;
import java.util.UUID;

/** Port: auth owns email addresses; this service only borrows one at a time and never stores it. */
public interface AccountDirectory {

  /** Empty when the account no longer exists; throws when the directory cannot be reached. */
  Optional<String> emailOf(UUID accountId);
}
