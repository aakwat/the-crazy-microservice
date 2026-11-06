package com.crazyms.auth.security;

import com.crazyms.auth.login.domain.ClientInfo;
import com.crazyms.auth.login.service.LoginAttemptService;
import jakarta.servlet.http.HttpServletRequest;
import java.util.Optional;
import java.util.UUID;
import org.springframework.context.event.EventListener;
import org.springframework.security.authentication.UsernamePasswordAuthenticationToken;
import org.springframework.security.authentication.event.AbstractAuthenticationFailureEvent;
import org.springframework.security.authentication.event.AuthenticationSuccessEvent;
import org.springframework.security.core.userdetails.UserDetails;
import org.springframework.stereotype.Component;
import org.springframework.web.context.request.RequestContextHolder;
import org.springframework.web.context.request.ServletRequestAttributes;

/** Turns form-login events into login outcomes; OAuth client authentication is not one. */
@Component
class LoginEventListener {

  private final LoginAttemptService attempts;

  LoginEventListener(LoginAttemptService attempts) {
    this.attempts = attempts;
  }

  @EventListener
  void onSuccess(AuthenticationSuccessEvent event) {
    if (event.getAuthentication() instanceof UsernamePasswordAuthenticationToken auth
        && auth.getPrincipal() instanceof UserDetails user) {
      String typed = request().map(r -> r.getParameter("username")).orElse(user.getUsername());
      attempts.recordSuccess(UUID.fromString(user.getUsername()), typed, client());
    }
  }

  @EventListener
  void onFailure(AbstractAuthenticationFailureEvent event) {
    if (event.getAuthentication() instanceof UsernamePasswordAuthenticationToken auth) {
      attempts.recordFailure(auth.getName(), client());
    }
  }

  private static ClientInfo client() {
    return request()
        .map(r -> new ClientInfo(r.getRemoteAddr(), r.getHeader("User-Agent")))
        .orElse(ClientInfo.UNKNOWN);
  }

  private static Optional<HttpServletRequest> request() {
    return RequestContextHolder.getRequestAttributes() instanceof ServletRequestAttributes attrs
        ? Optional.of(attrs.getRequest())
        : Optional.empty();
  }
}
