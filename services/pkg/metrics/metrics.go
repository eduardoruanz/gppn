// Package metrics provides a simple Prometheus-compatible metrics endpoint
// placeholder for GPPN services.
package metrics

import (
	"fmt"
	"net/http"
	"sync"
	"sync/atomic"
)

// Collector holds simple counters and gauges for a service.
type Collector struct {
	serviceName string
	counters    sync.Map // map[string]*int64
	gauges      sync.Map // map[string]*int64
}

// NewCollector creates a new metrics collector for the named service.
func NewCollector(serviceName string) *Collector {
	return &Collector{
		serviceName: serviceName,
	}
}

// IncrementCounter increments a named counter by 1.
func (c *Collector) IncrementCounter(name string) {
	val, _ := c.counters.LoadOrStore(name, new(int64))
	atomic.AddInt64(val.(*int64), 1)
}

// SetGauge sets a named gauge to the given value.
func (c *Collector) SetGauge(name string, value int64) {
	val, _ := c.gauges.LoadOrStore(name, new(int64))
	atomic.StoreInt64(val.(*int64), value)
}

// Handler returns an http.Handler that serves Prometheus-compatible metrics.
func (c *Collector) Handler() http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "text/plain; version=0.0.4; charset=utf-8")

		// Write counters.
		c.counters.Range(func(key, value any) bool {
			name := key.(string)
			val := atomic.LoadInt64(value.(*int64))
			fmt.Fprintf(w, "# TYPE %s_%s counter\n", c.serviceName, name)
			fmt.Fprintf(w, "%s_%s %d\n", c.serviceName, name, val)
			return true
		})

		// Write gauges.
		c.gauges.Range(func(key, value any) bool {
			name := key.(string)
			val := atomic.LoadInt64(value.(*int64))
			fmt.Fprintf(w, "# TYPE %s_%s gauge\n", c.serviceName, name)
			fmt.Fprintf(w, "%s_%s %d\n", c.serviceName, name, val)
			return true
		})
	})
}

// StartServer starts an HTTP server serving the /metrics endpoint on the given port.
// This function blocks, so it should be called in a goroutine.
func (c *Collector) StartServer(port int) error {
	mux := http.NewServeMux()
	mux.Handle("/metrics", c.Handler())

	addr := fmt.Sprintf(":%d", port)
	return http.ListenAndServe(addr, mux)
}
