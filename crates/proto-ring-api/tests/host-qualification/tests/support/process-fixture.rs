//! Independent synthetic process policy, bytes and runtime observations.
use super::{Bundle, get, put};
use serde_json::json;
use std::fs;
// Case, owned instance suffix, exit code, exact stdout, exact stderr.
type ExpectedProcess = (
    &'static str,
    &'static str,
    i64,
    &'static [u8],
    &'static [u8],
);
pub const PROCESSES: &[ExpectedProcess] = &[
    (
        "H02/governance-mutation",
        "governance",
        0,
        b"governance-written\n",
        b"",
    ),
    (
        "H02/untracked-mutation",
        "untracked",
        0,
        b"untracked-written\n",
        b"",
    ),
    (
        "H03/positive-write",
        "writer-positive",
        0,
        b"written\n",
        b"",
    ),
    (
        "H03/denied-write",
        "writer-denied",
        30,
        b"",
        b"started\nEROFS\n",
    ),
    (
        "H05/positive-write",
        "program-positive",
        0,
        b"written\n",
        b"",
    ),
    (
        "H05/program-denied",
        "program-denied",
        30,
        b"",
        b"started\nEROFS\n",
    ),
    (
        "H05/control-denied",
        "control-denied",
        30,
        b"",
        b"started\nEROFS\n",
    ),
    ("H06/simple", "simple", 0, b"simple\n", b""),
    (
        "H06/descendant-stream",
        "descendant-stream",
        0,
        b"child-stream\nparent-exit\n",
        b"",
    ),
    (
        "H06/descendant-writer",
        "descendant-writer",
        0,
        b"parent-exit\n",
        b"",
    ),
    (
        "H07/streams",
        "output",
        23,
        b"api03-stdout\0\xff\n",
        b"api03-stderr\xfe\n",
    ),
    (
        "H07/positive-write",
        "evidence-positive",
        0,
        b"written\n",
        b"",
    ),
    (
        "H07/evidence-denied",
        "evidence-denied",
        30,
        b"",
        b"started\nEROFS\n",
    ),
    (
        "H08/network-before",
        "network-before",
        0,
        b"api03-pong\n",
        b"",
    ),
    (
        "H08/network-denied",
        "network-denied",
        42,
        b"",
        b"started\nENETUNREACH\n",
    ),
    (
        "H08/network-after",
        "network-after",
        0,
        b"api03-pong\n",
        b"",
    ),
];
impl Bundle {
    pub(super) fn process(
        &mut self,
        prefix: &str,
        index: usize,
        code: i64,
        out: &[u8],
        err: &[u8],
    ) {
        let id = format!("{index:064x}");
        let (mode, args, rw) = match prefix {
            "governance" => (
                "governance",
                vec!["/owned/run/subject/authority.md".to_string()],
                true,
            ),
            "untracked" => (
                "untracked",
                vec!["/owned/run/subject/untracked-witness".into()],
                true,
            ),
            "policy" => ("policy", vec![], false),
            "simple" => ("simple", vec![], false),
            "output" => ("output", vec![], false),
            "descendant-stream" | "descendant-writer" => (
                "parent",
                vec![
                    format!("/owned/run/subject/{prefix}"),
                    if prefix.ends_with("stream") {
                        "stream".into()
                    } else {
                        "writer".into()
                    },
                    "api03-123-1".into(),
                    format!("api03-123-1-{prefix}"),
                ],
                true,
            ),
            "listener" => (
                "listener",
                vec!["/owned/run/subject/network-control".into()],
                true,
            ),
            "network-before" | "network-denied" | "network-after" => (
                "connect",
                vec![
                    "172.19.0.2".into(),
                    if prefix == "network-denied" {
                        "negative".into()
                    } else {
                        "positive".into()
                    },
                ],
                false,
            ),
            _ => (
                "write",
                vec![match prefix {
                    "writer-denied" => "/owned/run/subject/target".into(),
                    "program-denied" => "/program/probe.py".into(),
                    "control-denied" => "/control/target".into(),
                    "evidence-denied" => "/evidence-target/target".into(),
                    _ => format!("/owned/run/subject/{prefix}-target"),
                }],
                prefix.ends_with("positive"),
            ),
        };
        let network = if matches!(prefix, "listener" | "network-before" | "network-after") {
            "api03-123-1-network"
        } else {
            "none"
        };
        let argv: Vec<_> = ["python3", "-I", "-B", "/program/probe.py", mode]
            .into_iter()
            .map(str::to_owned)
            .chain(args)
            .collect();
        let mut mounts = vec![
            json!({"Type":"bind","Source":get(&self.root,"context.json")["program_root"],"Destination":"/program","RW":false}),
            json!({"Type":"bind","Source":"/owned/run/subject","Destination":"/owned/run/subject","RW":rw}),
        ];
        if prefix == "control-denied" {
            mounts.push(json!({"Type":"bind","Source":"/owned/run/control","Destination":"/control","RW":false}));
        }
        if prefix == "evidence-denied" {
            mounts.push(json!({"Type":"bind","Source":"/owned/run/sacrificial-evidence","Destination":"/evidence-target","RW":false}));
        }
        let inspected = json!([{"Id":id,"Name":format!("/api03-123-1-{prefix}"),"Image":"sha256:141fe9b385f3bb6d618c07ace1836c0b3c72bf525566ac3eb13cd1f405069877","Config":{"Hostname":&id[..12],"Labels":{"org.proto-ring.api03-qualification":"api03-123-1"},"User":"1001:1001","WorkingDir":"/owned/run/subject","Cmd":argv,"Env":super::environment_fixture::image_env()},"HostConfig":{"Privileged":false,"ReadonlyRootfs":true,"LogConfig":{"Type":"local","Config":{"max-size":"1m","max-file":"1"}},"PidsLimit":64,"Memory":536870912,"NanoCpus":1000000000,"PidMode":"","IpcMode":"private","CapDrop":["ALL"],"CapAdd":null,"SecurityOpt":["no-new-privileges"],"Devices":null,"DeviceRequests":null,"PortBindings":{},"NetworkMode":network,"Tmpfs":{"/tmp":"rw,nosuid,nodev,noexec,size=16m,mode=1777"}},"Mounts":mounts,"State":{"Running":false,"Pid":0,"ExitCode":code,"Error":"","StartedAt":"2026-01-01T00:00:00Z"}}]);
        let mut created = inspected.clone();
        created[0]["State"] = json!({"Running":false,"Pid":0,"ExitCode":0,"Error":"","StartedAt":"0001-01-01T00:00:00Z"});
        put(&self.root, &format!("{prefix}-created.json"), &created);
        put(&self.root, &format!("{prefix}-inspect.json"), &inspected);
        put(
            &self.root,
            &format!("{prefix}-launch.json"),
            &json!({"network":network,"user":"1001:1001","cwd":"/owned/run/subject","argv":argv,"env":inspected[0]["Config"]["Env"],"mounts":mounts.iter().map(|m|json!({"source":m["Source"],"destination":m["Destination"],"rw":m["RW"],"type":m["Type"]})).collect::<Vec<_>>()}),
        );
        if prefix == "listener" {
            let mut live = inspected.clone();
            live[0]["State"]["Running"] = json!(true);
            live[0]["State"]["Pid"] = json!(9000);
            live[0]["NetworkSettings"] =
                json!({"Networks":{"api03-123-1-network":{"IPAddress":"172.19.0.2"}}});
            put(&self.root, "listener-ready-inspect.json", &live);
        }
        fs::write(
            self.root.join(format!("{prefix}-create.stdout")),
            format!("{id}\n"),
        )
        .unwrap();
        for transport in ["create", "start", "wait", "logs"] {
            put(
                &self.root,
                &format!("{prefix}-{transport}-transport.json"),
                &json!({"code":0,"guard_triggered":false}),
            );
        }
        fs::write(
            self.root.join(format!("{prefix}-wait.stdout")),
            format!("{code}\n"),
        )
        .unwrap();
        fs::write(self.root.join(format!("{prefix}-logs.stdout")), out).unwrap();
        fs::write(self.root.join(format!("{prefix}-logs.stderr")), err).unwrap();
    }
}
