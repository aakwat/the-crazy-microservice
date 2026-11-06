package com.crazyms.auth.account.web.dto;

import com.crazyms.auth.account.domain.Account;
import com.crazyms.auth.account.domain.PasswordCredential;
import jakarta.validation.constraints.Email;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotNull;
import jakarta.validation.constraints.Size;

public record RegisterRequest(
    @NotBlank @Email @Size(max = Account.MAX_EMAIL_LENGTH) String email,
    @NotNull
        @Size(
            min = PasswordCredential.MIN_PASSWORD_LENGTH,
            max = PasswordCredential.MAX_PASSWORD_LENGTH)
        String password) {}
