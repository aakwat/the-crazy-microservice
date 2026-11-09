package com.crazyms.auth;

/** The OAuth clients and issuer the test context registers; shared by the config and the tests. */
final class TestClients {

  static final String ISSUER = "http://auth.test";
  static final String GATEWAY = "gateway";
  static final String GATEWAY_SECRET = "gateway-test-secret";
  static final String REDIRECT_URI = "http://127.0.0.1:9999/callback";
  static final String POST_LOGOUT_URI = "http://127.0.0.1:9999/logged-out";
  static final String JOBS = "jobs";
  static final String JOBS_SECRET = "jobs-test-secret";

  private TestClients() {}
}
