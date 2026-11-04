use proto_ring_engine::evidence_requirements as engine;

use super::declaration_fields::{authority, id, ids, path, target};
use crate::{
    EvidenceClasses, EvidenceClassesKind, EvidenceContext, EvidenceInstanceKind, EvidenceInstances,
    EvidenceProjection, EvidenceRequirementDeclaration, ModelError, VersionOne,
};

pub(crate) fn evidence_requirement(
    value: &engine::PersistentEvidenceRequirement,
) -> Result<EvidenceRequirementDeclaration, ModelError> {
    Ok(EvidenceRequirementDeclaration {
        responsibility: id(&value.responsibility)?,
        instances: EvidenceInstances {
            kind: match value.instances.kind {
                engine::InstantiationKind::Single => EvidenceInstanceKind::Single,
                engine::InstantiationKind::Source => EvidenceInstanceKind::Source,
            },
            source: value.instances.source.as_deref().map(id).transpose()?,
        },
        evidence_classes: EvidenceClasses {
            kind: match value.evidence_classes.kind {
                engine::EvidenceClassKind::Explicit => EvidenceClassesKind::Explicit,
                engine::EvidenceClassKind::Source => EvidenceClassesKind::Source,
            },
            classes: ids(&value.evidence_classes.classes)?,
            source: value
                .evidence_classes
                .source
                .as_deref()
                .map(id)
                .transpose()?,
        },
        subject_source: id(&value.subject_source)?,
        context: EvidenceContext {
            required: value.context.required,
            source: value.context.source.as_deref().map(id).transpose()?,
        },
        candidate_source: id(&value.candidate_source)?,
        target: value.target.as_ref().map(target).transpose()?,
    })
}

pub(crate) fn evidence(
    value: &engine::EvidenceRequirementRegistry,
) -> Result<EvidenceProjection, ModelError> {
    Ok(EvidenceProjection {
        model_version: VersionOne::try_from(u64::from(value.model_version))?,
        authority: authority(&value.authority.responsibility, &value.authority.source)?,
        requirements: value
            .requirements
            .iter()
            .map(|(key, value)| Ok((id(key)?, evidence_requirement(value)?)))
            .collect::<Result<_, ModelError>>()?,
        carrier: path(&value.carrier)?,
    })
}
