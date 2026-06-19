//! macOS menu bar integration
//!
//! Native macOS menu bar with status icon, context menu, and agent controls.
//! Only compiled on macOS with the `macos-menu-bar` feature flag.
//!
//! # Architecture
//!
//! The menu bar runs in a dedicated thread using NSApplication's run loop.
//! Communication with the main app happens via channels:
//! - `MenuBarEvent` for outbound events (menu clicks → main app)
//! - `MenuBarCommand` for inbound commands (main app → menu bar updates)
//!
//! # Status Icon States
//!
//! - **Idle**: Gray icon (SF Symbol `brain.head.profile`)
//! - **Working**: Orange pulsing icon
//! - **Error**: Red icon
//! - **Agents**: Blue icon with count badge

#![cfg(target_os = "macos")]
#![cfg(feature = "macos-menu-bar")]

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use cocoa::appkit::{
    NSApplication, NSButton, NSEventMask, NSImage, NSMenu, NSMenuItem, NSRunLoop,
    NSStatusBar, NSStatusItem, NSVariableStatusItemLength,
};
use cocoa::base::{id, nil, NO, YES};
use cocoa::foundation::{NSAutoreleasePool, NSString};
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use objc::{class, msg_send, sel, sel_impl};

/// Menu bar events sent to the main application
#[derive(Debug, Clone)]
pub enum MenuBarEvent {
    /// User clicked "Show Main Window"
    ShowMainWindow,
    /// User clicked "Status" menu item
    ShowStatus,
    /// User clicked "Spawn Agent" menu item
    SpawnAgent,
    /// User clicked "Cancel Agent" with agent ID
    CancelAgent(String),
    /// User clicked "Quit" menu item
    Quit,
}

/// Commands sent from the main app to update the menu bar
#[derive(Debug, Clone)]
pub enum MenuBarCommand {
    /// Update status icon state
    SetStatus(MenuBarStatus),
    /// Update agent count in menu
    UpdateAgentCount(usize),
    /// Update agent list for cancellation
    UpdateAgents(Vec<AgentInfo>),
    /// Show notification badge
    ShowBadge(String),
    /// Hide notification badge
    HideBadge,
}

/// Status icon states
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuBarStatus {
    /// Idle state (gray icon)
    Idle,
    /// Working state (orange icon)
    Working,
    /// Error state (red icon)
    Error,
    /// Custom status with text
    Custom(String),
}

impl Default for MenuBarStatus {
    fn default() -> Self {
        Self::Idle
    }
}

/// Agent information for the menu
#[derive(Debug, Clone)]
pub struct AgentInfo {
    /// Agent unique identifier
    pub id: String,
    /// Agent name/description
    pub name: String,
    /// Whether agent is currently running
    pub running: bool,
}

/// Menu bar manager
pub struct MenuBarManager {
    /// Sender for events to main app
    event_tx: Sender<MenuBarEvent>,
    /// Receiver for commands from main app
    command_rx: Receiver<MenuBarCommand>,
    /// Status item reference
    status_item: Option<id>,
    /// Current status
    current_status: MenuBarStatus,
    /// Current agent count
    agent_count: usize,
    /// Current agent list
    agents: Vec<AgentInfo>,
}

impl MenuBarManager {
    /// Create a new menu bar manager
    pub fn new() -> (Self, Sender<MenuBarCommand>, Receiver<MenuBarEvent>) {
        let (event_tx, event_rx) = mpsc::channel();
        let (command_tx, command_rx) = mpsc::channel();

        let manager = Self {
            event_tx,
            command_rx,
            status_item: None,
            current_status: MenuBarStatus::Idle,
            agent_count: 0,
            agents: Vec::new(),
        };

        (manager, command_tx, event_rx)
    }

    /// Run the menu bar (blocking - call from a dedicated thread)
    pub fn run(&mut self) {
        unsafe {
            let pool = NSAutoreleasePool::new(nil);

            // Create status item
            let status_bar = NSStatusBar::systemStatusBar(nil);
            let status_item = status_bar.statusItemWithLength_(NSVariableStatusItemLength);

            // Configure status item
            self.setup_status_item(status_item);
            self.status_item = Some(status_item);

            // Start event loop
            self.run_event_loop();

            pool.drain();
        }
    }

