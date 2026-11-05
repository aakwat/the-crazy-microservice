package com.crazyms.auth.config;

import jakarta.validation.Valid;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotEmpty;
import jakarta.validation.constraints.NotNull;
import java.nio.file.Path;
import java.time.Duration;
import java.util.List;
import java.util.Set;
import org.springframework.boot.context.properties.ConfigurationProperties;
import org.springframework.validation.annotation.Validated;

@Validated
@ConfigurationProperties("auth")
public record AuthProperties(
        @NotBlank String audience,
        @NotNull Duration accessTokenTtl,
        @NotNull Duration refreshTokenTtl,
        @NotBlank String defaultRole,
        @NotNull Path privateKeyPath,
        @NotNull Path publicKeyPath,
        @NotEmpty List<@Valid Client> clients) {

    /** One OAuth client of the deploying project, e.g. its gateway (browser login) or a worker (service token). */
    public record Client(
            @NotBlank String id,
            @NotBlank String secret,
            @NotEmpty Set<String> grantTypes,
            Set<String> redirectUris,
            @NotEmpty Set<String> scopes) {

        public Set<String> redirectUris() {
            return redirectUris == null ? Set.of() : redirectUris;
        }
    }
}
