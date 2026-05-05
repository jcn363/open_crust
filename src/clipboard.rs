use arboard::Clipboard;

pub struct ClipboardManager {
    clipboard: Option<Clipboard>,
}

impl ClipboardManager {
    pub fn new() -> Self {
        let clipboard = Clipboard::new().ok();
        Self { clipboard }
    }

    pub fn copy(&mut self, text: &str) -> bool {
        if let Some(ref mut cb) = self.clipboard {
            cb.set_text(text).is_ok()
        } else {
            false
        }
    }

    pub fn paste(&mut self) -> Option<String> {
        self.clipboard.as_mut().and_then(|cb| cb.get_text().ok())
    }
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new()
    }
}