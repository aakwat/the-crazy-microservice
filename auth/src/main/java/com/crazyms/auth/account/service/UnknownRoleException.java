package com.crazyms.auth.account.service;

public class UnknownRoleException extends RuntimeException {

  UnknownRoleException() {
    super("role is not one this project defines");
  }
}
