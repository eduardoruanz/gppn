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
git clone https://github.com/veritas-protocol/veritas.git
cd veritas
cargo build --release -p veritas-node -p veritas-cli

# Binaries at target/release/veritas-node and target/release/veritas
```

### Docker

```bash
docker pull ghcr.io/veritas-protocol/veritas/veritas-node:latest

docker run -d \
  --name veritas-node \
  -p 9000:9000 \
  -p 9001:9001 \
  -p 9002:9002 \
  -v veritas-data:/data \
  ghcr.io/veritas-protocol/veritas/veritas-node:latest
```

## Configuration

### Configuration File (`veritas.toml`)

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
path = "/data/veritas"

[metrics]
listen_port = 9002
enabled = true

[logging]
level = "info"    # trace, debug, info, warn, error
format = "json"   # json, text

[identity]
key_file = "/data/veritas/node.key"
is_issuer = true
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `VERITAS_CONFIG` | Config file path | `./veritas.toml` |
| `VERITAS_LOG_LEVEL` | Log level | `info` |
| `VERITAS_DATA_DIR` | Data directory | `./data` |
| `VERITAS_P2P_PORT` | P2P listen port | `9000` |
| `VERITAS_API_PORT` | API listen port | `9001` |

## Running

### Standalone

```bash
# Initialize
veritas init --data-dir /data/veritas

# Start
veritas-node --config /data/veritas/veritas.toml
```

### systemd Service

```ini
[Unit]
Description=Veritas Node
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=veritas
ExecStart=/usr/local/bin/veritas-node --config /etc/veritas/veritas.toml
Restart=on-failure
RestartSec=5
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
```

### Docker Compose (Multi-Node)

See `infra/docker/docker-compose.yml` for a 3-node testnet setup with registry-api, PostgreSQL, and DragonflyDB.

## Monitoring

### Prometheus Metrics

The node exposes Prometheus metrics on the configured metrics port (default: 9002):

```
# Scrape config for prometheus.yml
scrape_configs:
  - job_name: 'veritas-node'
    static_configs:
      - targets: ['localhost:9002']
```

Key metrics:
- `veritas_peers_connected` — Number of connected peers
- `veritas_credentials_issued` — Total credentials issued
- `veritas_proofs_verified` — Total ZK proofs verified
- `veritas_trust_attestations` — Trust attestation count

### Health Check

```bash
curl http://localhost:9001/api/v1/health
```

### Log Analysis

```bash
# Follow logs (JSON format)
journalctl -u veritas-node -f | jq .

# Filter by level
journalctl -u veritas-node -f | jq 'select(.level == "ERROR")'
```

## Security

- **Key Management**: Node identity keys are stored at the configured `key_file` path. Back up this file — losing it means losing your node's DID.
- **Firewall**: Only port 9000 (P2P) needs to be publicly accessible. API (9001) and metrics (9002) should be restricted.
- **TLS**: P2P connections use Noise protocol encryption. API should be placed behind a reverse proxy with TLS for production.

## Backup & Recovery

```bash
# Backup node data
tar -czf veritas-backup-$(date +%Y%m%d).tar.gz /data/veritas/

# Restore
tar -xzf veritas-backup-*.tar.gz -C /
```

The critical files to back up:
- `node.key` — node identity keypair (your DID)
- `db/` — RocksDB database (credentials, schemas, identity data)
- `veritas.toml` — configuration
