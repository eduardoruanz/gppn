// Package models defines the data structures used by the GPPN Explorer API.
package models

import "time"

// PaymentRecord represents a payment transaction in the GPPN network.
type PaymentRecord struct {
	ID              string    `json:"id"`
	Status          string    `json:"status"`
	Amount          string    `json:"amount"`
	Currency        string    `json:"currency"`
	SourceNode      string    `json:"source_node"`
	DestinationNode string    `json:"destination_node"`
	SettlementLayer string    `json:"settlement_layer"`
	TransactionHash string    `json:"transaction_hash,omitempty"`
	CreatedAt       time.Time `json:"created_at"`
	UpdatedAt       time.Time `json:"updated_at"`
	Fee             string    `json:"fee,omitempty"`
	Metadata        map[string]string `json:"metadata,omitempty"`
}

// PaymentListResponse is the response for listing payments.
type PaymentListResponse struct {
	Payments   []PaymentRecord `json:"payments"`
	TotalCount int             `json:"total_count"`
	Page       int             `json:"page"`
	PageSize   int             `json:"page_size"`
}

// NetworkStats contains statistics about the GPPN network.
type NetworkStats struct {
	TotalNodes       int       `json:"total_nodes"`
	ActiveNodes      int       `json:"active_nodes"`
	TotalPayments    int64     `json:"total_payments"`
	TotalVolume      string    `json:"total_volume"`
	AvgSettlementMs  int64     `json:"avg_settlement_ms"`
	SupportedChains  []string  `json:"supported_chains"`
	UptimePercentage float64   `json:"uptime_percentage"`
	LastUpdated      time.Time `json:"last_updated"`
}

// PeerRecord represents a peer node in the GPPN network.
type PeerRecord struct {
	ID              string    `json:"id"`
	Address         string    `json:"address"`
	Port            int       `json:"port"`
	Status          string    `json:"status"`
	Version         string    `json:"version"`
	Latency         int       `json:"latency_ms"`
	ConnectedSince  time.Time `json:"connected_since"`
	SupportedChains []string  `json:"supported_chains"`
	Region          string    `json:"region"`
}

// PeerListResponse is the response for listing peers.
type PeerListResponse struct {
	Peers      []PeerRecord `json:"peers"`
	TotalCount int          `json:"total_count"`
}

// ErrorResponse represents an API error response.
type ErrorResponse struct {
	Error   string `json:"error"`
	Code    int    `json:"code"`
	Message string `json:"message"`
}
