use crate::Operation;
use serde::{
    Deserialize, Deserializer,
    de::{DeserializeSeed, Error, MapAccess, SeqAccess, Visitor},
};
use serde_json::value::RawValue;
use std::{collections::BTreeSet, fmt};

/// Preserve root fields as raw syntax until duplicate checks finish. Never
/// collapse untrusted objects into maps before rejecting duplicate names.
#[derive(Default)]
pub(super) struct Root {
    pub(super) operation: Option<Operation>,
    pub(super) fields: Vec<(String, Box<RawValue>)>,
    ambiguous: bool,
    seen_operation: bool,
    pub(super) duplicate: bool,
}
struct RootSeed<'a>(&'a mut Root);
impl<'de> DeserializeSeed<'de> for RootSeed<'_> {
    type Value = ();
    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        deserializer.deserialize_map(self)
    }
}
impl<'de> Visitor<'de> for RootSeed<'_> {
    type Value = ();
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("one request object")
    }
    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<(), M::Error> {
        let mut seen = BTreeSet::new();
        while let Some(key) = map.next_key::<String>()? {
            if !seen.insert(key.clone()) {
                self.0.duplicate = true;
            }
            let value = map.next_value::<Box<RawValue>>();
            if key == "operation" {
                let candidate = value
                    .as_ref()
                    .ok()
                    .and_then(|raw| serde_json::from_str(raw.get()).ok());
                if candidate.is_none() || (self.0.seen_operation && candidate != self.0.operation) {
                    self.0.ambiguous = true;
                }
                self.0.seen_operation = true;
                self.0.operation = if self.0.ambiguous { None } else { candidate };
            }
            self.0.fields.push((key, value?));
        }
        Ok(())
    }
}

pub(super) fn root(bytes: &[u8]) -> (Root, bool) {
    let mut root = Root::default();
    let mut decoder = serde_json::Deserializer::from_slice(bytes);
    let valid = RootSeed(&mut root)
        .deserialize(&mut decoder)
        .and_then(|()| decoder.end())
        .is_ok();
    (root, valid)
}

struct Unique;
impl<'de> Deserialize<'de> for Unique {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(Self)
    }
}
impl<'de> Visitor<'de> for Unique {
    type Value = Self;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("JSON without duplicate keys")
    }
    fn visit_bool<E: Error>(self, _: bool) -> Result<Self, E> {
        Ok(self)
    }
    fn visit_i64<E: Error>(self, _: i64) -> Result<Self, E> {
        Ok(self)
    }
    fn visit_u64<E: Error>(self, _: u64) -> Result<Self, E> {
        Ok(self)
    }
    fn visit_f64<E: Error>(self, _: f64) -> Result<Self, E> {
        Ok(self)
    }
    fn visit_str<E: Error>(self, _: &str) -> Result<Self, E> {
        Ok(self)
    }
    fn visit_unit<E: Error>(self) -> Result<Self, E> {
        Ok(self)
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self, A::Error> {
        while seq.next_element::<Unique>()?.is_some() {}
        Ok(self)
    }
    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self, M::Error> {
        let mut names = BTreeSet::new();
        while let Some(key) = map.next_key::<String>()? {
            if !names.insert(key) {
                return Err(M::Error::custom("duplicate object key"));
            }
            map.next_value::<Unique>()?;
        }
        Ok(self)
    }
}
pub(super) fn unique(raw: &str) -> bool {
    serde_json::from_str::<Unique>(raw).is_ok()
}
