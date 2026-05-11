# OpenCrust Audit Memory

## Project Context

- **Language:** Rust 2024 edition
- **Build system:** Cargo
- **Key dependencies:** Ratatui 0.30.0, Tokio 1.52.1
- **Architecture:** Modular TUI platform for AI-powered coding tasks
- **Main modules:** ui.rs, llm.rs, tools.rs, mcp.rs, lsp.rs, skills.rs, rag.rs, permissions.rs, audit.rs, sessions.rs, desktop/

## Standards & Conventions

- Strict warnings-as-errors (clippy -D warnings)
- snake_case for functions/variables, PascalCase for types
- Result<T, Box<dyn std::error::Error>> for error handling
- Async code uses tokio::spawn, avoid blocking in async
- Tests via #[cfg(test)] modules
- Commits: phase-based naming or imperative descriptive titles

## Known Issues (To Track)

- *Will be populated during audit*

## Decisions

- *Will be populated during audit*
