#[path = "support/offline-bundle.rs"]
mod offline;
use offline::{Bundle, get, put};
use serde_json::Value;
fn rewrite(v: &mut Value, key: &str) {
    match v {
        Value::String(s) => {
            if key == "run_id" && s == "123" {
                *s = "999".into();
                return;
            }
            if s == "000000000000" {
                *s = "ff0000000000".into();
                return;
            }
            for index in (1..=16).chain([90, 91]) {
                if *s == format!("{index:064x}") {
                    *s = format!("ff{:062x}", index);
                    return;
                }
            }
            *s = s.replace("api03-123-1", "api03-999-1");
        }
        Value::Array(a) => {
            for x in a {
                rewrite(x, "");
            }
        }
        Value::Object(m) => {
            let old = std::mem::take(m);
            for (k, mut x) in old {
                rewrite(&mut x, &k);
                m.insert(k.replace("api03-123-1", "api03-999-1"), x);
            }
        }
        _ => {}
    }
}
#[test]
fn internally_coherent_evidence_from_two_valid_campaigns_cannot_be_mixed() {
    let a = Bundle::new();
    let b = Bundle::new();
    b.edit("context.json", |v| rewrite(v, ""));
    for entry in std::fs::read_dir(&b.root).unwrap() {
        let p = entry.unwrap();
        let name = p.file_name().into_string().unwrap();
        if name == "inventory.json" {
            continue;
        }
        let bytes = std::fs::read(p.path()).unwrap();
        if name.ends_with(".json") || name == "policy-logs.stdout" {
            let mut v: Value = serde_json::from_slice(&bytes).unwrap();
            rewrite(&mut v, "");
            put(&b.root, &name, &v);
        } else if name.ends_with("-create.stdout") {
            let id = String::from_utf8(bytes).unwrap();
            let index = u64::from_str_radix(id.trim(), 16).unwrap();
            std::fs::write(p.path(), format!("ff{index:062x}\n")).unwrap();
        }
    }
    b.rehash();
    api03_host_qualification::verification::verify(&a.root, &a.sha, "123", "1", &a.source).unwrap();
    api03_host_qualification::verification::verify(&b.root, &b.sha, "999", "1", &b.source).unwrap();
    assert_ne!(
        get(&a.root, "descendant-stream-created.json")[0]["Id"],
        get(&b.root, "descendant-stream-created.json")[0]["Id"]
    );
    for entry in std::fs::read_dir(&b.root).unwrap() {
        let p = entry.unwrap();
        let name = p.file_name().into_string().unwrap();
        if name.starts_with("descendant-stream-") {
            std::fs::copy(p.path(), a.root.join(name)).unwrap();
        }
    }
    a.rehash();
    let error =
        api03_host_qualification::verification::verify(&a.root, &a.sha, "123", "1", &a.source)
            .unwrap_err()
            .to_string();
    assert!(
        error.contains("H06/descendant-stream: launch policy: foreign container allocation"),
        "{error}"
    );
}
