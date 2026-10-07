#![forbid(unsafe_code)]

//! Implementation-private transport and differential harness for the #61 corpus.
//! This crate defines neither proto-ring semantics nor a public compatibility API.

pub mod compare;
pub mod fixture;
pub mod harness;
pub mod loader;
pub mod model;
pub mod rust_candidate;
