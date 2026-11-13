package com.crazyms.notifications;

import static org.awaitility.Awaitility.await;

import io.lettuce.core.Range;
import io.lettuce.core.RedisClient;
import io.lettuce.core.RedisURI;
import io.lettuce.core.StreamMessage;
import io.lettuce.core.api.StatefulRedisConnection;
import java.time.Duration;
import java.util.List;
import java.util.Map;

/** Puts envelopes on tickets.events, as the tickets relay does, and reads the dead letters. */
final class TestBus implements AutoCloseable {

  private final RedisClient client;
  private final StatefulRedisConnection<String, String> connection;

  TestBus(String host, int port, String password) {
    client =
        RedisClient.create(
            RedisURI.builder()
                .withHost(host)
                .withPort(port)
                .withPassword(password.toCharArray())
                .build());
    connection = client.connect();
  }

  /** The service creates its group, and with it the stream, on its first round. */
  void awaitConsumerGroup() {
    await()
        .atMost(Duration.ofSeconds(30))
        .until(() -> connection.sync().exists("tickets.events") == 1);
  }

  void publish(String envelope) {
    connection.sync().xadd("tickets.events", Map.of("envelope", envelope));
  }

  List<Map<String, String>> deadLetters() {
    return connection.sync().xrange("tickets.events.dlq", Range.create("-", "+")).stream()
        .map(StreamMessage::getBody)
        .toList();
  }

  @Override
  public void close() {
    connection.close();
    client.shutdown();
  }
}
