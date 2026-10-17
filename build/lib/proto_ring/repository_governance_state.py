"""Compose one exact, read-only repository-governance state."""

from __future__ import annotations

from dataclasses import dataclass
from enum import StrEnum
from pathlib import Path

from proto_ring import (
    evidence_requirements,
    governance_authority,
    governance_bindings,
    governance_bootstrap,
    governed_objects,
    projection_registry,
    repository_governance_model,
    repository_integrity,
    repository_state,
)
from proto_ring.governance_routing import ResolvedGovernanceRoute


__all__ = [
    "RepositoryGovernanceDiagnostic",
    "RepositoryGovernanceStage",
    "RepositoryGovernanceState",
    "RepositoryGovernanceStateError",
    "load",
]


class RepositoryGovernanceStage(StrEnum):
    """Construction stages available to controlled state diagnostics."""

    REPOSITORY_STATE = "repository_state"
    BOOTSTRAP = "bootstrap"
    REPOSITORY_GOVERNANCE_MODEL = "repository_governance_model"
    GOVERNANCE_AUTHORITY = "governance_authority"
    GOVERNED_OBJECTS = "governed_objects"
    GOVERNANCE_BINDINGS = "governance_bindings"
    REPOSITORY_INTEGRITY = "repository_integrity"
    PROJECTION_INTEGRITY = "projection_integrity"
    EVIDENCE_REQUIREMENTS = "evidence_requirements"
    COMPOSITION = "composition"


@dataclass(frozen=True)
class RepositoryGovernanceDiagnostic:
    """Identify one controlled construction stage and its underlying detail."""

    stage: RepositoryGovernanceStage
    detail: str


class RepositoryGovernanceStateError(ValueError):
    """Report controlled failure to construct a complete exact state."""

    def __init__(self, diagnostic: RepositoryGovernanceDiagnostic) -> None:
        self.diagnostic = diagnostic
        super().__init__(f"{diagnostic.stage.value}: {diagnostic.detail}")


@dataclass(frozen=True)
class RepositoryGovernanceState:
    """One coherent governance composition bound to an exact repository state."""

    repository: Path
    observed_state: repository_state.RepositoryState
    bootstrap: governance_bootstrap.GovernanceBootstrap
    repository_governance_model: repository_governance_model.RepositoryGovernanceModel
    governance_authority: governance_authority.GovernanceAuthorityProfile
    governed_objects: governed_objects.GovernedObjectCatalog | None
    governance_bindings: governance_bindings.GovernanceBindingRegistry
    repository_integrity: repository_integrity.ConsumerIntegrityProfile | None
    projection_registry: projection_registry.ProjectionRegistry | None
    evidence_requirements: evidence_requirements.EvidenceRequirementRegistry | None


@dataclass(frozen=True)
class _Composition:
    bootstrap: governance_bootstrap.GovernanceBootstrap
    model: repository_governance_model.RepositoryGovernanceModel
    authority: governance_authority.GovernanceAuthorityProfile
    objects: governed_objects.GovernedObjectCatalog | None
    bindings: governance_bindings.GovernanceBindingRegistry
    integrity: repository_integrity.ConsumerIntegrityProfile | None
    projections: projection_registry.ProjectionRegistry | None
    evidence: evidence_requirements.EvidenceRequirementRegistry | None


def _fail(stage: RepositoryGovernanceStage, detail: str) -> RepositoryGovernanceStateError:
    return RepositoryGovernanceStateError(
        RepositoryGovernanceDiagnostic(stage=stage, detail=detail)
    )


def _route(
    model: repository_governance_model.RepositoryGovernanceModel,
    capability_id: str,
    route_id: str,
) -> ResolvedGovernanceRoute:
    return model.capabilities[capability_id].routes[route_id]


def _compose(repository: Path) -> _Composition:
    try:
        bootstrap = governance_bootstrap.load(repository)
    except governance_bootstrap.GovernanceBootstrapError as error:
        raise _fail(RepositoryGovernanceStage.BOOTSTRAP, str(error)) from error

    try:
        model = repository_governance_model.from_bootstrap(bootstrap)
        if model.model_version != 2:
            raise repository_governance_model.RepositoryGovernanceModelError(
                "RepositoryGovernanceState requires Repository Governance Model "
                "model_version 2"
            )
    except repository_governance_model.RepositoryGovernanceModelError as error:
        raise _fail(
            RepositoryGovernanceStage.REPOSITORY_GOVERNANCE_MODEL, str(error)
        ) from error

    try:
        authority = governance_authority.load(
            repository, _route(model, "governance_authority", "profile")
        )
    except governance_authority.GovernanceAuthorityError as error:
        raise _fail(RepositoryGovernanceStage.GOVERNANCE_AUTHORITY, str(error)) from error

    objects: governed_objects.GovernedObjectCatalog | None = None
    if "governed_objects" in model.capabilities:
        try:
            objects = governed_objects.load(
                repository,
                _route(model, "governed_objects", "profile"),
                authority,
            )
        except governed_objects.GovernedObjectsError as error:
            raise _fail(RepositoryGovernanceStage.GOVERNED_OBJECTS, str(error)) from error

    try:
        bindings = governance_bindings.load(
            repository,
            _route(model, "shared_governance_provider", "registry"),
            authority,
            objects,
        )
    except governance_bindings.GovernanceBindingsError as error:
        raise _fail(RepositoryGovernanceStage.GOVERNANCE_BINDINGS, str(error)) from error

    try:
        repository_governance_model.validate_binding_capabilities(model, bindings)
    except repository_governance_model.RepositoryGovernanceModelError as error:
        raise _fail(RepositoryGovernanceStage.COMPOSITION, str(error)) from error

    integrity: repository_integrity.ConsumerIntegrityProfile | None = None
    if "repository_integrity" in model.capabilities:
        try:
            integrity = repository_integrity.load(
                repository,
                _route(model, "repository_integrity", "profile"),
                authority,
                objects,
            )
        except repository_integrity.RepositoryIntegrityError as error:
            raise _fail(RepositoryGovernanceStage.REPOSITORY_INTEGRITY, str(error)) from error

    projections: projection_registry.ProjectionRegistry | None = None
    if "projection_integrity" in model.capabilities:
        if integrity is None:
            raise _fail(
                RepositoryGovernanceStage.COMPOSITION,
                "projection_integrity requires repository_integrity",
            )
        try:
            projections = projection_registry.load(
                repository,
                _route(model, "projection_integrity", "registry"),
                authority,
                integrity,
                objects,
                bindings,
            )
        except projection_registry.ProjectionRegistryError as error:
            raise _fail(RepositoryGovernanceStage.PROJECTION_INTEGRITY, str(error)) from error

    evidence: evidence_requirements.EvidenceRequirementRegistry | None = None
    if "evidence_requirements" in model.capabilities:
        try:
            evidence = evidence_requirements.load(
                repository,
                _route(model, "evidence_requirements", "registry"),
                authority,
                objects,
            )
        except evidence_requirements.EvidenceRequirementsError as error:
            raise _fail(RepositoryGovernanceStage.EVIDENCE_REQUIREMENTS, str(error)) from error

    return _Composition(
        bootstrap,
        model,
        authority,
        objects,
        bindings,
        integrity,
        projections,
        evidence,
    )


