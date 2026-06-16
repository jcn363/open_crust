# Development Guide

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
