package com.crazyms.auth.account.web.dto;

import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.Size;

public record ChangeRoleRequest(@NotBlank @Size(max = 32) String role) {}
