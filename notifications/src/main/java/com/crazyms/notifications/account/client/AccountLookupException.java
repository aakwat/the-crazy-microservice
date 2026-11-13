package com.crazyms.notifications.account.client;

/** Auth answered, but not with an account or a 404; the event is retried later. */
public class AccountLookupException extends RuntimeException {

  AccountLookupException(int status) {
    super("account lookup failed with HTTP " + status);
  }
}
