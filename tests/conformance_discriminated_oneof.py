from __future__ import annotations

from collections.abc import Mapping

from jsonschema import Draft202012Validator
from jsonschema.validators import extend

_DRAFT_2020_12_SCHEMA_URI = "https://json-schema.org/draft/2020-12/schema"
_ORIGINAL_ONEOF = Draft202012Validator.VALIDATORS["oneOf"]


def _schema_allows_discriminated_oneof_fastpath(schema: object) -> bool:
    if not isinstance(schema, Mapping):
        return False
    if schema.get("$schema") != _DRAFT_2020_12_SCHEMA_URI:
        return False
    stack = [(schema, True)]
    while stack:
        value, is_root = stack.pop()
        if isinstance(value, Mapping):
            if not is_root and "$schema" in value:
                return False
            for keyword in ("$ref", "$dynamicRef"):
                if keyword in value:
                    reference = value[keyword]
                    if not isinstance(reference, str) or not reference.startswith("#"):
                        return False
            stack.extend((nested, False) for nested in value.values())
        elif isinstance(value, list):
            stack.extend((nested, False) for nested in value)
    return True


def _json_equal(left: object, right: object) -> bool:
    if isinstance(left, bool) or isinstance(right, bool):
        return type(left) is type(right) and left == right
    if isinstance(left, (int, float)) and isinstance(right, (int, float)):
        return left == right
    if type(left) is not type(right):
        return False
    if isinstance(left, list):
        return len(left) == len(right) and all(
            _json_equal(a, b) for a, b in zip(left, right)
        )
    if isinstance(left, dict):
        return left.keys() == right.keys() and all(
            _json_equal(left[key], right[key]) for key in left
        )
    return left == right


def _oneof_const_discriminator(
    one_of: object,
) -> tuple[str, tuple[object, ...]] | None:
    if not isinstance(one_of, list) or not one_of:
        return None
    if not all(isinstance(branch, Mapping) for branch in one_of):
        return None
    candidate_names: set[str] | None = None
    for branch in one_of:
        properties = branch.get("properties")
        if not isinstance(properties, Mapping):
            return None
        names = {
            name for name, subschema in properties.items()
            if isinstance(name, str) and isinstance(subschema, Mapping)
            and "const" in subschema
        }
        candidate_names = names if candidate_names is None else candidate_names & names
    for name in sorted(candidate_names or ()):
        values = tuple(branch["properties"][name]["const"] for branch in one_of)
        if all(
            not _json_equal(values[i], values[j])
            for i in range(len(values)) for j in range(i + 1, len(values))
        ):
            return name, values
    return None


def _discriminator_aware_one_of(validator, one_of, instance, schema):
    discriminator = _oneof_const_discriminator(one_of)
    if discriminator is None or not isinstance(instance, dict):
        yield from _ORIGINAL_ONEOF(validator, one_of, instance, schema)
        return
    name, const_values = discriminator
    if name not in instance:
        yield from _ORIGINAL_ONEOF(validator, one_of, instance, schema)
        return
    matches = [
        index for index, value in enumerate(const_values)
        if _json_equal(instance[name], value)
    ]
    if len(matches) != 1:
        yield from _ORIGINAL_ONEOF(validator, one_of, instance, schema)
        return
    index = matches[0]
    provisional_errors = list(
        validator.descend(instance, one_of[index], schema_path=index)
    )
    if provisional_errors:
        yield from _ORIGINAL_ONEOF(validator, one_of, instance, schema)


_DiscriminatedDraft202012Validator = extend(
    Draft202012Validator,
    validators={"oneOf": _discriminator_aware_one_of},
)
