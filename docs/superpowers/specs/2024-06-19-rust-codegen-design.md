# 2024-06-19 Rust Codegen Design

## 1. Crate Layout
```
opencrust-codegen/
├─ src/
│  ├─ lib.rs                // public API re‑exports
│  ├─ engine.rs             // wrapper around Tera & Handlebars
│  ├─ adapters/
│  │   ├─ csv.rs            // CSV → serde_json::Value
│  │   ├─ json.rs           // JSON file loader
│  │   ├─ http.rs           // reqwest GET/POST → serde_json::Value
│  │   └─ db.rs             // sqlx async queries → serde_json::Value
│  ├─ output.rs             // write_to(path, content) helper
│  └─ errors.rs             // CodegenError definition
├─ templates/               // bundled default templates (embedded)
├─ Cargo.toml
└─ README.md
```

## 2. Public API
```rust
/// Render a template with a JSON context.
pub fn generate(template_name: &str, ctx: &serde_json::Value) -> Result<String>;

/// Render a raw template string (useful for simple interpolation).
pub fn render_raw(template: &str, ctx: &serde_json::Value) -> Result<String>;

/// Write generated content to a file, creating parent directories as needed.
pub fn write_to<P: AsRef<Path>>(path: P, content: &str) -> Result<()>;
```
All functions return `anyhow::Result` with rich error context.

## 3. Template Handling
- **Bundled templates** are compiled into the binary via `include_str!` for fast access.
- **User‑provided templates** can be loaded from an absolute or relative path at runtime.
- **Engine selection**:
  - Full‑featured templates → **Tera** (supports loops, conditionals, filters).
  - Simple `${var}` interpolation → **Handlebars** (fallback for lightweight use‑cases).

## 4. Data‑Source Adapters
Each adapter implements:
```rust
pub trait DataSource {
    fn fetch(&self) -> Result<serde_json::Value>;
}
```
- **CSV** – reads a file, infers headers, returns an array of objects.
- **JSON** – loads any JSON file.
- **HTTP** – performs GET/POST with optional headers, returns parsed JSON.
- **DB** – async `sqlx` query → rows → JSON array.
Adapters are composable; a user can merge multiple sources into a single context before rendering.

## 5. Plugin Registration (OpenCrust integration)
Add an entry to `plugins/manifest.toml`:
```toml
[codegen]
module = "opencrust_codegen"
description = "Rust code generation library"
```
OpenCrust’s tool executor can then call:
```
/tool codegen.generate "my_template" "{ \"name\": \"Alice\" }"
```
The executor marshals the JSON string into `serde_json::Value` and returns the rendered code.

## 6. Error Handling
`CodegenError` enum (wrapped by `anyhow::Error`) covers:
- Template not found / parse error
- Data‑source failure (IO, network, DB)
- Rendering failure (type mismatch, missing variable)
All errors include the file/line where they originated for easy debugging.

## 7. Testing Strategy
- **Unit tests** for each adapter (mock CSV/JSON files, mock HTTP with `wiremock`).
- **Integration tests**: load a template, feed a combined context, assert exact output.
- **Snapshot tests** (`insta`) for generated code snippets to catch regressions.
- **CI step**: `cargo test && cargo check --examples` ensures generated code compiles.

## 8. Documentation
- `cargo doc --open` will generate API docs.
- User guide in `docs/codegen/` covering:
  - Adding templates
  - Using adapters
  - Plug‑in invocation from OpenCrust agents
  - Error handling patterns

## 9. CI Integration
Add to `.github/workflows/ci.yml`:
```yaml
- name: Codegen tests
  run: cargo test --package opencrust-codegen
- name: Verify generated code compiles
  run: |
    cargo run --bin codegen-example
    cargo check
```
---
