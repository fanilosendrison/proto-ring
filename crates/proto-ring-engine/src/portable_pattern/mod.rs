mod matcher;
mod model;
mod parser;

pub use model::{PortablePatternError, PortablePatternMatch};

use model::Program;

pub struct PortablePattern {
    source: String,
    program: Program,
}

impl PortablePattern {
    pub fn compile(source: &str) -> Result<Self, PortablePatternError> {
        Ok(Self {
            source: source.to_owned(),
            program: parser::parse(source)?,
        })
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn capture_count(&self) -> usize {
        self.program.capture_count
    }

    pub fn capture_names(&self) -> impl ExactSizeIterator<Item = (&str, usize)> {
        self.program
            .capture_names
            .iter()
            .map(|(name, index)| (name.as_str(), *index))
    }

    pub fn full_match(
        &self,
        candidate: &str,
    ) -> Result<Option<PortablePatternMatch>, PortablePatternError> {
        let scalars: Vec<char> = candidate.chars().collect();
        let Some(spans) = matcher::full_match(&self.program, &scalars) else {
            return Ok(None);
        };
        let captures = spans
            .into_iter()
            .map(|span| span.map(|(start, end)| scalars[start..end].iter().collect()))
            .collect();
        Ok(Some(PortablePatternMatch::new(
            captures,
            &self.program.capture_names,
        )))
    }
}

#[cfg(test)]
mod tests;
