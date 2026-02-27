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
	defaultPort = 50053
	defaultRPC  = "http://localhost:8545"
)

func main() {
	port := defaultPort
	if p := os.Getenv("GPPN_PORT"); p != "" {
		fmt.Sscanf(p, "%d", &port)
	}

	rpcEndpoint := defaultRPC

	adapter := NewStablecoinAdapter(rpcEndpoint)

	// Log supported currencies at startup.
	currencies := adapter.SupportedCurrencies()
	log.Printf("Stablecoin Settlement Adapter starting on port %d", port)
	log.Printf("Supported currencies: %v", currencies)
	log.Printf("RPC: %s", rpcEndpoint)

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
		Amount:      "1.00",
		Currency:    "USDC",
		FromAddress: "0x0000000000000000000000000000000000000000",
		ToAddress:   "0x0000000000000000000000000000000000000001",
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
	log.Println("Stablecoin Settlement Adapter stopped")
}
