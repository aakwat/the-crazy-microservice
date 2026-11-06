package problem

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestWriteSendsAnRFC9457Body(t *testing.T) {
	rec := httptest.NewRecorder()

	Write(rec, http.StatusTooManyRequests, "try again later")

	if rec.Code != http.StatusTooManyRequests || rec.Header().Get("Content-Type") != "application/problem+json" {
		t.Fatalf("got %d %q", rec.Code, rec.Header().Get("Content-Type"))
	}
	var got Details
	if err := json.Unmarshal(rec.Body.Bytes(), &got); err != nil {
		t.Fatal(err)
	}
	want := Details{Type: "about:blank", Title: "Too Many Requests", Status: 429, Detail: "try again later"}
	if got != want {
		t.Fatalf("got %+v, want %+v", got, want)
	}
}
