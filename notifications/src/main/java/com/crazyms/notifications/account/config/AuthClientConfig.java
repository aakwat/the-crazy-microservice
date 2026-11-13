package com.crazyms.notifications.account.config;

import java.time.Duration;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.http.client.SimpleClientHttpRequestFactory;
import org.springframework.http.converter.FormHttpMessageConverter;
import org.springframework.security.oauth2.client.AuthorizedClientServiceOAuth2AuthorizedClientManager;
import org.springframework.security.oauth2.client.OAuth2AuthorizedClientManager;
import org.springframework.security.oauth2.client.OAuth2AuthorizedClientProviderBuilder;
import org.springframework.security.oauth2.client.OAuth2AuthorizedClientService;
import org.springframework.security.oauth2.client.endpoint.RestClientClientCredentialsTokenResponseClient;
import org.springframework.security.oauth2.client.http.OAuth2ErrorResponseErrorHandler;
import org.springframework.security.oauth2.client.registration.ClientRegistrationRepository;
import org.springframework.security.oauth2.client.web.client.OAuth2ClientHttpRequestInterceptor;
import org.springframework.security.oauth2.core.http.converter.OAuth2AccessTokenResponseHttpMessageConverter;
import org.springframework.web.client.RestClient;

/** Calls to auth carry a client_credentials token, cached until it expires, and never hang. */
@Configuration(proxyBeanMethods = false)
class AuthClientConfig {

  /** The registration under spring.security.oauth2.client.registration. */
  static final String REGISTRATION = "auth";

  /** A slow auth must not stall the inbox; the event is simply retried. */
  private static final Duration TIMEOUT = Duration.ofSeconds(3);

  @Bean
  OAuth2AuthorizedClientManager authorizedClientManager(
      ClientRegistrationRepository registrations, OAuth2AuthorizedClientService clients) {
    var tokens = new RestClientClientCredentialsTokenResponseClient();
    tokens.setRestClient(tokenClient());
    var provider =
        OAuth2AuthorizedClientProviderBuilder.builder()
            .clientCredentials(c -> c.accessTokenResponseClient(tokens))
            .build();

    var manager = new AuthorizedClientServiceOAuth2AuthorizedClientManager(registrations, clients);
    manager.setAuthorizedClientProvider(provider);
    return manager;
  }

  @Bean
  RestClient authRestClient(AuthProperties props, OAuth2AuthorizedClientManager manager) {
    var interceptor = new OAuth2ClientHttpRequestInterceptor(manager);
    interceptor.setClientRegistrationIdResolver(request -> REGISTRATION);
    return RestClient.builder()
        .baseUrl(props.url())
        .requestFactory(timeouts())
        .requestInterceptor(interceptor)
        .build();
  }

  // Spring Security's default token client, plus the timeouts it leaves unset.
  private static RestClient tokenClient() {
    return RestClient.builder()
        .requestFactory(timeouts())
        .configureMessageConverters(c ->
            c.addCustomConverter(new FormHttpMessageConverter())
                .addCustomConverter(new OAuth2AccessTokenResponseHttpMessageConverter()))
        .defaultStatusHandler(new OAuth2ErrorResponseErrorHandler())
        .build();
  }

  private static SimpleClientHttpRequestFactory timeouts() {
    var factory = new SimpleClientHttpRequestFactory();
    factory.setConnectTimeout(TIMEOUT);
    factory.setReadTimeout(TIMEOUT);
    return factory;
  }
}
