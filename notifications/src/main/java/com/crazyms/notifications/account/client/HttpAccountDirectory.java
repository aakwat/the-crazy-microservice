package com.crazyms.notifications.account.client;

import com.crazyms.notifications.notification.domain.port.AccountDirectory;
import java.util.Optional;
import java.util.UUID;
import org.springframework.stereotype.Component;
import org.springframework.web.client.RestClient;

/** Reads one address from auth's internal lookup (contracts/openapi/auth.yaml). */
@Component
class HttpAccountDirectory implements AccountDirectory {

  private final RestClient auth;

  HttpAccountDirectory(RestClient authRestClient) {
    this.auth = authRestClient;
  }

  @Override
  public Optional<String> emailOf(UUID accountId) {
    return auth.get()
        .uri("/internal/accounts/{id}", accountId)
        .exchange((request, response) -> {
          int status = response.getStatusCode().value();
          if (status == 404) {
            return Optional.empty();
          }
          if (!response.getStatusCode().is2xxSuccessful()) {
            throw new AccountLookupException(status);
          }
          return Optional.ofNullable(response.bodyTo(AccountEmail.class)).map(AccountEmail::email);
        });
  }

  record AccountEmail(UUID id, String email) {}
}
