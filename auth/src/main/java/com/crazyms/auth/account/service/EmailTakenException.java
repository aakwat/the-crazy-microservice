package com.crazyms.auth.account.service;

public class EmailTakenException extends RuntimeException {

  EmailTakenException() {
    super("email already registered");
  }
}
