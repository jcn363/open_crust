//! Mode switching helpers

use crate::app::App;

impl App {
    pub fn enter_insert_mode(&mut self) {
        self.mode = crate::app::Mode::Insert;
    }

    pub fn enter_normal_mode(&mut self) {
        self.mode = crate::app::Mode::Normal;
    }
}
