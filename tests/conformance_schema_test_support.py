from jsonschema import ValidationError


def discriminated_branches():
    return [
        {
            "type": "object",
            "properties": {"kind": {"const": "A"}, "a": {"type": "string"}},
            "required": ["kind", "a"],
        },
        {
            "type": "object",
            "properties": {"kind": {"const": "B"}, "b": {"type": "integer"}},
            "required": ["kind", "b"],
        },
        {
            "type": "object",
            "properties": {"kind": {"const": "C"}, "c": {"type": "boolean"}},
            "required": ["kind", "c"],
        },
    ]


def normalize_validation_error(error):
    return {
        "message": error.message,
        "validator": error.validator,
        "validator_value": error.validator_value,
        "path": list(error.path),
        "schema_path": list(error.schema_path),
        "context": [normalize_validation_error(child) for child in error.context],
    }


def first_validation_error(validator_class, schema, instance):
    try:
        validator_class(schema).validate(instance)
    except ValidationError as error:
        return error
    raise AssertionError("expected validation failure")
