//! Independent synthetic H01/H05 inventories; no real kernel/image observations.
use super::{Bundle, digest, get, put};
use serde_json::{Value, json};
use std::fs;
pub(super) fn image_env() -> Value {
    json!([
        "PATH=/usr/local/bin:/usr/bin:/bin",
        "LANG=C.UTF-8",
        "PYTHON_VERSION=3.11.17"
    ])
}
pub(super) fn observations(b: &Bundle) {
    let mut image = get(&b.root, "image.json");
    image[0]["Config"]["Env"] = image_env();
    put(&b.root, "image.json", &image);
    put(
        &b.root,
        "kernel.json",
        &json!({"system":"Linux","release":"6.8.0-synthetic","architecture":"x86_64"}),
    );
    put(
        &b.root,
        "runtime-version.json",
        &json!({"client":"28.0.4","server":"28.0.4","api":"1.49","os":"linux","architecture":"amd64"}),
    );
    put(
        &b.root,
        "runtime-info.json",
        &json!({"server":"28.0.4","kernel":"6.8.0-synthetic","os":"Ubuntu24.04 synthetic","architecture":"x86_64","cgroup_driver":"systemd","cgroup_version":"2","security":["name=seccomp,profile=builtin"],"default_runtime":"runc","runtimes":{"runc":{"path":"runc"}}}),
    );
    fs::copy(b.root.join("runtime-info.json"), b.root.join("runtime.txt")).unwrap();
    let binary = b"SYNTHETIC interpreter bytes, never executed\n";
    fs::write(b.root.join("image-interpreter.bin"), binary).unwrap();
    let id = get(&b.root, "policy-created.json")[0]["Id"].clone();
    put(
        &b.root,
        "image-interpreter.json",
        &json!({"container_id":id,"image":"sha256:141fe9b385f3bb6d618c07ace1836c0b3c72bf525566ac3eb13cd1f405069877","path":"/usr/local/bin/python3","sha256":digest(binary),"bytes":binary.len()}),
    );
    put(
        &b.root,
        "policy-logs.stdout",
        &json!({"uid":1001,"gid":1001,"pid":1,"capabilities_zero":true,"no_new_privileges":true,"seccomp":2,"rootfs_erofs":true,"credential_names_absent":true,"environment_names":["HOME","HOSTNAME","LANG","PATH","PYTHON_VERSION"],"cwd":"/owned/run/subject","interpreter":"/usr/local/bin/python3","interpreter_sha256":digest(binary),"python":"3.11.17 (synthetic, not a runtime)","namespaces":{"pid":"pid:[123]","mnt":"mnt:[124]","net":"net:[125]","ipc":"ipc:[126]","user":"user:[127]"},"cgroup":"0::/\n","mountinfo":"10 1 0:1 / / ro,relatime - overlay overlay ro\n11 10 0:2 / /program ro,relatime - ext4 /synthetic/program ro\n12 10 0:3 / /owned/run/subject ro,relatime - ext4 /synthetic/subject ro\n"}),
    );
}
