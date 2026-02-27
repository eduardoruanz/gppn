package main

import (
	"fmt"
	"log"
	"net/http"
	"os"
	"os/signal"
	"syscall"

	"github.com/veritas-protocol/veritas/services/issuer-api/handlers"
)

const defaultPort = 8082

func main() {
	port := defaultPort
	if p := os.Getenv("VERITAS_PORT"); p != "" {
		fmt.Sscanf(p, "%d", &port)
	}

	issuerHandler := handlers.NewIssuerHandler()

	mux := http.NewServeMux()
	mux.HandleFunc("/api/v1/issue", issuerHandler.HandleIssue)
	mux.HandleFunc("/api/v1/revoke", issuerHandler.HandleRevoke)
	mux.HandleFunc("/api/v1/issued", issuerHandler.HandleListIssued)
	mux.HandleFunc("/api/v1/schemas", issuerHandler.HandleListSchemas)
	mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(`{"status":"healthy","service":"issuer-api"}`))
	})

	addr := fmt.Sprintf(":%d", port)
	server := &http.Server{Addr: addr, Handler: mux}

	go func() {
		log.Printf("Veritas Issuer API starting on %s", addr)
		log.Printf("Endpoints:")
		log.Printf("  POST /api/v1/issue     — Issue a credential")
		log.Printf("  POST /api/v1/revoke    — Revoke a credential")
		log.Printf("  GET  /api/v1/issued    — List issued credentials")
		log.Printf("  GET  /api/v1/schemas   — List credential schemas")
		log.Printf("  GET  /health")

		if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Fatalf("Server failed: %v", err)
		}
	}()

	sigCh := make(chan os.Signal, 1)
	signal.Notify(sigCh, syscall.SIGINT, syscall.SIGTERM)
	sig := <-sigCh
	log.Printf("Received signal %v, shutting down...", sig)
	server.Close()
	log.Println("Veritas Issuer API stopped")
}
