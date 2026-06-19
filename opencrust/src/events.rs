//! Event polling loop for the TUI
//!
//! Thin wrapper around crossterm's event polling to provide an async-compatible
//! interface. Polls for keyboard and mouse events at ~60 Hz frame rate.

use crossterm::event::{self, Event};
use std::time::Duration;

pub fn next_event() -> Result<Option<Event>, std::io::Error> {
    if event::poll(Duration::from_millis(16))? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}
