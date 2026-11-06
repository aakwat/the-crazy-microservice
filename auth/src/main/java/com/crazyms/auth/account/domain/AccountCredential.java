package com.crazyms.auth.account.domain;

/** What the login page needs in one read: the identity and its password credential. */
public record AccountCredential(Account account, PasswordCredential password) {}
