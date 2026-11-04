#![forbid(unsafe_code)]

use proto_ring_api::{
    ApiError, ErrorCode, ExplainQuery, ExplanationResult, Identifier, InspectSelector, Inspection,
    open,
};
use std::{io::Write, path::Path, process::ExitCode};

struct Views {
    complete: Inspection,
    targeted: Inspection,
    explanation: ExplanationResult,
}

fn observe(repository: &Path) -> Result<Views, ApiError> {
    let observation = open(repository)?;
    let id = Identifier::try_from("governance_authority").expect("nonempty example capability ID");
    let complete = observation.inspect(InspectSelector::All {})?;
    let targeted = observation.inspect(InspectSelector::Capability { id: id.clone() })?;
    let explanation = observation.explain(ExplainQuery::Capability { id })?;
    Ok(Views {
        complete,
        targeted,
        explanation,
    })
}

fn main() -> ExitCode {
    let mut args = std::env::args_os().skip(1);
    let repository = args.next().unwrap_or_else(|| ".".into());
    if args.next().is_some() {
        let _ = writeln!(
            std::io::stderr(),
            "usage: inspect-governance [NATIVE_REPOSITORY]"
        );
        return ExitCode::FAILURE;
    }
    match observe(Path::new(&repository)) {
        Ok(views) => {
            // This consumer prints a usage summary, not a wire API response or
            // a replacement for the complete values returned above.
            let mut stdout = std::io::stdout().lock();
            if writeln!(
                stdout,
                "complete={:?}\ntargeted={:?}\nexplanation={:?}",
                views.complete.selector(),
                views.targeted.selector(),
                views.explanation.query()
            )
            .and_then(|()| stdout.flush())
            .is_ok()
            {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(error) => {
            let label = match error.code {
                ErrorCode::GovernanceLoadFailed => "governance_load_failed",
                ErrorCode::ComponentNotDeclared => "component_not_declared",
                ErrorCode::SubjectNotFound => "subject_not_found",
                _ => "api_failure",
            };
            // Stage is typed. Detail is neither parsed nor used for control.
            let _ = writeln!(std::io::stderr(), "{label}: {:?}", error.stage);
            ExitCode::FAILURE
        }
    }
}
