use std::collections::BTreeSet;

use num_bigint::BigUint;

use super::model::{ClassItem, Node, NodeId, NodeProperties, PortablePatternError, Program};

struct GroupFrame {
    alternatives: Vec<NodeId>,
    sequence: Vec<NodeId>,
    capture: Option<usize>,
}

pub(super) fn parse(source: &str) -> Result<Program, PortablePatternError> {
    Parser {
        source: source.chars().collect(),
        position: 0,
        nodes: Vec::new(),
        properties: Vec::new(),
        frames: vec![GroupFrame {
            alternatives: Vec::new(),
            sequence: Vec::new(),
            capture: None,
        }],
        capture_count: 0,
        capture_names: Vec::new(),
        used_names: BTreeSet::new(),
    }
    .parse()
}

struct Parser {
    source: Vec<char>,
    position: usize,
    nodes: Vec<Node>,
    properties: Vec<NodeProperties>,
    frames: Vec<GroupFrame>,
    capture_count: usize,
    capture_names: Vec<(String, usize)>,
    used_names: BTreeSet<String>,
}

impl Parser {
    fn parse(mut self) -> Result<Program, PortablePatternError> {
        while let Some(&scalar) = self.source.get(self.position) {
            match scalar {
                '(' => self.open_group()?,
                ')' => self.close_group()?,
                '|' => self.finish_alternative(),
                '[' => {
                    let node = self.parse_character_class()?;
                    self.push_node(node);
                }
                '\\' => {
                    let literal = self.parse_outside_escape()?;
                    let node = self.add_node(Node::Literal(literal));
                    self.push_node(node);
                }
                '^' => {
                    self.position += 1;
                    let node = self.add_node(Node::StartAnchor);
                    self.push_node(node);
                }
                '$' => {
                    self.position += 1;
                    let node = self.add_node(Node::EndAnchor);
                    self.push_node(node);
                }
                '?' | '*' | '+' | '{' => self.apply_quantifier()?,
                '.' | ']' | '}' => return Err(self.invalid("metacharacter is not admitted here")),
                _ => {
                    self.position += 1;
                    let node = self.add_node(Node::Literal(scalar));
                    self.push_node(node);
                }
            }
        }
        if self.frames.len() != 1 {
            return Err(self.invalid("group is not closed"));
        }
        let frame = self.frames.pop().expect("top-level frame exists");
        let root = self.finish_frame(frame);
        Ok(Program {
            nodes: self.nodes,
            root,
            properties: self.properties,
            capture_count: self.capture_count,
            capture_names: self.capture_names,
        })
    }

    fn open_group(&mut self) -> Result<(), PortablePatternError> {
        self.position += 1;
        let capture = if self.source.get(self.position) != Some(&'?') {
            Some(self.next_capture(None)?)
        } else if self.source.get(self.position + 1) == Some(&':') {
            self.position += 2;
            None
        } else if self.source.get(self.position + 1) == Some(&'P')
            && self.source.get(self.position + 2) == Some(&'<')
        {
            self.position += 3;
            let start = self.position;
            while self
                .source
                .get(self.position)
                .is_some_and(|scalar| *scalar != '>')
            {
                self.position += 1;
            }
            if self.source.get(self.position) != Some(&'>') {
                return Err(self.invalid("named capture is not closed"));
            }
            let name: String = self.source[start..self.position].iter().collect();
            self.position += 1;
            if !valid_capture_name(&name) || !self.used_names.insert(name.clone()) {
                return Err(self.invalid("named capture is invalid or duplicated"));
            }
            Some(self.next_capture(Some(name))?)
        } else {
            return Err(self.invalid("group extension is not admitted"));
        };
        self.frames.push(GroupFrame {
            alternatives: Vec::new(),
            sequence: Vec::new(),
            capture,
        });
        Ok(())
    }

    fn next_capture(&mut self, name: Option<String>) -> Result<usize, PortablePatternError> {
        self.capture_count = self
            .capture_count
            .checked_add(1)
            .ok_or_else(|| self.invalid("capture count exceeds host address space"))?;
        if let Some(name) = name {
            self.capture_names.push((name, self.capture_count));
        }
        Ok(self.capture_count)
    }

