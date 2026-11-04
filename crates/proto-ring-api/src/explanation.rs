use serde::{Deserialize, Serialize, Serializer, ser::SerializeStruct};

use crate::{
    ExplainQuery, Identifier, ModelError, NativePath, ObservationProvenance, ObservationReference,
    SubjectProjection,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoleConsultation {
    #[serde(rename = "authority")]
    Authority,
    #[serde(rename = "secondary_representation")]
    SecondaryRepresentation,
    #[serde(rename = "non_authoritative")]
    NonAuthoritative,
    #[serde(rename = "UNKNOWN_RESPONSIBILITY")]
    UnknownResponsibility,
    #[serde(rename = "UNDECLARED")]
    Undeclared,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PrecedenceRelation {
    Outranks,
    DoesNotOutrank,
    UnknownResponsibility,
}

/// Declared facts only. A declaration's query is derived from its typed subject,
/// so an explanation cannot echo a different subject kind or identifier.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(try_from = "ExplanationWire")]
pub enum Explanation {
    AuthorityRole {
        responsibility: Identifier,
        source: Identifier,
        role: RoleConsultation,
    },
    AuthorityPrecedence {
        responsibility: Identifier,
        higher_source: Identifier,
        lower_source: Identifier,
        relation: PrecedenceRelation,
    },
    Declaration(Box<SubjectProjection>),
}

impl Explanation {
    pub fn query(&self) -> ExplainQuery {
        match self {
            Self::AuthorityRole {
                responsibility,
                source,
                ..
            } => ExplainQuery::AuthorityRole {
                responsibility: responsibility.clone(),
                source: source.clone(),
            },
            Self::AuthorityPrecedence {
                responsibility,
                higher_source,
                lower_source,
                ..
            } => ExplainQuery::AuthorityPrecedence {
                responsibility: responsibility.clone(),
                higher_source: higher_source.clone(),
                lower_source: lower_source.clone(),
            },
            Self::Declaration(subject) => subject.query(),
        }
    }
}

impl Serialize for Explanation {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct(
            "Explanation",
            match self {
                Self::AuthorityRole { .. } => 4,
                Self::AuthorityPrecedence { .. } => 5,
                Self::Declaration(_) => 3,
            },
        )?;
        match self {
            Self::AuthorityRole {
                responsibility,
                source,
                role,
            } => {
                state.serialize_field("kind", "authority_role")?;
                state.serialize_field("responsibility", responsibility)?;
                state.serialize_field("source", source)?;
                state.serialize_field("role", role)?;
            }
            Self::AuthorityPrecedence {
                responsibility,
                higher_source,
                lower_source,
                relation,
            } => {
                state.serialize_field("kind", "authority_precedence")?;
                state.serialize_field("responsibility", responsibility)?;
                state.serialize_field("higher_source", higher_source)?;
                state.serialize_field("lower_source", lower_source)?;
                state.serialize_field("relation", relation)?;
            }
            Self::Declaration(declaration) => {
                state.serialize_field("kind", "declaration")?;
                state.serialize_field("subject", &declaration.query())?;
                state.serialize_field("declaration", declaration)?;
            }
        }
        state.end()
    }
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum ExplanationWire {
    AuthorityRole {
        responsibility: Identifier,
        source: Identifier,
        role: RoleConsultation,
    },
    AuthorityPrecedence {
        responsibility: Identifier,
        higher_source: Identifier,
        lower_source: Identifier,
        relation: PrecedenceRelation,
    },
    Declaration {
        subject: ExplainQuery,
        declaration: Box<SubjectProjection>,
    },
}

impl TryFrom<ExplanationWire> for Explanation {
    type Error = ModelError;

    fn try_from(wire: ExplanationWire) -> Result<Self, Self::Error> {
        Ok(match wire {
            ExplanationWire::AuthorityRole {
                responsibility,
                source,
                role,
            } => Self::AuthorityRole {
                responsibility,
                source,
                role,
            },
            ExplanationWire::AuthorityPrecedence {
                responsibility,
                higher_source,
                lower_source,
                relation,
            } => Self::AuthorityPrecedence {
                responsibility,
                higher_source,
                lower_source,
                relation,
            },
            ExplanationWire::Declaration {
                subject,
                declaration,
            } => {
                if subject != declaration.query() {
                    return Err(ModelError::Inconsistent(
                        "declaration query does not match subject",
                    ));
                }
                Self::Declaration(declaration)
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(try_from = "ExplanationResultWire")]
pub struct ExplanationResult {
    provenance: ObservationProvenance,
    explanation: Explanation,
}

impl ExplanationResult {
    pub fn new(provenance: ObservationProvenance, explanation: Explanation) -> Self {
        Self {
            provenance,
            explanation,
        }
    }

    pub fn provenance(&self) -> &ObservationProvenance {
        &self.provenance
    }

    pub fn explanation(&self) -> &Explanation {
        &self.explanation
    }

    pub fn query(&self) -> ExplainQuery {
        self.explanation.query()
    }
}

impl Serialize for ExplanationResult {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("ExplanationResult", 5)?;
        state.serialize_field("observation", &self.provenance.observation)?;
        state.serialize_field("repository", &self.provenance.repository)?;
        state.serialize_field("scope_paths", &self.provenance.scope_paths)?;
        state.serialize_field("query", &self.query())?;
        state.serialize_field("explanation", &self.explanation)?;
        state.end()
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExplanationResultWire {
    observation: ObservationReference,
    repository: NativePath,
    scope_paths: Vec<NativePath>,
    query: ExplainQuery,
    explanation: Explanation,
}

impl TryFrom<ExplanationResultWire> for ExplanationResult {
    type Error = ModelError;

    fn try_from(wire: ExplanationResultWire) -> Result<Self, Self::Error> {
        if wire.query != wire.explanation.query() {
            return Err(ModelError::Inconsistent(
                "explanation query does not match result",
            ));
        }
        Ok(Self::new(
            ObservationProvenance {
                observation: wire.observation,
                repository: wire.repository,
                scope_paths: wire.scope_paths,
            },
            wire.explanation,
        ))
    }
}
