package com.crazyms.notifications.inbox.domain.port;

import com.crazyms.notifications.inbox.domain.InboxMessage;
import java.util.List;

/** Port: at-least-once delivery of ticket events to this service's consumer group. */
public interface EventInbox {

  /** New messages, waiting briefly when there are none. */
  List<InboxMessage> receive();

  /** Messages handed out earlier, to any replica, and never acknowledged. */
  List<InboxMessage> reclaimStuck();

  void acknowledge(InboxMessage message);

  /** Moves the message to the dead-letter stream for a human and acknowledges it. */
  void deadLetter(InboxMessage message);
}
