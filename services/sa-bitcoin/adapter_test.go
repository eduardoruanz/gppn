package main

import (
	"context"
	"testing"
)

func TestNewBitcoinAdapter(t *testing.T) {
	adapter := NewBitcoinAdapter("mainnet", "http://localhost:8332")
	if adapter == nil {
		t.Fatal("expected non-nil adapter")
	}
	if adapter.network != "mainnet" {
		t.Errorf("expected network mainnet, got %s", adapter.network)
	}
	if adapter.rpcEndpoint != "http://localhost:8332" {
		t.Errorf("expected rpcEndpoint http://localhost:8332, got %s", adapter.rpcEndpoint)
	}
}

func TestBitcoinAdapter_SupportedCurrencies(t *testing.T) {
	adapter := NewBitcoinAdapter("mainnet", "http://localhost:8332")
	currencies := adapter.SupportedCurrencies()

	if len(currencies) != 1 {
		t.Fatalf("expected 1 supported currency, got %d", len(currencies))
	}
	if currencies[0] != "BTC" {
		t.Errorf("expected BTC, got %s", currencies[0])
	}
}

func TestBitcoinAdapter_Initiate(t *testing.T) {
	adapter := NewBitcoinAdapter("mainnet", "http://localhost:8332")
	ctx := context.Background()

	req := SettlementRequest{
		PaymentID:   "pay-001",
		Amount:      "0.5",
		Currency:    "BTC",
		FromAddress: "bc1qSender",
		ToAddress:   "bc1qReceiver",
	}

	result, err := adapter.Initiate(ctx, req)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if result.TransactionID == "" {
		t.Error("expected non-empty transaction ID")
	}
	if result.Status != StatusPending {
		t.Errorf("expected status PENDING, got %s", result.Status)
	}
}

func TestBitcoinAdapter_Initiate_Validation(t *testing.T) {
	adapter := NewBitcoinAdapter("mainnet", "http://localhost:8332")
	ctx := context.Background()

	tests := []struct {
		name string
		req  SettlementRequest
	}{
		{
			name: "missing payment ID",
			req:  SettlementRequest{Amount: "1.0", ToAddress: "bc1qAddr"},
		},
		{
			name: "missing amount",
			req:  SettlementRequest{PaymentID: "pay-001", ToAddress: "bc1qAddr"},
		},
		{
			name: "missing to address",
			req:  SettlementRequest{PaymentID: "pay-001", Amount: "1.0"},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			_, err := adapter.Initiate(ctx, tt.req)
			if err == nil {
				t.Error("expected error, got nil")
			}
		})
	}
}

func TestBitcoinAdapter_Confirm(t *testing.T) {
	adapter := NewBitcoinAdapter("mainnet", "http://localhost:8332")
	ctx := context.Background()

	req := SettlementRequest{
		PaymentID:   "pay-002",
		Amount:      "1.0",
		Currency:    "BTC",
		FromAddress: "bc1qSender",
		ToAddress:   "bc1qReceiver",
	}
	result, err := adapter.Initiate(ctx, req)
	if err != nil {
		t.Fatalf("initiate failed: %v", err)
	}

	confirmed, err := adapter.Confirm(ctx, result.TransactionID)
	if err != nil {
		t.Fatalf("confirm failed: %v", err)
	}
	if confirmed.Status != StatusConfirmed {
		t.Errorf("expected status CONFIRMED, got %s", confirmed.Status)
	}
}

func TestBitcoinAdapter_Confirm_NotFound(t *testing.T) {
	adapter := NewBitcoinAdapter("mainnet", "http://localhost:8332")
	ctx := context.Background()

	_, err := adapter.Confirm(ctx, "nonexistent-tx")
	if err == nil {
		t.Error("expected error for nonexistent transaction")
	}
}

func TestBitcoinAdapter_Rollback(t *testing.T) {
	adapter := NewBitcoinAdapter("mainnet", "http://localhost:8332")
	ctx := context.Background()

	req := SettlementRequest{
		PaymentID:   "pay-003",
		Amount:      "0.1",
		Currency:    "BTC",
		FromAddress: "bc1qSender",
		ToAddress:   "bc1qReceiver",
	}
	result, err := adapter.Initiate(ctx, req)
	if err != nil {
		t.Fatalf("initiate failed: %v", err)
	}

	rolled, err := adapter.Rollback(ctx, result.TransactionID)
	if err != nil {
		t.Fatalf("rollback failed: %v", err)
	}
	if rolled.Status != StatusRolledBack {
		t.Errorf("expected status ROLLED_BACK, got %s", rolled.Status)
	}
}

func TestBitcoinAdapter_GetStatus(t *testing.T) {
	adapter := NewBitcoinAdapter("mainnet", "http://localhost:8332")
	ctx := context.Background()

	req := SettlementRequest{
		PaymentID:   "pay-004",
		Amount:      "0.25",
		Currency:    "BTC",
		FromAddress: "bc1qSender",
		ToAddress:   "bc1qReceiver",
	}
	result, err := adapter.Initiate(ctx, req)
	if err != nil {
		t.Fatalf("initiate failed: %v", err)
	}

	status, err := adapter.GetStatus(ctx, result.TransactionID)
	if err != nil {
		t.Fatalf("get status failed: %v", err)
	}
	if status.Status != StatusPending {
		t.Errorf("expected status PENDING, got %s", status.Status)
	}
}

func TestBitcoinAdapter_EstimateCost(t *testing.T) {
	adapter := NewBitcoinAdapter("mainnet", "http://localhost:8332")
	ctx := context.Background()

	req := SettlementRequest{
		PaymentID:   "pay-005",
		Amount:      "0.5",
		Currency:    "BTC",
		FromAddress: "bc1qSender",
		ToAddress:   "bc1qReceiver",
	}

	estimate, err := adapter.EstimateCost(ctx, req)
	if err != nil {
		t.Fatalf("estimate cost failed: %v", err)
	}
	if estimate.TotalFee == "" {
		t.Error("expected non-empty total fee")
	}
	if estimate.EstimatedTime == 0 {
		t.Error("expected non-zero estimated time")
	}
}

func TestBitcoinAdapter_EstimateCost_Validation(t *testing.T) {
	adapter := NewBitcoinAdapter("mainnet", "http://localhost:8332")
	ctx := context.Background()

	_, err := adapter.EstimateCost(ctx, SettlementRequest{})
	if err == nil {
		t.Error("expected error for empty amount")
	}
}
