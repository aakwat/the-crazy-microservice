package com.crazyms.auth.security;

import com.crazyms.auth.account.domain.Role;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.annotation.Order;
import org.springframework.security.config.annotation.web.builders.HttpSecurity;
import org.springframework.security.config.annotation.web.configurers.AbstractHttpConfigurer;
import org.springframework.security.config.http.SessionCreationPolicy;
import org.springframework.security.oauth2.server.resource.authentication.JwtAuthenticationConverter;
import org.springframework.security.oauth2.server.resource.authentication.JwtGrantedAuthoritiesConverter;
import org.springframework.security.web.SecurityFilterChain;

/** The admin API: bearer access tokens only (RFC 6750), no session, no login page. */
@Configuration(proxyBeanMethods = false)
class AccountAdminSecurityConfig {

  @Bean
  @Order(2)
  SecurityFilterChain accountAdminChain(HttpSecurity http) throws Exception {
    return http.securityMatcher("/auth/accounts/**")
        .authorizeHttpRequests(auth -> auth.anyRequest().hasRole(Role.ADMIN.name()))
        .oauth2ResourceServer(rs -> rs.jwt(jwt -> jwt.jwtAuthenticationConverter(rolesClaim())))
        .sessionManagement(s -> s.sessionCreationPolicy(SessionCreationPolicy.STATELESS))
        // Only a bearer header authenticates here, never a cookie, so CSRF cannot ride on it.
        .csrf(AbstractHttpConfigurer::disable)
        .build();
  }

  // Tokens carry roles in the "roles" claim; hasRole wants them as ROLE_ authorities.
  private static JwtAuthenticationConverter rolesClaim() {
    var authorities = new JwtGrantedAuthoritiesConverter();
    authorities.setAuthoritiesClaimName("roles");
    authorities.setAuthorityPrefix("ROLE_");
    var converter = new JwtAuthenticationConverter();
    converter.setJwtGrantedAuthoritiesConverter(authorities);
    return converter;
  }
}
