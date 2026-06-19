//! Provider abstraction layer for extensible integrations
//!
//! This module defines generic Provider traits that can be implemented for
//! different integration types (desktop, notifications, file pickers, tools, etc.)

pub mod desktop;
pub mod file_picker;
pub mod notifications;
pub mod plugin;
pub mod tool;

/// Generic provider trait that all providers must implement
pub trait Provider: Send + Sync {
    /// Unique identifier for this provider
    fn id(&self) -> &str;

    /// Human-readable name
    fn name(&self) -> &str;

    /// Check if this provider is available on the current system
    fn is_available(&self) -> bool;

    /// Priority for auto-selection (higher = preferred)
    fn priority(&self) -> u8 {
        50
    }
}

/// Provider registry for managing multiple providers of the same type
pub struct ProviderRegistry<P: Provider + ?Sized> {
    providers: Vec<Box<P>>,
}

impl<P: Provider + ?Sized> Default for ProviderRegistry<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: Provider + ?Sized> ProviderRegistry<P> {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    /// Register a new provider
    pub fn register(&mut self, provider: Box<P>) {
        self.providers.push(provider);
    }

    /// Get all registered providers
    pub fn all(&self) -> &[Box<P>] {
        &self.providers
    }

    /// Get available providers (sorted by priority, highest first)
    pub fn available(&self) -> Vec<&P> {
        let mut available: Vec<&P> = self
            .providers
            .iter()
            .filter(|p| p.is_available())
            .map(|p| p.as_ref())
            .collect();
        available.sort_by_key(|p| std::cmp::Reverse(p.priority()));
        available
    }

    /// Get the best available provider (highest priority)
    pub fn best(&self) -> Option<&P> {
        self.available().first().copied()
    }

    /// Get provider by ID
    pub fn get(&self, id: &str) -> Option<&P> {
        self.providers
            .iter()
            .find(|p| p.id() == id)
            .map(|p| p.as_ref())
    }
}
