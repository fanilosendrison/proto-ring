mod authority;
mod bindings;
mod bootstrap;
#[path = "declaration-fields.rs"]
mod declaration_fields;
#[path = "evidence-requirements.rs"]
mod evidence_requirements;
#[path = "governance-model.rs"]
mod governance_model;
#[path = "governed-objects.rs"]
mod governed_objects;
#[path = "projection-registry.rs"]
mod projection_registry;
#[path = "repository-integrity.rs"]
mod repository_integrity;
#[path = "structured-value.rs"]
mod structured_value;

pub(crate) use authority::authority;
pub(crate) use bindings::{binding, bindings};
pub(crate) use bootstrap::bootstrap;
pub(crate) use declaration_fields::{authority as authority_reference, path as carrier_path};
pub(crate) use evidence_requirements::{evidence, evidence_requirement};
pub(crate) use governance_model::{capability, model};
pub(crate) use governed_objects::{governed_object, objects};
pub(crate) use projection_registry::{projection, projections};
pub(crate) use repository_integrity::{integrity, required_profile_reference, validation};
