package main

import (
	"context"
	"testing"
)

func TestNewEthereumAdapter(t *testing.T) {
	adapter := NewEthereumAdapter(1, "http://localhost:8545")
	if adapter == nil {
		t.Fatal("expected non-nil adapter")
	}
	if adapter.chainID != 1 {
		t.Errorf("expected chainID 1, got %d", adapter.chainID)
	}
	if adapter.rpcEndpoint != "http://localhost:8545" {
		t.Errorf("expected rpcEndpoint http://localhost:8545, got %s", adapter.rpcEndpoint)
	}
}

func TestEthereumAdapter_SupportedCurrencies(t *testing.T) {
	adapter := NewEthereumAdapter(1, "http://localhost:8545")
	currencies := adapter.SupportedCurrencies()

	if len(currencies) != 2 {
		t.Fatalf("expected 2 supported currencies, got %d", len(currencies))
	}

	expected := map[string]bool{"ETH": true, "WETH": true}
	for _, c := range currencies {
		if !expected[c] {
			t.Errorf("unexpected currency: %s", c)
		}
	}
}

func TestEthereumAdapter_Initiate(t *testing.T) {
	adapter := NewEthereumAdapter(1, "http://localhost:8545")
	ctx := context.Background()

	req := SettlementRequest{
		PaymentID:   "pay-001",
		Amount:      "1.5",
		Currency:    "ETH",
		FromAddress: "0xSender",
		ToAddress:   "0xReceiver",
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

func TestEthereumAdapter_Initiate_Validation(t *testing.T) {
	adapter := NewEthereumAdapter(1, "http://localhost:8545")
	ctx := context.Background()

	tests := []struct {
		name string
		req  SettlementRequest
	}{
		{
			name: "missing payment ID",
			req:  SettlementRequest{Amount: "1.0", ToAddress: "0xAddr"},
		},
		{
			name: "missing amount",
			req:  SettlementRequest{PaymentID: "pay-001", ToAddress: "0xAddr"},
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

func TestEthereumAdapter_Confirm(t *testing.T) {
	adapter := NewEthereumAdapter(1, "http://localhost:8545")
	ctx := context.Background()

	// First initiate a transaction.
	req := SettlementRequest{
		PaymentID:   "pay-002",
		Amount:      "2.0",
		Currency:    "ETH",
		FromAddress: "0xSender",
		ToAddress:   "0xReceiver",
	}
	result, err := adapter.Initiate(ctx, req)
	if err != nil {
		t.Fatalf("initiate failed: %v", err)
	}

	// Confirm the transaction.
	confirmed, err := adapter.Confirm(ctx, result.TransactionID)
	if err != nil {
		t.Fatalf("confirm failed: %v", err)
	}
	if confirmed.Status != StatusConfirmed {
		t.Errorf("expected status CONFIRMED, got %s", confirmed.Status)
	}
}

func TestEthereumAdapter_Confirm_NotFound(t *testing.T) {
	adapter := NewEthereumAdapter(1, "http://localhost:8545")
	ctx := context.Background()

	_, err := adapter.Confirm(ctx, "nonexistent-tx")
	if err == nil {
		t.Error("expected error for nonexistent transaction")
	}
}

func TestEthereumAdapter_Rollback(t *testing.T) {
	adapter := NewEthereumAdapter(1, "http://localhost:8545")
	ctx := context.Background()

	req := SettlementRequest{
		PaymentID:   "pay-003",
		Amount:      "3.0",
		Currency:    "ETH",
		FromAddress: "0xSender",
		ToAddress:   "0xReceiver",
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

func TestEthereumAdapter_GetStatus(t *testing.T) {
	adapter := NewEthereumAdapter(1, "http://localhost:8545")
	ctx := context.Background()

	req := SettlementRequest{
		PaymentID:   "pay-004",
		Amount:      "4.0",
		Currency:    "ETH",
		FromAddress: "0xSender",
		ToAddress:   "0xReceiver",
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

func TestEthereumAdapter_EstimateCost(t *testing.T) {
	adapter := NewEthereumAdapter(1, "http://localhost:8545")
	ctx := context.Background()

	req := SettlementRequest{
		PaymentID:   "pay-005",
		Amount:      "5.0",
		Currency:    "ETH",
		FromAddress: "0xSender",
		ToAddress:   "0xReceiver",
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

func TestEthereumAdapter_EstimateCost_Validation(t *testing.T) {
	adapter := NewEthereumAdapter(1, "http://localhost:8545")
	ctx := context.Background()

	_, err := adapter.EstimateCost(ctx, SettlementRequest{})
	if err == nil {
		t.Error("expected error for empty amount")
	}
}
