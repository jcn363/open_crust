use crossterm::event::{self, Event};
use std::time::Duration;

pub async fn next_event() -> Result<Option<Event>, std::io::Error> {
    if event::poll(Duration::from_millis(16))? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}
