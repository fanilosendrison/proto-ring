use super::cargo_package_probe::{CargoPackageProbe, quoted};
use std::path::Path;

#[test]
fn isolated_engine_control_probe_exercises_real_drift_stages_and_measures_facade_provider_entries()
{
    let api = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = api.join("tests/support/canonical-controls.rs");
    let engine = api.join("../proto-ring-engine");
    let manifest = format!(
        "[workspace]\n[package]\nname='api02-canonical-controls'\nversion='0.1.0'\nedition='2024'\npublish=false\n[dependencies]\nproto-ring-api={{path={}}}\nproto-ring-engine={{path={},features=['conformance-internal']}}\nserde_json='1.0'\ntempfile='3.23'\n[[test]]\nname='canonical-controls'\npath={}\n",
        quoted(api),
        quoted(engine),
        quoted(source)
    );
    let probe = CargoPackageProbe::new("canonical-controls", &manifest);
    probe.lock_offline();
    let output = probe.cargo(&[
        "test",
        "--test",
        "canonical-controls",
        "--offline",
        "--locked",
        "--",
        "--nocapture",
    ]);
    let report = format!(
        "{}\n{}",
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap()
    );
    assert!(report.contains("5 passed; 0 failed"));
    for marker in [
        "VALID_STATE_DRIFT_STAGE=RepositoryState",
        "CORRUPTED_BINDING_STAGE=GovernanceBindings",
        "SCOPE_DRIFT_STAGE=Composition",
        "FACADE_PROVIDER_ENTRIES=0",
    ] {
        assert!(report.contains(marker), "{marker}");
    }
}
