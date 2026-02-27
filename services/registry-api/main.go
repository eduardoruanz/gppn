package main

import (
	"fmt"
	"log"
	"net/http"
	"os"
	"os/signal"
	"syscall"

	"github.com/veritas-protocol/veritas/services/registry-api/handlers"
)

const defaultPort = 8084

func main() {
	port := defaultPort
	if p := os.Getenv("VERITAS_PORT"); p != "" {
		fmt.Sscanf(p, "%d", &port)
	}

	registryHandler := handlers.NewRegistryHandler()

	mux := http.NewServeMux()
	mux.HandleFunc("/api/v1/dids", registryHandler.HandleDids)
	mux.HandleFunc("/api/v1/dids/", registryHandler.HandleDidByID)
	mux.HandleFunc("/api/v1/schemas", registryHandler.HandleSchemas)
	mux.HandleFunc("/api/v1/stats", registryHandler.HandleStats)
	mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(`{"status":"healthy","service":"registry-api"}`))
	})

	addr := fmt.Sprintf(":%d", port)
	server := &http.Server{Addr: addr, Handler: mux}

	go func() {
		log.Printf("Veritas Registry API starting on %s", addr)
		log.Printf("Endpoints:")
		log.Printf("  POST /api/v1/dids          — Register DID Document")
		log.Printf("  GET  /api/v1/dids/:did     — Resolve DID")
		log.Printf("  POST /api/v1/schemas       — Register schema")
		log.Printf("  GET  /api/v1/schemas       — List schemas")
		log.Printf("  GET  /api/v1/stats         — Registry stats")
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
	log.Println("Veritas Registry API stopped")
}
