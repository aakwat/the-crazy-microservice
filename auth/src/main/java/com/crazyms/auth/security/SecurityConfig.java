package com.crazyms.auth.security;

import jakarta.servlet.DispatcherType;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.annotation.Order;
import org.springframework.http.HttpMethod;
import org.springframework.security.config.Customizer;
import org.springframework.security.config.annotation.web.builders.HttpSecurity;
import org.springframework.security.crypto.argon2.Argon2PasswordEncoder;
import org.springframework.security.crypto.password.PasswordEncoder;
import org.springframework.security.web.SecurityFilterChain;

/** Everything outside the authorization server endpoints: the login page, sign-up and health. */
@Configuration(proxyBeanMethods = false)
class SecurityConfig {

    @Bean
    @Order(2)
    SecurityFilterChain applicationChain(HttpSecurity http) throws Exception {
        return http
                .authorizeHttpRequests(auth -> auth
                        .dispatcherTypeMatchers(DispatcherType.ERROR).permitAll()
                        .requestMatchers(HttpMethod.POST, "/auth/register").permitAll()
                        .requestMatchers(HttpMethod.GET, "/actuator/health/**").permitAll()
                        .anyRequest().authenticated())
                .formLogin(Customizer.withDefaults())
                // Sign-up is a JSON API that neither reads nor sets the session cookie, so CSRF cannot ride on it.
                .csrf(csrf -> csrf.ignoringRequestMatchers("/auth/register"))
                .build();
    }

    @Bean
    PasswordEncoder passwordEncoder() {
        // OWASP minimum for Argon2id: 19 MiB memory, 2 iterations, parallelism 1. Also hashes client secrets.
        return new Argon2PasswordEncoder(16, 32, 1, 19 * 1024, 2);
    }
}
