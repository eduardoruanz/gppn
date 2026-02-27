package main

import (
	"context"
	"testing"
)

func TestNewStablecoinAdapter(t *testing.T) {
	adapter := NewStablecoinAdapter("http://localhost:8545")
	if adapter == nil {
		t.Fatal("expected non-nil adapter")
	}
	if adapter.rpcEndpoint != "http://localhost:8545" {
		t.Errorf("expected rpcEndpoint http://localhost:8545, got %s", adapter.rpcEndpoint)
	}
	if len(adapter.tokenContracts) != 2 {
		t.Errorf("expected 2 token contracts, got %d", len(adapter.tokenContracts))
	}
}

func TestStablecoinAdapter_SupportedCurrencies(t *testing.T) {
	adapter := NewStablecoinAdapter("http://localhost:8545")
	currencies := adapter.SupportedCurrencies()

	if len(currencies) != 2 {
		t.Fatalf("expected 2 supported currencies, got %d", len(currencies))
	}

	expected := map[string]bool{"USDC": true, "USDT": true}
	for _, c := range currencies {
		if !expected[c] {
			t.Errorf("unexpected currency: %s", c)
		}
	}
}

func TestStablecoinAdapter_Initiate_USDC(t *testing.T) {
	adapter := NewStablecoinAdapter("http://localhost:8545")
	ctx := context.Background()

	req := SettlementRequest{
		PaymentID:   "pay-001",
		Amount:      "100.00",
		Currency:    "USDC",
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
	if result.Currency != "USDC" {
		t.Errorf("expected currency USDC, got %s", result.Currency)
	}
}

func TestStablecoinAdapter_Initiate_USDT(t *testing.T) {
	adapter := NewStablecoinAdapter("http://localhost:8545")
	ctx := context.Background()

	req := SettlementRequest{
		PaymentID:   "pay-002",
		Amount:      "250.50",
		Currency:    "USDT",
		FromAddress: "0xSender",
		ToAddress:   "0xReceiver",
	}

	result, err := adapter.Initiate(ctx, req)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if result.Currency != "USDT" {
		t.Errorf("expected currency USDT, got %s", result.Currency)
	}
}

func TestStablecoinAdapter_Initiate_UnsupportedCurrency(t *testing.T) {
	adapter := NewStablecoinAdapter("http://localhost:8545")
	ctx := context.Background()

	req := SettlementRequest{
		PaymentID:   "pay-003",
		Amount:      "100.00",
		Currency:    "DAI",
		FromAddress: "0xSender",
		ToAddress:   "0xReceiver",
	}

	_, err := adapter.Initiate(ctx, req)
	if err == nil {
		t.Error("expected error for unsupported currency")
	}
}

func TestStablecoinAdapter_Initiate_Validation(t *testing.T) {
	adapter := NewStablecoinAdapter("http://localhost:8545")
	ctx := context.Background()

	tests := []struct {
		name string
		req  SettlementRequest
	}{
		{
			name: "missing payment ID",
			req:  SettlementRequest{Amount: "100.00", Currency: "USDC", ToAddress: "0xAddr"},
		},
		{
			name: "missing amount",
			req:  SettlementRequest{PaymentID: "pay-001", Currency: "USDC", ToAddress: "0xAddr"},
		},
		{
			name: "missing currency",
			req:  SettlementRequest{PaymentID: "pay-001", Amount: "100.00", ToAddress: "0xAddr"},
		},
		{
			name: "missing to address",
			req:  SettlementRequest{PaymentID: "pay-001", Amount: "100.00", Currency: "USDC"},
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

func TestStablecoinAdapter_Confirm(t *testing.T) {
	adapter := NewStablecoinAdapter("http://localhost:8545")
	ctx := context.Background()

	req := SettlementRequest{
		PaymentID:   "pay-004",
		Amount:      "500.00",
		Currency:    "USDC",
		FromAddress: "0xSender",
		ToAddress:   "0xReceiver",
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

func TestStablecoinAdapter_Confirm_NotFound(t *testing.T) {
	adapter := NewStablecoinAdapter("http://localhost:8545")
	ctx := context.Background()

	_, err := adapter.Confirm(ctx, "nonexistent-tx")
	if err == nil {
		t.Error("expected error for nonexistent transaction")
	}
}

func TestStablecoinAdapter_Rollback(t *testing.T) {
	adapter := NewStablecoinAdapter("http://localhost:8545")
	ctx := context.Background()

	req := SettlementRequest{
		PaymentID:   "pay-005",
		Amount:      "75.00",
		Currency:    "USDT",
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

func TestStablecoinAdapter_GetStatus(t *testing.T) {
	adapter := NewStablecoinAdapter("http://localhost:8545")
	ctx := context.Background()

	req := SettlementRequest{
		PaymentID:   "pay-006",
		Amount:      "1000.00",
		Currency:    "USDC",
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

func TestStablecoinAdapter_EstimateCost(t *testing.T) {
	adapter := NewStablecoinAdapter("http://localhost:8545")
	ctx := context.Background()

	req := SettlementRequest{
		PaymentID: "pay-007",
		Amount:    "200.00",
		Currency:  "USDC",
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

func TestStablecoinAdapter_EstimateCost_Validation(t *testing.T) {
	adapter := NewStablecoinAdapter("http://localhost:8545")
	ctx := context.Background()

	_, err := adapter.EstimateCost(ctx, SettlementRequest{})
	if err == nil {
		t.Error("expected error for empty amount")
	}
}
