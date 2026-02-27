package handlers

import (
	"net/http"
	"time"

	"github.com/gppn-protocol/gppn/services/explorer-api/models"
)

// NetworkHandler handles network-related API endpoints.
type NetworkHandler struct{}

// NewNetworkHandler creates a new NetworkHandler.
func NewNetworkHandler() *NetworkHandler {
	return &NetworkHandler{}
}

// HandleNetworkStats handles GET /api/v1/network/stats.
func (h *NetworkHandler) HandleNetworkStats(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		writeError(w, http.StatusMethodNotAllowed, "method not allowed")
		return
	}

	stats := models.NetworkStats{
		TotalNodes:       42,
		ActiveNodes:      38,
		TotalPayments:    125847,
		TotalVolume:      "2584391.50",
		AvgSettlementMs:  3200,
		SupportedChains:  []string{"ethereum", "bitcoin", "stablecoin"},
		UptimePercentage: 99.87,
		LastUpdated:      time.Now().UTC(),
	}

	writeJSON(w, http.StatusOK, stats)
}

// HandleListPeers handles GET /api/v1/network/peers.
func (h *NetworkHandler) HandleListPeers(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		writeError(w, http.StatusMethodNotAllowed, "method not allowed")
		return
	}

	now := time.Now().UTC()
	peers := []models.PeerRecord{
		{
			ID:              "peer-001",
			Address:         "203.0.113.10",
			Port:            9735,
			Status:          "ACTIVE",
			Version:         "0.1.0",
			Latency:         12,
			ConnectedSince:  now.Add(-72 * time.Hour),
			SupportedChains: []string{"ethereum", "stablecoin"},
			Region:          "us-east-1",
		},
		{
			ID:              "peer-002",
			Address:         "198.51.100.20",
			Port:            9735,
			Status:          "ACTIVE",
			Version:         "0.1.0",
			Latency:         45,
			ConnectedSince:  now.Add(-48 * time.Hour),
			SupportedChains: []string{"bitcoin", "ethereum"},
			Region:          "eu-west-1",
		},
		{
			ID:              "peer-003",
			Address:         "192.0.2.30",
			Port:            9735,
			Status:          "INACTIVE",
			Version:         "0.0.9",
			Latency:         200,
			ConnectedSince:  now.Add(-24 * time.Hour),
			SupportedChains: []string{"ethereum"},
			Region:          "ap-southeast-1",
		},
	}

	resp := models.PeerListResponse{
		Peers:      peers,
		TotalCount: len(peers),
	}

	writeJSON(w, http.StatusOK, resp)
}
