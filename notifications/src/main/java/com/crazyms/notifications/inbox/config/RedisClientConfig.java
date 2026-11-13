package com.crazyms.notifications.inbox.config;

import io.lettuce.core.ClientOptions;
import io.lettuce.core.RedisClient;
import io.lettuce.core.RedisURI;
import io.lettuce.core.SocketOptions;
import java.time.Duration;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration(proxyBeanMethods = false)
class RedisClientConfig {

  /** Longer than the 2 s blocking read, short enough that an outage shows within one round. */
  private static final Duration TIMEOUT = Duration.ofSeconds(5);

  @Bean(destroyMethod = "shutdown")
  RedisClient eventsRedisClient(InboxProperties props) {
    var uri =
        RedisURI.builder()
            .withHost(props.host())
            .withPort(props.port())
            .withAuthentication(props.user(), props.password())
            .withTimeout(TIMEOUT)
            .build();
    var client = RedisClient.create(uri);
    client.setOptions(
        ClientOptions.builder()
            .socketOptions(SocketOptions.builder().connectTimeout(TIMEOUT).build())
            .build());
    return client;
  }
}
