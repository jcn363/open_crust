//! Mission Control rendering

use crate::orchestrator::task::{Task, TaskState};
use crate::ui::ThemeContext;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};

use super::state::MissionControlUI;

impl MissionControlUI {
    /// Render the DAG panel (left side)
    pub(crate) fn render_dag_panel(&mut self, f: &mut Frame, area: Rect, theme: &ThemeContext) {
        if self.tasks.is_empty() || self.layers.is_empty() {
            let empty =
                Paragraph::new("No active task graph").style(Style::default().fg(theme.dim()));
            f.render_widget(empty, area);
            return;
        }

        // We render the DAG using individual Paragraph widgets positioned at
        // their computed node positions. Since ratatui doesn't have a canvas,
        // we use a Paragraph with pre-formatted lines for the visible area.

        let node_width: u16 = super::types::NODE_WIDTH;
        let node_height: u16 = super::types::NODE_HEIGHT;

        // Build visible area based on scroll_offset
        let vis_height = area.height;
        let vis_start = self.scroll_offset as u16;

        // Draw edges as unicode box-drawing characters (before nodes so nodes render on top)
        if !self.edges.is_empty() {
            // Create a buffer of characters for the DAG area
            let mut buf: Vec<Vec<char>> = (0..area.height as usize)
                .map(|_| vec![' '; area.width as usize])
                .collect();
            // Helper to set a char in buffer, ignoring out-of-bounds
            let mut set_char = |x: i16, y: i16, c: char| {
                if x >= 0 && y >= 0 && (x as u16) < area.width && (y as u16) < area.height {
                    buf[y as usize][x as usize] = c;
                }
            };
            // For each edge, draw a line from source to target
            for (from_idx, to_idx) in &self.edges {
                if *from_idx >= self.node_positions.len() || *to_idx >= self.node_positions.len() {
                    continue;
                }
                let src = &self.node_positions[*from_idx];
                let dst = &self.node_positions[*to_idx];
                // Compute center of source node (bottom center)
                let src_cx = src.x as i16 + (node_width as i16) / 2;
                let src_cy = src.y as i16 + (node_height as i16) - 1; // bottom
                // Compute center of target node (top center)
                let dst_cx = dst.x as i16 + (node_width as i16) / 2;
                let dst_cy = dst.y as i16; // top
                // Draw vertical line from src bottom to dst top
                let start_y = src_cy;
                let end_y = dst_cy;
                if start_y <= end_y {
                    for y in start_y..=end_y {
                        set_char(src_cx, y, '│');
                    }
                } else {
                    for y in end_y..=start_y {
                        set_char(src_cx, y, '│');
                    }
                }
                // If horizontal offset, draw horizontal line at dst top
                if src_cx != dst_cx {
                    let (left, right) = if src_cx < dst_cx {
                        (src_cx, dst_cx)
                    } else {
                        (dst_cx, src_cx)
                    };
                    for x in left..=right {
                        set_char(x, dst_cy, '─');
                    }
                    // Adjust corners
                    if src_cx < dst_cx {
                        set_char(src_cx, dst_cy, '┌');
                        set_char(dst_cx, dst_cy, '┐');
                    } else {
                        set_char(dst_cx, dst_cy, '└');
                        set_char(src_cx, dst_cy, '┘');
                    }
                }
            }
            // Convert buffer to string
            let mut edge_text = String::new();
            for row in buf.iter() {
                let row_str: String = row.iter().collect();
                edge_text.push_str(&row_str);
                edge_text.push('\n');
            }
            let edge_para = Paragraph::new(edge_text).style(Style::default().fg(theme.dim()));
            f.render_widget(edge_para, area);
        }

        // Render each task node (on top of edges)
        for (task_idx, task) in self.tasks.iter().enumerate() {
            if task_idx >= self.node_positions.len() {
                continue;
            }
            let pos = &self.node_positions[task_idx];

            // Skip nodes outside the visible vertical range
            if pos.y + node_height < vis_start || pos.y > vis_start + vis_height {
                continue;
            }

            // Compute the actual on-screen position (apply scroll offset)
            let screen_y = pos.y.saturating_sub(vis_start);

            // Clamp to area bounds
            if pos.x >= area.width || screen_y >= area.height {
                continue;
            }

            let node_area = Rect::new(
                area.x + pos.x.min(area.width.saturating_sub(node_width + 2)),
                area.y + screen_y.min(area.height.saturating_sub(node_height + 2)),
                node_width.min(area.width.saturating_sub(pos.x + 2)),
                node_height,
            );

            // Skip if rectangle is invalid
            if node_area.width < 3 || node_area.height < 2 {
                continue;
            }

            let (icon, state_color) = Self::task_style(task, theme);
            let is_selected = task_idx == self.selected_index && self.active_panel == 0;

            // Build node content
            let desc_short =
                Self::truncate(&task.description, (node_width as usize).saturating_sub(4));
            let agent_short =
                Self::truncate(&task.agent_type, (node_width as usize).saturating_sub(6));
            let content = format!(" {} {}\n {} {}", icon, desc_short, "⎔", agent_short);

            let border_style = if is_selected {
                Style::default()
                    .fg(state_color)
                    .add_modifier(ratatui::style::Modifier::BOLD)
            } else {
                Style::default().fg(state_color)
            };

            let node = Paragraph::new(content)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(border_style),
                )
                .style(Style::default().fg(state_color))
                .wrap(Wrap { trim: true });

            f.render_widget(node, node_area);
        }
    }

    /// Render the detail panel (right side)
    pub(crate) fn render_detail_panel(&mut self, f: &mut Frame, area: Rect, theme: &ThemeContext) {
        if self.tasks.is_empty() {
            let empty = Paragraph::new("Select a task to view details")
                .style(Style::default().fg(theme.dim()));
            f.render_widget(empty, area);
            return;
        }

        // Split detail panel: top 60% task details, bottom 40% dashboard
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        // --- Task Details (top) ---
        let selected = self.selected_index.min(self.tasks.len().saturating_sub(1));
        let task = &self.tasks[selected];
        let (icon, _state_color) = Self::task_style(task, theme);

        let state_str = match &task.state {
            TaskState::Pending => "Pending".to_string(),
            TaskState::Running { agent_id } => format!("Running ({})", agent_id),
            TaskState::Completed { output } => {
                let truncated = Self::truncate(output, 80);
                format!("Completed: {}", truncated)
            }
            TaskState::Failed { error } => {
                let truncated = Self::truncate(error, 80);
                format!("Failed: {}", truncated)
            }
        };

        // Description [truncated to fit]
        let desc_display = Self::truncate(&task.description, 90);

        // Dependencies
        let dep_count = task.dependencies.len();
        let dep_info = if dep_count == 0 {
            "  No dependencies".to_string()
        } else {
            let dep_ids: Vec<String> = task
                .dependencies
                .iter()
                .map(|id| {
                    // Try to find the task name by looking it up
                    self.tasks
                        .iter()
                        .find(|t| t.id == *id)
                        .map(|t| Self::truncate(&t.description, 20))
                        .unwrap_or_else(|| id.to_string()[..8].to_string())
                })
                .collect();
            format!("  Depends on: {}", dep_ids.join(", "))
        };

        // Dependents (tasks that depend on this one)
        let dependents: Vec<&Task> = self
            .tasks
            .iter()
            .filter(|t| t.dependencies.contains(&task.id))
            .collect();
        let dependents_info = if dependents.is_empty() {
            "  No dependents".to_string()
        } else {
            let dep_names: Vec<String> = dependents
                .iter()
                .map(|t| Self::truncate(&t.description, 20))
                .collect();
            format!("  Blocking: {}", dep_names.join(", "))
        };

        let detail_text = format!(
            " {} {}\n\n\
             Agent: {}\n\
             State: {}\n\n\
             {}\n\
             {}\n\n\
             {}",
            icon,
            desc_display,
            task.agent_type,
            state_str,
            dep_info,
            dependents_info,
            if dep_count > 0 {
                format!("({} dependencies)", dep_count)
            } else {
                String::new()
            },
        );

        let detail_block = if self.active_panel == 1 {
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Task #{}", selected))
                .border_style(Style::default().fg(theme.accent))
        } else {
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Task #{}", selected))
        };

        let detail = Paragraph::new(detail_text)
            .block(detail_block)
            .style(Style::default().fg(theme.fg))
            .wrap(Wrap { trim: true });
        f.render_widget(detail, chunks[0]);

        // --- Dashboard (bottom) ---
        let s = &self.stats;

        let total_bar = if s.total > 0 {
            s.completed as f64 / s.total as f64
        } else {
            0.0
        };

        let dash_text = format!(
            " Total: {}  Pending: {}  Running: {}\n\
             Completed: {}  Failed: {}\n\
             Progress: [{:<20}] {:.0}%",
            s.total,
            s.pending,
            s.running,
            s.completed,
            s.failed,
            "█".repeat((total_bar * 20.0) as usize),
            total_bar * 100.0,
        );

        let dash = Paragraph::new(dash_text)
            .block(Block::default().borders(Borders::ALL).title("Dashboard"))
            .style(Style::default().fg(theme.fg));
        f.render_widget(dash, chunks[1]);
    }

    /// Render the agent panel
    pub(crate) fn render_agent_panel(&mut self, f: &mut Frame, area: Rect, theme: &ThemeContext) {
        let agents = &self.agent_panel.agents;
        let selected = self.agent_panel.selected;
        let show_logs = self.agent_panel.show_logs;

        if agents.is_empty() {
            let empty = Paragraph::new("No background agents\n\nPress 'n' to start a new agent")
                .style(Style::default().fg(theme.dim()))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Background Agents ")
                        .border_style(Style::default().fg(theme.accent)),
                );
            f.render_widget(empty, area);
            return;
        }

        // Split area: list on left, logs on right (if show_logs)
        let chunks = if show_logs {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area)
        } else {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(100)])
                .split(area)
        };

        // --- Agent List (left) ---
        let list_area = chunks[0];
        let list_block = Block::default()
            .borders(Borders::ALL)
            .title(" Background Agents ")
            .border_style(if self.active_panel == 2 {
                Style::default().fg(theme.accent)
            } else {
                Style::default().fg(theme.fg)
            });
        let list_inner = list_block.inner(list_area);
        f.render_widget(list_block, list_area);

        // Build agent list text
        let mut list_text = String::new();
        for (idx, agent) in agents.iter().enumerate() {
            let is_selected = idx == selected && self.active_panel == 2;
            let status_icon = match &agent.status {
                crate::background_agents::AgentStatus::Pending => "⏳",
                crate::background_agents::AgentStatus::Running => "▶",
                crate::background_agents::AgentStatus::Completed { .. } => "✅",
                crate::background_agents::AgentStatus::Failed { .. } => "❌",
                crate::background_agents::AgentStatus::Cancelled => "⏸",
            };
            let _status_color = match &agent.status {
                crate::background_agents::AgentStatus::Pending => theme.fg,
                crate::background_agents::AgentStatus::Running => theme.warning(),
                crate::background_agents::AgentStatus::Completed { .. } => theme.success(),
                crate::background_agents::AgentStatus::Failed { .. } => theme.error(),
                crate::background_agents::AgentStatus::Cancelled => theme.dim(),
            };

            let name = Self::truncate(&agent.name, 20);
            let progress = format!("{}%", agent.progress);
            let line = format!(
                " {} {:<20} {:>4} {}",
                status_icon, name, progress, agent.provider
            );

            if is_selected {
                list_text.push_str(&format!("► {}\n", line));
            } else {
                list_text.push_str(&format!("  {}\n", line));
            }
        }

        let list_style = if self.active_panel == 2 {
            Style::default().fg(theme.fg)
        } else {
            Style::default().fg(theme.dim())
        };
        let list_para = Paragraph::new(list_text)
            .style(list_style)
            .wrap(Wrap { trim: true });
        f.render_widget(list_para, list_inner);

        // --- Agent Logs (right, if show_logs) ---
        if show_logs && selected < agents.len() {
            let log_area = chunks[1];
            let agent = &agents[selected];
            let log_block = Block::default()
                .borders(Borders::ALL)
                .title(format!(" Logs: {} ", agent.name))
                .border_style(Style::default().fg(theme.accent));
            let log_inner = log_block.inner(log_area);
            f.render_widget(log_block, log_area);

            let log_text = if agent.log.is_empty() {
                "No logs yet".to_string()
            } else {
                agent.log.join("\n")
            };
            let log_para = Paragraph::new(log_text)
                .style(Style::default().fg(theme.fg))
                .wrap(Wrap { trim: true });
            f.render_widget(log_para, log_inner);
        }
    }

    /// Main render function
    pub fn render(&mut self, f: &mut Frame, area: Rect, theme: &ThemeContext) {
        // Handle agent panel (full screen)
        if self.active_panel == 2 {
            let agent_block = Block::default()
                .borders(Borders::ALL)
                .title(" Background Agents ")
                .border_style(Style::default().fg(theme.accent));
            let agent_inner = agent_block.inner(area);
            f.render_widget(agent_block, area);
            self.render_agent_panel(f, agent_inner, theme);
            return;
        }

        // Normal DAG + Detail layout
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        // DAG panel (left 60%)
        let dag_border_style = if self.active_panel == 0 {
            Style::default().fg(theme.accent)
        } else {
            Style::default().fg(theme.fg)
        };
        let dag_block = Block::default()
            .borders(Borders::ALL)
            .title(" Task Graph ")
            .border_style(dag_border_style)
            .style(Style::default().fg(theme.accent));
        let dag_inner = dag_block.inner(chunks[0]);
        f.render_widget(dag_block, chunks[0]);

        if self.tasks.is_empty() {
            let empty =
                Paragraph::new("No active task graph").style(Style::default().fg(theme.dim()));
            f.render_widget(empty, dag_inner);
        } else {
            self.render_dag_panel(f, dag_inner, theme);
        }

        // Detail panel (right 40%)
        let detail_border_style = if self.active_panel == 1 {
            Style::default().fg(theme.accent)
        } else {
            Style::default().fg(theme.fg)
        };
        let detail_block = Block::default()
            .borders(Borders::ALL)
            .title(" Details ")
            .border_style(detail_border_style);
        let detail_inner = detail_block.inner(chunks[1]);
        f.render_widget(detail_block, chunks[1]);

        if self.tasks.is_empty() {
            let empty = Paragraph::new("Select a task to view details")
                .style(Style::default().fg(theme.dim()));
            f.render_widget(empty, detail_inner);
        } else {
            self.render_detail_panel(f, detail_inner, theme);
        }
    }
}
