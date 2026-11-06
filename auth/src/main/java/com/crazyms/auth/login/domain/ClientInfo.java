package com.crazyms.auth.login.domain;

/** Where a login came from; attacker-controlled, so bounded and never trusted. */
public record ClientInfo(String ipAddress, String userAgent) {

  public static final ClientInfo UNKNOWN = new ClientInfo(null, null);

  private static final int MAX_USER_AGENT = 500;

  public ClientInfo {
    if (userAgent != null && userAgent.length() > MAX_USER_AGENT) {
      userAgent = userAgent.substring(0, MAX_USER_AGENT);
    }
  }
}
