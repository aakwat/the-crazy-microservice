package com.crazyms.auth.shared.id;

import static org.assertj.core.api.Assertions.assertThat;

import java.time.Instant;
import org.junit.jupiter.api.Test;

class UuidV7Test {

  @Test
  void encodesVersionVariantAndTimestamp() {
    var at = Instant.parse("2026-11-05T10:00:00.123Z");
    var id = UuidV7.at(at);

    assertThat(id.version()).isEqualTo(7);
    assertThat(id.variant()).isEqualTo(2);
    assertThat(id.getMostSignificantBits() >>> 16).isEqualTo(at.toEpochMilli());
  }

  @Test
  void laterMillisecondSortsAfter() {
    var first = UuidV7.at(Instant.ofEpochMilli(1_000));
    var second = UuidV7.at(Instant.ofEpochMilli(1_001));

    assertThat(first.toString()).isLessThan(second.toString());
  }
}
