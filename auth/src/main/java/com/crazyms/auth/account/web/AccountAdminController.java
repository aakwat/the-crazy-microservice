package com.crazyms.auth.account.web;

import com.crazyms.auth.account.service.AccountService;
import com.crazyms.auth.account.web.dto.AccountResponse;
import com.crazyms.auth.account.web.dto.ChangeRoleRequest;
import jakarta.validation.Valid;
import java.util.UUID;
import org.springframework.http.HttpStatus;
import org.springframework.security.core.annotation.AuthenticationPrincipal;
import org.springframework.security.oauth2.jwt.Jwt;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PutMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.RestController;
import org.springframework.web.server.ResponseStatusException;

/** Role management; its security chain admits only access tokens carrying the admin role. */
@RestController
@RequestMapping("/auth/accounts")
class AccountAdminController {

  private final AccountService accounts;

  AccountAdminController(AccountService accounts) {
    this.accounts = accounts;
  }

  @GetMapping
  AccountResponse findByEmail(@RequestParam String email, @AuthenticationPrincipal Jwt caller) {
    return accounts
        .findByEmailAsAdmin(UUID.fromString(caller.getSubject()), email)
        .map(AccountResponse::from)
        .orElseThrow(this::notFound);
  }

  @PutMapping("/{id}/role")
  AccountResponse changeRole(
      @PathVariable UUID id,
      @Valid @RequestBody ChangeRoleRequest request,
      @AuthenticationPrincipal Jwt caller) {
    var actor = UUID.fromString(caller.getSubject());
    return accounts
        .changeRole(actor, id, request.role())
        .map(AccountResponse::from)
        .orElseThrow(this::notFound);
  }

  private ResponseStatusException notFound() {
    return new ResponseStatusException(HttpStatus.NOT_FOUND, "no such account");
  }
}
