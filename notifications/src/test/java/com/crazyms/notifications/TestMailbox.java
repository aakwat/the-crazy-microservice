package com.crazyms.notifications;

import java.io.IOException;
import java.net.URI;
import java.net.URLEncoder;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;
import tools.jackson.databind.json.JsonMapper;

/** Reads what reached Mailpit, through its HTTP API. */
final class TestMailbox {

  private final String apiUrl;
  private final HttpClient http = HttpClient.newHttpClient();
  private final JsonMapper json = JsonMapper.builder().build();

  TestMailbox(String apiUrl) {
    this.apiUrl = apiUrl;
  }

  List<String> subjectsTo(String address) throws IOException, InterruptedException {
    var query = URLEncoder.encode("to:" + address, StandardCharsets.UTF_8);
    var request =
        HttpRequest.newBuilder(URI.create(apiUrl + "/api/v1/search?query=" + query)).build();
    var body = http.send(request, HttpResponse.BodyHandlers.ofString()).body();

    var subjects = new ArrayList<String>();
    json.readTree(body)
        .path("messages")
        .forEach(m -> subjects.add(m.path("Subject").stringValue()));
    return subjects;
  }
}
