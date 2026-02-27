// Package handlers implements the HTTP request handlers for the GPPN Gateway service.
package handlers

import (
	"encoding/json"
	"fmt"
	"net/http"
	"strings"
	"sync"
	"time"
)

// SendPaymentRequest represents a request to send a payment through the GPPN network.
type SendPaymentRequest struct {
	Amount          string `json:"amount"`
	Currency        string `json:"currency"`
	DestinationNode string `json:"destination_node"`
	SettlementLayer string `json:"settlement_layer"`
	Memo            string `json:"memo,omitempty"`
}

// SendPaymentResponse represents the response after initiating a payment.
type SendPaymentResponse struct {
	PaymentID   string    `json:"payment_id"`
	Status      string    `json:"status"`
	CreatedAt   time.Time `json:"created_at"`
	Message     string    `json:"message"`
}

// PaymentStatusResponse represents the status of a payment.
type PaymentStatusResponse struct {
	PaymentID       string    `json:"payment_id"`
	Status          string    `json:"status"`
	Amount          string    `json:"amount"`
	Currency        string    `json:"currency"`
	DestinationNode string    `json:"destination_node"`
	SettlementLayer string    `json:"settlement_layer"`
	CreatedAt       time.Time `json:"created_at"`
	UpdatedAt       time.Time `json:"updated_at"`
	TransactionHash string    `json:"transaction_hash,omitempty"`
}

// ErrorResponse represents an API error response.
type ErrorResponse struct {
	Error   string `json:"error"`
	Code    int    `json:"code"`
	Message string `json:"message"`
}

// GatewayHandler handles gateway-related API endpoints.
type GatewayHandler struct {
	mu       sync.RWMutex
	payments map[string]*PaymentStatusResponse
	counter  int
}

// NewGatewayHandler creates a new GatewayHandler.
func NewGatewayHandler() *GatewayHandler {
	return &GatewayHandler{
		payments: make(map[string]*PaymentStatusResponse),
	}
}

// HandleSendPayment handles POST /api/v1/send.
func (h *GatewayHandler) HandleSendPayment(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		writeError(w, http.StatusMethodNotAllowed, "method not allowed")
		return
	}

	var req SendPaymentRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, fmt.Sprintf("invalid request body: %v", err))
		return
	}

	// Validate required fields.
	if req.Amount == "" {
		writeError(w, http.StatusBadRequest, "amount is required")
		return
	}
	if req.Currency == "" {
		writeError(w, http.StatusBadRequest, "currency is required")
		return
	}
	if req.DestinationNode == "" {
		writeError(w, http.StatusBadRequest, "destination_node is required")
		return
	}
	if req.SettlementLayer == "" {
		writeError(w, http.StatusBadRequest, "settlement_layer is required")
		return
	}

	// Validate settlement layer.
	validLayers := map[string]bool{
		"ethereum":   true,
		"bitcoin":    true,
		"stablecoin": true,
	}
	if !validLayers[req.SettlementLayer] {
		writeError(w, http.StatusBadRequest, fmt.Sprintf("unsupported settlement layer: %s (valid: ethereum, bitcoin, stablecoin)", req.SettlementLayer))
		return
	}

	now := time.Now().UTC()

	h.mu.Lock()
	h.counter++
	paymentID := fmt.Sprintf("gppn-pay-%06d", h.counter)

	// Store the payment for status lookups.
	h.payments[paymentID] = &PaymentStatusResponse{
		PaymentID:       paymentID,
		Status:          "PENDING",
		Amount:          req.Amount,
		Currency:        req.Currency,
		DestinationNode: req.DestinationNode,
		SettlementLayer: req.SettlementLayer,
		CreatedAt:       now,
		UpdatedAt:       now,
	}
	h.mu.Unlock()

	resp := SendPaymentResponse{
		PaymentID: paymentID,
		Status:    "PENDING",
		CreatedAt: now,
		Message:   fmt.Sprintf("Payment initiated via %s settlement layer", req.SettlementLayer),
	}

	writeJSON(w, http.StatusCreated, resp)
}

// HandleGetStatus handles GET /api/v1/status/:id.
func (h *GatewayHandler) HandleGetStatus(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		writeError(w, http.StatusMethodNotAllowed, "method not allowed")
		return
	}

	// Extract the payment ID from the URL path.
	// Expected path: /api/v1/status/{id}
	parts := strings.Split(strings.TrimPrefix(r.URL.Path, "/api/v1/status/"), "/")
	paymentID := parts[0]

	if paymentID == "" {
		writeError(w, http.StatusBadRequest, "payment ID is required")
		return
	}

	h.mu.RLock()
	payment, exists := h.payments[paymentID]
	h.mu.RUnlock()

	if !exists {
		writeError(w, http.StatusNotFound, fmt.Sprintf("payment %s not found", paymentID))
		return
	}

	// Return a copy.
	resp := *payment
	writeJSON(w, http.StatusOK, resp)
}

// writeJSON writes a JSON response with the given status code.
func writeJSON(w http.ResponseWriter, status int, data interface{}) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	json.NewEncoder(w).Encode(data)
}

// writeError writes a JSON error response.
func writeError(w http.ResponseWriter, status int, message string) {
	resp := ErrorResponse{
		Error:   http.StatusText(status),
		Code:    status,
		Message: message,
	}
	writeJSON(w, status, resp)
}
