use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

/// Offline, separately resolved Cargo package: no workspace feature unification
/// and no lock on the parent test process's target directory.
pub struct CargoPackageProbe {
    pub directory: tempfile::TempDir,
    pub workspace: PathBuf,
    pub target: PathBuf,
    name: &'static str,
}
impl CargoPackageProbe {
    pub fn new(name: &'static str, manifest: &str) -> Self {
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let directory = tempfile::Builder::new()
            .prefix("api02-t7-probe-")
            .tempdir_in(workspace.join("target"))
            .unwrap();
        fs::write(directory.path().join("Cargo.toml"), manifest).unwrap();
        fs::copy(
            workspace.join("Cargo.lock"),
            directory.path().join("Cargo.lock"),
        )
        .unwrap();
        let target = workspace.join("target/api02-consumer-probes");
        Self {
            directory,
            workspace,
            target,
            name,
        }
    }
    pub fn cargo(&self, arguments: &[&str]) -> Output {
        let started = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let output = Command::new(env!("CARGO"))
            .args(arguments)
            .current_dir(self.directory.path())
            .env("CARGO_TARGET_DIR", &self.target)
            .env("CARGO_NET_OFFLINE", "true")
            .output()
            .unwrap();
        let ended = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let log = format!(
            "START_UNIX_NS={started}\nCOMMAND=cargo {arguments:?}\nCWD={}\nCARGO_TARGET_DIR={}\nCARGO_NET_OFFLINE=true\nSTDOUT:\n{}\nSTDERR:\n{}\nEND_UNIX_NS={ended}\nEXIT_CODE={:?}\n",
            self.directory.path().display(),
            self.target.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
            output.status.code()
        );
        let evidence = self.workspace.join("target/api02-evidence");
        fs::create_dir_all(&evidence).unwrap();
        fs::write(
            evidence.join(format!("t7-probe-{}-{started}.log", self.name)),
            log,
        )
        .unwrap();
        assert!(
            output.status.success(),
            "cargo {arguments:?}:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        output
    }
    pub fn lock_offline(&self) {
        self.cargo(&["generate-lockfile", "--offline"]);
    }
    pub fn binary(&self, name: &str) -> PathBuf {
        self.target
            .join("debug")
            .join(format!("{name}{}", std::env::consts::EXE_SUFFIX))
    }
}
pub fn quoted(path: impl AsRef<Path>) -> String {
    serde_json::to_string(path.as_ref().to_str().unwrap()).unwrap()
}
