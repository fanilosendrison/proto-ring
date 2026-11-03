#!/usr/bin/env python3
"""Validate API-01 candidate wire examples. Not engine or security qualification."""
import json
from pathlib import Path
from jsonschema import Draft202012Validator
from referencing import Registry, Resource

ROOT=Path(__file__).resolve().parents[1]
SCHEMAS=ROOT/'schemas'
registry=Registry()
for path in SCHEMAS.glob('*.json'):
    document=json.loads(path.read_text())
    Draft202012Validator.check_schema(document)
    registry=registry.with_resource(document['$id'],Resource.from_contents(document))

schema_req=json.loads((SCHEMAS/'request.schema.json').read_text())
schema_rsp=json.loads((SCHEMAS/'response.schema.json').read_text())
schema_local=json.loads((SCHEMAS/'local-context.schema.json').read_text())
validators={
    'request':Draft202012Validator(schema_req,registry=registry),
    'response':Draft202012Validator(schema_rsp,registry=registry),
    'local':Draft202012Validator(schema_local,registry=registry),
}

def strict_object_pairs(items):
    d={}
    for k,v in items:
        if k in d: raise ValueError('duplicate JSON key: '+k)
        d[k]=v
    return d

cases=json.loads((ROOT/'conformance/schema-cases.json').read_text())
fail=[]
for case in cases:
    p=ROOT/case['document']
    try:
        data=json.loads(p.read_bytes(),object_pairs_hook=strict_object_pairs)
        kind='local' if p.name=='local-context.json' else ('request' if p.name.startswith('request-') or p.name.startswith('negative-') else 'response')
        errors=list(validators[kind].iter_errors(data))
        valid=not errors
        why=str(errors[0].message) if errors else 'valid'
    except (ValueError,UnicodeError) as ex:
        valid=False
        why=str(ex)
    if valid!=(case['expect']=='accept'):
        fail.append((case['id'],case['expect'],why))
if fail:
    for x in fail:print('FAIL:',x)
    raise SystemExit(1)
print(f'SCHEMA_VALIDATION_PASS={len(cases)}; SEMANTIC_RUNTIME_CASES_NOT_QUALIFIED=31')
