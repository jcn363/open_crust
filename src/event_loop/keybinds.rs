use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Check if a key event matches a keybind string (e.g., "ctrl+c", "ctrl+shift+k")
pub fn check_key_match(key: &KeyEvent, keybind_str: &str) -> bool {
    for combo in keybind_str.split(',') {
        let mut target_modifiers = KeyModifiers::empty();
        let mut target_code = None;

        for part in combo.trim().split('+') {
            match part.to_lowercase().as_str() {
                "ctrl" => target_modifiers.insert(KeyModifiers::CONTROL),
                "alt" => target_modifiers.insert(KeyModifiers::ALT),
                "shift" => target_modifiers.insert(KeyModifiers::SHIFT),
                "return" | "enter" => target_code = Some(KeyCode::Enter),
                "backspace" => target_code = Some(KeyCode::Backspace),
                "delete" => target_code = Some(KeyCode::Delete),
                "esc" | "escape" => target_code = Some(KeyCode::Esc),
                "up" => target_code = Some(KeyCode::Up),
                "down" => target_code = Some(KeyCode::Down),
                "left" => target_code = Some(KeyCode::Left),
                "right" => target_code = Some(KeyCode::Right),
                c if c.len() == 1 => target_code = c.chars().next().map(KeyCode::Char),
                _ => {}
            }
        }

        if let Some(code) = target_code
            && key.code == code
            && key.modifiers.contains(target_modifiers)
        {
            return true;
        }
    }
    false
}