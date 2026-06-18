//! Service Locator / Dependency Injection pattern following vLLM Phase 1 approach
//!
//! This module provides a central service registry that allows components to be
//! registered and retrieved by type. This follows the vLLM pattern of simulating
//! DI with a central service module.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// Global service registry
#[allow(dead_code, reason = "New service registry - will be integrated in future phases")]
static SERVICE_REGISTRY: OnceLock<ServiceRegistry> = OnceLock::new();

/// Service registry for dependency injection
#[allow(dead_code, reason = "New service registry - will be integrated in future phases")]
pub struct ServiceRegistry {
    services: Mutex<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
}

impl ServiceRegistry {
    /// Get the global service registry instance
    #[allow(dead_code, reason = "New service registry - will be integrated in future phases")]
    pub fn global() -> &'static ServiceRegistry {
        SERVICE_REGISTRY.get_or_init(|| ServiceRegistry {
            services: Mutex::new(HashMap::new()),
        })
    }

    /// Register a service in the registry
    #[allow(dead_code, reason = "New service registry - will be integrated in future phases")]
    pub fn register<T: Any + Send + Sync + 'static>(&self, service: Arc<T>) {
        let mut services = self.services.lock().unwrap();
        services.insert(TypeId::of::<T>(), Box::new(service));
    }

    /// Get a service from the registry
    #[allow(dead_code, reason = "New service registry - will be integrated in future phases")]
    pub fn get<T: Any + Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        let services = self.services.lock().unwrap();
        services
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<Arc<T>>())
            .cloned()
    }

    /// Get or initialize a service with a factory function
    #[allow(dead_code, reason = "New service registry - will be integrated in future phases")]
    pub fn get_or_init<T: Any + Send + Sync + 'static, F: FnOnce() -> T>(&self, factory: F) -> Arc<T> {
        let mut services = self.services.lock().unwrap();
        
        if let Some(existing) = services.get(&TypeId::of::<T>()) {
            return existing.downcast_ref::<Arc<T>>().unwrap().clone();
        }
        
        let service = Arc::new(factory());
        services.insert(TypeId::of::<T>(), Box::new(service.clone()));
        service
    }

    /// Check if a service is registered
    #[allow(dead_code, reason = "New service registry - will be integrated in future phases")]
    pub fn contains<T: Any + Send + Sync + 'static>(&self) -> bool {
        let services = self.services.lock().unwrap();
        services.contains_key(&TypeId::of::<T>())
    }

    /// Remove a service from the registry
    #[allow(dead_code, reason = "New service registry - will be integrated in future phases")]
    pub fn remove<T: Any + Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        let mut services = self.services.lock().unwrap();
        services
            .remove(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast::<Arc<T>>().ok())
            .map(|arc| *arc)
    }

    /// Clear all services
    #[allow(dead_code, reason = "New service registry - will be integrated in future phases")]
    pub fn clear(&self) {
        let mut services = self.services.lock().unwrap();
        services.clear();
    }
}

/// Convenience macro for registering services
#[macro_export]
macro_rules! register_service {
    ($service:expr) => {
        $crate::core::services::ServiceRegistry::global().register($service);
    };
}

/// Convenience macro for getting services
#[macro_export]
macro_rules! get_service {
    ($type:ty) => {
        $crate::core::services::ServiceRegistry::global().get::<$type>()
    };
}

/// Convenience macro for getting or initializing services
#[macro_export]
macro_rules! get_or_init_service {
    ($type:ty, $factory:expr) => {
        $crate::core::services::ServiceRegistry::global().get_or_init::<$type, _>($factory)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestService {
        value: String,
    }

    #[test]
    fn test_service_registry() {
        let registry = ServiceRegistry::global();
        registry.clear();
        
        // Register a service (as Arc)
        let service = Arc::new(TestService { value: "test".to_string() });
        registry.register(service);
        
        // Get the service
        let retrieved: Option<Arc<TestService>> = registry.get();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().value, "test");
        
        // Test get_or_init
        let registry2 = ServiceRegistry::global();
        registry2.clear();
        
        let service2: Arc<TestService> = registry2.get_or_init(|| TestService { value: "init".to_string() });
        assert_eq!(service2.value, "init");
        
        // Second call should return the same instance
        let service3: Arc<TestService> = registry2.get_or_init(|| TestService { value: "different".to_string() });
        assert_eq!(service3.value, "init");
        assert!(Arc::ptr_eq(&service2, &service3));
    }
}