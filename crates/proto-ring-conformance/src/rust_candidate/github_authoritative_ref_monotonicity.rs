#![forbid(unsafe_code)]

use std::fs;
use std::path::{Component, Path, PathBuf};

use proto_ring_engine::github_authoritative_ref_monotonicity::conformance_internal;
use serde::Deserialize;
use serde_json::Value;

use crate::fixture;
use crate::model::Observation;

use super::transport::{fixture_failure, record, result, string_value};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProviderFixture {
    binding: BindingFixture,
    interactions: Vec<InteractionFixture>,
    kind: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BindingFixture {
    bytes_hex: String,
    path: String,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum InteractionFixture {
    HttpResponse { status: u16, body_hex: String },
    HttpError { status: u16 },
    NetworkError { message: String },
    TlsVerificationError { message: String },
    Timeout { message: String },
    IoError { message: String },
}

enum Interaction {
    HttpResponse { status: u16, body: Vec<u8> },
    HttpError { status: u16 },
    NetworkError,
    TlsVerificationError,
    Timeout,
    IoError,
}

pub(super) fn execute(value: &Value) -> Result<Observation, fixture::FixtureError> {
    let fixture: ProviderFixture = serde_json::from_value(value.clone())
        .map_err(|error| fixture_failure(format!("invalid GitHub ARM fixture: {error}")))?;
    if fixture.kind != "provider_observation" {
        return Err(fixture_failure("unsupported GitHub ARM fixture kind"));
    }
    let temporary = tempfile::tempdir()
        .map_err(|error| fixture_failure(format!("cannot create fixture root: {error}")))?;
    let binding_path = write_binding(temporary.path(), &fixture.binding)?;
    let interaction_count = fixture.interactions.len();
    let mut fake = FakeTransport::new(fixture.interactions)?;
    let observation =
        conformance_internal::observe_with_transport(&binding_path, |request| fake.get(request));
    fake.require_complete(interaction_count)?;
    Ok(result(record(vec![(
        "status",
        string_value(observation.status.serialized()),
    )])))
}

fn write_binding(root: &Path, binding: &BindingFixture) -> Result<PathBuf, fixture::FixtureError> {
    let relative = Path::new(&binding.path);
    if binding.path.is_empty()
        || relative.is_absolute()
        || relative
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::RootDir))
    {
        return Err(fixture_failure("binding path escapes fixture root"));
    }
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| fixture_failure(format!("binding parent failed: {error}")))?;
    }
    fs::write(&path, decode_hex(&binding.bytes_hex)?)
        .map_err(|error| fixture_failure(format!("binding write failed: {error}")))?;
    Ok(path)
}

fn decode_hex(value: &str) -> Result<Vec<u8>, fixture::FixtureError> {
    let (pairs, remainder) = value.as_bytes().as_chunks::<2>();
    if !remainder.is_empty() {
        return Err(fixture_failure("hex input has odd length"));
    }
    pairs
        .iter()
        .map(|pair| {
            let text = std::str::from_utf8(pair)
                .map_err(|_| fixture_failure("invalid fixture hex input"))?;
            u8::from_str_radix(text, 16).map_err(|_| fixture_failure("invalid fixture hex input"))
        })
        .collect()
}

struct FakeTransport {
    interactions: Vec<Interaction>,
    position: usize,
    requests: Vec<conformance_internal::HttpRequest>,
    invalid_request: bool,
    extra_request: bool,
}

impl FakeTransport {
    fn new(interactions: Vec<InteractionFixture>) -> Result<Self, fixture::FixtureError> {
        let interactions = interactions
            .into_iter()
            .map(prepare_interaction)
            .collect::<Result<_, _>>()?;
        Ok(Self {
            interactions,
            position: 0,
            requests: Vec::new(),
            invalid_request: false,
            extra_request: false,
        })
    }

    fn get(
        &mut self,
        request: &conformance_internal::HttpRequest,
    ) -> Result<conformance_internal::HttpResponse, conformance_internal::HttpFailure> {
        self.requests.push(request.clone());
        if !valid_request(request, self.position + 1) {
            self.invalid_request = true;
            return Err(conformance_internal::HttpFailure::Io);
        }
        let Some(interaction) = self.interactions.get(self.position) else {
            self.extra_request = true;
            return Err(conformance_internal::HttpFailure::Io);
        };
        self.position += 1;
        match interaction {
            Interaction::HttpResponse { status, body } => Ok(conformance_internal::HttpResponse {
                status: *status,
                body: body.clone(),
            }),
            Interaction::HttpError { status } => Ok(conformance_internal::HttpResponse {
                status: *status,
                body: Vec::new(),
            }),
            Interaction::NetworkError => Err(conformance_internal::HttpFailure::Network),
            Interaction::TlsVerificationError => Err(conformance_internal::HttpFailure::Tls),
            Interaction::Timeout => Err(conformance_internal::HttpFailure::Timeout),
            Interaction::IoError => Err(conformance_internal::HttpFailure::Io),
        }
    }

    fn require_complete(&self, expected: usize) -> Result<(), fixture::FixtureError> {
        if self.invalid_request {
            return Err(fixture_failure(
                "GitHub ARM request did not match transport contract",
            ));
        }
        if self.extra_request {
            return Err(fixture_failure("GitHub ARM made an extra provider request"));
        }
        if self.position != expected || self.requests.len() != expected {
            return Err(fixture_failure(
                "GitHub ARM did not consume every interaction",
            ));
        }
        Ok(())
    }
}

fn prepare_interaction(
    interaction: InteractionFixture,
) -> Result<Interaction, fixture::FixtureError> {
    Ok(match interaction {
        InteractionFixture::HttpResponse { status, body_hex } => Interaction::HttpResponse {
            status,
            body: decode_hex(&body_hex)?,
        },
        InteractionFixture::HttpError { status } => Interaction::HttpError { status },
        InteractionFixture::NetworkError { message } => {
            let _ = message;
            Interaction::NetworkError
        }
        InteractionFixture::TlsVerificationError { message } => {
            let _ = message;
            Interaction::TlsVerificationError
        }
        InteractionFixture::Timeout { message } => {
            let _ = message;
            Interaction::Timeout
        }
        InteractionFixture::IoError { message } => {
            let _ = message;
            Interaction::IoError
        }
    })
}

fn valid_request(request: &conformance_internal::HttpRequest, page: usize) -> bool {
    let expected_headers = [
        ("Accept", "application/vnd.github+json"),
        ("X-GitHub-Api-Version", "2026-03-10"),
        ("User-Agent", "proto-ring-github-arm-conformance"),
    ];
    request.url.starts_with("https://api.github.com/repos/")
        && request.url.ends_with(&format!("?per_page=100&page={page}"))
        && request.headers == expected_headers
        && !request
            .headers
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case("authorization"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_response_hex_is_rejected_as_fixture_transport() {
        let fixture = serde_json::json!({
            "kind": "provider_observation",
            "binding": { "path": "binding.md", "bytes_hex": "23" },
            "interactions": [
                { "kind": "http_response", "status": 200, "body_hex": "zz" }
            ]
        });
        let error = execute(&fixture).expect_err("invalid hex must be a fixture error");
        assert!(error.to_string().contains("invalid fixture hex input"));
    }
}
