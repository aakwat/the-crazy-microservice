package com.crazyms.auth.authserver.key;

import jakarta.validation.constraints.NotNull;
import java.nio.file.Path;
import org.springframework.boot.context.properties.ConfigurationProperties;
import org.springframework.validation.annotation.Validated;

@Validated
@ConfigurationProperties("auth.key")
record KeyProperties(@NotNull Path privatePath, @NotNull Path publicPath) {}
