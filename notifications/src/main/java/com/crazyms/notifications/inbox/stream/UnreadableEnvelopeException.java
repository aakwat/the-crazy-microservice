package com.crazyms.notifications.inbox.stream;

/** The entry breaks the event contract; retrying cannot fix it, so it goes straight aside. */
class UnreadableEnvelopeException extends RuntimeException {

  UnreadableEnvelopeException(String reason) {
    super(reason);
  }
}
