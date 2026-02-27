package main

import (
	"fmt"
	"log"
	"net/http"
	"os"
	"os/signal"
	"syscall"

	"github.com/gppn-protocol/gppn/services/gateway/handlers"
	"github.com/gppn-protocol/gppn/services/gateway/middleware"
)

const defaultPort = 8081

func main() {
	port := defaultPort
	if p := os.Getenv("GPPN_PORT"); p != "" {
		fmt.Sscanf(p, "%d", &port)
	}

	gatewayHandler := handlers.NewGatewayHandler()
	auth := middleware.NewAuthMiddleware()

	mux := http.NewServeMux()

	// Protected endpoints (require API key).
	mux.Handle("/api/v1/send", auth.AuthenticateFunc(gatewayHandler.HandleSendPayment))
	mux.Handle("/api/v1/status/", auth.AuthenticateFunc(gatewayHandler.HandleGetStatus))

	// Health check endpoint (no auth required).
	mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(`{"status":"healthy","service":"gateway"}`))
	})

	addr := fmt.Sprintf(":%d", port)
	server := &http.Server{
		Addr:    addr,
		Handler: mux,
	}

	// Start server in a goroutine.
	go func() {
		log.Printf("GPPN Gateway starting on %s", addr)
		log.Printf("Endpoints:")
		log.Printf("  POST /api/v1/send       (requires API key)")
		log.Printf("  GET  /api/v1/status/{id} (requires API key)")
		log.Printf("  GET  /health")
		log.Printf("Auth: X-API-Key header or Authorization: Bearer <key>")

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
	log.Println("GPPN Gateway stopped")
}
