use std::rc::Rc;

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Create a centered popup area within the given rectangle.
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Adaptive popup sizing for small terminals
    let adjusted_percent_x = if r.width < 50 { 
        // Very narrow terminal: use full width with small margins
        if r.width < 30 { 90 } else { 80 } 
    } else if r.width < 80 {
        // Narrow terminal: reduce width
        percent_x.saturating_sub(10)
    } else {
        percent_x
    };
    
    let adjusted_percent_y = if r.height < 20 {
        // Short terminal: use more vertical space
        if r.height < 15 { 85 } else { 75 }
    } else if r.height < 30 {
        // Moderately short terminal: slight reduction
        percent_y.saturating_sub(10)
    } else {
        percent_y
    };

    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - adjusted_percent_y) / 2),
                Constraint::Percentage(adjusted_percent_y),
                Constraint::Percentage((100 - adjusted_percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - adjusted_percent_x) / 2),
                Constraint::Percentage(adjusted_percent_x),
                Constraint::Percentage((100 - adjusted_percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

/// Build the main vertical layout: Tabs, Content, Input, Status.
pub fn main_layout(area: Rect) -> Rc<[Rect]> {
    // Adaptive layout based on terminal height
    let tabs_height = if area.height < 20 { 2 } else { 3 };
    let input_height = if area.height < 20 { 2 } else { 3 };
    let status_height = 1;
    
    Layout::default()
        .direction(Direction::Vertical)
        .margin(if area.height < 20 { 0 } else { 1 })
        .constraints(
            [
                Constraint::Length(tabs_height), // Tabs
                Constraint::Min(1),              // Main content
                Constraint::Length(input_height), // Input
                Constraint::Length(status_height), // Status
            ]
            .as_ref(),
        )
        .split(area)
}

/// Split a content area into sidebar + main, returning (sidebar, main).
/// When show is false, sidebar has zero width.
pub fn sidebar_layout(area: Rect, show: bool) -> (Rect, Rect) {
    // Adaptive sidebar width based on terminal width
    let sidebar_width = if area.width < 80 {
        // Narrow terminal: smaller sidebar or hidden
        if show && area.width >= 60 { 15 } else { 0 }
    } else if area.width < 120 {
        // Medium terminal: proportional sidebar
        if show { 20 } else { 0 }
    } else {
        // Wide terminal: full sidebar
        if show { 25 } else { 0 }
    };
    
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(sidebar_width), Constraint::Min(0)].as_ref())
        .split(area);
    (chunks[0], chunks[1])
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
        // With adaptive sizing for 100x100 terminal, should be close to 50% but may vary slightly
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
        assert_eq!(sidebar.width, 20);
        assert_eq!(main.width, 80);
    }
}
