//! Vim mode cursor movement and editing helpers

use crate::app::App;

impl App {
    // Vim Mode helper methods
    pub fn move_cursor_left(&mut self) {
        if self.vim_cursor_pos > 0 {
            self.vim_cursor_pos -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        let char_count = self.input.chars().count();
        if self.vim_cursor_pos < char_count {
            self.vim_cursor_pos += 1;
        }
    }

    pub fn move_to_next_word(&mut self) {
        let chars: Vec<char> = self.input.chars().collect();
        let mut pos = self.vim_cursor_pos;
        // Skip current word
        while pos < chars.len() && !chars[pos].is_whitespace() {
            pos += 1;
        }
        // Skip whitespace
        while pos < chars.len() && chars[pos].is_whitespace() {
            pos += 1;
        }
        self.vim_cursor_pos = pos.min(chars.len());
    }

    pub fn move_to_prev_word(&mut self) {
        let chars: Vec<char> = self.input.chars().collect();
        let mut pos = self.vim_cursor_pos;
        // Skip whitespace backwards
        while pos > 0 && chars[pos - 1].is_whitespace() {
            pos -= 1;
        }
        // Skip current word backwards
        while pos > 0 && !chars[pos - 1].is_whitespace() {
            pos -= 1;
        }
        self.vim_cursor_pos = pos;
    }

    pub fn move_to_line_start(&mut self) {
        self.vim_cursor_pos = 0;
    }

    pub fn move_to_line_end(&mut self) {
        self.vim_cursor_pos = self.input.chars().count();
    }

    pub fn delete_line(&mut self) {
        self.input.clear();
        self.vim_cursor_pos = 0;
    }

    pub fn yank_line(&self, clipboard: &mut crate::clipboard::ClipboardManager) -> bool {
        clipboard.copy(&self.input)
    }
}