    fn close_group(&mut self) -> Result<(), PortablePatternError> {
        if self.frames.len() == 1 {
            return Err(self.invalid("closing parenthesis has no group"));
        }
        self.position += 1;
        let frame = self.frames.pop().expect("nested frame exists");
        let capture = frame.capture;
        let child = self.finish_frame(frame);
        let group = self.add_node(Node::Group { child, capture });
        self.push_node(group);
        Ok(())
    }

    fn finish_alternative(&mut self) {
        self.position += 1;
        let sequence = std::mem::take(&mut self.frames.last_mut().expect("frame exists").sequence);
        let node = self.finish_sequence(sequence);
        self.frames
            .last_mut()
            .expect("frame exists")
            .alternatives
            .push(node);
    }

    fn finish_frame(&mut self, mut frame: GroupFrame) -> NodeId {
        let last = self.finish_sequence(frame.sequence);
        frame.alternatives.push(last);
        if frame.alternatives.len() == 1 {
            frame.alternatives[0]
        } else {
            self.add_node(Node::Alternation(frame.alternatives))
        }
    }

    fn finish_sequence(&mut self, sequence: Vec<NodeId>) -> NodeId {
        match sequence.as_slice() {
            [] => self.add_node(Node::Empty),
            [only] => *only,
            _ => self.add_node(Node::Sequence(sequence)),
        }
    }

    fn parse_outside_escape(&mut self) -> Result<char, PortablePatternError> {
        self.position += 1;
        let scalar = self
            .source
            .get(self.position)
            .copied()
            .ok_or_else(|| self.invalid("escape has no escaped scalar"))?;
        if !matches!(
            scalar,
            '\\' | '^' | '$' | '.' | '[' | ']' | '(' | ')' | '|' | '?' | '*' | '+' | '{' | '}'
        ) {
            return Err(self.invalid("escape is outside the admitted vocabulary"));
        }
        self.position += 1;
        Ok(scalar)
    }

    fn parse_character_class(&mut self) -> Result<NodeId, PortablePatternError> {
        self.position += 1;
        let negated = if self.source.get(self.position) == Some(&'^') {
            self.position += 1;
            true
        } else {
            false
        };
        let mut tokens = Vec::new();
        loop {
            let scalar = self
                .source
                .get(self.position)
                .copied()
                .ok_or_else(|| self.invalid("character class is not closed"))?;
            if scalar == ']' {
                self.position += 1;
                break;
            }
            if scalar == '\\' {
                self.position += 1;
                let escaped = self
                    .source
                    .get(self.position)
                    .copied()
                    .ok_or_else(|| self.invalid("class escape has no scalar"))?;
                if !matches!(escaped, ']' | '\\' | '-' | '^') {
                    return Err(self.invalid("class escape is outside the admitted vocabulary"));
                }
                tokens.push((escaped, true));
            } else {
                tokens.push((scalar, false));
            }
            self.position += 1;
        }
        if tokens.is_empty() {
            return Err(self.invalid("character class is empty"));
        }
        let items = self.class_items(&tokens)?;
        Ok(self.add_node(Node::CharacterClass { negated, items }))
    }

    fn class_items(&self, tokens: &[(char, bool)]) -> Result<Vec<ClassItem>, PortablePatternError> {
        let mut items = Vec::new();
        let mut index = 0;
        while index < tokens.len() {
            let (current, escaped) = tokens[index];
            if current == '-' && !escaped {
                if index == 0 || index + 1 == tokens.len() {
                    items.push(ClassItem::Literal('-'));
                    index += 1;
                    continue;
                }
                return Err(self.invalid("class hyphen cannot be reassociated"));
            }
            if index + 2 < tokens.len() && tokens[index + 1] == ('-', false) {
                let end = tokens[index + 2].0;
                if !current.is_ascii() || !end.is_ascii() || current > end {
                    return Err(self.invalid("character class range is invalid"));
                }
                items.push(ClassItem::Range(current, end));
                index += 3;
            } else {
                items.push(ClassItem::Literal(current));
                index += 1;
            }
        }
        Ok(items)
    }

