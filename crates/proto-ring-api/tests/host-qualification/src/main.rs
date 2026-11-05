use api03_host_qualification::{
    Result,
    campaign_context::Context,
    docker::Docker,
    ensure,
    evidence::{Campaign, Evidence},
    policy, probes, source,
    source_state::Guard,
    subject::Subject,
    verification,
};
use std::path::{Path, PathBuf};
fn main() -> std::process::ExitCode {
    match execute() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("qualification incomplete: {e}");
            std::process::ExitCode::FAILURE
        }
    }
}
fn execute() -> Result<()> {
    let a: Vec<_> = std::env::args().skip(1).collect();
    match a.as_slice(){
 [mode] if mode=="cleanup"=>api03_host_qualification::cleanup::fallback(),
 [mode,root,sha,run,attempt,source] if mode=="verify"=>verification::verify(Path::new(root),sha,run,attempt,Path::new(source)),
 [mode,sha] if mode=="campaign"=>campaign(sha),
 _=>Err("usage: campaign EXPECTED_SHA | verify EVIDENCE_ROOT SHA RUN ATTEMPT SOURCE_GIT_ROOT | cleanup".into()),
 }
}
fn campaign(expected: &str) -> Result<()> {
    ensure(
        cfg!(target_os = "linux") && policy::source_sha(expected),
        "only reviewed Linux job may execute campaign",
    )?;
    let parent = PathBuf::from(std::env::var("RUNNER_TEMP")?);
    let run = std::env::var("GITHUB_RUN_ID")?;
    let attempt = std::env::var("GITHUB_RUN_ATTEMPT")?;
    ensure(
        !run.is_empty() && run.bytes().all(|b| b.is_ascii_digit()) && attempt == "1",
        "invalid run or rerun; no automatic retry",
    )?;
    let e = Evidence::create(&parent.join("api03-host-evidence/campaign"))?;
    let mut campaign = Campaign::new(
        expected.into(),
        "0".repeat(40),
        run.clone(),
        attempt.clone(),
    );
    e.json("initial-campaign.json", &campaign)?;
    let source = match source::inventory(&e, expected) {
        Ok(s) => s,
        Err(error) => {
            e.bytes("failure.txt", error.to_string().as_bytes())?;
            e.json("campaign.json", &campaign)?;
            e.inventory()?;
            return Err(error);
        }
    };
    campaign.source_tree = source.tree.clone();
    let source_root = std::env::current_dir()?.canonicalize()?;
    let program = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .canonicalize()?;
    let executable = std::env::current_exe()?;
    e.json("controller.json",&serde_json::json!({"source_sha":expected,"source_tree":source.tree,"binary_sha256":policy::sha256(&std::fs::read(&executable)?),"executable":executable}))?;
    let mut guard = Guard::new(source);
    let before = guard.check(&e, &source_root, "before_probes");
    if let Err(error) = before {
        guard.finish(&e)?;
        e.bytes("failure.txt", error.to_string().as_bytes())?;
        e.json("campaign.json", &campaign)?;
        e.inventory()?;
        return Err(error);
    }
    let subject = Subject::new(&parent, &e)?;
    let mut docker = Docker::new(e.clone(), format!("api03-{run}-{attempt}"), program.clone())?;
    Context::capture(&e, &subject, &source_root, &docker.user, &campaign)?;
    let outcome = probes::run(&mut campaign, &mut docker, &subject);
    match docker.cleanup() {
        Ok((containers, networks)) => {
            campaign.cleanup_verified = containers.is_empty() && networks.is_empty();
            campaign.cleanup_containers = containers;
            campaign.cleanup_networks = networks;
        }
        Err(error) => e.bytes("cleanup-error.txt", error.to_string().as_bytes())?,
    }
    let final_source = guard.check(&e, &source_root, "after_probes");
    guard.finish(&e)?;
    if let Err(error) = &outcome {
        e.bytes("failure.txt", error.to_string().as_bytes())?;
    }
    e.json("campaign.json", &campaign)?;
    e.inventory()?;
    outcome?;
    final_source?;
    verification::verify(&e.root, expected, &run, &attempt, &source_root)
}
