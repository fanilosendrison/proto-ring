use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use num_bigint::BigUint;

pub(super) type NodeId = usize;
pub(super) type CaptureSpan = (usize, usize);

#[derive(Clone, Debug)]
pub(super) enum ClassItem {
    Literal(char),
    Range(char, char),
}

#[derive(Clone, Debug)]
pub(super) enum Node {
    Empty,
    Literal(char),
    CharacterClass {
        negated: bool,
        items: Vec<ClassItem>,
    },
    StartAnchor,
    EndAnchor,
    Sequence(Vec<NodeId>),
    Alternation(Vec<NodeId>),
    Group {
        child: NodeId,
        capture: Option<usize>,
    },
    Repeat {
        child: NodeId,
        minimum: BigUint,
        maximum: Option<BigUint>,
    },
}

#[derive(Clone, Debug)]
pub(super) struct NodeProperties {
    pub(super) nullable: bool,
    pub(super) minimum_consumption: BigUint,
}

impl NodeProperties {
    pub(super) fn derive(node: &Node, known: &[Self]) -> Self {
        match node {
            Node::Empty | Node::StartAnchor | Node::EndAnchor => Self {
                nullable: true,
                minimum_consumption: BigUint::from(0_u8),
            },
            Node::Literal(_) | Node::CharacterClass { .. } => Self {
                nullable: false,
                minimum_consumption: BigUint::from(1_u8),
            },
            Node::Sequence(children) => Self {
                nullable: children.iter().all(|child| known[*child].nullable),
                minimum_consumption: children.iter().fold(BigUint::from(0_u8), |sum, child| {
                    sum + &known[*child].minimum_consumption
                }),
            },
            Node::Alternation(children) => Self {
                nullable: children.iter().any(|child| known[*child].nullable),
                minimum_consumption: children
                    .iter()
                    .map(|child| known[*child].minimum_consumption.clone())
                    .min()
                    .expect("alternation has a branch"),
            },
            Node::Group { child, .. } => known[*child].clone(),
            Node::Repeat { child, minimum, .. } => Self {
                nullable: minimum == &BigUint::from(0_u8) || known[*child].nullable,
                minimum_consumption: &known[*child].minimum_consumption * minimum,
            },
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct Program {
    pub(super) nodes: Vec<Node>,
    pub(super) root: NodeId,
    pub(super) properties: Vec<NodeProperties>,
    pub(super) capture_count: usize,
    pub(super) capture_names: Vec<(String, usize)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortablePatternError {
    detail: String,
}

impl PortablePatternError {
    pub(super) fn invalid(detail: impl Into<String>) -> Self {
        Self {
            detail: detail.into(),
        }
    }
}

impl Display for PortablePatternError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "invalid Canonical Portable Pattern: {}",
            self.detail
        )
    }
}

impl Error for PortablePatternError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortablePatternMatch {
    captures: Vec<Option<String>>,
    capture_names: BTreeMap<String, usize>,
}

impl PortablePatternMatch {
    pub(super) fn new(captures: Vec<Option<String>>, capture_names: &[(String, usize)]) -> Self {
        Self {
            captures,
            capture_names: capture_names.iter().cloned().collect(),
        }
    }

    pub fn capture_count(&self) -> usize {
        self.captures.len()
    }

    pub fn positional_capture(&self, one_based_index: usize) -> Option<&str> {
        one_based_index
            .checked_sub(1)
            .and_then(|index| self.captures.get(index))
            .and_then(Option::as_deref)
    }

    pub fn named_capture(&self, name: &str) -> Option<&str> {
        self.capture_names
            .get(name)
            .and_then(|index| self.positional_capture(*index))
    }
}
