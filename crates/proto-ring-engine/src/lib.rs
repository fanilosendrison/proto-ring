#![forbid(unsafe_code)]

//! Implementation-private Rust reference engine.
//!
//! #62 established the empty substrate. #63 implements six foundational
//! responsibilities, and #65 adds Governance Authority and Governed Objects.
//! Normative contracts and `conformance/v1` remain authority. The crate layout
//! is not normative architecture, and #28 owns the later public agent-facing API.

pub mod evidence_requirements;
pub mod exact_evidence_binding;
pub mod governance_authority;
pub mod governance_bindings;
pub mod governance_bootstrap;
pub mod governance_routing;
pub mod governed_objects;
pub mod projection_registry;
pub mod repository_governance_model;
pub mod repository_integrity;
pub mod structured_data;

mod declaration_support;
mod structured_data_yaml;
