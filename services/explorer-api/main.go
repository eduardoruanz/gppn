package main

import (
	"fmt"
	"log"
	"net/http"
	"os"
	"os/signal"
	"syscall"

	"github.com/gppn-protocol/gppn/services/explorer-api/handlers"
)

const defaultPort = 8080

func main() {
	port := defaultPort
	if p := os.Getenv("GPPN_PORT"); p != "" {
		fmt.Sscanf(p, "%d", &port)
	}

	paymentHandler := handlers.NewPaymentHandler()
	networkHandler := handlers.NewNetworkHandler()

	mux := http.NewServeMux()

	// Payment endpoints.
	mux.HandleFunc("/api/v1/payments", paymentHandler.HandleListPayments)
	mux.HandleFunc("/api/v1/payments/", paymentHandler.HandleGetPayment)

	// Network endpoints.
	mux.HandleFunc("/api/v1/network/stats", networkHandler.HandleNetworkStats)
	mux.HandleFunc("/api/v1/network/peers", networkHandler.HandleListPeers)

	// Health check endpoint.
	mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(`{"status":"healthy","service":"explorer-api"}`))
	})

	addr := fmt.Sprintf(":%d", port)
	server := &http.Server{
		Addr:    addr,
		Handler: mux,
	}

	// Start server in a goroutine.
	go func() {
		log.Printf("GPPN Explorer API starting on %s", addr)
		log.Printf("Endpoints:")
		log.Printf("  GET /api/v1/payments")
		log.Printf("  GET /api/v1/payments/{id}")
		log.Printf("  GET /api/v1/network/stats")
		log.Printf("  GET /api/v1/network/peers")
		log.Printf("  GET /health")

		if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Fatalf("Server failed: %v", err)
		}
	}()

	// Wait for shutdown signal.
	sigCh := make(chan os.Signal, 1)
	signal.Notify(sigCh, syscall.SIGINT, syscall.SIGTERM)

	sig := <-sigCh
	log.Printf("Received signal %v, shutting down...", sig)
	server.Close()
	log.Println("GPPN Explorer API stopped")
}
