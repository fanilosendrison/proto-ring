use std::env;
use std::process::Command;

fn main() {
    let manifest_dir = env::var_os("CARGO_MANIFEST_DIR")
        .expect("Cargo must provide the compatibility crate manifest directory");
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(manifest_dir)
        .output()
        .expect("cannot execute git rev-parse HEAD for compatibility source identity");
    assert!(
        output.status.success(),
        "git rev-parse HEAD failed for compatibility source identity"
    );
    let stdout = String::from_utf8(output.stdout)
        .expect("git rev-parse HEAD did not return UTF-8 source identity");
    let source_commit = stdout.trim_end_matches(['\r', '\n']);
    assert!(
        source_commit.len() == 40
            && source_commit
                .bytes()
                .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f')),
        "git rev-parse HEAD did not return 40 lowercase hexadecimal characters"
    );
    println!("cargo:rustc-env=PROTO_RING_SOURCE_COMMIT={source_commit}");
}
