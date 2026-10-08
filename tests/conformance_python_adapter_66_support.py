from __future__ import annotations

import os
from pathlib import Path
import stat

from proto_ring import governance_bindings, repository_integrity


def integrity_profile(root: Path, data: dict[str, object]):
    definitions = {}
    for item in data["validations"]:
        definitions[item["id"]] = repository_integrity.ValidationDefinition(
            item["id"],
            item["responsibility"],
            tuple(item["prerequisites"]),
            None,
            repository_integrity.ValidationInstances("single"),
            repository_integrity.CommandBinding(
                item["environment"],
                tuple(item["arguments"]),
                frozenset(item["undetermined_exit_codes"]),
            ),
        )
    authority = data["authority"]
    return repository_integrity.ConsumerIntegrityProfile(
        root,
        root / data["carrier"],
        1,
        repository_integrity.ProfileAuthority(
            authority["responsibility"], authority["source"]
        ),
        frozenset(data["environments"]),
        data["continue_after_non_satisfied"],
        definitions,
        tuple(data["order"]),
        data["identity"],
    )


def binding_registry(root: Path, data: dict[str, object]):
    values = {}
    for item in data["bindings"]:
        kind = governance_bindings.BindingKind(item["kind"])
        if kind is governance_bindings.BindingKind.EXECUTABLE_PROVIDER:
            identity = governance_bindings.ExecutableProviderIdentity(
                item["repository"], item["commit"]
            )
        else:
            identity = governance_bindings.GovernanceContractIdentity(
                item["repository"], item["commit"], item["path"]
            )
        values[item["id"]] = governance_bindings.GovernanceBinding(
            item["id"],
            kind,
            governance_bindings.GovernanceBindingScope(
                governance_bindings.ScopeKind(item["scope_kind"]),
                item.get("capability"),
            ),
            identity,
            governance_bindings.BindingAuthority(
                item["responsibility"], item["source"]
            ),
        )
    return governance_bindings.GovernanceBindingRegistry(
        root, root / data["carrier"], 1, data["source"], values
    )


def repository_snapshot(root: Path) -> tuple[tuple[object, ...], ...]:
    entries: list[tuple[object, ...]] = []
    for path in sorted(root.rglob("*"), key=lambda item: item.relative_to(root).as_posix()):
        relative = path.relative_to(root).as_posix()
        metadata = path.lstat()
        mode = stat.S_IMODE(metadata.st_mode)
        if path.is_symlink():
            entries.append((relative, "symlink", mode, os.readlink(path)))
        elif path.is_dir():
            entries.append((relative, "directory", mode, None))
        else:
            entries.append((relative, "file", mode, path.read_bytes()))
    return tuple(entries)


def binding_observation(binding):
    scope = binding.scope
    identity = binding.identity
    governed_object = scope.governed_object
    return {
        "id": binding.id,
        "kind": binding.kind.value,
        "scope": {
            "kind": scope.kind.value,
            "capability": scope.capability_id,
            "interface": None if governed_object is None else governed_object.interface_id,
            "object": None if governed_object is None else governed_object.object_id,
        },
        "identity": {
            "repository": identity.repository,
            "commit": identity.commit,
            "path": getattr(identity, "path", None),
        },
        "authority": {
            "responsibility": binding.authority.responsibility_id,
            "source": binding.authority.source_id,
        },
    }


def projection_observation(projection):
    target = projection.target
    return {
        "id": projection.id,
        "responsibility": projection.responsibility_id,
        "canonical_source": projection.canonical_source_id,
        "secondary_source": projection.secondary_source_id,
        "mode": projection.mode.value,
        "validation": projection.validation_id,
        "generator_source": projection.generator_source_id,
        "boundary_source": projection.boundary_source_id,
        "target": None
        if target is None
        else {"interface": target.interface_id, "object": target.object_id},
        "binding": projection.binding_id,
    }


def validation_observation(validation):
    target = validation.target
    return {
        "id": validation.validation_id,
        "responsibility": validation.responsibility,
        "prerequisites": list(validation.prerequisites),
        "target": None
        if target is None
        else {"interface": target.interface_id, "object": target.object_id},
        "instances": {
            "kind": validation.instances.kind,
            "mode": validation.instances.mode,
            "selectors": [
                {"kind": selector.kind, "value": selector.value}
                for selector in validation.instances.selectors
            ],
        },
        "command": {
            "environment": validation.command.environment,
            "arguments": list(validation.command.arguments),
            "undetermined_exit_codes": sorted(validation.command.undetermined_exit_codes),
        },
    }


def requirement_observation(requirement):
    target = requirement.target
    classes = requirement.evidence_classes
    return {
        "id": requirement.id,
        "responsibility": requirement.responsibility_id,
        "target": None
        if target is None
        else {"interface": target.interface_id, "object": target.object_id},
        "instances": {
            "kind": requirement.instances.kind.value,
            "source": requirement.instances.source_id,
        },
        "evidence_classes": {
            "kind": classes.kind.value,
            "classes": None
            if classes.explicit_classes is None
            else sorted(classes.explicit_classes),
            "source": classes.source_id,
        },
        "subject_source": requirement.subject_source_id,
        "context": {
            "required": requirement.context.required,
            "source": requirement.context.source_id,
        },
        "candidate_source": requirement.candidate_source_id,
    }
