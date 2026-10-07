#![forbid(unsafe_code)]

//! Implementation-private Rust reference engine.
//!
//! #62 established the empty substrate. #63 implements exactly its six
//! foundational responsibilities. Normative contracts and `conformance/v1`
//! remain authority. The crate layout is not normative architecture, and #28
//! owns the later public agent-facing API.

pub mod governance_bootstrap;
pub mod governance_routing;
pub mod repository_governance_model;
pub mod structured_data;

mod structured_data_yaml;
