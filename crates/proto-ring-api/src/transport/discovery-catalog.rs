use super::{
    contract_schema::{bundle, canonical},
    failure::internal,
};
use crate::{ApiError, Discovery};
use serde_json::{Map, Value, json};

/// The complete accepted v1 catalog, not a list of currently delivered services.
/// Uses only compile-time embedded canonical schema data; no repository or I/O.
pub fn discover() -> Result<Discovery, ApiError> {
    let api = canonical()?;
    let branches = api["$defs"]["catalog_entry"]["oneOf"]
        .as_array()
        .ok_or_else(|| internal("missing canonical catalog"))?;
    let requests = api["$defs"]["request"]["oneOf"]
        .as_array()
        .ok_or_else(|| internal("missing canonical requests"))?;
    let responses = api["$defs"]["response"]["oneOf"]
        .as_array()
        .ok_or_else(|| internal("missing canonical responses"))?;
    let mut entries = Vec::new();
    for branch in branches {
        let properties = branch["properties"]
            .as_object()
            .ok_or_else(|| internal("invalid catalog branch"))?;
        let mut entry: Map<String, Value> = properties
            .iter()
            .filter_map(|(key, value)| value.get("const").map(|value| (key.clone(), value.clone())))
            .collect();
        let operation = entry
            .get("operation")
            .ok_or_else(|| internal("missing catalog operation"))?;
        let request = requests
            .iter()
            .find(|branch| &branch["properties"]["operation"]["const"] == operation)
            .ok_or_else(|| internal("missing operation request"))?;
        let selected: Vec<_> = responses
            .iter()
            .filter(|branch| &branch["properties"]["operation"]["const"] == operation)
            .cloned()
            .collect();
        if selected.is_empty() {
            return Err(internal("missing operation responses"));
        }
        entry.insert("request_schema".into(), bundle(request)?);
        entry.insert(
            "response_schema".into(),
            bundle(&json!({"oneOf":selected}))?,
        );
        entries.push(Value::Object(entry));
    }
    serde_json::from_value(json!({"operations":entries,"schema_draft":api["$schema"]}))
        .map_err(|_| internal("invalid canonical discovery data"))
}
