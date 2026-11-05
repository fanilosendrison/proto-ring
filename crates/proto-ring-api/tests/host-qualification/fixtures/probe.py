"""Finite inert actions only; no consumer commands or credential reads."""
import errno
import hashlib
import json
import os
from pathlib import Path
import socket
import runpy
import subprocess
import sys
import time

STREAM_LIMIT = 65536


def rendezvous(path):
    deadline = time.monotonic() + 20
    while not path.is_file():
        if time.monotonic() >= deadline:
            raise RuntimeError("experiment rendezvous guard; incomplete")
        time.sleep(0.02)


def write(path, data):
    # Existence/read check distinguishes wrong path and absent target from refusal.
    assert path.is_file()
    assert path.read_bytes()
    try:
        with path.open("wb") as stream:
            stream.write(data)
    except OSError as error:
        if error.errno != errno.EROFS:
            raise
        os.write(2, b"started\nEROFS\n")
        raise SystemExit(30)
    os.write(1, b"written\n")


def process_policy():
    status = dict(line.split(":", 1) for line in Path("/proc/self/status").read_text().splitlines() if ":" in line)
    assert os.getuid() != 0 and os.getgid() != 0 and os.getpid() == 1
    for key in ("CapInh", "CapPrm", "CapEff", "CapBnd", "CapAmb"):
        assert int(status[key].strip(), 16) == 0
    assert status["NoNewPrivs"].strip() == "1"
    assert status["Seccomp"].strip() == "2"
    try:
        Path("/var/tmp/api03-owned-rootfs-test").write_bytes(b"inert")
    except OSError as error:
        assert error.errno == errno.EROFS
    else:
        raise AssertionError("writable rootfs")
    sensitive = ("GITHUB_TOKEN", "GH_TOKEN", "OPENAI_API_KEY", "CODEX_API_KEY", "ACTIONS_RUNTIME_TOKEN", "ACTIONS_ID_TOKEN_REQUEST_TOKEN", "SSH_AUTH_SOCK")
    assert not any(key in os.environ for key in sensitive)
    allowed = {"PATH", "LANG", "GPG_KEY", "PYTHON_VERSION", "PYTHON_SHA256", "HOME", "HOSTNAME"}
    assert set(os.environ) <= allowed
    print(json.dumps({
        "uid": os.getuid(), "gid": os.getgid(), "pid": os.getpid(),
        "capabilities_zero": True, "no_new_privileges": True,
        "seccomp": 2, "rootfs_erofs": True, "credential_names_absent": True,
        "environment_names": sorted(os.environ), "cwd": os.getcwd(),
        "interpreter": sys.executable,
        "interpreter_sha256": hashlib.sha256(Path(sys.executable).read_bytes()).hexdigest(),
        "python": sys.version, "namespaces": {name: os.readlink("/proc/self/ns/" + name) for name in ("pid", "mnt", "net", "ipc", "user")},
        "cgroup": Path("/proc/self/cgroup").read_text(),
        "mountinfo": Path("/proc/self/mountinfo").read_text(),
    }, sort_keys=True))


mode = sys.argv[1]
if mode == "policy":
    process_policy()
elif mode == "write":
    write(Path(sys.argv[2]), b"after\n")
elif mode == "governance":
    path = Path(sys.argv[2])
    data = path.read_bytes()
    assert data.endswith(b"# Authority\n")
    path.write_bytes(data + b"\nBounded inert change.\n")
    os.write(1, b"governance-written\n")
elif mode == "untracked":
    path = Path(sys.argv[2])
    with path.open("xb") as stream:
        stream.write(b"inert-untracked\n")
    os.write(1, b"untracked-written\n")
elif mode == "simple":
    os.write(1, b"simple\n")
elif mode == "output":
    os.write(1, b"api03-stdout\x00\xff\n")
    os.write(2, b"api03-stderr\xfe\n")
    raise SystemExit(23)
elif mode in ("parent", "child"):
    root, kind, campaign, instance = Path(sys.argv[2]), sys.argv[3], sys.argv[4], sys.argv[5]
    binding = json.loads((root / "launch-binding.json").read_bytes())
    assert binding["campaign"] == campaign and binding["instance"] == instance
    assert os.environ["HOSTNAME"] == binding["container_id"][:12]
    publish = runpy.run_path("/program/rendezvous.py")["publish"]
    record = dict(binding, hostname=os.environ["HOSTNAME"],
                  pid_namespace=os.readlink("/proc/self/ns/pid"),
                  pid=os.getpid(), ppid=os.getppid(), write_fd=None, kind="parent")
    if mode == "parent":
        subprocess.Popen([sys.executable, "-I", "-B", "/program/probe.py", "child",
                          str(root), kind, campaign, instance])
        publish(root, instance + "-parent-ready.json", record)
        rendezvous(root / (instance + "-child-ready.json"))
        rendezvous(root / "release-parent")
        os.write(1, b"parent-exit\n")
        os._exit(0)
    else:
        descriptor = os.open(root / "writer", os.O_WRONLY | os.O_APPEND)
        os.write(descriptor, b"child-started\n")
        if kind == "stream":
            os.write(1, b"child-stream\n")
        else:
            os.close(1)
            os.close(2)
        record.update(kind=kind, write_fd=descriptor)
        publish(root, instance + "-child-ready.json", record)
        rendezvous(root / "release-child")
        os.write(descriptor, b"child-released\n")
        os.close(descriptor)
elif mode == "listener":
    root = Path(sys.argv[2])
    with socket.socket() as listener:
        listener.bind(("0.0.0.0", 8765))
        listener.listen(4)
        listener.settimeout(20)
        (root / "listener-ready").write_bytes(b"ready\n")
        for _ in range(2):
            connection, _ = listener.accept()
            with connection:
                connection.sendall(b"api03-pong\n")
elif mode == "connect":
    endpoint, negative = sys.argv[2], sys.argv[3] == "negative"
    with socket.socket() as connection:
        connection.settimeout(3)
        try:
            connection.connect((endpoint, 8765))
        except OSError as error:
            if not negative or error.errno != errno.ENETUNREACH:
                raise
            os.write(2, b"started\nENETUNREACH\n")
            raise SystemExit(42)
        if negative:
            os.write(2, b"unexpected-connect\n")
            raise SystemExit(43)
        data = connection.recv(64)
        assert data == b"api03-pong\n"
        os.write(1, data)
else:
    raise RuntimeError("unknown fixed probe mode")
