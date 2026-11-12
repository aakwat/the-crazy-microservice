package com.crazyms.auth.security;

import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.annotation.Order;
import org.springframework.security.config.annotation.web.builders.HttpSecurity;
import org.springframework.security.config.annotation.web.configurers.AbstractHttpConfigurer;
import org.springframework.security.config.http.SessionCreationPolicy;
import org.springframework.security.web.SecurityFilterChain;

/** Service-to-service reads: bearer access tokens with the matching scope, never a browser. */
@Configuration(proxyBeanMethods = false)
class InternalApiSecurityConfig {

  /** Only client_credentials clients are granted it; the gateway's user tokens never carry it. */
  static final String ACCOUNTS_READ = "SCOPE_accounts.read";

  @Bean
  @Order(3)
  SecurityFilterChain internalApiChain(HttpSecurity http) throws Exception {
    return http.securityMatcher("/internal/**")
        .authorizeHttpRequests(auth -> auth.anyRequest().hasAuthority(ACCOUNTS_READ))
        .oauth2ResourceServer(rs -> rs.jwt(jwt -> {}))
        .sessionManagement(s -> s.sessionCreationPolicy(SessionCreationPolicy.STATELESS))
        // Only a bearer header authenticates here, never a cookie, so CSRF cannot ride on it.
        .csrf(AbstractHttpConfigurer::disable)
        .build();
  }
}
