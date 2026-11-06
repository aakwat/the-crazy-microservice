package com.crazyms.auth.shared.id;

import java.security.SecureRandom;
import java.time.Instant;
import java.util.UUID;

/** RFC 9562 UUIDv7: 48-bit Unix milliseconds, then random bits, so ids sort by creation time. */
public final class UuidV7 {

  private static final SecureRandom RANDOM = new SecureRandom();

  private UuidV7() {}

  public static UUID at(Instant instant) {
    long millis = instant.toEpochMilli();
    long msb = (millis << 16) | 0x7000L | (RANDOM.nextLong() & 0x0FFFL);
    long lsb = (RANDOM.nextLong() & 0x3FFF_FFFF_FFFF_FFFFL) | 0x8000_0000_0000_0000L;
    return new UUID(msb, lsb);
  }
}
