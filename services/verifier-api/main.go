package main

import (
	"fmt"
	"log"
	"net/http"
	"os"
	"os/signal"
	"syscall"

	"github.com/veritas-protocol/veritas/services/verifier-api/handlers"
)

const defaultPort = 8083

func main() {
	port := defaultPort
	if p := os.Getenv("VERITAS_PORT"); p != "" {
		fmt.Sscanf(p, "%d", &port)
	}

	verifierHandler := handlers.NewVerifierHandler()

	mux := http.NewServeMux()
	mux.HandleFunc("/api/v1/verify", verifierHandler.HandleVerify)
	mux.HandleFunc("/api/v1/proof-request", verifierHandler.HandleProofRequest)
	mux.HandleFunc("/api/v1/verify-proof", verifierHandler.HandleVerifyProof)
	mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(`{"status":"healthy","service":"verifier-api"}`))
	})

	addr := fmt.Sprintf(":%d", port)
	server := &http.Server{Addr: addr, Handler: mux}

	go func() {
		log.Printf("Veritas Verifier API starting on %s", addr)
		log.Printf("Endpoints:")
		log.Printf("  POST /api/v1/verify        — Verify a presentation")
		log.Printf("  POST /api/v1/proof-request  — Create proof request")
		log.Printf("  POST /api/v1/verify-proof   — Verify proof response")
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
	log.Println("Veritas Verifier API stopped")
}
