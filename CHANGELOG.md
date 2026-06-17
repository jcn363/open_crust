# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.2.0] - 2026-06-17

### Added
- **PTY/TTY Stability Improvements**: Enhanced interactive PTY session handling with robust process management, improved stdin/stdout/stderr piping, and reliable session cleanup on drop. Added comprehensive test coverage for PTY spawn, write, and lifecycle operations.
- **Documentation Site Foundation**: Added Zola-based documentation site structure (`site/`) with configuration for OpenCrust documentation at https://opencrust.github.io/opencrust/. Includes navigation, search, theming, and SEO configuration.
- **Audit Export UI Polish**: Improved audit export command with better formatting, CSV/JSON/Syslog format support, and enhanced evidence package generation with SHA256 manifests and chain-of-custody logging.
- **Desktop Integration Dependencies**: Added `ssh2`, `nix`, and `libc` dependencies for enhanced desktop integration capabilities including SSH support and native file picker/notification backends.

### Changed
- Updated `ssh2` dependency from 0.10 to 0.9 for improved compatibility
- Updated `nix` to 0.28 with proper feature configuration
- Enhanced audit log redaction with JWT token detection and Bearer token handling

### Fixed
- PTY session cleanup now properly joins reader/stderr threads on drop
- Audit export handles empty entries gracefully
- Compliance evidence package verification includes all exported audit.csv and.json exports

### Security
- Audit log redaction now covers JWT tokens (three-part base64) and Authorization Bearer tokens
- Compliance mode enforces append-only audit trails (no rotation/deletion)

## [1.0.0] - 2026-06-17

### Added
- Initial release of OpenCrust v1.0.0
- Production TUI platform for AI-powered coding
- MCP/LSP integration with JSON-RPC communication
- Multi-agent orchestration with background agent dashboard
- Semantic code search with RAG
- Granular permissions and network gating
- Persistent audit logging with CSV/JSON export
- Enterprise compliance packaging (SOC2, HIPAA, GDPR, SOX)
- Custom tools and skills system
- Session management with fork/merge
- Token budget management and cost control
- Desktop integration (notifications, file picker, theme detection)

### Security
- Hardcoded key removal
- Command injection prevention
- Path validation and sanitization
- Audit log redaction for sensitive data

## [0.2.0] - 2026-05-15

### Added
- Plugin/extension system with dynamic loading
- Multi-repository support
- Citation system for plugins
- Background agent dashboard UI
- Enterprise compliance packaging

## [0.1.3.1] - 2026-05-01

### Fixed
- Critical security vulnerabilities (DAN bypass, command injection, arg sanitization)
- Performance issues (cache eviction, fuzzy matching, context summarization)

## [0.1.3] - 2026-04-20

### Added
- Auto-formatters with 20+ language support
- Custom commands from `.opencrust/commands/`
- @ file fuzzy search picker

## [0.1.2] - 2026-04-10

### Added
- Comprehensive GUI/UX audit and fixes
- OpenCode dark theme design system
- Plugin subsystem integration

## [0.1.1] - 2026-04-01

### Added
- Initial public release
- Core TUI with chat/tasks views
- LLM client with tool execution loop
- Basic MCP server integration
