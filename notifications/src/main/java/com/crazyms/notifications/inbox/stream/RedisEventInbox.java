package com.crazyms.notifications.inbox.stream;

import com.crazyms.notifications.inbox.config.InboxProperties;
import com.crazyms.notifications.inbox.domain.InboxMessage;
import com.crazyms.notifications.inbox.domain.port.EventInbox;
import io.lettuce.core.Consumer;
import io.lettuce.core.Limit;
import io.lettuce.core.Range;
import io.lettuce.core.RedisBusyException;
import io.lettuce.core.RedisClient;
import io.lettuce.core.StreamMessage;
import io.lettuce.core.XAddArgs;
import io.lettuce.core.XAutoClaimArgs;
import io.lettuce.core.XGroupCreateArgs;
import io.lettuce.core.XReadArgs;
import io.lettuce.core.XReadArgs.StreamOffset;
import io.lettuce.core.api.StatefulRedisConnection;
import io.lettuce.core.api.sync.RedisCommands;
import jakarta.annotation.PreDestroy;
import java.time.Duration;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.function.ToLongFunction;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Component;

/** tickets.events through the consumer group "notifications" (CLAUDE.md, events). */
@Component
class RedisEventInbox implements EventInbox {

  private static final Logger log = LoggerFactory.getLogger(RedisEventInbox.class);

  static final String STREAM = "tickets.events";
  static final String DEAD_LETTERS = "tickets.events.dlq";
  static final String GROUP = "notifications";
  static final String ENVELOPE_FIELD = "envelope";

  private static final int BATCH = 10;
  private static final Duration BLOCK = Duration.ofSeconds(2);

  /** Dead letters wait for a human; trimming bounds them if nobody looks. */
  private static final long MAX_DEAD_LETTERS = 10_000;

  private final RedisClient client;
  private final EnvelopeReader reader;
  private final InboxProperties props;
  private final Consumer<String> consumer;
  private StatefulRedisConnection<String, String> connection;

  RedisEventInbox(RedisClient client, EnvelopeReader reader, InboxProperties props) {
    this.client = client;
    this.reader = reader;
    this.props = props;
    // Every replica reads as its own consumer; a container's hostname is unique per replica.
    this.consumer = Consumer.from(GROUP, System.getenv().getOrDefault("HOSTNAME", GROUP));
  }

  @Override
  public List<InboxMessage> receive() {
    var entries =
        redis()
            .xreadgroup(
                consumer,
                XReadArgs.Builder.block(BLOCK).count(BATCH),
                StreamOffset.lastConsumed(STREAM));
    return toMessages(entries, id -> 1);
  }

  @Override
  public List<InboxMessage> reclaimStuck() {
    var claimed =
        redis()
            .xautoclaim(
                STREAM,
                XAutoClaimArgs.Builder.xautoclaim(consumer, props.stuckAfter(), "0-0")
                    .count(BATCH));
    return toMessages(claimed.getMessages(), this::deliveries);
  }

  @Override
  public void acknowledge(InboxMessage message) {
    redis().xack(STREAM, GROUP, message.id());
  }

  @Override
  public void deadLetter(InboxMessage message) {
    moveToDeadLetters(
        message.id(), message.envelope(), message.deliveries(), "too many deliveries");
  }

  @PreDestroy
  synchronized void close() {
    if (connection != null) {
      connection.close();
    }
  }

  private List<InboxMessage> toMessages(
      List<StreamMessage<String, String>> entries, ToLongFunction<String> deliveries) {
    var messages = new ArrayList<InboxMessage>(entries.size());
    for (var entry : entries) {
      var envelope = entry.getBody().get(ENVELOPE_FIELD);
      long count = deliveries.applyAsLong(entry.getId());
      try {
        messages.add(new InboxMessage(entry.getId(), count, envelope, reader.read(envelope)));
      } catch (UnreadableEnvelopeException e) {
        moveToDeadLetters(entry.getId(), envelope, count, e.getMessage());
      }
    }
    return messages;
  }

  private long deliveries(String id) {
    var pending = redis().xpending(STREAM, GROUP, Range.create(id, id), Limit.from(1));
    return pending.isEmpty() ? 1 : pending.getFirst().getRedeliveryCount();
  }

  private void moveToDeadLetters(String id, String envelope, long deliveries, String reason) {
    var fields = new HashMap<String, String>();
    fields.put("message_id", id);
    fields.put("reason", reason);
    fields.put("deliveries", Long.toString(deliveries));
    if (envelope != null) {
      fields.put(ENVELOPE_FIELD, envelope);
    }

    var redis = redis();
    redis.xadd(
        DEAD_LETTERS, XAddArgs.Builder.maxlen(MAX_DEAD_LETTERS).approximateTrimming(), fields);
    redis.xack(STREAM, GROUP, id);

    log.atWarn()
        .addKeyValue("message_id", id)
        .addKeyValue("reason", reason)
        .log("inbox.dead_lettered");
  }

  // Opened on first use, so the service starts, and stays healthy, while the bus is down.
  private synchronized RedisCommands<String, String> redis() {
    if (connection == null) {
      var opened = client.connect();
      try {
        createGroup(opened.sync());
      } catch (RuntimeException e) {
        opened.close();
        throw e;
      }
      connection = opened;
    }
    return connection.sync();
  }

  // "$": a new group starts at the end of the stream; after that Redis keeps its place.
  private static void createGroup(RedisCommands<String, String> redis) {
    try {
      redis.xgroupCreate(StreamOffset.latest(STREAM), GROUP, XGroupCreateArgs.Builder.mkstream());
    } catch (RedisBusyException groupAlreadyExists) {
      // Expected on every start after the first.
    }
  }
}
