package com.crazyms.notifications;

import com.sun.net.httpserver.HttpExchange;
import com.sun.net.httpserver.HttpServer;
import java.io.IOException;
import java.net.InetSocketAddress;
import java.nio.charset.StandardCharsets;
import java.util.Map;
import java.util.Set;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;

/** Stands in for auth: hands out a token and answers /internal/accounts from a map. */
final class AuthStub implements AutoCloseable {

  static final String TOKEN = "stub-token";

  private final HttpServer server;
  private final Map<UUID, String> emails = new ConcurrentHashMap<>();
  private final Set<UUID> failing = ConcurrentHashMap.newKeySet();

  AuthStub() throws IOException {
    server = HttpServer.create(new InetSocketAddress("127.0.0.1", 0), 0);
    server.createContext("/oauth2/token", this::token);
    server.createContext("/internal/accounts/", this::account);
    server.start();
  }

  String url() {
    return "http://127.0.0.1:" + server.getAddress().getPort();
  }

  void addAccount(UUID id, String email) {
    emails.put(id, email);
  }

  void failLookupsFor(UUID id) {
    failing.add(id);
  }

  void recover(UUID id) {
    failing.remove(id);
  }

  @Override
  public void close() {
    server.stop(0);
  }

  private void token(HttpExchange exchange) throws IOException {
    respond(
        exchange,
        200,
        "{\"access_token\":\"" + TOKEN + "\",\"token_type\":\"Bearer\",\"expires_in\":300}");
  }

  private void account(HttpExchange exchange) throws IOException {
    if (!("Bearer " + TOKEN).equals(exchange.getRequestHeaders().getFirst("Authorization"))) {
      respond(exchange, 401, "{}");
      return;
    }
    var path = exchange.getRequestURI().getPath();
    var id = UUID.fromString(path.substring(path.lastIndexOf('/') + 1));
    var email = emails.get(id);
    if (failing.contains(id)) {
      respond(exchange, 500, "{}");
    } else if (email == null) {
      respond(exchange, 404, "{}");
    } else {
      respond(exchange, 200, "{\"id\":\"" + id + "\",\"email\":\"" + email + "\"}");
    }
  }

  private static void respond(HttpExchange exchange, int status, String json) throws IOException {
    var body = json.getBytes(StandardCharsets.UTF_8);
    exchange.getResponseHeaders().add("Content-Type", "application/json");
    exchange.sendResponseHeaders(status, body.length);
    try (var out = exchange.getResponseBody()) {
      out.write(body);
    }
  }
}
