# Development Guide

## Provider Abstraction Layer

### Overview

The `providers/` module provides a generic trait-based abstraction for extensible integrations. Each integration type (desktop, notifications, file pickers, tools, plugins) has its own provider trait that extends the base `Provider` trait.

### Key Files

- `src/providers/mod.rs` - Base `Provider` trait and `ProviderRegistry<P>`
- `src/providers/desktop.rs` - `DesktopProvider` trait and `DefaultDesktopProvider`
- `src/providers/notifications.rs` - `NotificationProvider` trait
- `src/providers/file_picker.rs` - `FilePickerProvider` trait
- `src/providers/tool.rs` - `ToolProvider` trait
- `src/providers/plugin.rs` - `PluginProvider` trait and `PluginWrapper`

### Adding a New Provider Type

1. Create a new file in `src/providers/` (e.g., `my_provider.rs`)
2. Define a trait extending `Provider` with your specific methods
3. Implement the trait for your provider(s)
4. Create a type alias for the registry: `type MyProviderRegistry = ProviderRegistry<dyn MyProvider>;`
5. Add a constructor function (e.g., `default_my_registry()`)
6. Export the module in `src/providers/mod.rs`

### Example: Custom Provider

```

## Adding New Dependencies and Feature Flags

When adding a new crate dependency or a Cargo feature, follow these steps to keep the project consistent and the documentation up‑to‑date:

1. **Add the dependency** in the appropriate `Cargo.toml` (workspace root for shared crates, or the specific crate for crate‑local usage). Use the `dep:` syntax for optional dependencies and enable them via a feature flag if they are not required for the default build.
2. **Expose the feature** in the crate’s `Cargo.toml` under `[features]`. Name the feature after what it enables (e.g., `macos-menu-bar`).
3. **Guard platform‑specific code** with `#[cfg(feature = "macos-menu-bar")]` or `#[cfg(target_os = "macos")]` as appropriate.
4. **Update documentation**:
   - Add a brief description of the new feature to the README’s *Installation* section (the `cargo install --features …` example already shows the `macos-menu-bar` flag).
   - Document the purpose of the new dependency in this guide, explaining why it is needed and any runtime implications.
   - If the dependency introduces new configuration options, add them to `docs/CONFIGURATION.md` under the relevant section.
5. **Run the full test suite** with `cargo test --all-features` to ensure the new feature does not break existing functionality.
6. **Run clippy and fmt** to keep code quality standards.

### Example: Adding the `dirs` crate

- **Dependency**: `dirs = "6.0.0"` (already declared in the workspace). It provides cross‑platform standard directories for configuration, cache, and data files.
- **Feature flag**: None needed; the crate is always compiled because it is used by the core `opencrust` crate.
- **Usage**: Import with `use dirs::config_dir;` and use the returned `PathBuf` to locate `~/.config/opencrust`.
- **Documentation**: Mentioned in this section and referenced in `docs/CONFIGURATION.md` where the config file location is described.

Following this checklist ensures new dependencies and features are introduced cleanly, with proper documentation and test coverage.
```

## Adding New Dependencies and Feature Flags

When adding a new crate dependency or a Cargo feature, follow these steps to keep the project consistent and the documentation up‑to‑date:

1. **Add the dependency** in the appropriate `Cargo.toml` (workspace root for shared crates, or the specific crate for crate‑local usage). Use the `dep:` syntax for optional dependencies and enable them via a feature flag if they are not required for the default build.
2. **Expose the feature** in the crate’s `Cargo.toml` under `[features]`. Name the feature after what it enables (e.g., `macos-menu-bar`).
3. **Guard platform‑specific code** with `#[cfg(feature = "macos-menu-bar")]` or `#[cfg(target_os = "macos")]` as appropriate.
4. **Update documentation**:
   - Add a brief description of the new feature to the README’s *Installation* section (the `cargo install --features …` example already shows the `macos-menu-bar` flag).
   - Document the purpose of the new dependency in this guide, explaining why it is needed and any runtime implications.
   - If the dependency introduces new configuration options, add them to `docs/CONFIGURATION.md` under the relevant section.
5. **Run the full test suite** with `cargo test --all-features` to ensure the new feature does not break existing functionality.
6. **Run clippy and fmt** to keep code quality standards.

### Example: Adding the `dirs` crate

- **Dependency**: `dirs = "6.0.0"` (already declared in the workspace). It provides cross‑platform standard directories for configuration, cache, and data files.
- **Feature flag**: None needed; the crate is always compiled because it is used by the core `opencrust` crate.
- **Usage**: Import with `use dirs::config_dir;` and use the returned `PathBuf` to locate `~/.config/opencrust`.
- **Documentation**: Mentioned in this section and referenced in `docs/CONFIGURATION.md` where the config file location is described.

Following this checklist ensures new dependencies and features are introduced cleanly, with proper documentation and test coverage.rust

```
// src/providers/my_provider.rs
use crate::providers::Provider;

pub trait MyProvider: Provider {
    fn my_method(&self) -> String;
}

pub struct MyProviderImpl;

impl Provider for MyProviderImpl {
    fn id(&self) -> &str { "my_provider" }
    fn name(&self) -> &str { "My Provider" }
    fn is_available(&self) -> bool { true }
    fn priority(&self) -> u8 { 50 }
}

impl MyProvider for MyProviderImpl {
    fn my_method(&self) -> String { "Hello".to_string() }
}

pub type MyProviderRegistry = crate::providers::ProviderRegistry<dyn MyProvider>;

pub fn default_my_registry() -> MyProviderRegistry {
    let mut registry = MyProviderRegistry::new();
    registry.register(Box::new(MyProviderImpl));
    registry
}
```

---

## Mode Handlers Implementation

### Overview
The mode handlers system allows different UI states (Normal, Insert, Review, etc.) to handle key events independently. Each mode has its own handler module implementing the `ModeHandler` trait.

### Key Files
- `src/event_loop/modes/types.rs` - Defines `ModeHandler` trait and `ModeAction` enum
- `src/event_loop/modes/mod.rs` - Routes key events to appropriate handler
- Individual handler modules (normal.rs, insert.rs, etc.)

### Implementation Steps
1. Create new handler module in `event_loop/modes/`
2. Implement `ModeHandler` trait with `handle_key()` method
3. Add mode to `dispatch_mode()` router in `mod.rs`
4. Update keybindings in `ui.rs` if needed
5. Add tests in `tests.rs` for new mode

### Example: Insert Mode

```rust
// src/event_loop/modes/insert.rs
use crate::event_loop::modes::types::ModeHandler;

pub struct InsertHandler;

impl ModeHandler for InsertHandler {
    fn handle_key(&mut self, app: &mut App, key: KeyEvent, ctx: &mut HandlerContext) -> ModeAction {
        // Handle insert mode key events
        match key.code {
            KeyCode::Char('i') => ModeAction::Continue,
            KeyCode::Char('e') => ModeAction::SwitchMode(Mode::Normal),
            _ => ModeAction::Continue
        }
    }
}
```
