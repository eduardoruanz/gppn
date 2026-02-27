package main

import (
	"context"
	"fmt"
	"log"
	"net"
	"os"
	"os/signal"
	"syscall"
)

const (
	defaultPort    = 50052
	defaultNetwork = "mainnet"
	defaultRPC     = "http://localhost:8332"
)

func main() {
	port := defaultPort
	if p := os.Getenv("GPPN_PORT"); p != "" {
		fmt.Sscanf(p, "%d", &port)
	}

	network := defaultNetwork
	rpcEndpoint := defaultRPC

	adapter := NewBitcoinAdapter(network, rpcEndpoint)

	// Log supported currencies at startup.
	currencies := adapter.SupportedCurrencies()
	log.Printf("Bitcoin Settlement Adapter starting on port %d", port)
	log.Printf("Supported currencies: %v", currencies)
	log.Printf("Network: %s, RPC: %s", network, rpcEndpoint)

	// Create a TCP listener for the gRPC server.
	lis, err := net.Listen("tcp", fmt.Sprintf(":%d", port))
	if err != nil {
		log.Fatalf("Failed to listen on port %d: %v", port, err)
	}

	// In a full implementation, we would register the adapter with a gRPC server here.
	log.Printf("gRPC server listening on %s", lis.Addr().String())

	// Demonstrate that the adapter is functional.
	ctx := context.Background()
	req := SettlementRequest{
		PaymentID:   "startup-check",
		Amount:      "0.00001",
		Currency:    "BTC",
		FromAddress: "bc1q0000000000000000000000000000000000000000",
		ToAddress:   "bc1q0000000000000000000000000000000000000001",
	}
	estimate, err := adapter.EstimateCost(ctx, req)
	if err != nil {
		log.Fatalf("Adapter health check failed: %v", err)
	}
	log.Printf("Adapter health check passed. Estimated fee: %s, time: %s", estimate.TotalFee, estimate.EstimatedTime)

	// Wait for shutdown signal.
	sigCh := make(chan os.Signal, 1)
	signal.Notify(sigCh, syscall.SIGINT, syscall.SIGTERM)

	sig := <-sigCh
	log.Printf("Received signal %v, shutting down...", sig)
	lis.Close()
	log.Println("Bitcoin Settlement Adapter stopped")
}
