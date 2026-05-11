date: 2026-05-06
topic: LinuxMint Cinnamon GUI/UX Support
status: validated

## Problem Statement

OpenCrust currently operates as a pure TUI (Terminal UI) application using Ratatui. Users on LinuxMint Cinnamon desktop environment may want **desktop integration capabilities** while maintaining the terminal-first experience. This design adds optional desktop features like system notifications, native file pickers, and desktop environment awareness.

**What we're solving:**
- Need for system notifications when long-running tasks complete
- Desire for native file selection dialogs (instead of terminal prompts)
- Appreciation for desktop theme awareness (light/dark mode)
- All while preserving the TUI's portability and cross-platform compatibility

**Constraints:**
- TUI must remain the primary interface
- All desktop features must be opt-in and optional
- Graceful degradation if Cinnamon-specific tools unavailable
- No breaking changes to existing platform support

---

## Constraints

| Constraint | Impact |
|------------|--------|
| TUI-first interface | Desktop features are overlays, not replacements |
| Cross-platform support | Features must fail gracefully on non-Linux platforms |
| Security requirements | No privilege escalation for desktop integrations |
| No runtime dependencies | All new dependencies must be optional/yanked |
| Rust 2024 edition | Must compile with strict warnings-as-errors |

---

## Approach

Create a **desktop integration module** that provides Cinnamon-specific enhancements while keeping the core TUI architecture intact.

**Architecture pattern:**
```
TUI Application → Desktop Integration Module → Platform-Specific Helper → System
     ↓                                      ↓
  (user action)                        (notify-send, zenity, etc.)
```

**Implementation strategy:**
1. **Desktop detection module** - Identify Cinnamon environment
2. **Notification system** - System notifications for task completion
3. **File picker integration** - Native file selection (optional)
4. **Theme awareness** - Light/dark mode detection for future use

**Why this approach:**
- Minimal code changes to existing architecture
- Clear separation of concerns (desktop module is self-contained)
- Easy to extend for other desktop environments later
- Users can opt-in/out via configuration

---

## Architecture

```
opencrust/
├── src/
│   ├── desktop/
│   │   ├── mod.rs              # Module entry, exports
│   │   ├── detection.rs        # Desktop environment detection
│   │   ├── notifications.rs    # System notification support
│   │   └── file_picker.rs      # Native file dialog (optional)
│   ├── app.rs                  # (unchanged - TUI)
│   ├── ui.rs                   # (unchanged - Ratatui rendering)
│   └── config.rs               # (extends with desktop config)
```

**Component dependencies:**
```
Desktop Module
     ↓ (optional dependencies)
notify-rust (Linux notifications)
     ↓ (optional dependencies)
zenity/yad (file picker - via shell commands)
```

**No new mandatory dependencies** - all desktop features use existing system tools or optional crates.

---

## Components

### Desktop Detection Module (`desktop/detection.rs`)

**Responsibility:** Identify if running on Cinnamon desktop

**Functions:**
- `is_cinnamon() -> bool` - Checks environment variables and process list
- `desktop_env() -> DesktopEnv` - Returns enum variant for current DE

**Implementation:**
```rust
// Checks XDG_CURRENT_DESKTOP, XDG_SESSION_DESKTOP
// Falls back to ps check for "cinnamon" process
// Returns DesktopEnv::{Cinnamon, Gnome, KDE, Unknown}
```

### Notifications Module (`desktop/notifications.rs`)

**Responsibility:** Send system notifications to desktop

**Functions:**
- `send_notification(title: &str, message: &str) -> Result<(), NotificationError>`
- `notification_available() -> bool` - Check if notification system works

**Implementation:**
- Linux: Uses `notify-send` via shell or `notify-rust` crate
- Fallback: No-op if notification system unavailable
- Configurable via `desktop.notifications = true/false`

### File Picker Module (`desktop/file_picker.rs`)

**Responsibility:** Provide native file selection dialog

**Functions:**
- `pick_file() -> Option<PathBuf>` - Returns selected file path
- `file_picker_available() -> bool` - Check if zenity/yad available

**Implementation:**
- Uses `zenity --file-selection` or `yad --file-selection`
- Fallback:Terminal prompt via `App` if unavailable
- Configurable via `desktop.file_picker = true/false`

---

## Data Flow

```
TUI Event (e.g., "Save As" or task completion)
     ↓
Desktop Integration Module
     ↓
Check config[desktop.feature] → enabled?
     ↓ (yes)
Platform-Specific Implementation
     ↓
System Calls (notify-send, zenity, etc.)
     ↓
User sees native Cinnamon dialog/notification
```

**Fallback path (if feature unavailable):**
```
TUI Event → Desktop Module → Feature not available
     ↓
Continue with TUI behavior (no interruption)
```

---

## Error Handling Strategy

| Error Type | Handling |
|------------|----------|
| Desktop feature not available | Silently fall back to TUI |
| Notification system disabled | No-op, app continues |
| zenity/yad not installed | Terminal prompt instead |
| Shell command failure | Log to stderr, continue |

**Key principle:** Never block or fail the application due to desktop integration issues.

---

## Testing Strategy

**Unit Tests:**
- `desktop/detection.rs::test_is_cinnamon()` - Verify detection logic
- `desktop/notifications.rs::test_send_notification()` - Verify no panic
- `desktop/file_picker.rs::test_file_picker_available()` - Check for tools

**Integration Tests:**
- Run on Cinnamon environment, verify notification appears
- Test with zenity installed/uninstalled
- Test on non-Linux platform (should no-op gracefully)

**CI/CD:**
- Run on standard Linux without desktop (no breakage)
- Optional desktop integration tests on LinuxMint runner (future)

---

## Open Questions

| Question | Resolution |
|----------|------------|
| Should we add `notify-rust` as optional dependency? | Yes, with `desktop-notifications` feature flag |
| How to handle light/dark theme detection? | For future enhancement; detect via Cinnamon settings |
| Should file picker be enabled by default? | No - opt-in only via config |

---

## Implementation Plan

**Phase 1: Core module**
1. Create `src/desktop/mod.rs` - Module structure
2. Create `src/desktop/detection.rs` - Environment detection
3. Add `DesktopEnv` enum to config.rs

**Phase 2: Notifications**
4. Create `src/desktop/notifications.rs` - System notification support
5. Add notification feature to `config.rs` (enabled by default on Linux)

**Phase 3: File picker** (optional)
6. Create `src/desktop/file_picker.rs` - Zenity wrapper
7. Add file picker feature to config.rs

**Phase 4: Integration**
8. Wire desktop module into `main.rs` or `app.rs` as needed
9. Test on Cinnamon environment

**Phase 5: Documentation**
10. Update AGENTS.md with desktop module info
11. Add config.example.json example for desktop features