    /// Setup the status item with icon and menu
    unsafe fn setup_status_item(&mut self, status_item: id) {
        // Set title (icon)
        let title = NSString::alloc(nil).init_str("🦀");
        let _: () = msg_send![status_item, setTitle:title];

        // Set tooltip
        let tooltip = NSString::alloc(nil).init_str("OpenCrust - AI Coding Agent");
        let _: () = msg_send![status_item, setToolTip:tooltip];

        // Create menu
        let menu = NSMenu::alloc(nil).init();
        self.build_menu(menu, status_item);

        // Attach menu
        let _: () = msg_send![status_item, setMenu:menu];
    }

    /// Build the context menu
    unsafe fn build_menu(&self, menu: id, status_item: id) {
        // Clear existing menu items
        let _: () = msg_send![menu, removeAllItems];

        // Show Main Window
        let show_item = NSMenuItem::alloc(nil)
            .initWithTitle_action_keyEquivalent_(
                NSString::alloc(nil).init_str("Show OpenCrust"),
                sel!(showMainWindow:),
                NSString::alloc(nil).init_str(""),
            );
        let _: () = msg_send![show_item, setTarget:self as *const Self as id];
        let _: () = msg_send![menu, addItem:show_item];

        // Separator
        let separator = NSMenuItem::separatorItem(nil);
        let _: () = msg_send![menu, addItem:separator];

        // Status section
        let status_header = NSMenuItem::alloc(nil)
            .initWithTitle_action_keyEquivalent_(
                NSString::alloc(nil).init_str("Status"),
                nil,
                NSString::alloc(nil).init_str(""),
            );
        let _: () = msg_send![status_header, setEnabled:NO];
        let _: () = msg_send![menu, addItem:status_header];

        // Status indicator
        let status_text = match &self.current_status {
            MenuBarStatus::Idle => "● Idle",
            MenuBarStatus::Working => "● Working...",
            MenuBarStatus::Error => "● Error",
            MenuBarStatus::Custom(text) => text.as_str(),
        };
        let status_item = NSMenuItem::alloc(nil)
            .initWithTitle_action_keyEquivalent_(
                NSString::alloc(nil).init_str(status_text),
                nil,
                NSString::alloc(nil).init_str(""),
            );
        let _: () = msg_send![status_item, setEnabled:NO];
        let _: () = msg_send![menu, addItem:status_item];

        // Separator
        let separator = NSMenuItem::separatorItem(nil);
        let _: () = msg_send![menu, addItem:separator];

        // Agents section
        let agents_header = NSMenuItem::alloc(nil)
            .initWithTitle_action_keyEquivalent_(
                NSString::alloc(nil).init_str(&format!("Agents ({})", self.agent_count)),
                nil,
                NSString::alloc(nil).init_str(""),
            );
        let _: () = msg_send![agents_header, setEnabled:NO];
        let _: () = msg_send![menu, addItem:agents_header];

        // Spawn Agent
        let spawn_item = NSMenuItem::alloc(nil)
            .initWithTitle_action_keyEquivalent_(
                NSString::alloc(nil).init_str("Spawn Agent"),
                sel!(spawnAgent:),
                NSString::alloc(nil).init_str(""),
            );
        let _: () = msg_send![spawn_item, setTarget:self as *const Self as id];
        let _: () = msg_send![menu, addItem:spawn_item];

        // Agent list (if any)
        if !self.agents.is_empty() {
            let separator = NSMenuItem::separatorItem(nil);
            let _: () = msg_send![menu, addItem:separator];

            for agent in &self.agents {
                let agent_name = format!(
                    "{}{}",
                    if agent.running { "● " } else { "○ " },
                    agent.name
                );
                let agent_item = NSMenuItem::alloc(nil)
                    .initWithTitle_action_keyEquivalent_(
                        NSString::alloc(nil).init_str(&agent_name),
                        sel!(cancelAgent:),
                        NSString::alloc(nil).init_str(""),
                    );
                let _: () = msg_send![agent_item, setTarget:self as *const Self as id];
                let _: () = msg_send![agent_item, setRepresentedObject:NSString::alloc(nil).init_str(&agent.id)];
                let _: () = msg_send![menu, addItem:agent_item];
            }
        }

        // Separator
        let separator = NSMenuItem::separatorItem(nil);
        let _: () = msg_send![menu, addItem:separator];

        // Quit
        let quit_item = NSMenuItem::alloc(nil)
            .initWithTitle_action_keyEquivalent_(
                NSString::alloc(nil).init_str("Quit OpenCrust"),
                sel!(quitApp:),
                NSString::alloc(nil).init_str("q"),
            );
        let _: () = msg_send![quit_item, setTarget:self as *const Self as id];
        let _: () = msg_send![menu, addItem:quit_item];
    }

