"""Bounded experiment-only atomic no-replace readiness publication."""
import json
import os
from pathlib import Path
import tempfile


def publish(directory, final_name, record):
    directory = Path(directory)
    data = json.dumps(record, sort_keys=True).encode("utf-8")
    if len(data) > 4096 or Path(final_name).name != final_name:
        raise ValueError("invalid experiment rendezvous")
    descriptor, temporary = tempfile.mkstemp(prefix=".ready-", dir=directory)
    try:
        with os.fdopen(descriptor, "wb") as stream:
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
        # Atomic EEXIST rather than absence-check + replace. The closed inode is
        # complete before its final name becomes visible, even before tmp unlink.
        os.link(temporary, directory / final_name)
    finally:
        os.unlink(temporary)
    descriptor = os.open(directory, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)
