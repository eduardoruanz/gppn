# Node Operator Guide

## System Requirements

| Resource | Minimum | Recommended |
|----------|---------|-------------|
| CPU | 2 cores | 4+ cores |
| RAM | 2 GB | 8 GB |
| Storage | 10 GB SSD | 100 GB NVMe |
| Network | 10 Mbps | 100 Mbps |
| OS | Linux (glibc 2.31+) | Debian 12 / Ubuntu 22.04 |

## Installation

### From Source

```bash
git clone https://github.com/gppn-protocol/gppn.git
cd gppn
cargo build --release -p gppn-node -p gppn-cli

# Binaries at target/release/gppn-node and target/release/gppn-cli
```

### Docker

```bash
docker pull ghcr.io/gppn-protocol/gppn/gppn-node:latest

docker run -d \
  --name gppn-node \
  -p 9000:9000 \
  -p 9001:9001 \
  -p 9002:9002 \
  -v gppn-data:/data \
  ghcr.io/gppn-protocol/gppn/gppn-node:latest
```

## Configuration

### Configuration File (`gppn.toml`)

```toml
[network]
listen_port = 9000
external_address = "/ip4/203.0.113.1/tcp/9000"
bootstrap_peers = [
    "/ip4/198.51.100.1/tcp/9000/p2p/12D3KooW...",
    "/ip4/198.51.100.2/tcp/9000/p2p/12D3KooW...",
]
max_peers = 50

[api]
listen_port = 9001
enabled = true

[storage]
path = "/data/gppn"

[metrics]
listen_port = 9002
enabled = true

[logging]
level = "info"    # trace, debug, info, warn, error
format = "json"   # json, text

[identity]
key_file = "/data/gppn/node.key"
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `GPPN_CONFIG` | Config file path | `./gppn.toml` |
| `GPPN_LOG_LEVEL` | Log level | `info` |
| `GPPN_DATA_DIR` | Data directory | `./data` |
| `GPPN_P2P_PORT` | P2P listen port | `9000` |
| `GPPN_API_PORT` | API listen port | `9001` |

## Running

### Standalone

```bash
# Initialize
gppn-cli init --data-dir /data/gppn

# Start
gppn-node --config /data/gppn/gppn.toml
```

### systemd Service

```ini
[Unit]
Description=GPPN Node
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=gppn
ExecStart=/usr/local/bin/gppn-node --config /etc/gppn/gppn.toml
Restart=on-failure
RestartSec=5
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
```

### Docker Compose (Multi-Node)

See `infra/docker/docker-compose.yml` for a 3-node testnet setup with PostgreSQL and DragonflyDB.

## Monitoring

### Prometheus Metrics

The node exposes Prometheus metrics on the configured metrics port (default: 9002):

```
# Scrape config for prometheus.yml
scrape_configs:
  - job_name: 'gppn-node'
    static_configs:
      - targets: ['localhost:9002']
```

Key metrics:
- `gppn_peers_connected` — Number of connected peers
- `gppn_payments_total` — Total payments processed (by status)
- `gppn_routing_table_size` — Number of routes in DRT
- `gppn_settlement_duration_seconds` — Settlement latency histogram

### Health Check

```bash
curl http://localhost:9001/health
```

### Log Analysis

```bash
# Follow logs (JSON format)
journalctl -u gppn-node -f | jq .

# Filter by level
journalctl -u gppn-node -f | jq 'select(.level == "ERROR")'
```

## Security

- **Key Management**: Node identity keys are stored at the configured `key_file` path. Back up this file — losing it means losing your node's identity.
- **Firewall**: Only port 9000 (P2P) needs to be publicly accessible. API (9001) and metrics (9002) should be restricted.
- **TLS**: P2P connections use Noise protocol encryption. API should be placed behind a reverse proxy with TLS for production.

## Backup & Recovery

```bash
# Backup node data
tar -czf gppn-backup-$(date +%Y%m%d).tar.gz /data/gppn/

# Restore
tar -xzf gppn-backup-*.tar.gz -C /
```

The critical files to back up:
- `node.key` — node identity keypair
- `db/` — RocksDB database (payments, routing, identity data)
- `gppn.toml` — configuration
