//! Explicit synthetic subject/Git resource witnesses, not RepositoryState encoding.
use super::{Bundle, put};
use serde_json::{Value, json};
pub(super) fn observations(b: &Bundle) {
    let subject = identity(101, 16832);
    put(&b.root, "subject-root-identity.json", &subject);
    let roles = [
        (".git", true),
        (".git/HEAD", false),
        (".git/index", false),
        (".git/config", false),
        (".git/objects", true),
        (".git/objects/info", true),
        (".git/refs", true),
        (".git/refs/heads", true),
        (".git/refs/heads/main", false),
    ];
    let mut administration = Vec::new();
    for (index, (relative, directory)) in roles.into_iter().enumerate() {
        let path = format!("/owned/run/subject/{relative}");
        administration.push(json!({"path":path,"resolved":path,"present":true,"identity":identity(110+index as u64,if directory{16832}else{33152})}));
    }
    for relative in [".git/objects/info/alternates", ".git/commondir"] {
        administration.push(json!({"path":format!("/owned/run/subject/{relative}"),"resolved":null,"present":false,"identity":null}));
    }
    put(
        &b.root,
        "positive-git-identities.json",
        &json!({"campaign":"api03-123-1","subject":{"path":"/owned/run/subject","resolved":"/owned/run/subject","present":true,"identity":subject},"administration":administration}),
    );
}
pub(super) fn invalidations(b: &Bundle) {
    put(
        &b.root,
        "replaced-binding.json",
        &json!({"campaign":"api03-123-1","path":"/owned/run/coverage-binding","before":identity(301,33152),"after":identity(302,33152),"checkpoint_refused":true}),
    );
    put(
        &b.root,
        "guard-loss.json",
        &json!({"campaign":"api03-123-1","path":"/owned/run/subject","before":identity(101,16832),"after":identity(501,16832),"protection_active":false,"next_capture_refused":true}),
    );
}
fn identity(inode: u64, mode: u32) -> Value {
    json!({"device":1,"inode":inode,"mode":mode,"links":if mode==16832{2}else{1}})
}
