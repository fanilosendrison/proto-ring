#[path = "support/git-governance.rs"]
mod git_governance;

use git_governance::GitGovernance;
use proto_ring_api::{CoherenceBasis, ErrorCode, GovernanceStage, NativePath, open};

#[test]
fn minimal_real_git_observation_exposes_original_root_and_explicit_scope() {
    let fixture = GitGovernance::minimal();
    let observed = open(&fixture.root).unwrap();
    assert_eq!(observed.repository(), fixture.root);
    assert_eq!(
        git_governance::native_units(observed.repository()),
        git_governance::native_units(&fixture.root)
    );
    assert_eq!(
        observed.provenance().repository.to_path_buf().unwrap(),
        fixture.root
    );
    let expected: Vec<_> = ["AGENTS.md", "authority.md", "bindings.md"]
        .into_iter()
        .map(std::path::PathBuf::from)
        .collect();
    assert_eq!(observed.scope_paths(), expected);
    assert_eq!(
        observed.provenance().scope_paths,
        expected
            .iter()
            .map(|p| NativePath::from_path(p).unwrap())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        observed.reference().coherence_basis,
        CoherenceBasis::OrdinaryCapture
    );
    assert!(observed.profile_reference().is_none());
}

#[test]
fn full_real_git_observation_retains_full_profile_without_executing_it() {
    let fixture = GitGovernance::full();
    let observed = open(&fixture.root).unwrap();
    assert!(observed.profile_reference().is_some());
    assert_eq!(observed.scope_paths().len(), 7);
    let public = format!("{observed:?}");
    assert!(!public.contains("RAW-IDENTITY-SENTINEL"));
    assert!(!public.contains("RepositoryGovernanceState"));
    let serialized = serde_json::to_string(observed.provenance()).unwrap();
    assert!(!serialized.contains("RAW-IDENTITY-SENTINEL"));
}

#[test]
fn retained_observation_is_unchanged_after_edit_and_reopen() {
    let fixture = GitGovernance::full();
    let observed = open(&fixture.root).unwrap();
    let original = observed.provenance().clone();
    let original_profile = observed.profile_reference().unwrap().clone();
    let mut profile = fixture.read("integrity.md");
    profile["repository_integrity"]["validations"]["second"]["command"]["arguments"] =
        serde_json::json!(["changed-second-validation"]);
    fixture.write("integrity.md", profile);
    let reopened = open(&fixture.root).unwrap();
    assert_eq!(observed.provenance(), &original);
    assert_eq!(observed.profile_reference(), Some(&original_profile));
    assert_eq!(
        observed.reference().repository,
        reopened.reference().repository
    );
    assert_eq!(observed.reference().scope, reopened.reference().scope);
    assert_ne!(observed.reference().state, reopened.reference().state);
    assert_ne!(observed.profile_reference(), reopened.profile_reference());
    std::fs::remove_dir_all(&fixture.root).unwrap();
    assert_eq!(observed.provenance(), &original);
    assert_eq!(observed.repository(), fixture.root);
    assert_eq!(observed.scope_paths().len(), 7);
    assert!(open(&fixture.root).is_err());
}

#[test]
fn canonical_load_errors_remain_structured_without_partial_observations() {
    let not_git = tempfile::tempdir().unwrap();
    let error = open(not_git.path()).unwrap_err();
    assert_eq!(error.code, ErrorCode::GovernanceLoadFailed);
    assert_eq!(error.stage, Some(GovernanceStage::RepositoryState));
    assert_eq!(error.effects_possible, Some(false));
    let fixture = GitGovernance::minimal();
    std::fs::remove_file(fixture.root.join("AGENTS.md")).unwrap();
    let error = open(&fixture.root).unwrap_err();
    assert_eq!(error.stage, Some(GovernanceStage::Bootstrap));
    assert!(error.detail.is_some());
    assert!(error.attempt_id.is_none());
}

#[test]
fn actual_filesystem_root_preserves_the_canonical_engine_units() {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let fixture = GitGovernance::named(std::ffi::OsStr::from_bytes(b"repository-\xff"), false);
        let observed = open(&fixture.root).unwrap();
        assert_eq!(
            observed.repository().as_os_str().as_bytes(),
            fixture.root.as_os_str().as_bytes()
        );
        assert_eq!(
            observed.provenance().repository.to_path_buf().unwrap(),
            fixture.root
        );
        // Some macOS filesystems resolve the raw suffix to Unicode "%FF".
        // That is not evidence that non-UTF-8 names survived on the filesystem.
        if fixture.root.to_str().is_some() {
            assert!(matches!(
                observed.provenance().repository,
                NativePath::Utf8 { .. }
            ));
        } else {
            assert!(matches!(
                observed.provenance().repository,
                NativePath::UnixBytesBase64 { .. }
            ));
        }
    }
    #[cfg(windows)]
    {
        let fixture = GitGovernance::named(std::ffi::OsStr::new("repository-é"), false);
        let observed = open(&fixture.root).unwrap();
        assert_eq!(
            observed.provenance().repository.to_path_buf().unwrap(),
            fixture.root
        );
    }
}
