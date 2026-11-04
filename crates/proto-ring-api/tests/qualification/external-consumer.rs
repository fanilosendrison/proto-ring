use super::{
    cargo_package_probe::{CargoPackageProbe, quoted},
    git_governance::GitGovernance,
};
use std::{path::Path, process::Command};

#[test]
fn standalone_root_api_only_example_compiles_offline_and_runs_real_valid_and_error_paths() {
    let api = Path::new(env!("CARGO_MANIFEST_DIR"));
    let example = api.join("examples/inspect-governance.rs");
    let manifest = format!(
        "[workspace]\n[package]\nname='api02-external-consumer'\nversion='0.1.0'\nedition='2024'\npublish=false\n[dependencies]\nproto-ring-api={{path={}}}\n[[bin]]\nname='api02-external-consumer'\npath={}\n",
        quoted(api),
        quoted(&example)
    );
    let probe = CargoPackageProbe::new("external-consumer", &manifest);
    probe.lock_offline();
    probe.cargo(&["build", "--offline", "--locked"]);
    let source = std::fs::read_to_string(&example).unwrap();
    assert!(!source.contains("proto_ring_engine"));
    for fixture in [GitGovernance::minimal(), GitGovernance::full()] {
        let output = Command::new(probe.binary("api02-external-consumer"))
            .arg(&fixture.root)
            .output()
            .unwrap();
        assert!(output.status.success(), "{:?}", output.stderr);
        assert!(output.stderr.is_empty());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("complete=All"));
        assert!(stdout.contains("targeted=Capability"));
        assert!(stdout.contains("explanation=Capability"));
        eprintln!("EXTERNAL_CONSUMER_VALID_STDOUT:\n{stdout}");
    }
    let invalid = tempfile::tempdir().unwrap();
    let output = Command::new(probe.binary("api02-external-consumer"))
        .arg(invalid.path())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("governance_load_failed: Some(RepositoryState)"));
    eprintln!("EXTERNAL_CONSUMER_ERROR_STDERR:\n{stderr}");
}
