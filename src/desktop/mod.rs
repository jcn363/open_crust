//! Cinnamon desktop integration module
//!
//! Provides desktop environment detection, system notifications, and native file pickers
//! for LinuxMint Cinnamon while keeping the core TUI unchanged.
//!
//! # Dead Code Policy
//! This module contains functions marked as dead code because they are **intentional scaffolding**
//! for planned future enhancements. These include:
//! - **File pickers**: Multiple backend support (Nemo, Zenity, KDialog) for different desktop environments
//! - **Notifications**: DBus integration for rich notifications with actions (currently using notify-send)
//! - **Detection**: Helper utilities for desktop environment capabilities
//!
//! These are preserved to accelerate future development. The module is exposed as `pub` to support
//! experimental CLI subcommands or external integration. Production code currently uses only
//! `send_notification()` from the notifications submodule.

#![allow(dead_code)]

pub mod detection;
pub mod notifications;
pub mod file_picker;