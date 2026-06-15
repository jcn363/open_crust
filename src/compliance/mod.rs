//! Enterprise Compliance Packaging — SOC2-ready evidence, policies, and reporting
//!
//! Provides production-grade compliance tooling for regulated environments:
//!
//! - **Evidence packages** with SHA256 manifests, chain-of-custody logging,
//!   and tamper-evident sealing
//! - **Compliance profiles** (SOC2, HIPAA, GDPR, SOX) with predefined
//!   control mappings
//! - **Policy enforcement** with rule evaluation and violation reporting
//! - **Structured reports** in multiple formats (text, JSON, CSV, HTML)
//!
//! ## Architecture
//!
//! ```text
//! EvidencePackage     → timestamped directory with audit exports + manifest
//! ComplianceProfile   → named profile with control mappings
//! CompliancePolicy    → declarative rules evaluated against audit entries
//! ComplianceReport    → structured summary with approval rates, trends
//! ```

pub mod evidence;
pub mod policies;
pub mod profiles;
pub mod reports;

// Re-export types used by other crate modules.
pub use evidence::EvidencePackage;
pub use reports::{ComplianceManager, ComplianceReport};
