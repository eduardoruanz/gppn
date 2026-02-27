// Package handlers implements the HTTP request handlers for the Veritas Gateway service.
package handlers

import (
	"encoding/json"
	"fmt"
	"net/http"
	"strings"
	"sync"
	"time"
)

// IssueCredentialRequest represents a request to issue a credential.
type IssueCredentialRequest struct {
	SubjectDID     string                 `json:"subject_did"`
	CredentialType []string               `json:"credential_type"`
	Claims         map[string]interface{} `json:"claims"`
}

// IssueCredentialResponse is returned after issuing a credential.
type IssueCredentialResponse struct {
	CredentialID string    `json:"credential_id"`
	Issuer       string    `json:"issuer"`
	Subject      string    `json:"subject"`
	Status       string    `json:"status"`
	CreatedAt    time.Time `json:"created_at"`
}

// VerifyCredentialRequest represents a request to verify a credential.
type VerifyCredentialRequest struct {
	Credential map[string]interface{} `json:"credential"`
}

// GenerateProofRequest represents a request to generate a ZK proof.
type GenerateProofRequest struct {
	ProofType string                 `json:"proof_type"`
	Params    map[string]interface{} `json:"params"`
}

// ErrorResponse represents an API error response.
type ErrorResponse struct {
	Error   string `json:"error"`
	Code    int    `json:"code"`
	Message string `json:"message"`
}

// GatewayHandler handles gateway-related API endpoints.
type GatewayHandler struct {
	mu          sync.RWMutex
	credentials map[string]*IssueCredentialResponse
	counter     int
}

// NewGatewayHandler creates a new GatewayHandler.
func NewGatewayHandler() *GatewayHandler {
	return &GatewayHandler{
		credentials: make(map[string]*IssueCredentialResponse),
	}
}

// HandleIssueCredential handles POST /api/v1/credentials/issue.
func (h *GatewayHandler) HandleIssueCredential(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		writeError(w, http.StatusMethodNotAllowed, "method not allowed")
		return
	}

	var req IssueCredentialRequest
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
	credID := fmt.Sprintf("gw-vc-%06d", h.counter)
	h.credentials[credID] = &IssueCredentialResponse{
		CredentialID: credID,
		Issuer:       "did:veritas:key:gateway",
		Subject:      req.SubjectDID,
		Status:       "issued",
		CreatedAt:    now,
	}
	h.mu.Unlock()

	resp := IssueCredentialResponse{
		CredentialID: credID,
		Issuer:       "did:veritas:key:gateway",
		Subject:      req.SubjectDID,
		Status:       "issued",
		CreatedAt:    now,
	}

	writeJSON(w, http.StatusCreated, resp)
}

// HandleVerifyCredential handles POST /api/v1/credentials/verify.
func (h *GatewayHandler) HandleVerifyCredential(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		writeError(w, http.StatusMethodNotAllowed, "method not allowed")
		return
	}

	var req VerifyCredentialRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, fmt.Sprintf("invalid request body: %v", err))
		return
	}

	valid := req.Credential["issuer"] != nil && req.Credential["subject"] != nil

	writeJSON(w, http.StatusOK, map[string]interface{}{
		"valid": valid,
		"checks": []map[string]interface{}{
			{"name": "has_issuer", "passed": req.Credential["issuer"] != nil},
			{"name": "has_subject", "passed": req.Credential["subject"] != nil},
		},
	})
}

// HandleGenerateProof handles POST /api/v1/proofs/generate.
func (h *GatewayHandler) HandleGenerateProof(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		writeError(w, http.StatusMethodNotAllowed, "method not allowed")
		return
	}

	var req GenerateProofRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, fmt.Sprintf("invalid request body: %v", err))
		return
	}

	if req.ProofType == "" {
		writeError(w, http.StatusBadRequest, "proof_type is required")
		return
	}

	validTypes := map[string]bool{"age": true, "residency": true, "kyc_level": true}
	if !validTypes[req.ProofType] {
		writeError(w, http.StatusBadRequest, fmt.Sprintf("unsupported proof_type: %s", req.ProofType))
		return
	}

	writeJSON(w, http.StatusOK, map[string]interface{}{
		"proof_type": req.ProofType,
		"status":     "generated",
		"message":    "proof generation delegated to Veritas node",
	})
}

// HandleResolve handles GET /api/v1/identity/:did.
func (h *GatewayHandler) HandleResolve(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		writeError(w, http.StatusMethodNotAllowed, "method not allowed")
		return
	}

	did := strings.TrimPrefix(r.URL.Path, "/api/v1/identity/")
	if did == "" {
		writeError(w, http.StatusBadRequest, "DID is required")
		return
	}

	writeJSON(w, http.StatusOK, map[string]interface{}{
		"did":    did,
		"status": "resolution delegated to Veritas node",
	})
}

func writeJSON(w http.ResponseWriter, status int, data interface{}) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	json.NewEncoder(w).Encode(data)
}

func writeError(w http.ResponseWriter, status int, message string) {
	resp := ErrorResponse{
		Error:   http.StatusText(status),
		Code:    status,
		Message: message,
	}
	writeJSON(w, status, resp)
}
