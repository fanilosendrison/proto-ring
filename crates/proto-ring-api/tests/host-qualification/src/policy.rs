//! Fixed experiment policy, not a product host contract.
pub const IMAGE: &str = "docker.io/library/python@sha256:8a00ecb83ed9d0864dfd102a1bdb2067fda907ceb30bb86d09da9ee558b19590";
pub const CONFIG: &str = "sha256:141fe9b385f3bb6d618c07ace1836c0b3c72bf525566ac3eb13cd1f405069877";
pub const LABEL: &str = "org.proto-ring.api03-qualification";
pub const STREAM_LIMIT: usize = 65_536;
pub const REQUIRED: &[(&str, &[&str])] = &[
    ("H01", &["runner", "image", "runtime", "process-policy"]),
    ("H02", &["governance-mutation", "untracked-mutation"]),
    ("H03", &["positive-write", "denied-write"]),
    (
        "H04",
        &[
            "positive-coverage",
            "external-git-dir",
            "external-common-dir",
            "escaping-path",
            "replaced-binding",
        ],
    ),
    (
        "H05",
        &[
            "launch-binding",
            "positive-write",
            "program-denied",
            "control-denied",
            "environment",
        ],
    ),
    ("H06", &["simple", "descendant-stream", "descendant-writer"]),
    ("H07", &["streams", "positive-write", "evidence-denied"]),
    (
        "H08",
        &[
            "guard-loss",
            "network-before",
            "network-denied",
            "network-after",
        ],
    ),
];
pub fn hex(data: &[u8]) -> String {
    data.iter().map(|byte| format!("{byte:02x}")).collect()
}
pub fn sha256(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    hex(&Sha256::digest(data))
}
pub fn source_sha(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
