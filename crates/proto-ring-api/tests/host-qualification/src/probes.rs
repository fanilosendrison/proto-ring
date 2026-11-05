//! Fixed H01-H08 orchestration, separate from process data and verdict verification.
use crate::write_probes::{denied, run_output, target_unchanged, writable_control};
use crate::{
    Result,
    coverage::{self, equal},
    docker::Docker,
    ensure,
    evidence::{Campaign, Case, Check, Evidence, Status},
    lifetime, policy,
    subject::Subject,
    transport, verification,
};
use serde_json::{Value, json};
use std::fs;

fn record(
    campaign: &mut Campaign,
    evidence: &Evidence,
    name: &str,
    result: Result<Vec<Check>>,
) -> Result<()> {
    let result = result.and_then(|checks| {
        verification::checks(&evidence.root, &checks)?;
        Ok(checks)
    });
    match result {
        Ok(checks) => {
            *campaign.cases.get_mut(name).ok_or("undeclared subcase")? = Case {
                status: Status::Pass,
                reason: "controller observations and required checks passed".into(),
                checks,
            };
            Ok(())
        }
        Err(error) => {
            *campaign.cases.get_mut(name).ok_or("undeclared subcase")? = Case {
                status: Status::Fail,
                reason: error.to_string(),
                checks: Vec::new(),
            };
            Err(error)
        }
    }
}
pub fn run(campaign: &mut Campaign, docker: &mut Docker, subject: &Subject) -> Result<()> {
    let e = docker.evidence.clone();
    record(campaign, &e, "H01/runner", runner(&e))?;
    for (case, action) in [
        (
            "H01/runtime",
            Docker::prepare_runtime as fn(&mut Docker) -> Result<()>,
        ),
        ("H01/image", Docker::prepare_image),
    ] {
        if let Err(error) = action(docker) {
            let outcome = campaign.cases.get_mut(case).ok_or("setup case absent")?;
            outcome.status = Status::Unavailable;
            outcome.reason = error.to_string();
            return Err(error);
        }
        if case == "H01/runtime" {
            let info = verification::json(&e.root, "runtime-info.json")?;
            record(
                campaign,
                &e,
                case,
                Ok(vec![equal(
                    json!(
                        info["logging_drivers"]
                            .as_array()
                            .is_some_and(|v| v.contains(&json!("local")))
                    ),
                    json!(true),
                )]),
            )?;
        }
    }
    let image = verification::json(&e.root, "image.json")?;
    record(
        campaign,
        &e,
        "H01/image",
        Ok(vec![
            equal(image[0]["Id"].clone(), json!(policy::CONFIG)),
            equal(image[0]["Architecture"].clone(), json!("amd64")),
            equal(image[0]["Os"].clone(), json!("linux")),
        ]),
    )?;
    // Cover the whole standalone subject and real Git administration before RGS.
    record(
        campaign,
        &e,
        "H04/positive-coverage",
        subject
            .coverage(&e, "positive")
            .map(|covered| vec![equal(json!(covered), json!(true))]),
    )?;
    let id = docker.run(
        "policy",
        "policy",
        &[],
        &subject.root,
        &[subject.mount(false)],
        None,
    )?;
    crate::environment_evidence::capture_interpreter(docker, &id)?;
    let process: Value = serde_json::from_slice(&fs::read(e.root.join("policy-logs.stdout"))?)?;
    let mut checks = vec![
        Docker::closed("policy", &id),
        equal(process["pid"].clone(), json!(1)),
        equal(process["seccomp"].clone(), json!(2)),
    ];
    for name in [
        "capabilities_zero",
        "no_new_privileges",
        "rootfs_erofs",
        "credential_names_absent",
    ] {
        checks.push(equal(process[name].clone(), json!(true)));
    }
    let uid: u32 = docker.user.split(':').next().ok_or("uid absent")?.parse()?;
    checks.push(equal(process["uid"].clone(), json!(uid)));
    record(campaign, &e, "H01/process-policy", Ok(checks))?;

    let context: crate::campaign_context::Context =
        serde_json::from_slice(&fs::read(e.root.join("context.json"))?)?;
    let before_record = crate::mutation_evidence::capture_before(&e, &subject.root, "governance")?;
    let before = before_record["observation"].clone();
    e.json("rgs-before.json", &before)?;
    let mut checks = run_output(
        docker,
        subject,
        "governance",
        "governance",
        &[subject
            .root
            .join("authority.md")
            .to_string_lossy()
            .into_owned()],
        true,
        b"governance-written\n",
    )?;
    let after = subject.observation()?;
    e.json("rgs-after.json", &after)?;
    let id = verification::json(&e.root, "governance-created.json")?[0]["Id"]
        .as_str()
        .ok_or("governance container identity")?
        .to_owned();
    crate::mutation_evidence::capture_after(&e, &context, "governance", before_record, &id)?;
    checks.push(Check::DifferentCanonical { before, after });
    let expected = [
        include_bytes!("../fixtures/authority.md").as_slice(),
        b"\nBounded inert change.\n",
    ]
    .concat();
    checks.push(target_unchanged(
        &e,
        "governance",
        &subject.root.join("authority.md"),
        &expected,
    )?);
    record(campaign, &e, "H02/governance-mutation", Ok(checks))?;
    let before_record = crate::mutation_evidence::capture_before(&e, &subject.root, "untracked")?;
    let before = before_record["observation"].clone();
    let state_before = proto_ring_engine::repository_state::capture(&subject.root, &[])?;
    let mut checks = run_output(
        docker,
        subject,
        "untracked",
        "untracked",
        &[subject
            .root
            .join("untracked-witness")
            .to_string_lossy()
            .into_owned()],
        true,
        b"untracked-written\n",
    )?;
    let id = verification::json(&e.root, "untracked-created.json")?[0]["Id"]
        .as_str()
        .ok_or("untracked container identity")?
        .to_owned();
    crate::mutation_evidence::capture_after(&e, &context, "untracked", before_record, &id)?;
    let state_after = proto_ring_engine::repository_state::capture(&subject.root, &[])?;
    checks.push(equal(
        json!(state_before.identity != state_after.identity),
        json!(true),
    ));
    checks.push(Check::DifferentCanonical {
        before,
        after: subject.observation()?,
    });
    record(campaign, &e, "H02/untracked-mutation", Ok(checks))?;

    record(
        campaign,
        &e,
        "H03/positive-write",
        writable_control(docker, subject, "writer-positive"),
    )?;
    let mut checks = denied(
        docker,
        subject,
        "writer-denied",
        subject.root.join("target").to_str().ok_or("target path")?,
        &[subject.mount(false)],
    )?;
    checks.push(target_unchanged(
        &e,
        "writer-denied",
        &subject.root.join("target"),
        b"before\n",
    )?);
    record(campaign, &e, "H03/denied-write", Ok(checks))?;
    for (name, checks) in coverage::counterexamples(subject, &e)? {
        record(campaign, &e, &name, Ok(checks))?;
    }

    record(
        campaign,
        &e,
        "H05/launch-binding",
        Ok(vec![equal(
            json!(policy::sha256(&fs::read(docker.program.join("probe.py"))?)),
            json!(policy::sha256(include_bytes!("../fixtures/probe.py"))),
        )]),
    )?;
    record(
        campaign,
        &e,
        "H05/positive-write",
        writable_control(docker, subject, "program-positive"),
    )?;
    let mut checks = denied(
        docker,
        subject,
        "program-denied",
        "/program/probe.py",
        &[subject.mount(false)],
    )?;
    checks.push(target_unchanged(
        &e,
        "program-denied",
        &docker.program.join("probe.py"),
        include_bytes!("../fixtures/probe.py"),
    )?);
    record(campaign, &e, "H05/program-denied", Ok(checks))?;
    let control = subject.owned.path().join("control");
    fs::create_dir(&control)?;
    fs::write(control.join("target"), b"before\n")?;
    let binds = vec![
        subject.mount(false),
        (control.clone(), "/control".into(), false),
    ];
    let mut checks = denied(docker, subject, "control-denied", "/control/target", &binds)?;
    checks.push(target_unchanged(
        &e,
        "control-denied",
        &control.join("target"),
        b"before\n",
    )?);
    record(campaign, &e, "H05/control-denied", Ok(checks))?;
    record(
        campaign,
        &e,
        "H05/environment",
        Ok(vec![
            equal(process["credential_names_absent"].clone(), json!(true)),
            equal(
                json!(
                    process["interpreter_sha256"]
                        .as_str()
                        .is_some_and(|s| s.len() == 64)
                ),
                json!(true),
            ),
        ]),
    )?;

    record(
        campaign,
        &e,
        "H06/simple",
        run_output(docker, subject, "simple", "simple", &[], false, b"simple\n"),
    )?;
    record(
        campaign,
        &e,
        "H06/descendant-stream",
        lifetime::descendants(docker, subject, "descendant-stream", "stream"),
    )?;
    record(
        campaign,
        &e,
        "H06/descendant-writer",
        lifetime::descendants(docker, subject, "descendant-writer", "writer"),
    )?;
    let id = docker.run(
        "output",
        "output",
        &[],
        &subject.root,
        &[subject.mount(false)],
        None,
    )?;
    record(
        campaign,
        &e,
        "H07/streams",
        Ok(vec![
            transport::stream_check(
                "output",
                23,
                b"api03-stdout\x00\xff\n",
                b"api03-stderr\xfe\n",
            ),
            Docker::closed("output", &id),
        ]),
    )?;
    record(
        campaign,
        &e,
        "H07/positive-write",
        writable_control(docker, subject, "evidence-positive"),
    )?;
    let sacrificial = subject.owned.path().join("sacrificial-evidence");
    fs::create_dir(&sacrificial)?;
    fs::write(sacrificial.join("target"), b"before\n")?;
    let mut checks = denied(
        docker,
        subject,
        "evidence-denied",
        "/evidence-target/target",
        &[
            subject.mount(false),
            (sacrificial.clone(), "/evidence-target".into(), false),
        ],
    )?;
    checks.push(target_unchanged(
        &e,
        "evidence-denied",
        &sacrificial.join("target"),
        b"before\n",
    )?);
    record(campaign, &e, "H07/evidence-denied", Ok(checks))?;
    record(
        campaign,
        &e,
        "H08/guard-loss",
        coverage::invalidate(subject, &e),
    )?;
    for (name, checks) in lifetime::network(docker, subject)? {
        record(campaign, &e, &name, Ok(checks))?;
    }
    Ok(())
}
fn runner(e: &Evidence) -> Result<Vec<Check>> {
    let mut values = serde_json::Map::new();
    for name in ["RUNNER_OS", "RUNNER_ARCH", "ImageOS", "ImageVersion"] {
        values.insert(
            name.into(),
            json!(std::env::var(name).map_err(|_| format!("{name} unavailable"))?),
        );
    }
    ensure(
        values["RUNNER_OS"] == "Linux"
            && values["RUNNER_ARCH"] == "X64"
            && values["ImageOS"] == "ubuntu24"
            && values["ImageVersion"]
                .as_str()
                .is_some_and(|s| !s.is_empty()),
        "candidate runner mismatch",
    )?;
    e.json("runner.json", &values)?;
    crate::environment_evidence::capture_kernel(e)?;
    Ok(vec![
        equal(
            Value::Object(values.clone())["RUNNER_OS"].clone(),
            json!("Linux"),
        ),
        equal(Value::Object(values)["ImageOS"].clone(), json!("ubuntu24")),
    ])
}
