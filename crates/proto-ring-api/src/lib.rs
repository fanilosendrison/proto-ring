#![forbid(unsafe_code)]
//! Nonexecuting API-02: discovery, canonical opening, retained inspection and
//! explanation, strict JSON transport and the `proto-ring` CLI.
//!
//! `open` retains one canonical engine load. `inspect` and `explain` use only
//! that state. JSON/CLI requests share decoding, dispatch and bounded encoding.
//! The full v1 catalog is discoverable, but both API-03 services return explicit
//! missing-service failures in this development build. Execution and integrated
//! v1 runtime qualification are not claimed. Deserializing a DTO alone does not
//! establish governance authority or validate the common transport boundary.
//! No loaded `GovernanceObservation` can be fabricated from these models.

mod transport;
pub use transport::*;

mod components;
mod discovery;
#[path = "explain-query.rs"]
mod explain_query;
mod explanation;
#[path = "explanation-resolve.rs"]
mod explanation_resolve;
#[path = "explicit-null.rs"]
mod explicit_null;
#[path = "inspect-selector.rs"]
mod inspect_selector;
mod inspection;
#[path = "inspection-resolve.rs"]
mod inspection_resolve;
#[path = "load-error.rs"]
mod load_error;
#[path = "model-error.rs"]
mod model_error;
mod models;
#[path = "native-codec.rs"]
mod native_codec;
#[path = "native-path.rs"]
mod native_path;
#[path = "numeric-tags.rs"]
mod numeric_tags;
mod observation;
#[path = "observation-reference.rs"]
mod observation_reference;
#[path = "path-error.rs"]
mod path_error;
mod projections;
mod protocol;
#[path = "query-error.rs"]
mod query_error;
#[path = "reference-derivation.rs"]
mod reference_derivation;
#[path = "reference-hash.rs"]
mod reference_hash;
#[path = "required-nullable.rs"]
mod required_nullable;
#[path = "scalar-values.rs"]
mod scalar_values;
#[path = "structured-value.rs"]
mod structured_value;
#[path = "subject-projection.rs"]
mod subject_projection;
#[path = "subject-resolution.rs"]
mod subject_resolution;

pub use components::*;
pub use discovery::*;
pub use explain_query::*;
pub use explanation::*;
pub use explicit_null::*;
pub use inspect_selector::*;
pub use inspection::*;
pub use model_error::*;
pub use models::*;
pub use native_path::*;
pub use numeric_tags::*;
pub use observation::*;
pub use observation_reference::*;
pub use path_error::*;
pub use protocol::*;
pub use scalar_values::*;
pub use structured_value::*;
pub use subject_projection::*;
