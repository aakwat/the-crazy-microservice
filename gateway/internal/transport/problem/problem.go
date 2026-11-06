// Package problem writes RFC 9457 problem details, the error body every service in this project returns.
package problem

import (
	"encoding/json"
	"net/http"
)

// Details is the RFC 9457 body; type stays about:blank, so title is the HTTP status text (section 4.2.1).
type Details struct {
	Type   string `json:"type"`
	Title  string `json:"title"`
	Status int    `json:"status"`
	Detail string `json:"detail,omitempty"`
}

// Write sends status with a problem body; detail must be safe to show a client, never an internal error.
func Write(w http.ResponseWriter, status int, detail string) {
	w.Header().Set("Content-Type", "application/problem+json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(Details{
		Type:   "about:blank",
		Title:  http.StatusText(status),
		Status: status,
		Detail: detail,
	})
}
