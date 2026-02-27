// Package config provides common configuration loading for GPPN services.
// It uses a simple TOML-like format for configuration files.
package config

import (
	"bufio"
	"fmt"
	"os"
	"strconv"
	"strings"
)

// AppConfig holds the common configuration for all GPPN services.
type AppConfig struct {
	Name        string
	Port        int
	LogLevel    string
	MetricsPort int
}

// DefaultConfig returns an AppConfig with sensible defaults.
func DefaultConfig(name string) AppConfig {
	return AppConfig{
		Name:        name,
		Port:        8080,
		LogLevel:    "info",
		MetricsPort: 9090,
	}
}

// LoadFromFile loads configuration from a TOML-style file.
// The file format is simple key = "value" or key = number pairs.
// Lines starting with # are comments.
func LoadFromFile(path string) (AppConfig, error) {
	cfg := DefaultConfig("")

	f, err := os.Open(path)
	if err != nil {
		return cfg, fmt.Errorf("config: failed to open file %s: %w", path, err)
	}
	defer f.Close()

	scanner := bufio.NewScanner(f)
	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())

		// Skip empty lines and comments.
		if line == "" || strings.HasPrefix(line, "#") || strings.HasPrefix(line, "[") {
			continue
		}

		parts := strings.SplitN(line, "=", 2)
		if len(parts) != 2 {
			continue
		}

		key := strings.TrimSpace(parts[0])
		value := strings.TrimSpace(parts[1])
		// Remove surrounding quotes if present.
		value = strings.Trim(value, `"'`)

		switch key {
		case "name":
			cfg.Name = value
		case "port":
			p, err := strconv.Atoi(value)
			if err != nil {
				return cfg, fmt.Errorf("config: invalid port value %q: %w", value, err)
			}
			cfg.Port = p
		case "log_level":
			cfg.LogLevel = value
		case "metrics_port":
			p, err := strconv.Atoi(value)
			if err != nil {
				return cfg, fmt.Errorf("config: invalid metrics_port value %q: %w", value, err)
			}
			cfg.MetricsPort = p
		}
	}

	if err := scanner.Err(); err != nil {
		return cfg, fmt.Errorf("config: error reading file: %w", err)
	}

	return cfg, nil
}

// LoadFromEnv loads configuration from environment variables.
// Environment variables take the form GPPN_<KEY> (e.g., GPPN_PORT, GPPN_LOG_LEVEL).
func LoadFromEnv(name string) AppConfig {
	cfg := DefaultConfig(name)

	if v := os.Getenv("GPPN_PORT"); v != "" {
		if p, err := strconv.Atoi(v); err == nil {
			cfg.Port = p
		}
	}

	if v := os.Getenv("GPPN_LOG_LEVEL"); v != "" {
		cfg.LogLevel = v
	}

	if v := os.Getenv("GPPN_METRICS_PORT"); v != "" {
		if p, err := strconv.Atoi(v); err == nil {
			cfg.MetricsPort = p
		}
	}

	return cfg
}
