// Package handlers implements the HTTP request handlers for the Veritas Verifier API.
package handlers

import (
	"encoding/json"
	"fmt"
	"net/http"
	"sync"
	"time"
)

// VerifyRequest represents a request to verify a credential presentation.
type VerifyRequest struct {
	Credential map[string]interface{} `json:"credential"`
}

// VerifyResponse is returned after verification.
type VerifyResponse struct {
	Valid  bool          `json:"valid"`
	Checks []VerifyCheck `json:"checks"`
}

// VerifyCheck is an individual verification check.
type VerifyCheck struct {
	Name   string  `json:"name"`
	Passed bool    `json:"passed"`
	Detail *string `json:"detail,omitempty"`
}

// ProofRequest represents a proof request to be sent to a holder.
type ProofRequest struct {
	ProofType    string                 `json:"proof_type"`
	Requirements map[string]interface{} `json:"requirements"`
}

// ProofRequestResponse is returned after creating a proof request.
type ProofRequestResponse struct {
	RequestID  string    `json:"request_id"`
	ProofType  string    `json:"proof_type"`
	Status     string    `json:"status"`
	CreatedAt  time.Time `json:"created_at"`
}

// VerifyProofRequest represents a proof to be verified.
type VerifyProofRequest struct {
	RequestID string                 `json:"request_id"`
	ProofData map[string]interface{} `json:"proof_data"`
}

// VerifierHandler handles verifier API endpoints.
type VerifierHandler struct {
	mu       sync.RWMutex
	requests map[string]*ProofRequestResponse
	counter  int
}

// NewVerifierHandler creates a new VerifierHandler.
func NewVerifierHandler() *VerifierHandler {
	return &VerifierHandler{
		requests: make(map[string]*ProofRequestResponse),
	}
}

// HandleVerify handles POST /api/v1/verify.
func (h *VerifierHandler) HandleVerify(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		writeError(w, http.StatusMethodNotAllowed, "method not allowed")
		return
	}

	var req VerifyRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, fmt.Sprintf("invalid request body: %v", err))
		return
	}

	if len(req.Credential) == 0 {
		writeError(w, http.StatusBadRequest, "credential is required")
		return
	}

	// Check for required credential fields.
	checks := []VerifyCheck{
		{Name: "has_issuer", Passed: req.Credential["issuer"] != nil},
		{Name: "has_subject", Passed: req.Credential["subject"] != nil},
		{Name: "has_claims", Passed: req.Credential["claims"] != nil},
		{Name: "has_proof", Passed: req.Credential["proof_signature"] != nil},
	}

	allPassed := true
	for _, c := range checks {
		if !c.Passed {
			allPassed = false
		}
	}

	resp := VerifyResponse{
		Valid:  allPassed,
		Checks: checks,
	}

	writeJSON(w, http.StatusOK, resp)
}

// HandleProofRequest handles POST /api/v1/proof-request.
func (h *VerifierHandler) HandleProofRequest(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		writeError(w, http.StatusMethodNotAllowed, "method not allowed")
		return
	}

	var req ProofRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, fmt.Sprintf("invalid request body: %v", err))
		return
	}

	if req.ProofType == "" {
		writeError(w, http.StatusBadRequest, "proof_type is required")
		return
	}

	now := time.Now().UTC()

	h.mu.Lock()
	h.counter++
	requestID := fmt.Sprintf("proof-req-%06d", h.counter)
	h.requests[requestID] = &ProofRequestResponse{
		RequestID: requestID,
		ProofType: req.ProofType,
		Status:    "PENDING",
		CreatedAt: now,
	}
	h.mu.Unlock()

	writeJSON(w, http.StatusCreated, ProofRequestResponse{
		RequestID: requestID,
		ProofType: req.ProofType,
		Status:    "PENDING",
		CreatedAt: now,
	})
}

// HandleVerifyProof handles POST /api/v1/verify-proof.
func (h *VerifierHandler) HandleVerifyProof(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		writeError(w, http.StatusMethodNotAllowed, "method not allowed")
		return
	}

	var req VerifyProofRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, fmt.Sprintf("invalid request body: %v", err))
		return
	}

	if len(req.ProofData) == 0 {
		writeError(w, http.StatusBadRequest, "proof_data is required")
		return
	}

	// Simplified proof verification — checks structure only.
	valid := req.ProofData["commitment"] != nil || req.ProofData["proof_json"] != nil

	writeJSON(w, http.StatusOK, map[string]interface{}{
		"request_id": req.RequestID,
		"valid":      valid,
		"status":     "VERIFIED",
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
