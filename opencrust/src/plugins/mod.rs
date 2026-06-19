//! Plugin/Extension System — discover, load, and manage OpenCrust plugins
//!
//! Plugins extend OpenCrust with new capabilities: custom tools, event hooks,
//! UI panels, and protocol integrations. Each plugin is a directory under
//! `~/.config/opencrust/plugins/<name>/` containing a `plugin.json` manifest
//! and optionally scripts, WASM modules, or configuration files.
//!
//! ## Manifest format (`plugin.json`)
//!
//! ```json
//! {
//!   "name": "my-plugin",
//!   "version": "1.0.0",
//!   "description": "Integrates with FooBar API",
//!   "author": "You",
//!   "entry": "main.sh",
//!   "hooks": ["on_tool_execute", "on_message"],
//!   "tools": ["my_custom_tool"],
//!   "dependencies": [],
//!   "citations": [
//!     {
//!       "id": "ref1",
//!       "title": "OpenAI GPT-4 Technical Report",
//!       "author": "OpenAI",
//!       "source": "https://cdn.openai.com/papers/gpt-4.pdf",
//!       "date": "2023-03-14",
//!       "context": "Language model capabilities",
//!       "verified": true
//!     }
//!   ],
//!   "enabled": true
//! }
//! ```
//!
//! ## Hook System
//!
//! Plugins can register hooks that fire at specific points in the
//! OpenCrust lifecycle. Built-in hook points:
//!
//! - `on_startup` — called when OpenCrust initializes
//! - `on_shutdown` — called before OpenCrust exits
//! - `on_tool_execute` — before a tool runs (can modify/block)
//! - `on_message` — when a message is received
//! - `on_response` — when an LLM response is generated
//! - `on_session_save` — when a session is persisted

pub(crate) mod helpers;
pub mod manager;
#[cfg(test)]
mod tests;
pub mod types;

pub use manager::PluginManager;
// Citation, Plugin, PluginError, PluginStats are available via types:: if needed
