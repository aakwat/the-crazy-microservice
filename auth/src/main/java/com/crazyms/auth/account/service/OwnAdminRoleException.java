package com.crazyms.auth.account.service;

public class OwnAdminRoleException extends RuntimeException {

  OwnAdminRoleException() {
    super("admins cannot remove their own admin role");
  }
}
