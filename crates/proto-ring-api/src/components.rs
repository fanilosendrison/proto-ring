use serde::{Deserialize, Serialize};

use crate::{
    AuthorityProjection, BindingsProjection, BootstrapProjection, EvidenceProjection, ExplicitNull,
    InspectSelector, IntegrityProfileProjection, ModelProjection, ObjectsProjection,
    ProjectionsProjection,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresentTag {
    Present,
}

/// Mandatory component: no absent variant is representable.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Present<T> {
    pub presence: PresentTag,
    pub data: T,
}

impl<T> Present<T> {
    pub fn new(data: T) -> Self {
        Self {
            presence: PresentTag::Present,
            data,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "presence",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Presence<T> {
    Present(T),
    Absent(ExplicitNull),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Components {
    pub bootstrap: Present<BootstrapProjection>,
    pub model: Present<ModelProjection>,
    pub authority: Present<AuthorityProjection>,
    pub governed_objects: Presence<ObjectsProjection>,
    pub bindings: Present<BindingsProjection>,
    pub repository_integrity: Presence<IntegrityProfileProjection>,
    pub projections: Presence<ProjectionsProjection>,
    pub evidence_requirements: Presence<EvidenceProjection>,
}

/// Complete present component data; absence is never a successful component view.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "name",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ComponentProjection {
    Bootstrap(BootstrapProjection),
    Model(ModelProjection),
    Authority(AuthorityProjection),
    GovernedObjects(ObjectsProjection),
    Bindings(BindingsProjection),
    RepositoryIntegrity(IntegrityProfileProjection),
    Projections(ProjectionsProjection),
    EvidenceRequirements(EvidenceProjection),
}

impl ComponentProjection {
    pub fn selector(&self) -> InspectSelector {
        match self {
            Self::Bootstrap(_) => InspectSelector::Bootstrap {},
            Self::Model(_) => InspectSelector::Model {},
            Self::Authority(_) => InspectSelector::Authority {},
            Self::GovernedObjects(_) => InspectSelector::GovernedObjects {},
            Self::Bindings(_) => InspectSelector::Bindings {},
            Self::RepositoryIntegrity(_) => InspectSelector::RepositoryIntegrity {},
            Self::Projections(_) => InspectSelector::Projections {},
            Self::EvidenceRequirements(_) => InspectSelector::EvidenceRequirements {},
        }
    }
}
