use serde::{Deserialize, Serialize, Serializer, ser::SerializeStruct};

use crate::{
    ComponentProjection, Components, InspectSelector, ModelError, NativePath,
    ObservationProvenance, ObservationReference, SubjectProjection,
};

/// Result data only; no operation or repository access is performed by these DTOs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InspectionResult {
    All(Box<Components>),
    Component(Box<ComponentProjection>),
    Subject(Box<SubjectProjection>),
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(try_from = "InspectionWire")]
pub struct Inspection {
    provenance: ObservationProvenance,
    selector: InspectSelector,
    result: InspectionResult,
}

impl Inspection {
    pub fn new(
        provenance: ObservationProvenance,
        result: InspectionResult,
    ) -> Result<Self, ModelError> {
        let selector = match &result {
            InspectionResult::All(_) => InspectSelector::All {},
            InspectionResult::Component(component) => component.selector(),
            InspectionResult::Subject(subject) => InspectSelector::try_from(subject.query())?,
        };
        Ok(Self {
            provenance,
            selector,
            result,
        })
    }

    pub fn provenance(&self) -> &ObservationProvenance {
        &self.provenance
    }

    pub fn selector(&self) -> &InspectSelector {
        &self.selector
    }

    pub fn result(&self) -> &InspectionResult {
        &self.result
    }
}

impl Serialize for Inspection {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("Inspection", 6)?;
        state.serialize_field("observation", &self.provenance.observation)?;
        state.serialize_field("repository", &self.provenance.repository)?;
        state.serialize_field("scope_paths", &self.provenance.scope_paths)?;
        state.serialize_field("selector", &self.selector)?;
        match &self.result {
            InspectionResult::All(components) => {
                state.serialize_field("result_kind", "all")?;
                state.serialize_field("components", components)?;
            }
            InspectionResult::Component(component) => {
                state.serialize_field("result_kind", "component")?;
                state.serialize_field("component", component)?;
            }
            InspectionResult::Subject(subject) => {
                state.serialize_field("result_kind", "subject")?;
                state.serialize_field("subject", subject)?;
            }
        }
        state.end()
    }
}

// Explicit wire variants make surplus/null payload fields errors, without
// flatten+deny_unknown_fields interactions or untyped governance values.
#[derive(Deserialize)]
#[serde(tag = "result_kind", rename_all = "snake_case", deny_unknown_fields)]
enum InspectionWire {
    All {
        observation: ObservationReference,
        repository: NativePath,
        scope_paths: Vec<NativePath>,
        selector: InspectSelector,
        components: Box<Components>,
    },
    Component {
        observation: ObservationReference,
        repository: NativePath,
        scope_paths: Vec<NativePath>,
        selector: InspectSelector,
        component: Box<ComponentProjection>,
    },
    Subject {
        observation: ObservationReference,
        repository: NativePath,
        scope_paths: Vec<NativePath>,
        selector: InspectSelector,
        subject: Box<SubjectProjection>,
    },
}

impl TryFrom<InspectionWire> for Inspection {
    type Error = ModelError;

    fn try_from(wire: InspectionWire) -> Result<Self, Self::Error> {
        let (observation, repository, scope_paths, selector, result) = match wire {
            InspectionWire::All {
                observation,
                repository,
                scope_paths,
                selector,
                components,
            } => (
                observation,
                repository,
                scope_paths,
                selector,
                InspectionResult::All(components),
            ),
            InspectionWire::Component {
                observation,
                repository,
                scope_paths,
                selector,
                component,
            } => (
                observation,
                repository,
                scope_paths,
                selector,
                InspectionResult::Component(component),
            ),
            InspectionWire::Subject {
                observation,
                repository,
                scope_paths,
                selector,
                subject,
            } => (
                observation,
                repository,
                scope_paths,
                selector,
                InspectionResult::Subject(subject),
            ),
        };
        let result = Self::new(
            ObservationProvenance {
                observation,
                repository,
                scope_paths,
            },
            result,
        )?;
        if result.selector != selector {
            return Err(ModelError::Inconsistent(
                "inspection selector does not match result",
            ));
        }
        Ok(result)
    }
}
