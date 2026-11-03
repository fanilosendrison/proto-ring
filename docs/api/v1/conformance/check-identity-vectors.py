#!/usr/bin/env python3
"""Known-answer checks of the candidate PUBLIC hash framing, not the engine oracle."""
import base64, hashlib, json
from pathlib import Path

FILE=Path(__file__).with_name('identity-vectors.json')
def be(x):return int(x).to_bytes(8,'big',signed=False)
def frame(x):return be(len(x))+x
def public_hash(domain,parts):
    payload=frame(b'proto-ring-api01-v1')+frame(domain.encode('utf-8'))+be(len(parts))
    payload+=b''.join(frame(x) for x in parts)
    return hashlib.sha256(payload).digest()
def from_b64(s):return base64.b64decode(s,validate=True)
def calculate(vector):
    raw=[from_b64(x) for x in vector['parts_base64']]
    return public_hash(vector['domain'],raw).hex()
def fixtures():
    root_unix=b'/tmp/repo'; scope=[b'AGENTS.md',b'docs/profile.md']; native='unix'
    repo=public_hash('rgs-repository-v1',[native.encode(),root_unix]);sc=public_hash('rgs-scope-v1',[repo,be(len(scope))]+scope)
    state=public_hash('rgs-state-v1',[repo,sc,b'\x00\x01candidate'])
    root_wide='C:\\repo'.encode('utf-16-be');windows=public_hash('rgs-repository-v1',[b'windows',root_wide])
    profile=public_hash('ri-profile-v1',[b'\x01test-profile']);context=public_hash('ri-context-v1',[b'\x02test-context'])
    ri_scope=public_hash('ri-scope-v1',[repo,be(1),b'AGENTS.md']);ri_state=public_hash('ri-state-v1',[repo,ri_scope,b'\x00\xffri'])
    objects=[
      ('unix_repository','rgs-repository-v1',[b'unix',root_unix],repo),
      ('windows_repository','rgs-repository-v1',[b'windows',root_wide],windows),
      ('unix_rgs_scope','rgs-scope-v1',[repo,be(len(scope))]+scope,sc),
      ('unix_rgs_state','rgs-state-v1',[repo,sc,b'\x00\x01candidate'],state),
      ('ri_profile','ri-profile-v1',[b'\x01test-profile'],profile),
      ('ri_context','ri-context-v1',[b'\x02test-context'],context),
      ('ri_scope','ri-scope-v1',[repo,be(1),b'AGENTS.md'],ri_scope),
      ('ri_state','ri-state-v1',[repo,ri_scope,b'\x00\xffri'],ri_state),
    ]
    return [{'id':id,'domain':domain,'parts_base64':[base64.b64encode(x).decode('ascii') for x in parts],'expected_sha256_lowerhex':digest.hex()} for id,domain,parts,digest in objects]

def main():
  if not FILE.exists():
    FILE.write_text(json.dumps({'status':'candidate_known_answer_vectors_not_engine_observation','vectors':fixtures()},indent=2)+'\n')
  rows=json.loads(FILE.read_text())['vectors'];assert len(rows)==8
  for entry in rows:
    found=calculate(entry)
    assert found==entry['expected_sha256_lowerhex'],(entry['id'],found)
  assert fixtures()==rows,'Identity vector semantics diverged from frozen KAT data'
  print('PUBLIC_IDENTITY_KAT_PASS=8; NO_RUST_ENGINE_ATTESTATION')
if __name__=='__main__': main()
