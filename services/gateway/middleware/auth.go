// Package middleware provides HTTP middleware for the GPPN Gateway service.
package middleware

import (
	"log"
	"net/http"
	"strings"
)

const (
	// APIKeyHeader is the HTTP header used for API key authentication.
	APIKeyHeader = "X-API-Key"

	// BearerPrefix is the prefix for Bearer token authentication.
	BearerPrefix = "Bearer "

	// stubAPIKey is a placeholder API key for development/testing.
	stubAPIKey = "gppn-dev-api-key-placeholder"
)

// AuthMiddleware provides API key authentication for protected endpoints.
type AuthMiddleware struct {
	// validKeys holds the set of valid API keys.
	// In production, this would be backed by a database or key management service.
	validKeys map[string]bool
}

// NewAuthMiddleware creates a new AuthMiddleware with default stub keys.
func NewAuthMiddleware() *AuthMiddleware {
	return &AuthMiddleware{
		validKeys: map[string]bool{
			stubAPIKey: true,
		},
	}
}

// NewAuthMiddlewareWithKeys creates a new AuthMiddleware with the given valid keys.
func NewAuthMiddlewareWithKeys(keys []string) *AuthMiddleware {
	m := &AuthMiddleware{
		validKeys: make(map[string]bool, len(keys)),
	}
	for _, k := range keys {
		m.validKeys[k] = true
	}
	return m
}

// Authenticate wraps an http.Handler with API key authentication.
// It checks for the API key in the X-API-Key header or as a Bearer token
// in the Authorization header.
func (m *AuthMiddleware) Authenticate(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		apiKey := r.Header.Get(APIKeyHeader)

		// Also check Authorization header for Bearer token.
		if apiKey == "" {
			authHeader := r.Header.Get("Authorization")
			if strings.HasPrefix(authHeader, BearerPrefix) {
				apiKey = strings.TrimPrefix(authHeader, BearerPrefix)
			}
		}

		if apiKey == "" {
			log.Printf("Auth: request rejected - no API key provided from %s", r.RemoteAddr)
			http.Error(w, `{"error":"unauthorized","code":401,"message":"API key is required. Provide it via X-API-Key header or Authorization: Bearer <key>"}`, http.StatusUnauthorized)
			return
		}

		if !m.validKeys[apiKey] {
			log.Printf("Auth: request rejected - invalid API key from %s", r.RemoteAddr)
			http.Error(w, `{"error":"forbidden","code":403,"message":"invalid API key"}`, http.StatusForbidden)
			return
		}

		// API key is valid, proceed to the next handler.
		next.ServeHTTP(w, r)
	})
}

// AuthenticateFunc wraps an http.HandlerFunc with API key authentication.
func (m *AuthMiddleware) AuthenticateFunc(next http.HandlerFunc) http.Handler {
	return m.Authenticate(http.HandlerFunc(next))
}
