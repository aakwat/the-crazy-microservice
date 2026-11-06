package com.crazyms.auth.authserver.repository;

import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.jdbc.core.JdbcOperations;
import org.springframework.security.oauth2.server.authorization.JdbcOAuth2AuthorizationConsentService;
import org.springframework.security.oauth2.server.authorization.JdbcOAuth2AuthorizationService;
import org.springframework.security.oauth2.server.authorization.OAuth2AuthorizationConsentService;
import org.springframework.security.oauth2.server.authorization.OAuth2AuthorizationService;
import org.springframework.security.oauth2.server.authorization.client.RegisteredClientRepository;

/** Postgres-backed stores, so a login can continue on any replica. */
@Configuration(proxyBeanMethods = false)
class AuthorizationStoreConfig {

  @Bean
  OAuth2AuthorizationService authorizationService(
      JdbcOperations jdbc, RegisteredClientRepository clients) {
    return new JdbcOAuth2AuthorizationService(jdbc, clients);
  }

  @Bean
  OAuth2AuthorizationConsentService authorizationConsentService(
      JdbcOperations jdbc, RegisteredClientRepository clients) {
    return new JdbcOAuth2AuthorizationConsentService(jdbc, clients);
  }
}
