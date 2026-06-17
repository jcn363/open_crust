# Private Model Deployment Guide

This guide covers deploying OpenCrust with private/local models for air-gapped and regulated environments.

## Overview

OpenCrust supports multiple LLM providers. For private deployments, the recommended approach is using **Ollama** for local model serving, with optional air-gapped configurations.

---

## Ollama Deployment

### Quick Start

```bash
# Install Ollama
curl -fsSL https://ollama.com/install.sh | sh

# Start Ollama server
ollama serve

# Pull a model
ollama pull llama3.1:8b
ollama pull codellama:13b
ollama pull mistral:7b
```

### Configure OpenCrust for Ollama

```json
{
  "provider": "ollama",
  "model": "llama3.1:8b",
  "ollama_base_url": "http://localhost:11434",
  "temperature": 0.2,
  "max_tokens": 4096
}
```

Or via CLI:
```bash
opencrust --provider ollama --model llama3.1:8b
```

### Model Recommendations

| Use Case | Model | Size | Notes |
|----------|-------|------|-------|
| General coding | `llama3.1:8b` | 4.7 GB | Best balance of speed/quality |
| Complex reasoning | `llama3.1:70b` | 40 GB | Requires 48GB+ RAM |
| Code-specific | `codellama:13b` | 7.3 GB | Optimized for code generation |
| Fast iteration | `qwen2.5-coder:7b` | 4.2 GB | Excellent for code tasks |
| Lightweight | `phi3:mini` | 2.3 GB | Good for simple tasks |

---

## Air-Gapped Deployment

### Prerequisites

1. **No internet access** on target machine
2. **Ollama pre-installed** or installed via package manager
3. **Model files transferred** via secure media

### Step 1: Prepare Models on Connected Machine

```bash
# On internet-connected machine
ollama pull llama3.1:8b
ollama pull codellama:13b

# Export models
mkdir -p /tmp/ollama-models
ollama save llama3.1:8b /tmp/ollama-models/llama3.1-8b.tar
ollama save codellama:13b /tmp/ollama-models/codellama-13b.tar

# Transfer via secure media (encrypted USB, etc.)
```

### Step 2: Import Models on Air-Gapped Machine

```bash
# On air-gapped machine
ollama load -i /media/secure/llama3.1-8b.tar
ollama load -i /media/secure/codellama-13b.tar

# Verify
ollama list
```

### Step 3: Configure OpenCrust

```json
{
  "provider": "ollama",
  "model": "llama3.1:8b",
  "ollama_base_url": "http://localhost:11434",
  "compliance_mode": true,
  "audit_retention_days": 365,
  "allowed_domains": []
}
```

**Key air-gap settings:**
- `allowed_domains: []` — Blocks all external network access
- `compliance_mode: true` — Immutable audit trail
- No API keys required

---

## Systemd Service for Production

```ini
# /etc/systemd/system/ollama.service
[Unit]
Description=Ollama LLM Server
After=network-online.target

[Service]
ExecStart=/usr/local/bin/ollama serve
User=ollama
Group=ollama
Restart=always
RestartSec=3
Environment="OLLAMA_HOST=0.0.0.0:11434"
Environment="OLLAMA_MODELS=/var/lib/ollama/models"
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now ollama
```

---

## OpenCrust as Systemd Service

```ini
# /etc/systemd/system/opencrust.service
[Unit]
Description=OpenCrust AI Coding Assistant
After=ollama.service
Requires=ollama.service

[Service]
Type=simple
ExecStart=/usr/local/bin/opencrust
User=developer
WorkingDirectory=/home/developer/project
Environment="OPENCRUST_CONFIG=/etc/opencrust/config.json"
Restart=on-failure
RestartSec=5

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ReadWritePaths=/home/developer/project /var/log/opencrust
ProtectHome=false

[Install]
WantedBy=multi-user.target
```

---

## Network Isolation

### Firewall Rules (iptables/nftables)

```bash
# Allow only localhost Ollama
iptables -A OUTPUT -d 127.0.0.1 -p tcp --dport 11434 -j ACCEPT
iptables -A OUTPUT -p tcp -p tcp -j DROP

# Or with nftables
nft add rule inet filter output ip daddr 127.0.0.1 tcp dport 11434 accept
nft add rule inet filter output tcp drop
```

### OpenCrust Config for Network Isolation

```json
{
  "allowed_domains": [],
  "network_gating": true,
  "compliance_mode": true
}
```

---

## Model Customization

### Create Custom Modelfile

```dockerfile
# Modelfile
FROM llama3.1:8b

# System prompt for coding assistant
SYSTEM """You are OpenCrust, an expert coding assistant.
Follow the user's instructions precisely.
Write clean, idiomatic code with proper error handling.
Never use unwrap() or expect() in production code."""

# Parameters
PARAMETER temperature 0.1
PARAMETER top_p 0.9
PARAMETER num_ctx 8192
PARAMETER stop "<|eot_id|>"
```

```bash
ollama create opencrust-coder -f Modelfile
```

### Use Custom Model

```json
{
  "provider": "ollama",
  "model": "opencrust-coder"
}
```

---

## Compliance & Auditing

### Enable Full Compliance Mode

```json
{
  "compliance_mode": true,
  "audit_retention_days": 2555,
  "audit_max_size_bytes": 1073741824,
  "allowed_domains": []
}
```

### Generate Compliance Reports

```bash
# SOC 2 Type II report
opencrust compliance generate --format soc2 --output-dir /audit/reports

# Export audit logs for SIEM
opencrust compliance export --format syslog --syslog-server log-collector.internal --from 2026-01-01

# Build evidence package
opencrust compliance evidence --output-dir /audit/evidence
```

---

## Hardware Requirements

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| RAM | 16 GB | 64 GB+ |
| CPU | 8 cores | 16+ cores |
| GPU | Optional | NVIDIA 24GB+ VRAM |
| Storage | 50 GB SSD | 500 GB NVMe |
| Network | Isolated | Air-gapped |

### GPU Acceleration

```bash
# NVIDIA GPU support
ollama serve --gpu-layers 999

# Verify GPU usage
ollama run llama3.1:8b "test" --verbose
```

---

## Troubleshooting

### Ollama Connection Refused

```bash
# Check service
systemctl status ollama

# Check port
ss -tlnp | grep 11434

# Check logs
journalctl -u ollama -f
```

### Out of Memory

```bash
# Reduce context window
# In config.json:
"max_tokens": 2048

# Or use smaller model
"model": "phi3:mini"
```

### Model Not Found

```bash
# List available
ollama list

# Pull if missing (requires internet)
ollama pull llama3.1:8b
```

---

## Security Checklist

- [ ] Air-gapped network (no internet access)
- [ ] `allowed_domains: []` in config
- [ ] `compliance_mode: true` enabled
- [ ] Audit logs forwarded to SIEM
- [ ] Model files verified (checksums)
- [ ] Systemd hardening applied
- [ ] Firewall rules blocking egress
- [ ] Regular evidence package generation scheduled
- [ ] Backup/retention policy for audit logs

---

## References

- [Ollama Documentation](https://github.com/ollama/ollama)
- [OpenCrust Compliance](docs/COMPLIANCE.md)
- [OpenCrust Configuration](docs/CONFIGURATION.md)
- [SOC 2 Compliance](docs/COMPLIANCE_PACKAGING.md)