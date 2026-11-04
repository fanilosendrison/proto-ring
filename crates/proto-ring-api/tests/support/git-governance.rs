use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Value, json};

/// Self-contained unborn Git repository; no user config, objects or credentials.
pub struct GitGovernance {
    _temporary: tempfile::TempDir,
    pub root: PathBuf,
}

impl GitGovernance {
    pub fn minimal() -> Self {
        Self::named(OsStr::new("repository"), false)
    }

    pub fn full() -> Self {
        Self::named(OsStr::new("repository"), true)
    }

    pub fn named(name: &OsStr, full: bool) -> Self {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().join(name);
        std::fs::create_dir(&root).unwrap();
        let root = root.canonicalize().unwrap();
        let fixture = Self {
            _temporary: temporary,
            root,
        };
        fixture.git(&["init", "-q"]);
        fixture.git(&["symbolic-ref", "HEAD", "refs/heads/main"]);
        fixture.git(&["config", "core.autocrlf", "false"]);
        let mut capabilities = json!({
            "governance_authority":{"configuration":{},"routes":{"profile":"authority.md"}},
            "shared_governance_provider":{"configuration":{},"routes":{"registry":"bindings.md"}}
        });
        let mut sources = json!({"bindings":{"repository_target":"bindings.md"}});
        let mut roles = json!({"bindings":"authority"});
        if full {
            for (capability, route, carrier, source) in [
                ("governed_objects", "profile", "objects.md", "objects"),
                (
                    "repository_integrity",
                    "profile",
                    "integrity.md",
                    "integrity",
                ),
                (
                    "projection_integrity",
                    "registry",
                    "projections.md",
                    "projections",
                ),
                (
                    "evidence_requirements",
                    "registry",
                    "evidence.md",
                    "evidence",
                ),
            ] {
                capabilities[capability] = json!({"configuration":{},"routes":{route:carrier}});
                sources[source] = json!({"repository_target":carrier});
                roles[source] = json!("authority");
            }
        }
        fixture.write(
            "AGENTS.md",
            json!({"repository_governance":{
                "model_version":2,"provider":{"id":"proto-ring","binding":{
                    "capability":"shared_governance_provider","route":"registry"
                }},"capabilities":capabilities
            }}),
        );
        fixture.write(
            "authority.md",
            json!({"governance_authority":{
                "model_version":1,"sources":sources,"responsibilities":{
                    "quality":{"roles":roles,"precedence":[]}
                }
            }}),
        );
        fixture.write("bindings.md", json!({"governance_bindings":{
            "model_version":1,"source":"bindings","bindings":{
                "provider":{"kind":"executable_provider","scope":{"kind":"logical_provider"},
                    "identity":{"repository":"example/provider","commit":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"},
                    "authority":{"responsibility":"quality","source":"bindings"}}
            }
        }}));
        if full {
            fixture.write(
                "objects.md",
                json!({"governed_objects":{
                    "model_version":1,"interfaces":{"repository":{"objects":{
                        "tree":{"responsibilities":["quality"],"relations":[]}
                    }}}
                }}),
            );
            let validation = json!({"responsibility":"quality","prerequisites":[],
                "instances":{"kind":"single"},"command":{"kind":"command","environment":"declared-only",
                    "arguments":["RAW-IDENTITY-SENTINEL","never-execute"],"undetermined_exit_codes":[127]}});
            fixture.write("integrity.md", json!({"repository_integrity":{
                "model_version":1,"authority":{"responsibility":"quality","source":"integrity"},
                "environments":["declared-only"],"continue_after_non_satisfied":false,
                "validations":{"first":validation,"second":validation},"order":["second","first"]
            }}));
            fixture.write("projections.md", json!({"projection_registry":{
                "model_version":1,"authority":{"responsibility":"quality","source":"projections"},"projections":{}
            }}));
            fixture.write("evidence.md", json!({"evidence_requirements":{
                "model_version":1,"authority":{"responsibility":"quality","source":"evidence"},"requirements":{}
            }}));
        }
        fixture.git(&["add", "-A"]);
        fixture
    }

    pub fn write(&self, path: &str, metadata: Value) {
        let text = format!(
            "---\n{}\n---\n# Fixture\n",
            serde_json::to_string_pretty(&metadata).unwrap()
        );
        std::fs::write(self.root.join(path), text).unwrap();
    }

    pub fn read(&self, path: &str) -> Value {
        let text = std::fs::read_to_string(self.root.join(path)).unwrap();
        serde_json::from_str(text.split("---").nth(1).unwrap()).unwrap()
    }

    pub fn git(&self, arguments: &[&str]) -> Vec<u8> {
        let mut command = Command::new("git");
        for (key, _) in std::env::vars_os() {
            if key
                .to_str()
                .is_some_and(|key| key.to_ascii_uppercase().starts_with("GIT_"))
            {
                command.env_remove(key);
            }
        }
        let output = command
            .arg("-C")
            .arg(&self.root)
            .args(arguments)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env(
                "GIT_CONFIG_GLOBAL",
                self.root.join(".git/absent-global-config"),
            )
            .env("GIT_TERMINAL_PROMPT", "0")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {arguments:?}: {:?}",
            output.stderr
        );
        output.stdout
    }
}

pub fn native_units(path: &Path) -> Vec<u8> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        path.as_os_str().as_bytes().to_vec()
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        path.as_os_str()
            .encode_wide()
            .flat_map(u16::to_be_bytes)
            .collect()
    }
}