    /// Run the event loop
    unsafe fn run_event_loop(&mut self) {
        let app = NSApplication::sharedApplication(nil);
        let run_loop = NSRunLoop::currentRunLoop(nil);

        loop {
            // Check for commands from main app
            while let Ok(command) = self.command_rx.try_recv() {
                self.handle_command(command);
            }

            // Process events
            let date = cocoa::foundation::NSDate::distantPast(nil);
            let _: () = msg_send![run_loop, runMode:NSDefaultRunLoopMode beforeDate:date];

            // Small sleep to prevent CPU spinning
            thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    /// Handle a command from the main app
    unsafe fn handle_command(&mut self, command: MenuBarCommand) {
        match command {
            MenuBarCommand::SetStatus(status) => {
                self.current_status = status;
                self.update_icon();
            }
            MenuBarCommand::UpdateAgentCount(count) => {
                self.agent_count = count;
            }
            MenuBarCommand::UpdateAgents(agents) => {
                self.agents = agents;
            }
            MenuBarCommand::ShowBadge(text) => {
                // Update title with badge
                let title = format!("🦀 {}", text);
                let title_str = NSString::alloc(nil).init_str(&title);
                if let Some(item) = self.status_item {
                    let _: () = msg_send![item, setTitle:title_str];
                }
            }
            MenuBarCommand::HideBadge => {
                // Reset title
                let title = NSString::alloc(nil).init_str("🦀");
                if let Some(item) = self.status_item {
                    let _: () = msg_send![item, setTitle:title];
                }
            }
        }
    }

    /// Update the status icon
    unsafe fn update_icon(&mut self) {
        // Icon updates would use NSImage with SF Symbols
        // For now, we use text-based icons
        if let Some(item) = self.status_item {
            let title = match &self.current_status {
                MenuBarStatus::Idle => "🦀",
                MenuBarStatus::Working => "🔧",
                MenuBarStatus::Error => "❌",
                MenuBarStatus::Custom(_) => "🦀",
            };
            let title_str = NSString::alloc(nil).init_str(title);
            let _: () = msg_send![item, setTitle:title_str];
        }
    }

    // Action methods (called by menu items via objc runtime)

    /// Show main window action
    unsafe fn show_main_window(&self) {
        let _ = self.event_tx.send(MenuBarEvent::ShowMainWindow);
    }

    /// Spawn agent action
    unsafe fn spawn_agent(&self) {
        let _ = self.event_tx.send(MenuBarEvent::SpawnAgent);
    }

    /// Cancel agent action
    unsafe fn cancel_agent(&self, agent_id: &str) {
        let _ = self.event_tx.send(MenuBarEvent::CancelAgent(agent_id.to_string()));
    }

    /// Quit app action
    unsafe fn quit_app(&self) {
        let _ = self.event_tx.send(MenuBarEvent::Quit);
    }
}

/// Start the menu bar in a background thread
///
/// Returns handles for sending commands and receiving events.
pub fn start_menu_bar() -> (Sender<MenuBarCommand>, Receiver<MenuBarEvent>) {
    let (mut manager, command_tx, event_rx) = MenuBarManager::new();

    thread::spawn(move || {
        manager.run();
    });

    (command_tx, event_rx)
}

/// Check if menu bar is available (macOS + feature flag)
pub fn is_menu_bar_available() -> bool {
    true // Always true when this module is compiled
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_bar_status_default() {
        let status = MenuBarStatus::default();
        assert_eq!(status, MenuBarStatus::Idle);
    }

    #[test]
    fn test_agent_info() {
        let agent = AgentInfo {
            id: "agent-1".to_string(),
            name: "Test Agent".to_string(),
            running: true,
        };
        assert_eq!(agent.id, "agent-1");
        assert!(agent.running);
    }

    #[test]
    fn test_menu_bar_available() {
        assert!(is_menu_bar_available());
    }
}
