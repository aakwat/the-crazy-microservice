package com.crazyms.auth.security;

import com.crazyms.auth.account.domain.Account;
import com.crazyms.auth.account.domain.PasswordCredential;
import jakarta.servlet.FilterChain;
import jakarta.servlet.ServletException;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;
import java.io.IOException;
import org.springframework.web.filter.OncePerRequestFilter;

/** No valid credential is this long, so oversized logins fail like bad ones before Argon2 runs. */
class LoginInputLimitFilter extends OncePerRequestFilter {

  @Override
  protected boolean shouldNotFilter(HttpServletRequest request) {
    return !("POST".equals(request.getMethod()) && "/login".equals(request.getServletPath()));
  }

  @Override
  protected void doFilterInternal(
      HttpServletRequest request, HttpServletResponse response, FilterChain chain)
      throws ServletException, IOException {
    if (exceeds(request.getParameter("username"), Account.MAX_EMAIL_LENGTH)
        || exceeds(request.getParameter("password"), PasswordCredential.MAX_PASSWORD_LENGTH)) {
      response.sendRedirect(request.getContextPath() + "/login?error");
      return;
    }
    chain.doFilter(request, response);
  }

  private static boolean exceeds(String value, int max) {
    return value != null && value.length() > max;
  }
}
