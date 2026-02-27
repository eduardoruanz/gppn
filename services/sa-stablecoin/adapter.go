// Package main implements the Stablecoin settlement adapter for the GPPN protocol.
// It supports USDC and USDT stablecoins.
package main

import (
	"context"
	"fmt"
	"sync"
	"time"
)

// SettlementStatus represents the current state of a settlement transaction.
type SettlementStatus string

const (
	StatusPending    SettlementStatus = "PENDING"
	StatusConfirmed  SettlementStatus = "CONFIRMED"
	StatusFailed     SettlementStatus = "FAILED"
	StatusRolledBack SettlementStatus = "ROLLED_BACK"
)

// Supported stablecoin tokens.
const (
	CurrencyUSDC = "USDC"
	CurrencyUSDT = "USDT"
)

// SettlementRequest contains the parameters for initiating a settlement.
type SettlementRequest struct {
	PaymentID   string
	Amount      string // Decimal string representation (in token units)
	Currency    string // Must be USDC or USDT
	FromAddress string
	ToAddress   string
	ChainID     int // The blockchain to settle on (e.g., 1 for Ethereum mainnet)
}

// SettlementResult contains the result of a settlement operation.
type SettlementResult struct {
	TransactionID string
	Status        SettlementStatus
	Timestamp     time.Time
	Fee           string
	Currency      string
	Message       string
}

// CostEstimate contains the estimated cost for a settlement.
type CostEstimate struct {
	GasFee        string
	NetworkFee    string
	TotalFee      string
	EstimatedTime time.Duration
}

// SettlementAdapter defines the interface that all settlement adapters must implement.
type SettlementAdapter interface {
	Initiate(ctx context.Context, req SettlementRequest) (*SettlementResult, error)
	Confirm(ctx context.Context, transactionID string) (*SettlementResult, error)
	Rollback(ctx context.Context, transactionID string) (*SettlementResult, error)
	GetStatus(ctx context.Context, transactionID string) (*SettlementResult, error)
	EstimateCost(ctx context.Context, req SettlementRequest) (*CostEstimate, error)
	SupportedCurrencies() []string
}

// StablecoinAdapter implements the SettlementAdapter interface for stablecoin settlements.
// It supports USDC and USDT on EVM-compatible chains.
type StablecoinAdapter struct {
	mu           sync.RWMutex
	transactions map[string]*SettlementResult
	rpcEndpoint  string
	// tokenContracts maps currency to contract address (stub addresses).
	tokenContracts map[string]string
}

// NewStablecoinAdapter creates a new StablecoinAdapter instance.
func NewStablecoinAdapter(rpcEndpoint string) *StablecoinAdapter {
	return &StablecoinAdapter{
		transactions: make(map[string]*SettlementResult),
		rpcEndpoint:  rpcEndpoint,
		tokenContracts: map[string]string{
			CurrencyUSDC: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC on Ethereum mainnet
			CurrencyUSDT: "0xdAC17F958D2ee523a2206206994597C13D831ec7", // USDT on Ethereum mainnet
		},
	}
}

// isSupportedCurrency checks whether the given currency is supported.
func (a *StablecoinAdapter) isSupportedCurrency(currency string) bool {
	_, ok := a.tokenContracts[currency]
	return ok
}

