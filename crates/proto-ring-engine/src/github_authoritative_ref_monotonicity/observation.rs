use std::collections::BTreeSet;
use std::path::Path;

use serde_json::Value;

use super::binding::{self, Binding};
use super::http::{HttpRequest, HttpTransport};
use super::{ArmDiagnostic, ArmDiagnosticKind, ArmObservation, ArmStatus};

pub(super) const PAGE_SIZE: usize = 100;
pub(super) const MAX_PAGES: usize = 100;
const API_BASE: &str = "https://api.github.com";
const REQUIRED_TYPES: [&str; 2] = ["deletion", "non_fast_forward"];

pub(super) fn observe_with_transport(
    binding_path: &Path,
    transport: &mut impl HttpTransport,
) -> ArmObservation {
    let binding = match binding::load(binding_path) {
        Ok(binding) => binding,
        Err(_) => return report(ArmStatus::Violated, ArmDiagnosticKind::BindingInvalid),
    };
    let observed = match observe_all_pages(&binding, transport) {
        Ok(observed) => observed,
        Err(()) => {
            return report(
                ArmStatus::Undetermined,
                ArmDiagnosticKind::ProviderObservationUnavailable,
            );
        }
    };
    if REQUIRED_TYPES
        .iter()
        .all(|required| observed.contains(*required))
    {
        ArmObservation {
            status: ArmStatus::Satisfied,
            diagnostics: Vec::new(),
        }
    } else {
        report(
            ArmStatus::Violated,
            ArmDiagnosticKind::RequiredProtectionsMissing,
        )
    }
}

fn observe_all_pages(
    binding: &Binding,
    transport: &mut impl HttpTransport,
) -> Result<BTreeSet<String>, ()> {
    let mut observed = BTreeSet::new();
    for page in 1..=MAX_PAGES {
        let request = HttpRequest::github(request_url(binding, page));
        let response = transport.get(&request).map_err(|_| ())?;
        if !(200..300).contains(&response.status) {
            return Err(());
        }
        let items = decode_page(&response.body)?;
        for item in &items {
            let object = item.as_object().ok_or(())?;
            let rule_type = object
                .get("type")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .ok_or(())?;
            let ruleset_id = provider_integer(object.get("ruleset_id").ok_or(())?)?;
            if ruleset_id == binding.ruleset_id {
                observed.insert(rule_type.to_owned());
            }
        }
        if items.len() < PAGE_SIZE {
            return Ok(observed);
        }
    }
    Err(())
}

fn decode_page(body: &[u8]) -> Result<Vec<Value>, ()> {
    let text = std::str::from_utf8(body).map_err(|_| ())?;
    let value: Value = serde_json::from_str(text).map_err(|_| ())?;
    value.as_array().cloned().ok_or(())
}

fn provider_integer(value: &Value) -> Result<String, ()> {
    let text = value.as_number().ok_or(())?.to_string();
    let digits = text.strip_prefix('-').unwrap_or(&text);
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(());
    }
    Ok(text)
}

fn request_url(binding: &Binding, page: usize) -> String {
    format!(
        "{API_BASE}/repos/{}/{}/rules/branches/{}?per_page={PAGE_SIZE}&page={page}",
        percent_encode(&binding.owner),
        percent_encode(&binding.repository),
        percent_encode(&binding.branch),
    )
}

fn percent_encode(segment: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut encoded = String::with_capacity(segment.len());
    for byte in segment.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push('%');
            encoded.push(char::from(HEX[usize::from(byte >> 4)]));
            encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
    }
    encoded
}

fn report(status: ArmStatus, kind: ArmDiagnosticKind) -> ArmObservation {
    ArmObservation {
        status,
        diagnostics: vec![ArmDiagnostic { kind }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dynamic_url_path_segments_use_rfc3986_byte_encoding() {
        let binding = Binding {
            owner: "owner name".to_owned(),
            repository: "répo".to_owned(),
            branch: "release/x".to_owned(),
            ruleset_id: "1".to_owned(),
        };
        assert_eq!(
            request_url(&binding, 7),
            "https://api.github.com/repos/owner%20name/r%C3%A9po/rules/branches/release%2Fx?per_page=100&page=7"
        );
    }
}
