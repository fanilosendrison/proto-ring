//! Pure exact comparison of consumer-owned evidence identity tokens.

use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindingStatus {
    Match,
    Mismatch,
    Undetermined,
}

impl BindingStatus {
    pub const fn serialized(self) -> &'static str {
        match self {
            Self::Match => "MATCH",
            Self::Mismatch => "MISMATCH",
            Self::Undetermined => "UNDETERMINED",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceRequirement {
    pub admitted_classes: BTreeSet<String>,
    pub subject_identity: Option<Vec<u8>>,
    pub context_identity: Option<Vec<u8>>,
    pub context_required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvidenceRequirementError {
    EmptyAdmissionSet,
    EmptyAdmittedClass,
    EmptySubjectIdentity,
    EmptyContextIdentity,
    ForbiddenContextIdentity,
}

impl EvidenceRequirement {
    pub fn new(
        admitted_classes: BTreeSet<String>,
        subject_identity: Option<Vec<u8>>,
        context_identity: Option<Vec<u8>>,
        context_required: bool,
    ) -> Result<Self, EvidenceRequirementError> {
        if admitted_classes.is_empty() {
            return Err(EvidenceRequirementError::EmptyAdmissionSet);
        }
        if admitted_classes.iter().any(String::is_empty) {
            return Err(EvidenceRequirementError::EmptyAdmittedClass);
        }
        if subject_identity.as_ref().is_some_and(Vec::is_empty) {
            return Err(EvidenceRequirementError::EmptySubjectIdentity);
        }
        if context_identity.as_ref().is_some_and(Vec::is_empty) {
            return Err(EvidenceRequirementError::EmptyContextIdentity);
        }
        if !context_required && context_identity.is_some() {
            return Err(EvidenceRequirementError::ForbiddenContextIdentity);
        }
        Ok(Self {
            admitted_classes,
            subject_identity,
            context_identity,
            context_required,
        })
    }

    pub fn evaluate(&self, candidate: Option<&EvidenceBinding>) -> BindingStatus {
        let Some(required_subject) = self.subject_identity.as_ref() else {
            return BindingStatus::Undetermined;
        };
        let required_context = if self.context_required {
            let Some(context) = self.context_identity.as_ref() else {
                return BindingStatus::Undetermined;
            };
            Some(context)
        } else {
            None
        };
        let Some(candidate) = candidate else {
            return BindingStatus::Undetermined;
        };
        let Some(evidence_class) = candidate
            .evidence_class
            .as_ref()
            .filter(|value| !value.is_empty())
        else {
            return BindingStatus::Undetermined;
        };
        let Some(subject) = candidate
            .subject_identity
            .as_ref()
            .filter(|value| !value.is_empty())
        else {
            return BindingStatus::Undetermined;
        };
        let candidate_context = if self.context_required {
            let Some(context) = candidate
                .context_identity
                .as_ref()
                .filter(|value| !value.is_empty())
            else {
                return BindingStatus::Undetermined;
            };
            Some(context)
        } else {
            None
        };
        if !self.admitted_classes.contains(evidence_class)
            || subject != required_subject
            || required_context
                .zip(candidate_context)
                .is_some_and(|(left, right)| left != right)
        {
            BindingStatus::Mismatch
        } else {
            BindingStatus::Match
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceBinding {
    pub evidence_class: Option<String>,
    pub subject_identity: Option<Vec<u8>>,
    pub context_identity: Option<Vec<u8>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn requirement(classes: &[&str], context_required: bool) -> EvidenceRequirement {
        EvidenceRequirement::new(
            classes.iter().map(|value| (*value).to_owned()).collect(),
            Some(b"subject".to_vec()),
            context_required.then(|| b"context".to_vec()),
            context_required,
        )
        .unwrap()
    }

    #[test]
    fn determinacy_precedes_known_mismatch() {
        let mut requirement = requirement(&["proof"], false);
        requirement.subject_identity = None;
        let candidate = EvidenceBinding {
            evidence_class: Some("other".to_owned()),
            subject_identity: Some(b"different".to_vec()),
            context_identity: None,
        };
        assert_eq!(
            requirement.evaluate(Some(&candidate)),
            BindingStatus::Undetermined
        );
    }

    #[test]
    fn context_is_ignored_when_not_required() {
        let candidate = EvidenceBinding {
            evidence_class: Some("proof".to_owned()),
            subject_identity: Some(b"subject".to_vec()),
            context_identity: Some(Vec::new()),
        };
        assert_eq!(
            requirement(&["proof"], false).evaluate(Some(&candidate)),
            BindingStatus::Match
        );
    }

    #[test]
    fn any_exact_admitted_class_matches() {
        let candidate = EvidenceBinding {
            evidence_class: Some("attestation".to_owned()),
            subject_identity: Some(b"subject".to_vec()),
            context_identity: None,
        };
        assert_eq!(
            requirement(&["proof", "attestation"], false).evaluate(Some(&candidate)),
            BindingStatus::Match
        );
    }
}
