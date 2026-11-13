package com.crazyms.notifications.inbox.stream;

import com.crazyms.notifications.notification.domain.TicketEvent;
import com.crazyms.notifications.notification.domain.TicketEventType;
import java.util.UUID;
import org.springframework.stereotype.Component;
import tools.jackson.core.JacksonException;
import tools.jackson.databind.JsonNode;
import tools.jackson.databind.json.JsonMapper;

/** Reads a tickets.events envelope (contracts/events) and refuses anything off contract. */
@Component
class EnvelopeReader {

  /** The only envelope version this reader understands; a new one needs code here first. */
  private static final int VERSION = 1;

  private final JsonMapper json;

  EnvelopeReader(JsonMapper json) {
    this.json = json;
  }

  TicketEvent read(String envelope) {
    var root = parse(envelope);
    var version = root.path("version");
    if (!version.isIntegralNumber() || version.intValue() != VERSION) {
      throw new UnreadableEnvelopeException("unsupported version");
    }
    var type =
        TicketEventType.fromWire(text(root, "type"))
            .orElseThrow(() -> new UnreadableEnvelopeException("unknown type"));

    var payload = root.path("payload");
    return new TicketEvent(
        uuid(root, "id"),
        type,
        uuid(payload, "ticket_id"),
        text(payload, "reference"),
        text(payload, "status"),
        uuid(payload, "requester_id"),
        nullableUuid(payload, "assignee_id"));
  }

  private JsonNode parse(String envelope) {
    if (envelope == null) {
      throw new UnreadableEnvelopeException("no envelope field");
    }
    try {
      return json.readTree(envelope);
    } catch (JacksonException e) {
      throw new UnreadableEnvelopeException("not JSON");
    }
  }

  private static String text(JsonNode node, String field) {
    var value = node.path(field);
    if (!value.isString()) {
      throw new UnreadableEnvelopeException("missing " + field);
    }
    return value.stringValue();
  }

  private static UUID uuid(JsonNode node, String field) {
    try {
      return UUID.fromString(text(node, field));
    } catch (IllegalArgumentException e) {
      throw new UnreadableEnvelopeException("bad " + field);
    }
  }

  private static UUID nullableUuid(JsonNode node, String field) {
    var value = node.path(field);
    return value.isNull() || value.isMissingNode() ? null : uuid(node, field);
  }
}
