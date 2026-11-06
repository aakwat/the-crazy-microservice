package com.crazyms.auth.authserver.web;

import java.util.Set;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.annotation.Order;
import org.springframework.http.MediaType;
import org.springframework.security.config.Customizer;
import org.springframework.security.config.annotation.web.builders.HttpSecurity;
import org.springframework.security.web.SecurityFilterChain;
import org.springframework.security.web.authentication.LoginUrlAuthenticationEntryPoint;
import org.springframework.security.web.util.matcher.MediaTypeRequestMatcher;

/** Security for the OAuth/OIDC protocol endpoints only; the app chain handles the rest. */
@Configuration(proxyBeanMethods = false)
class AuthorizationServerSecurityConfig {

  @Bean
  @Order(1)
  SecurityFilterChain authorizationServerChain(HttpSecurity http) throws Exception {
    return http.oauth2AuthorizationServer(
            server -> {
              http.securityMatcher(server.getEndpointsMatcher());
              server.oidc(Customizer.withDefaults());
            })
        .authorizeHttpRequests(auth -> auth.anyRequest().authenticated())
        // Unauthenticated browsers go to the login page; API clients get 401.
        .exceptionHandling(e ->
            e.defaultAuthenticationEntryPointFor(
                new LoginUrlAuthenticationEntryPoint("/login"), browserRequests()))
        // /userinfo is itself protected by an access token.
        .oauth2ResourceServer(rs -> rs.jwt(Customizer.withDefaults()))
        .build();
  }

  private static MediaTypeRequestMatcher browserRequests() {
    var matcher = new MediaTypeRequestMatcher(MediaType.TEXT_HTML);
    // API clients send Accept: */*, which would otherwise match text/html and redirect.
    matcher.setIgnoredMediaTypes(Set.of(MediaType.ALL));
    return matcher;
  }
}
