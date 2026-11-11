package com.crazyms.auth.shared.web;

import com.crazyms.auth.account.service.EmailTakenException;
import com.crazyms.auth.account.service.NotAdminException;
import com.crazyms.auth.account.service.OwnAdminRoleException;
import com.crazyms.auth.account.service.UnknownRoleException;
import org.springframework.http.HttpStatus;
import org.springframework.http.ProblemDetail;
import org.springframework.web.bind.annotation.ExceptionHandler;
import org.springframework.web.bind.annotation.RestControllerAdvice;
import org.springframework.web.servlet.mvc.method.annotation.ResponseEntityExceptionHandler;

/** Errors leave as RFC 9457 problem+json; the base class covers validation and status. */
@RestControllerAdvice
class ApiErrors extends ResponseEntityExceptionHandler {

  @ExceptionHandler({EmailTakenException.class, OwnAdminRoleException.class})
  ProblemDetail conflict(RuntimeException e) {
    return ProblemDetail.forStatusAndDetail(HttpStatus.CONFLICT, e.getMessage());
  }

  @ExceptionHandler(NotAdminException.class)
  ProblemDetail notAdmin(NotAdminException e) {
    return ProblemDetail.forStatusAndDetail(HttpStatus.FORBIDDEN, e.getMessage());
  }

  @ExceptionHandler(UnknownRoleException.class)
  ProblemDetail unknownRole(UnknownRoleException e) {
    return ProblemDetail.forStatusAndDetail(HttpStatus.UNPROCESSABLE_ENTITY, e.getMessage());
  }
}
