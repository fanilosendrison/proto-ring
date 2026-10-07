#![forbid(unsafe_code)]

//! Implementation-private reference-engine substrate for the Rust migration.
//!
//! Issue #62 intentionally implements zero proto-ring semantic responsibilities.
//! Normative contracts and the #61 corpus remain authority. Issues #63–#69 own
//! the incremental semantic ports. This crate is not #28's public agent-facing
//! API.