    fn apply_quantifier(&mut self) -> Result<(), PortablePatternError> {
        let child = self
            .frames
            .last_mut()
            .expect("frame exists")
            .sequence
            .pop()
            .ok_or_else(|| self.invalid("quantifier has no atom"))?;
        if !matches!(
            self.nodes[child],
            Node::Literal(_) | Node::CharacterClass { .. } | Node::Group { .. }
        ) {
            return Err(self.invalid("node is not quantifiable"));
        }
        let (minimum, maximum) = self.parse_quantifier_bounds()?;
        if maximum.is_none() && self.properties[child].nullable {
            return Err(self.invalid("unbounded repetition child is nullable"));
        }
        let repeat = self.add_node(Node::Repeat {
            child,
            minimum,
            maximum,
        });
        self.push_node(repeat);
        Ok(())
    }

    fn parse_quantifier_bounds(
        &mut self,
    ) -> Result<(BigUint, Option<BigUint>), PortablePatternError> {
        match self.source[self.position] {
            '?' => {
                self.position += 1;
                Ok((BigUint::from(0_u8), Some(BigUint::from(1_u8))))
            }
            '*' => {
                self.position += 1;
                Ok((BigUint::from(0_u8), None))
            }
            '+' => {
                self.position += 1;
                Ok((BigUint::from(1_u8), None))
            }
            '{' => self.parse_braced_quantifier(),
            _ => unreachable!("quantifier dispatch is closed"),
        }
    }

    fn parse_braced_quantifier(
        &mut self,
    ) -> Result<(BigUint, Option<BigUint>), PortablePatternError> {
        self.position += 1;
        let start = self.position;
        while self
            .source
            .get(self.position)
            .is_some_and(|scalar| *scalar != '}')
        {
            self.position += 1;
        }
        if self.source.get(self.position) != Some(&'}') {
            return Err(self.invalid("quantifier is not closed"));
        }
        let body = &self.source[start..self.position];
        self.position += 1;
        let commas: Vec<_> = body
            .iter()
            .enumerate()
            .filter_map(|(index, scalar)| (*scalar == ',').then_some(index))
            .collect();
        match commas.as_slice() {
            [] => {
                let exact = decimal_bound(body, self)?;
                Ok((exact.clone(), Some(exact)))
            }
            [comma] => {
                let minimum = decimal_bound(&body[..*comma], self)?;
                let maximum = if *comma + 1 == body.len() {
                    None
                } else {
                    Some(decimal_bound(&body[*comma + 1..], self)?)
                };
                if maximum.as_ref().is_some_and(|maximum| minimum > *maximum) {
                    return Err(self.invalid("quantifier minimum exceeds maximum"));
                }
                Ok((minimum, maximum))
            }
            _ => Err(self.invalid("quantifier contains multiple commas")),
        }
    }

    fn push_node(&mut self, node: NodeId) {
        self.frames
            .last_mut()
            .expect("frame exists")
            .sequence
            .push(node);
    }

    fn add_node(&mut self, node: Node) -> NodeId {
        let properties = NodeProperties::derive(&node, &self.properties);
        let id = self.nodes.len();
        self.nodes.push(node);
        self.properties.push(properties);
        id
    }

    fn invalid(&self, detail: &str) -> PortablePatternError {
        PortablePatternError::invalid(detail)
    }
}

fn decimal_bound(chars: &[char], parser: &Parser) -> Result<BigUint, PortablePatternError> {
    if chars.is_empty()
        || (chars.len() > 1 && chars[0] == '0')
        || chars.iter().any(|scalar| !scalar.is_ascii_digit())
    {
        return Err(parser.invalid("quantifier bound is not canonical decimal"));
    }
    let decimal: String = chars.iter().collect();
    BigUint::parse_bytes(decimal.as_bytes(), 10)
        .ok_or_else(|| parser.invalid("quantifier bound is not decimal"))
}

fn valid_capture_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte == b'_' || byte.is_ascii_alphabetic())
        && bytes.all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
}
