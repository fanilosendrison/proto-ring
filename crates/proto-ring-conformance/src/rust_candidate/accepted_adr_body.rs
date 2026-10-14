#![forbid(unsafe_code)]

use proto_ring_engine::accepted_adr_body::{self, AcceptedBodyDiagnosticKind};

use crate::fixture::{self, MaterializedFixture};
use crate::model::{Observation, TransportValue};

use super::transport::{exact_record, fixture_failure, record, result, string_value};

pub(super) fn execute(fixture: &MaterializedFixture) -> Result<Observation, fixture::FixtureError> {
    let fields = exact_record(
        fixture.arguments(),
        "Accepted ADR Body invocation",
        &["operation"],
    )?;
    if fixture::string(fields["operation"])? != "check_accepted_body" {
        return Err(fixture_failure("unsupported Accepted ADR Body operation"));
    }
    let repository = fixture
        .root()
        .ok_or_else(|| fixture_failure("Accepted ADR Body check requires repository plan"))?;
    let report = accepted_adr_body::check(repository);
    let category = if report.diagnostics.is_empty() {
        TransportValue::Null(())
    } else if report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.kind == AcceptedBodyDiagnosticKind::BodyChanged)
    {
        string_value("body_changed")
    } else {
        string_value("historical_state_invalid")
    };
    let status = if report.diagnostics.is_empty() {
        "PASS"
    } else {
        "NON_PASS"
    };
    Ok(result(record(vec![
        ("category", category),
        ("status", string_value(status)),
    ])))
}
