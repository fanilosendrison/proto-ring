"""Derive discovery fixtures from the canonical candidate schema, never a second API."""
import copy
from jsonschema import Draft202012Validator

DRAFT = 'https://json-schema.org/draft/2020-12/schema'


def bundle(api, fragment):
    """Copy the transitive local definition closure into one standalone schema."""
    definitions = api['$defs']
    needed = set()

    def visit(value):
        if isinstance(value, dict):
            if '$ref' in value:
                reference = value['$ref']
                if not reference.startswith('#/$defs/'):
                    raise ValueError('Nonlocal schema reference: ' + reference)
                name = reference.removeprefix('#/$defs/')
                if '/' in name or name not in definitions:
                    raise ValueError('Unknown definition: ' + reference)
                if name not in needed:
                    needed.add(name)
                    visit(definitions[name])
            for child in value.values():
                visit(child)
        elif isinstance(value, list):
            for child in value:
                visit(child)

    visit(fragment)
    return {'$schema': DRAFT, **copy.deepcopy(fragment),
            '$defs': {name: copy.deepcopy(definitions[name]) for name in sorted(needed)}}


def operation_schemas(api, operation):
    definitions = api['$defs']
    request = next(branch for branch in definitions['request']['oneOf']
                   if branch['properties']['operation']['const'] == operation)
    responses = []
    for branch in definitions['response']['oneOf']:
        field = branch['properties']['operation']
        if set(field) != {'const'}:
            raise ValueError('Canonical response branches require exact operation discriminants')
        if field['const'] == operation:
            responses.append(copy.deepcopy(branch))
    return bundle(api, request), bundle(api, {'oneOf': responses})


def discovery_response(api):
    """The closed catalog constants and definition graph are owned by api.schema.json."""
    entries = []
    for branch in api['$defs']['catalog_entry']['oneOf']:
        properties = branch['properties']
        entry = {key: copy.deepcopy(value['const']) for key, value in properties.items()
                 if 'const' in value}
        entry['request_schema'], entry['response_schema'] = operation_schemas(api, entry['operation'])
        entries.append(entry)
    return {'api_version': 1, 'operation': 'discover', 'outcome': 'result',
            'value': {'operations': entries, 'schema_draft': DRAFT}}


def verify_discovery(api, document, fixtures):
    # Independent conformance oracle for the owner-fixed inventory; this is not
    # production dispatch or a separately published API definition.
    expected = [
        ('discover', ['pure_projection'], []),
        ('inspect', ['local_observation'], ['repository_read', 'rgs_load']),
        ('explain', ['local_observation', 'pure_projection'],
         ['repository_read', 'rgs_load', 'query_resolution']),
        ('check_repository_integrity', ['local_observation', 'consumer_command_execution'],
         ['repository_read', 'coherent_workspace', 'rgs_load', 'ri_declared',
          'profile_admission', 'realization', 'effect_plan_admission', 'receipt_budget']),
        ('read_execution_output', ['read_host_output'], ['receipt_access', 'retained_output']),
    ]
    actual = [(entry['operation'], entry['effects'], entry['preconditions'])
              for entry in document['value']['operations']]
    if actual != expected:
        raise AssertionError('Operation order, effects or preconditions differ from fixed v1 inventory')
    if document != discovery_response(api):
        raise AssertionError('Published discovery differs from canonical schema derivation')
    for entry in document['value']['operations']:
        operation = entry['operation']
        for target in ('request', 'response'):
            schema = entry[target + '_schema']
            Draft202012Validator.check_schema(schema)
            validator = Draft202012Validator(schema)
            matching = [data for declared_target, data in fixtures
                        if declared_target == target and data.get('operation') == operation]
            if not matching:
                raise AssertionError('No advertised ' + operation + ' ' + target + ' fixture')
            for data in matching:
                validator.validate(data)
        # bundle() traverses every reference, including those not exercised by fixtures.
        # Equality above proves all embedded definitions are the canonical local closure.
