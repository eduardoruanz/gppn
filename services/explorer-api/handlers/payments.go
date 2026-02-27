// Package handlers implements the HTTP request handlers for the GPPN Explorer API.
package handlers

import (
	"encoding/json"
	"net/http"
	"strings"
	"time"

	"github.com/gppn-protocol/gppn/services/explorer-api/models"
)

// PaymentHandler handles payment-related API endpoints.
type PaymentHandler struct{}

// NewPaymentHandler creates a new PaymentHandler.
func NewPaymentHandler() *PaymentHandler {
	return &PaymentHandler{}
}

// stubPayments returns sample payment records for demonstration.
func stubPayments() []models.PaymentRecord {
	now := time.Now().UTC()
	return []models.PaymentRecord{
		{
			ID:              "pay-001",
			Status:          "CONFIRMED",
			Amount:          "1.5",
			Currency:        "ETH",
			SourceNode:      "node-us-east-1",
			DestinationNode: "node-eu-west-1",
			SettlementLayer: "ethereum",
			TransactionHash: "0xabc123def456",
			CreatedAt:       now.Add(-2 * time.Hour),
			UpdatedAt:       now.Add(-1 * time.Hour),
			Fee:             "0.002",
		},
		{
			ID:              "pay-002",
			Status:          "PENDING",
			Amount:          "500.00",
			Currency:        "USDC",
			SourceNode:      "node-ap-southeast-1",
			DestinationNode: "node-us-west-2",
			SettlementLayer: "stablecoin",
			CreatedAt:       now.Add(-30 * time.Minute),
			UpdatedAt:       now.Add(-30 * time.Minute),
			Fee:             "0.003",
		},
		{
			ID:              "pay-003",
			Status:          "CONFIRMED",
			Amount:          "0.05",
			Currency:        "BTC",
			SourceNode:      "node-eu-central-1",
			DestinationNode: "node-us-east-1",
			SettlementLayer: "bitcoin",
			TransactionHash: "btc_tx_789xyz",
			CreatedAt:       now.Add(-24 * time.Hour),
			UpdatedAt:       now.Add(-23 * time.Hour),
			Fee:             "0.00005",
		},
	}
}

// HandleListPayments handles GET /api/v1/payments.
func (h *PaymentHandler) HandleListPayments(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		writeError(w, http.StatusMethodNotAllowed, "method not allowed")
		return
	}

	payments := stubPayments()

	resp := models.PaymentListResponse{
		Payments:   payments,
		TotalCount: len(payments),
		Page:       1,
		PageSize:   20,
	}

	writeJSON(w, http.StatusOK, resp)
}

// HandleGetPayment handles GET /api/v1/payments/:id.
func (h *PaymentHandler) HandleGetPayment(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		writeError(w, http.StatusMethodNotAllowed, "method not allowed")
		return
	}

	// Extract the payment ID from the URL path.
	// Expected path: /api/v1/payments/{id}
	parts := strings.Split(strings.TrimPrefix(r.URL.Path, "/api/v1/payments/"), "/")
	paymentID := parts[0]

	if paymentID == "" {
		writeError(w, http.StatusBadRequest, "payment ID is required")
		return
	}

	// Search stub payments for the requested ID.
	for _, p := range stubPayments() {
		if p.ID == paymentID {
			writeJSON(w, http.StatusOK, p)
			return
		}
	}

	writeError(w, http.StatusNotFound, "payment not found")
}

// writeJSON writes a JSON response with the given status code.
func writeJSON(w http.ResponseWriter, status int, data interface{}) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	json.NewEncoder(w).Encode(data)
}

// writeError writes a JSON error response.
func writeError(w http.ResponseWriter, status int, message string) {
	resp := models.ErrorResponse{
		Error:   http.StatusText(status),
		Code:    status,
		Message: message,
	}
	writeJSON(w, status, resp)
}
