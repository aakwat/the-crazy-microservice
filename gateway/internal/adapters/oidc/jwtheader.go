package oidc

import (
	"encoding/base64"
	"encoding/json"
	"errors"
	"strings"
)

// headerType reads a JWT's typ header without verifying it; callers verify the signature separately.
func headerType(raw string) (string, error) {
	encoded, _, ok := strings.Cut(raw, ".")
	if !ok {
		return "", errors.New("not a JWT")
	}
	decoded, err := base64.RawURLEncoding.DecodeString(encoded)
	if err != nil {
		return "", err
	}
	var header struct {
		Typ string `json:"typ"`
	}
	if err := json.Unmarshal(decoded, &header); err != nil {
		return "", err
	}
	return header.Typ, nil
}
