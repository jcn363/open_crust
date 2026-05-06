//! Cinnamon desktop integration module
//!
//! Provides desktop environment detection, system notifications, and native file pickers
//! for LinuxMint Cinnamon while keeping the core TUI unchanged.

#![allow(dead_code)]

pub mod detection;
pub mod notifications;
pub mod file_picker;