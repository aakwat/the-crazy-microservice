package com.crazyms.auth.account.web;

import com.crazyms.auth.account.service.AccountService;
import com.crazyms.auth.account.web.dto.AccountResponse;
import com.crazyms.auth.account.web.dto.RegisterRequest;
import jakarta.validation.Valid;
import org.springframework.http.HttpStatus;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.ResponseStatus;
import org.springframework.web.bind.annotation.RestController;

/** Self-service sign-up; all token work uses the standard OAuth endpoints. */
@RestController
@RequestMapping("/auth")
class AccountController {

  private final AccountService accounts;

  AccountController(AccountService accounts) {
    this.accounts = accounts;
  }

  @PostMapping("/register")
  @ResponseStatus(HttpStatus.CREATED)
  AccountResponse register(@Valid @RequestBody RegisterRequest request) {
    return AccountResponse.from(accounts.register(request.email(), request.password()));
  }
}
