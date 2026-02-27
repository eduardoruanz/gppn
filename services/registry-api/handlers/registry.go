// Package handlers implements the HTTP request handlers for the Veritas Registry API.
package handlers

import (
	"encoding/json"
	"fmt"
	"net/http"
	"strings"
	"sync"
	"time"
)

// DidRecord represents a registered DID document in the registry.
type DidRecord struct {
	DID        string                 `json:"did"`
	Document   map[string]interface{} `json:"document"`
	RegisteredAt time.Time            `json:"registered_at"`
}

// SchemaRecord represents a registered credential schema.
type SchemaRecord struct {
	ID          string                 `json:"id"`
	Name        string                 `json:"name"`
	Version     string                 `json:"version"`
	Claims      []string               `json:"claims"`
	RegisteredAt time.Time             `json:"registered_at"`
}

// RegistryHandler handles registry API endpoints.
type RegistryHandler struct {
	mu      sync.RWMutex
	dids    map[string]*DidRecord
	schemas map[string]*SchemaRecord
}

// NewRegistryHandler creates a new RegistryHandler.
func NewRegistryHandler() *RegistryHandler {
	return &RegistryHandler{
		dids:    make(map[string]*DidRecord),
		schemas: make(map[string]*SchemaRecord),
	}
}

// HandleDids handles POST /api/v1/dids (register) and GET /api/v1/dids (list).
func (h *RegistryHandler) HandleDids(w http.ResponseWriter, r *http.Request) {
	switch r.Method {
	case http.MethodPost:
		h.registerDid(w, r)
	case http.MethodGet:
		h.listDids(w, r)
	default:
		writeError(w, http.StatusMethodNotAllowed, "method not allowed")
	}
}

// HandleDidByID handles GET /api/v1/dids/:did.
func (h *RegistryHandler) HandleDidByID(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		writeError(w, http.StatusMethodNotAllowed, "method not allowed")
		return
	}

	did := strings.TrimPrefix(r.URL.Path, "/api/v1/dids/")
	if did == "" {
		writeError(w, http.StatusBadRequest, "DID is required")
		return
	}

	h.mu.RLock()
	record, exists := h.dids[did]
	h.mu.RUnlock()

	if !exists {
		writeError(w, http.StatusNotFound, fmt.Sprintf("DID %s not found", did))
		return
	}

	writeJSON(w, http.StatusOK, record)
}

// HandleSchemas handles POST /api/v1/schemas (register) and GET /api/v1/schemas (list).
func (h *RegistryHandler) HandleSchemas(w http.ResponseWriter, r *http.Request) {
	switch r.Method {
	case http.MethodPost:
		h.registerSchema(w, r)
	case http.MethodGet:
		h.listSchemas(w, r)
	default:
		writeError(w, http.StatusMethodNotAllowed, "method not allowed")
	}
}

// HandleStats handles GET /api/v1/stats.
func (h *RegistryHandler) HandleStats(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		writeError(w, http.StatusMethodNotAllowed, "method not allowed")
		return
	}

	h.mu.RLock()
	didCount := len(h.dids)
	schemaCount := len(h.schemas)
	h.mu.RUnlock()

	writeJSON(w, http.StatusOK, map[string]interface{}{
		"total_dids":    didCount,
		"total_schemas": schemaCount,
		"status":        "operational",
	})
}

func (h *RegistryHandler) registerDid(w http.ResponseWriter, r *http.Request) {
	var req struct {
		DID      string                 `json:"did"`
		Document map[string]interface{} `json:"document"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, fmt.Sprintf("invalid request body: %v", err))
		return
	}
	if req.DID == "" {
		writeError(w, http.StatusBadRequest, "did is required")
		return
	}

	now := time.Now().UTC()
	record := &DidRecord{
		DID:          req.DID,
		Document:     req.Document,
		RegisteredAt: now,
	}

	h.mu.Lock()
	h.dids[req.DID] = record
	h.mu.Unlock()

	writeJSON(w, http.StatusCreated, record)
}

func (h *RegistryHandler) listDids(w http.ResponseWriter, r *http.Request) {
	h.mu.RLock()
	dids := make([]DidRecord, 0, len(h.dids))
	for _, d := range h.dids {
		dids = append(dids, *d)
	}
	h.mu.RUnlock()

	writeJSON(w, http.StatusOK, map[string]interface{}{
		"dids":  dids,
		"count": len(dids),
	})
}

func (h *RegistryHandler) registerSchema(w http.ResponseWriter, r *http.Request) {
	var req SchemaRecord
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, fmt.Sprintf("invalid request body: %v", err))
		return
	}
	if req.ID == "" {
		writeError(w, http.StatusBadRequest, "id is required")
		return
	}

	req.RegisteredAt = time.Now().UTC()

	h.mu.Lock()
	h.schemas[req.ID] = &req
	h.mu.Unlock()

	writeJSON(w, http.StatusCreated, req)
}

func (h *RegistryHandler) listSchemas(w http.ResponseWriter, r *http.Request) {
	h.mu.RLock()
	schemas := make([]SchemaRecord, 0, len(h.schemas))
	for _, s := range h.schemas {
		schemas = append(schemas, *s)
	}
	h.mu.RUnlock()

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
