package com.crazyms.notifications.inbox.worker;

import com.crazyms.notifications.inbox.service.InboxProcessor;
import java.time.Duration;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.boot.SpringApplication;
import org.springframework.context.ConfigurableApplicationContext;
import org.springframework.context.SmartLifecycle;
import org.springframework.stereotype.Component;

/** Runs inbox rounds on its own thread: failures back off, a crash takes the process down too. */
@Component
class InboxWorker implements SmartLifecycle {

  private static final Logger log = LoggerFactory.getLogger(InboxWorker.class);

  private static final Duration MIN_BACKOFF = Duration.ofMillis(500);
  private static final Duration MAX_BACKOFF = Duration.ofSeconds(30);

  /** Below the container's stop grace period, so a round in flight finishes first. */
  private static final Duration DRAIN = Duration.ofSeconds(10);

  private final InboxProcessor processor;
  private final InboxLiveness liveness;
  private final ConfigurableApplicationContext context;
  private volatile boolean running;
  private Thread thread;

  InboxWorker(
      InboxProcessor processor, InboxLiveness liveness, ConfigurableApplicationContext context) {
    this.processor = processor;
    this.liveness = liveness;
    this.context = context;
  }

  @Override
  public void start() {
    running = true;
    thread = Thread.ofPlatform().name("inbox-worker").daemon().start(this::run);
  }

  @Override
  public void stop() {
    running = false;
    try {
      thread.join(DRAIN);
    } catch (InterruptedException e) {
      Thread.currentThread().interrupt();
    }
    thread.interrupt();
  }

  @Override
  public boolean isRunning() {
    return running;
  }

  private void run() {
    try {
      loop();
    } catch (InterruptedException stopping) {
      Thread.currentThread().interrupt();
    } catch (Error e) {
      log.atError().addKeyValue("error", e.getClass().getName()).log("inbox.worker_crashed");
      exitProcess();
    }
  }

  private void loop() throws InterruptedException {
    int failures = 0;
    while (running) {
      failures = round(failures);
      liveness.roundFinished();
      if (failures > 0) {
        Thread.sleep(backoff(failures));
      }
    }
  }

  /** Returns the number of failed rounds in a row. */
  private int round(int failures) {
    try {
      processor.processOnce();
      return 0;
    } catch (RuntimeException e) {
      log.atWarn()
          .addKeyValue("error", e.getClass().getSimpleName())
          .addKeyValue("retry_in_ms", backoff(failures + 1).toMillis())
          .log("inbox.round_failed");
      return failures + 1;
    }
  }

  static Duration backoff(int failures) {
    var wait = MIN_BACKOFF.multipliedBy(1L << Math.min(failures - 1, 6));
    return wait.compareTo(MAX_BACKOFF) > 0 ? MAX_BACKOFF : wait;
  }

  // A dead worker must not look healthy; exiting lets the container restart it.
  private void exitProcess() {
    Thread.ofPlatform().start(() -> System.exit(SpringApplication.exit(context, () -> 1)));
  }
}