def _relative_path(repository: Path, path: Path) -> str:
    try:
        return path.relative_to(repository).as_posix()
    except ValueError as error:
        raise _fail(
            RepositoryGovernanceStage.COMPOSITION,
            "governance carrier is outside the repository",
        ) from error


def _add_route_scope(
    scope: set[str], repository: Path, route: ResolvedGovernanceRoute
) -> None:
    scope.add(_relative_path(repository, route.target))
    try:
        declared_identity = repository_state._explicit_scope_path_identity(
            repository, route.declared_path
        )
    except repository_state.StateCaptureError as error:
        raise _fail(
            RepositoryGovernanceStage.REPOSITORY_STATE,
            f"route scope admissibility could not be determined: {error}",
        ) from error
    if declared_identity is not None:
        scope.add(declared_identity)


def _add_bootstrap_scope(
    scope: set[str], repository: Path, bootstrap: governance_bootstrap.GovernanceBootstrap
) -> None:
    declared_path = _relative_path(repository, bootstrap.carrier)
    try:
        resolved_target = bootstrap.carrier.resolve(strict=True)
    except (OSError, RuntimeError) as error:
        raise _fail(
            RepositoryGovernanceStage.REPOSITORY_STATE,
            f"cannot resolve bootstrap carrier target: {error}",
        ) from error
    try:
        resolved_target.relative_to(repository)
    except ValueError as error:
        raise _fail(
            RepositoryGovernanceStage.REPOSITORY_STATE,
            "bootstrap-resolved target is outside the repository observation root",
        ) from error
    scope.add(declared_path)
    scope.add(_relative_path(repository, resolved_target))


def _observation_scope(repository: Path, composition: _Composition) -> tuple[str, ...]:
    scope: set[str] = set()
    _add_bootstrap_scope(scope, repository, composition.bootstrap)
    for capability in composition.model.capabilities.values():
        for route in capability.routes.values():
            _add_route_scope(scope, repository, route)

    carriers = (
        composition.authority.carrier,
        composition.objects.carrier if composition.objects is not None else None,
        composition.bindings.carrier,
        composition.integrity.carrier if composition.integrity is not None else None,
        composition.projections.carrier if composition.projections is not None else None,
        composition.evidence.carrier if composition.evidence is not None else None,
    )
    for carrier in carriers:
        if carrier is not None:
            scope.add(_relative_path(repository, carrier))

    for source in composition.authority.sources.values():
        if source.repository_target is not None:
            _add_route_scope(scope, repository, source.repository_target)
    return tuple(sorted(scope))


def _capture(repository: Path, scope: tuple[str, ...]) -> repository_state.RepositoryState:
    try:
        return repository_state.capture(repository, scope)
    except repository_state.StateCaptureError as error:
        raise _fail(RepositoryGovernanceStage.REPOSITORY_STATE, str(error)) from error


def load(repository: Path) -> RepositoryGovernanceState:
    """Construct one stable, exact, non-executing RepositoryGovernanceState."""

    try:
        root = repository_state.resolve_worktree_root(repository)
    except repository_state.StateCaptureError as error:
        raise _fail(RepositoryGovernanceStage.REPOSITORY_STATE, str(error)) from error

    discovery = _compose(root)
    discovery_scope = _observation_scope(root, discovery)
    first_state = _capture(root, discovery_scope)

    authoritative = _compose(root)
    authoritative_scope = _observation_scope(root, authoritative)
    if discovery_scope != authoritative_scope:
        raise _fail(
            RepositoryGovernanceStage.COMPOSITION,
            "observation scope changed during state construction",
        )

    final_state = _capture(root, discovery_scope)
    if first_state.identity != final_state.identity:
        raise _fail(
            RepositoryGovernanceStage.REPOSITORY_STATE,
            "repository state changed during governance composition",
        )

    return RepositoryGovernanceState(
        repository=root,
        observed_state=final_state,
        bootstrap=authoritative.bootstrap,
        repository_governance_model=authoritative.model,
        governance_authority=authoritative.authority,
        governed_objects=authoritative.objects,
        governance_bindings=authoritative.bindings,
        repository_integrity=authoritative.integrity,
        projection_registry=authoritative.projections,
        evidence_requirements=authoritative.evidence,
    )
