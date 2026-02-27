// Package handlers implements the HTTP request handlers for the Veritas Issuer API.
package handlers

import (
	"encoding/json"
	"fmt"
	"net/http"
	"sync"
	"time"
)

// IssueRequest represents a request to issue a verifiable credential.
type IssueRequest struct {
	SubjectDID     string                 `json:"subject_did"`
	CredentialType []string               `json:"credential_type"`
	Claims         map[string]interface{} `json:"claims"`
	ExpiresIn      string                 `json:"expires_in,omitempty"`
}

// IssueResponse is returned after issuing a credential.
type IssueResponse struct {
	CredentialID string    `json:"credential_id"`
	Issuer       string    `json:"issuer"`
	Subject      string    `json:"subject"`
	Status       string    `json:"status"`
	IssuedAt     time.Time `json:"issued_at"`
}

// RevokeRequest represents a request to revoke a credential.
type RevokeRequest struct {
	CredentialID string `json:"credential_id"`
	Reason       string `json:"reason,omitempty"`
}

// CredentialRecord is stored for each issued credential.
type CredentialRecord struct {
	CredentialID   string                 `json:"credential_id"`
	SubjectDID     string                 `json:"subject_did"`
	CredentialType []string               `json:"credential_type"`
	Claims         map[string]interface{} `json:"claims"`
	Status         string                 `json:"status"`
	IssuedAt       time.Time              `json:"issued_at"`
	RevokedAt      *time.Time             `json:"revoked_at,omitempty"`
}

// IssuerHandler handles issuer API endpoints.
type IssuerHandler struct {
	mu          sync.RWMutex
	credentials map[string]*CredentialRecord
	counter     int
}

// NewIssuerHandler creates a new IssuerHandler.
func NewIssuerHandler() *IssuerHandler {
	return &IssuerHandler{
		credentials: make(map[string]*CredentialRecord),
	}
}

// HandleIssue handles POST /api/v1/issue.
func (h *IssuerHandler) HandleIssue(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		writeError(w, http.StatusMethodNotAllowed, "method not allowed")
		return
	}

	var req IssueRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, fmt.Sprintf("invalid request body: %v", err))
		return
	}

	if req.SubjectDID == "" {
		writeError(w, http.StatusBadRequest, "subject_did is required")
		return
	}
	if len(req.CredentialType) == 0 {
		writeError(w, http.StatusBadRequest, "credential_type is required")
		return
	}

	now := time.Now().UTC()

	h.mu.Lock()
	h.counter++
	credID := fmt.Sprintf("vc-%06d", h.counter)

	h.credentials[credID] = &CredentialRecord{
		CredentialID:   credID,
		SubjectDID:     req.SubjectDID,
		CredentialType: req.CredentialType,
		Claims:         req.Claims,
		Status:         "ACTIVE",
		IssuedAt:       now,
	}
	h.mu.Unlock()

	resp := IssueResponse{
		CredentialID: credID,
		Issuer:       "did:veritas:key:issuer-api",
		Subject:      req.SubjectDID,
		Status:       "ACTIVE",
		IssuedAt:     now,
	}

	writeJSON(w, http.StatusCreated, resp)
}

// HandleRevoke handles POST /api/v1/revoke.
func (h *IssuerHandler) HandleRevoke(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		writeError(w, http.StatusMethodNotAllowed, "method not allowed")
		return
	}

	var req RevokeRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, fmt.Sprintf("invalid request body: %v", err))
		return
	}

	if req.CredentialID == "" {
		writeError(w, http.StatusBadRequest, "credential_id is required")
		return
	}

	h.mu.Lock()
	cred, exists := h.credentials[req.CredentialID]
	if exists {
		now := time.Now().UTC()
		cred.Status = "REVOKED"
		cred.RevokedAt = &now
	}
	h.mu.Unlock()

	if !exists {
		writeError(w, http.StatusNotFound, fmt.Sprintf("credential %s not found", req.CredentialID))
		return
	}

	writeJSON(w, http.StatusOK, map[string]string{
		"credential_id": req.CredentialID,
		"status":        "REVOKED",
	})
}

// HandleListIssued handles GET /api/v1/issued.
func (h *IssuerHandler) HandleListIssued(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		writeError(w, http.StatusMethodNotAllowed, "method not allowed")
		return
	}

	h.mu.RLock()
	creds := make([]CredentialRecord, 0, len(h.credentials))
	for _, c := range h.credentials {
		creds = append(creds, *c)
	}
	h.mu.RUnlock()

	writeJSON(w, http.StatusOK, map[string]interface{}{
		"credentials": creds,
		"count":       len(creds),
	})
}

// HandleListSchemas handles GET /api/v1/schemas.
func (h *IssuerHandler) HandleListSchemas(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		writeError(w, http.StatusMethodNotAllowed, "method not allowed")
		return
	}

	schemas := []map[string]interface{}{
		{"id": "kyc-basic-v1", "name": "KYC Basic", "version": "1.0"},
		{"id": "age-verification-v1", "name": "Age Verification", "version": "1.0"},
		{"id": "residency-v1", "name": "Residency", "version": "1.0"},
		{"id": "humanity-proof-v1", "name": "Humanity Proof", "version": "1.0"},
	}

	writeJSON(w, http.StatusOK, map[string]interface{}{
		"schemas": schemas,
		"count":   len(schemas),
	})
}

func writeJSON(w http.ResponseWriter, status int, data interface{}) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	json.NewEncoder(w).Encode(data)
}

func writeError(w http.ResponseWriter, status int, message string) {
	resp := map[string]interface{}{
		"error":   http.StatusText(status),
		"code":    status,
		"message": message,
	}
	writeJSON(w, status, resp)
}
