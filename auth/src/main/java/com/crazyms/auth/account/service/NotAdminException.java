package com.crazyms.auth.account.service;

public class NotAdminException extends RuntimeException {

  NotAdminException() {
    super("the caller is no longer an admin");
  }
}
