// Package logging provides structured logging setup for GPPN services
// using the standard library's log/slog package.
package logging

import (
	"log/slog"
	"os"
	"strings"
)

// Setup initializes and returns a structured logger configured for the given
// service name and log level. The logger outputs JSON-formatted log entries
// to stdout.
func Setup(serviceName, level string) *slog.Logger {
	var logLevel slog.Level
	switch strings.ToLower(level) {
	case "debug":
		logLevel = slog.LevelDebug
	case "info":
		logLevel = slog.LevelInfo
	case "warn", "warning":
		logLevel = slog.LevelWarn
	case "error":
		logLevel = slog.LevelError
	default:
		logLevel = slog.LevelInfo
	}

	handler := slog.NewJSONHandler(os.Stdout, &slog.HandlerOptions{
		Level: logLevel,
	})

	logger := slog.New(handler).With(
		slog.String("service", serviceName),
	)

	return logger
}

// SetDefault configures the default slog logger for the given service.
func SetDefault(serviceName, level string) {
	logger := Setup(serviceName, level)
	slog.SetDefault(logger)
}
