#!/usr/bin/env python3
"""Independent public framing/codec KATs, not engine-derived observations."""
import base64
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def be(value):
    return int(value).to_bytes(8, 'big', signed=False)


def frame(value):
    return be(len(value)) + value


def public_hash(domain, parts):
    payload = frame(b'proto-ring-api01-v1') + frame(domain.encode()) + be(len(parts))
    return hashlib.sha256(payload + b''.join(frame(part) for part in parts)).hexdigest()


def canonical_base64(value):
    raw = base64.b64decode(value, validate=True)
    if base64.b64encode(raw).decode('ascii') != value:
        raise ValueError('noncanonical Base64')
    return raw


def native_units(path, platform):
    kind, value = path['kind'], path['value']
    if platform not in {'unix', 'windows'}:
        raise ValueError('unsupported platform')
    if kind == 'utf8':
        raw = value.encode('utf-8') if platform == 'unix' else value.encode('utf-16-be')
    elif kind == 'unix_bytes_base64' and platform == 'unix':
        raw = canonical_base64(value)
    elif kind == 'windows_wide_be_base64' and platform == 'windows':
        raw = canonical_base64(value)
    else:
        raise ValueError('native platform mismatch')
    if not raw:
        raise ValueError('empty path')
    if platform == 'unix' and b'\0' in raw:
        raise ValueError('NUL byte')
    if platform == 'windows' and (len(raw) % 2 or any(raw[i:i + 2] == b'\0\0' for i in range(0, len(raw), 2))):
        raise ValueError('invalid native wide units')
    return raw


def main():
    document = json.loads((ROOT / 'identity-vectors.json').read_text())
    rows = document['vectors']
    assert len({row['id'] for row in rows}) == len(rows)
    for row in rows:
        parts = [canonical_base64(part) for part in row['parts_base64']]
        assert public_hash(row['domain'], parts) == row['expected_sha256_lowerhex'], row['id']
        if row['domain'] == 'ri-profile-v1':
            assert row['native_word_bits'] in [32, 64]
            assert parts[0] == be(row['native_word_bits'])
        if 'scope_input_units_base64' in row:
            incoming = [canonical_base64(value) for value in row['scope_input_units_base64']]
            ordered = sorted(set(incoming))
            assert parts[1] == be(len(ordered)) and parts[2:] == ordered
    widths = {row['native_word_bits']: row['expected_sha256_lowerhex'] for row in rows
              if row['id'] in {'ri_profile_word32', 'ri_profile_word64'}}
    assert widths[32] != widths[64]
    paths = document['native_path_cases']
    for case in paths:
        try:
            actual = native_units(case['path'], case['platform'])
            valid = True
        except (ValueError, UnicodeError):
            valid = False
        assert valid == case['valid'], case['id']
        if valid:
            assert actual.hex() == case['expected_units_hex'], case['id']
    print(f'PUBLIC_IDENTITY_KAT_PASS={len(rows)}; NO_RUST_ENGINE_ATTESTATION')
    print(f'NATIVE_PATH_CODEC_CASES_PASS={len(paths)}; NO_ENGINE_PATH_ROUNDTRIP_ATTESTATION')


if __name__ == '__main__':
    main()
