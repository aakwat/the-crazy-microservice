package com.crazyms.auth.authserver.client;

import jakarta.validation.Valid;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotEmpty;
import java.util.List;
import java.util.Set;
import org.springframework.boot.context.properties.ConfigurationProperties;
import org.springframework.validation.annotation.Validated;

@Validated
@ConfigurationProperties("auth.client")
record ClientProperties(@NotEmpty List<@Valid Registration> registrations) {

  /** One OAuth client: e.g. the gateway (browser login) or a worker (service token). */
  record Registration(
      @NotBlank String id,
      @NotBlank String secret,
      @NotEmpty Set<String> grantTypes,
      Set<String> redirectUris,
      @NotEmpty Set<String> scopes) {

    Registration {
      redirectUris = redirectUris == null ? Set.of() : redirectUris;
    }
  }
}
