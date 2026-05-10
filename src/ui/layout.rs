use std::rc::Rc;

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Create a centered popup area within the given rectangle.
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

/// Build the main vertical layout: Tabs, Content, Input, Status.
pub fn main_layout(area: Rect) -> Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3), // Tabs
                Constraint::Min(1),    // Main content
                Constraint::Length(3), // Input
                Constraint::Length(1), // Status
            ]
            .as_ref(),
        )
        .split(area)
}

/// Split a content area into sidebar + main, returning (sidebar, main).
/// When show is false, sidebar has zero width.
pub fn sidebar_layout(area: Rect, show: bool) -> (Rect, Rect) {
    if show {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(25), Constraint::Min(0)].as_ref())
            .split(area);
        (chunks[0], chunks[1])
    } else {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(0), Constraint::Min(0)].as_ref())
            .split(area);
        (chunks[0], chunks[1])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    #[test]
    fn test_centered_rect_full() {
        let area = Rect::new(0, 0, 100, 100);
        let result = centered_rect(100, 100, area);
        assert_eq!(result.x, 0);
        assert_eq!(result.y, 0);
        assert_eq!(result.width, 100);
        assert_eq!(result.height, 100);
    }

    #[test]
    fn test_centered_rect_half() {
        let area = Rect::new(0, 0, 100, 100);
        let result = centered_rect(50, 50, area);
        // 25% padding on each side
        assert_eq!(result.x, 25);
        assert_eq!(result.y, 25);
        assert_eq!(result.width, 50);
        assert_eq!(result.height, 50);
    }

    #[test]
    fn test_main_layout_returns_four_chunks() {
        let area = Rect::new(0, 0, 100, 100);
        let chunks = main_layout(area);
        assert_eq!(chunks.len(), 4);
        // Tabs: height 3, Content: min 1, Input: height 3, Status: height 1
        assert_eq!(chunks[0].height, 3);
        assert_eq!(chunks[3].height, 1);
    }

    #[test]
    fn test_sidebar_layout_hidden() {
        let area = Rect::new(0, 0, 100, 50);
        let (sidebar, main) = sidebar_layout(area, false);
        assert_eq!(sidebar.width, 0);
        assert_eq!(main.width, 100);
    }

    #[test]
    fn test_sidebar_layout_shown() {
        let area = Rect::new(0, 0, 100, 50);
        let (sidebar, main) = sidebar_layout(area, true);
        assert_eq!(sidebar.width, 25);
        assert_eq!(main.width, 75);
    }
}