// Initiate starts a new stablecoin settlement transaction.
// This is a stub implementation that simulates an ERC-20 token transfer.
func (a *StablecoinAdapter) Initiate(ctx context.Context, req SettlementRequest) (*SettlementResult, error) {
	if req.PaymentID == "" {
		return nil, fmt.Errorf("stablecoin: payment ID is required")
	}
	if req.Amount == "" {
		return nil, fmt.Errorf("stablecoin: amount is required")
	}
	if req.Currency == "" {
		return nil, fmt.Errorf("stablecoin: currency is required")
	}
	if !a.isSupportedCurrency(req.Currency) {
		return nil, fmt.Errorf("stablecoin: unsupported currency %q (supported: USDC, USDT)", req.Currency)
	}
	if req.ToAddress == "" {
		return nil, fmt.Errorf("stablecoin: destination address is required")
	}

	// Simulate generating a transaction hash.
	txID := fmt.Sprintf("0xsc_%s_%s_%d", req.Currency, req.PaymentID, time.Now().UnixNano())

	result := &SettlementResult{
		TransactionID: txID,
		Status:        StatusPending,
		Timestamp:     time.Now().UTC(),
		Fee:           "0.003",
		Currency:      req.Currency,
		Message:       fmt.Sprintf("%s transfer submitted to network", req.Currency),
	}

	a.mu.Lock()
	a.transactions[txID] = result
	a.mu.Unlock()

	return result, nil
}

// Confirm confirms a pending stablecoin settlement transaction.
func (a *StablecoinAdapter) Confirm(ctx context.Context, transactionID string) (*SettlementResult, error) {
	a.mu.Lock()
	defer a.mu.Unlock()

	result, exists := a.transactions[transactionID]
	if !exists {
		return nil, fmt.Errorf("stablecoin: transaction %s not found", transactionID)
	}

	if result.Status != StatusPending {
		return nil, fmt.Errorf("stablecoin: transaction %s is not in pending state (current: %s)", transactionID, result.Status)
	}

	result.Status = StatusConfirmed
	result.Timestamp = time.Now().UTC()
	result.Message = fmt.Sprintf("%s transfer confirmed on chain", result.Currency)

	return result, nil
}

// Rollback rolls back a pending stablecoin settlement transaction.
func (a *StablecoinAdapter) Rollback(ctx context.Context, transactionID string) (*SettlementResult, error) {
	a.mu.Lock()
	defer a.mu.Unlock()

	result, exists := a.transactions[transactionID]
	if !exists {
		return nil, fmt.Errorf("stablecoin: transaction %s not found", transactionID)
	}

	if result.Status != StatusPending {
		return nil, fmt.Errorf("stablecoin: transaction %s is not in pending state (current: %s)", transactionID, result.Status)
	}

	result.Status = StatusRolledBack
	result.Timestamp = time.Now().UTC()
	result.Message = fmt.Sprintf("%s transfer rolled back", result.Currency)

	return result, nil
}

// GetStatus retrieves the current status of a stablecoin settlement transaction.
func (a *StablecoinAdapter) GetStatus(ctx context.Context, transactionID string) (*SettlementResult, error) {
	a.mu.RLock()
	defer a.mu.RUnlock()

	result, exists := a.transactions[transactionID]
	if !exists {
		return nil, fmt.Errorf("stablecoin: transaction %s not found", transactionID)
	}

	// Return a copy to avoid data races.
	copy := *result
	return &copy, nil
}

// EstimateCost provides a cost estimate for a stablecoin settlement.
// Stablecoin transfers typically have lower fees since the value is stable.
func (a *StablecoinAdapter) EstimateCost(ctx context.Context, req SettlementRequest) (*CostEstimate, error) {
	if req.Amount == "" {
		return nil, fmt.Errorf("stablecoin: amount is required for cost estimation")
	}
	if req.Currency != "" && !a.isSupportedCurrency(req.Currency) {
		return nil, fmt.Errorf("stablecoin: unsupported currency %q", req.Currency)
	}

	// Stub: return fixed estimated costs for ERC-20 transfers.
	return &CostEstimate{
		GasFee:        "0.002",
		NetworkFee:    "0.001",
		TotalFee:      "0.003",
		EstimatedTime: 15 * time.Second,
	}, nil
}

// SupportedCurrencies returns the list of currencies supported by this adapter.
func (a *StablecoinAdapter) SupportedCurrencies() []string {
	return []string{CurrencyUSDC, CurrencyUSDT}
}

// Compile-time check that StablecoinAdapter implements SettlementAdapter.
var _ SettlementAdapter = (*StablecoinAdapter)(nil)
