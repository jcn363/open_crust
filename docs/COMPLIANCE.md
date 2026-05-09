# Enterprise Compliance

OpenCrust provides built-in audit logging, evidence packaging, and compliance reporting for SOC 2, ISO 27001, and internal audit requirements. This document covers how to configure and use these features.

**For security model context:** See **docs/SECURITY.md**  
**For audit internals:** See **src/audit.rs** and **src/compliance.rs**

---

## SOC 2 Control Mapping

| Control | Requirement | OpenCrust Feature |
|---------|-------------|-------------------|
| **CC6.1** | Logical and physical access controls | Permission enforcement, network gating, file access patterns |
| **CC6.7** | Restrict physical access to information | Audit logging of all tool executions with session tracking |
| **A1.2** | System monitoring and logging | Structured audit export, compliance reporting, log rotation |

---

## Audit CLI Reference

### Export audit logs

```bash
# Export all logs within a date range as CSV (default)
opencrust audit export --from 2025-01-01 --to 2025-12-31

# Export as JSON
opencrust audit export --from 2025-01-01 --to 2025-12-31 --format json

# Export to a specific file
opencrust audit export --from 2025-01-01 --to 2025-12-31 --output /tmp/audit-export.json
```

### Query audit logs

```bash
# Show all log entries within a date range
opencrust audit query --from 2025-01-01 --to 2025-12-31

# Filter by action (partial match on tool name)
opencrust audit query --action "file_read"

# Filter by approval status
opencrust audit query --status approved
opencrust audit query --status denied
```

### Generate evidence package

```bash
opencrust audit evidence
opencrust audit evidence --output-dir /var/audit/evidence
```

Creates a structured evidence directory with SHA256 manifest:

```
evidence-<timestamp>/
├── audit-export.json      # All log entries
├── sha256.manifest        # SHA256 checksums for every file
└── report.json            # Summary statistics
```

### Generate compliance report

```bash
opencrust audit report
opencrust audit report --from 2025-01-01 --to 2025-12-31
```

Outputs a comprehensive compliance report including:

- Total approved/denied actions
- Per-tool breakdown
- Session summary
- Evidence package integrity check

---

## Compliance Mode

Enable compliance mode in `config.json` to prevent log rotation and deletion:

```json
{
  "compliance_mode": true,
  "audit_retention_days": 365,
  "audit_max_size_bytes": 1073741824
}
```

**What compliance mode does:**

- Prevents log rotation (logs are append-only)
- Prevents cleanup/deletion of old logs
- Ensures an unbroken, immutable audit trail
- Logs grow unbounded — you must manage storage externally

---

## Evidence Package Structure

The `opencrust audit evidence` command produces a tamper-evident package:

```
evidence-2025-01-01T120000/
├── audit-export.json      # All audit entries as structured JSON
├── sha256.manifest        # SHA256 hashes for integrity verification
└── report.json            # Summary with action counts, sessions, status
```

To verify integrity after archiving:

```bash
cd evidence-2025-01-01T120000
sha256sum -c sha256.manifest
```

---

## Production Deployment

For production environments requiring compliance:

1. **Enable compliance mode** — set `compliance_mode: true` to prevent any tampering with audit logs
2. **Set retention policy** — configure `audit_retention_days` (default 365) and `audit_max_size_bytes` (default 10 MB)
3. **Schedule regular evidence exports** — run `opencrust audit evidence` via cron/systemd timer
4. **Archive evidence packages** to durable external storage (S3, Glacier, etc.)
5. **Monitor disk usage** — compliance mode prevents log rotation; ensure sufficient disk space

Example systemd timer for daily evidence export:

```
[Unit]
Description=OpenCrust daily audit evidence export

[Timer]
OnCalendar=daily
Persistent=true

[Install]
WantedBy=timers.target
```

---

## FAQ

**Does compliance mode affect performance?**  
No. The only change is that log rotation and cleanup are skipped. Append logging is the same fast path.

**Can I rotate logs manually in compliance mode?**  
No — compliance mode prevents all rotation and deletion. You must manage archival externally.

**What happens when the log file exceeds the disk?**  
OpenCrust will log an error and continue. The running process is not affected, but new entries may be lost. Monitor disk usage externally when compliance mode is enabled.

**Are evidence packages cryptographically signed?**  
Evidence packages include SHA256 manifests for integrity verification. For production use, we recommend signing the manifest with your own key management infrastructure.
