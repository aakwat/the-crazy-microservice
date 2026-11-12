package com.crazyms.auth.account.web;

import com.crazyms.auth.account.service.AccountService;
import com.crazyms.auth.account.web.dto.AccountEmailResponse;
import java.util.UUID;
import org.springframework.http.HttpStatus;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;
import org.springframework.web.server.ResponseStatusException;

/** Lookups for other services; the gateway 404s /internal, so browsers never reach it. */
@RestController
@RequestMapping("/internal/accounts")
class InternalAccountController {

  private final AccountService accounts;

  InternalAccountController(AccountService accounts) {
    this.accounts = accounts;
  }

  @GetMapping("/{id}")
  AccountEmailResponse find(@PathVariable UUID id) {
    return accounts
        .find(id)
        .map(AccountEmailResponse::from)
        .orElseThrow(() -> new ResponseStatusException(HttpStatus.NOT_FOUND, "no such account"));
  }
}
