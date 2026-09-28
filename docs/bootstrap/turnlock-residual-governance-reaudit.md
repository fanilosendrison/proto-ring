---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "extraction-analysis"
domain: "proto-ring-bootstrap"
severity: "strict"
name: "Residual Turnlock governance re-audit against current Ruu"
---

# Residual Turnlock governance re-audit against current Ruu

Status: non-normative extraction analysis

## Exact evidence baselines

This analysis is bounded to these exact repository states:

```text
proto-ring
fanilosendrison/proto-ring
ee14ae8ef67640d0bb53c90a7cd00e08580a68ce

Turnlock
fanilosendrison/turnlock-rust
0c2fd68974466df37105682d9aa8d034ed22dad2

Ruu
fanilosendrison/ruu
e27adc9290b4781396cb5c9745014ca5427e98ac
```

The historical seed inventory is
[`turnlock-extraction-audit.md`](turnlock-extraction-audit.md), which examined
Turnlock revision `7534a4667ec9190aec4021f90bfae2fa8a046dae`.
That document remains non-normative historical bootstrap analysis. Its candidate
inventory is provenance, not a current extraction plan.

## Maturity and authority boundary

Turnlock remains, in its own repository authority:

```text
a normative specification and formal-verification corpus
that precedes implementation
```

Its current formal workspace states:

```text
No executable canonical formal model exists yet.
```

Current repository evidence also shows no declared formal realization, no
populated mechanical result record, and no executed hostile-review campaign.
The current Gate A protocol bundles contain no eligible reviewer profiles, and
the Gate A campaign runner remains under active construction authority.

Therefore:

```text
demonstrated
≠
mature
≠
extractible now
```

Maturity is candidate-specific. A concrete mechanism can be useful and detailed
while remaining provisional machinery for Turnlock's evolving specification,
formal-assurance architecture, implementation boundary, or review method.
File existence, code volume, tests, schema complexity, current use, and the
historical audit classification do not establish maturity.

Proto-ring owns only accepted consumer-independent contracts and mechanisms.
Turnlock and Ruu retain their product semantics, decisions, profiles, evidence,
qualification or assurance truth, local paths, historical migrations, and
repository-specific bindings. External shared protocols remain external.

## Method

Each historical or current-tree candidate was processed in this order:

```text
inspect current Turnlock responsibility and authority
→ assess candidate-specific maturity
→ check existing proto-ring ownership
→ check external shared ownership
→ confront with current Ruu evidence where applicable
→ preserve consumer-specific distinctions
→ assign exactly one terminal disposition
```

The derivation is not `Turnlock ∩ Ruu`. Absence from Ruu is not evidence against
a justified Turnlock candidate, and elaboration in Turnlock is not evidence for
extraction. The only terminal dispositions used are `COVERED`, `EXTRACT`,
`DEFER_UNSTABLE`, `CONSUMER`, `EXTERNAL`, and `OBSOLETE`.

## Evidence summary

Current Turnlock evidence includes its repository-governance profiles, ADR
profile and tooling, accepted ADR-041 through ADR-051, formal-assurance graph and
schemas, review protocol and meta-schema chain, formal traceability checker,
generated mapping, and active Gate A runner construction briefs. Current Ruu
confrontation includes its repository-governance profiles, distinct ADR profile
and tooling, ADR-029 through ADR-031 as currently amended, immutable
qualification lineage and manifests, and historical, post-baseline, and Git
smoke replay mechanisms.

Existing proto-ring ownership was checked against Shared Governance Provider,
Projection Integrity, Repository Integrity, ADR Metadata Primitives, Canonical
ADR Identity, Accepted ADR Body Immutability, Authoritative Ref Monotonicity,
Normative Terminology, Canonical Governance Routing, the Git whitespace
mechanism, and GitHub Authoritative Ref Monotonicity live conformance.

## Candidate matrix

| Candidate | Responsibility | Evidence and provenance | Ruu confrontation | Owner or overlap | Analysis and maturity | Disposition | Trigger or follow-up |
| --------- | -------------- | ----------------------- | ----------------- | ---------------- | --------------------- | ----------- | -------------------- |
| WORK-01 | Shared unready, ready, active, review, and done roles with repository-local Project Status mappings. | Historical work-state candidate; current `docs/repository-governance/turnlock-rust-engineering.md`. | `docs/repository-governance/ruu-engineering.md` consumes the same shared workflow roles through a local profile. | GitHub Engineering Projects operational protocol. | Both repositories explicitly delegate the shared operational roles; their field mappings remain local profiles. Proto-ring must not create a second protocol authority. | EXTERNAL | — |
| WORK-02 | Shared work-item Kind roles: Agent Task, Follow-up, and Finding. | Historical work-state candidate; current Turnlock Engineering profile. | Current Ruu Engineering profile uses the same shared Kind roles. | GitHub Engineering Projects operational protocol. | Kind is already part of the external shared work-management protocol. | EXTERNAL | — |
| WORK-03 | GitHub Issue state, Project fields, native parent/sub-issue and blocking relations, and native pull-request relationships own live work state instead of Markdown mirrors. | Historical work-state candidate; current Turnlock Engineering and Projection Integrity bindings. | Ruu Engineering and `ruu-projection-integrity.md` preserve the same native-owner boundary. | GitHub Engineering Projects operational protocol; proto-ring Projection Integrity overlaps only the generic anti-mirroring/currentness rule. | The external protocol is the operational shared owner. Projection Integrity supports, but does not replace, that protocol. | EXTERNAL | — |
| WORK-04 | Priority semantics, portfolio revalidation, readiness/scheduling distinction, and autonomous pickup. | Historical work-state candidate; current Turnlock Engineering profile has a protection branch, current-phase logic, revalidation triggers, and pickup rules. | Ruu Engineering uses materially narrower scheduling semantics. | Consumer repositories. | One stable consumer-independent scheduling contract is not demonstrated. | CONSUMER | — |
| WORK-05 | Phase and engineering lifecycle taxonomy. | Turnlock Engineering uses Product Semantics, Formal Architecture, Formal Verification, and Implementation. | Ruu Engineering uses Specification, Formal Verification, and Implementation. | Consumer repositories. | The materially different taxonomies must not be generalized. | CONSUMER | — |
| WORK-06 | Durable validated-finding and work routing. | Historical work-routing candidate; current Turnlock Engineering and discovery-classification profiles. | Current Ruu Engineering and discovery-classification profiles bind the same external procedures locally. | GitHub Engineering Projects operational protocol; `engineering-discovery-classification`. | Durable work mechanics and semantic-discovery routing already have external owners. | EXTERNAL | — |
| DISC-01 | Discovery categories `derived-from-existing-authority`, `decision-required`, and `authority-conflict-or-uncertain`. | Historical discovery candidate; current `turnlock-rust-discovery-classification.md`. | Current `ruu-discovery-classification.md` consumes the same categories. | `engineering-discovery-classification`. | The shared classification authority already exists outside proto-ring. | EXTERNAL | — |
| DISC-02 | Earliest-unresolved-authority routing and prohibition on downstream semantic ratification. | Historical discovery candidate; Turnlock AGENTS, discovery profile, and ADR-041. | Ruu AGENTS and discovery profile retain the same external routing discipline across a different authority chain. | `engineering-discovery-classification`. | The procedural authority is external and must not be duplicated. | EXTERNAL | — |
| DISC-03 | Repository-specific mapping from discovery layers to normative, formal, qualification, implementation, integration, and repository artifacts. | Turnlock discovery profile maps shared layers to Turnlock authorities. | Ruu discovery profile maps them to Ruu's different authority and qualification surfaces. | Consumer repositories. | The bindings depend on each consumer's authority graph. | CONSUMER | — |
| ADR-01 | Byte-exact parsing, safe loading, schema validation, hashing, containment, structured configuration, and relation-reference checking. | Historical ADR-tooling candidate; current Turnlock and Ruu ADR tools import `proto_ring.adr_metadata`. | Ruu tooling consumes the same primitives with local policy. | ADR Metadata Primitives. | The accepted proto-ring primitive contract already owns this bounded responsibility. | COVERED | — |
| ADR-02 | Canonical logical ADR identity resolution. | Historical ADR-tooling candidate; current profiles route canonical corpora. | Ruu uses its own profile through the shared resolver. | Canonical ADR Identity. | Single logical identity resolution is already owned without taking corpus policy. | COVERED | — |
| ADR-03 | Accepted ADR decision-body immutability. | Current Turnlock accepted-ADR history and shared checker binding. | Ruu retains its accepted decision history under the same shared boundary. | Accepted ADR Body Immutability. | The existing contract owns exact accepted-body sealing. | COVERED | — |
| ADR-04 | ADR profile vocabulary, profile version, required sections, and commands. | `docs/adr/adr-profile.yaml` defines Turnlock-specific profile fields and commands. | Ruu's profile differs in H1 policy, date policy, legacy fields, history handling, and commands. | Consumer repositories. | Proto-ring explicitly does not own a universal ADR profile. | CONSUMER | — |
| ADR-05 | Canonical schema provenance and pins, vendored coordinates, local-overlay identity, and composition policy. | Turnlock ADR profile and `scripts/adr-metadata.py`. | Ruu uses distinct overlay identity and local composition policy. | Consumer repositories; ADR Metadata Primitives cover only generic loading, hashing, and validation mechanics. | Exact schema authority, source coordinates, digest, overlay, and composition policy remain local. | CONSUMER | — |
| ADR-06 | Corpus-wide identity and representation policy: ID width, contiguity, retained IDs, filename and H1 policy, dates, legacy exceptions, and lifecycle rules. | Turnlock ADR profile and local validator. | Ruu's materially different profile and validator demonstrate consumer policy variance. | Consumer repositories; Canonical ADR Identity covers only one logical resolution. | Corpus policy is not the shared single-record resolver. | CONSUMER | — |
| ADR-07 | ADR metadata migration evidence. | Turnlock profile, migration evidence, and local historical-hash validation. | Ruu has a different baseline, migrated corpus, hashes, and migration scope. | Consumer repositories. | Baselines, ancestry assumptions, preservation claims, and historical scope are consumer evidence. | CONSUMER | — |
| ADR-08 | Maintained annotated ADR history. | Turnlock profile, annotated-history grammar, markers, links, and narration checks. | Ruu does not use the same maintained annotated-history contract. | Consumer repositories; Projection Integrity covers only generic projection obligations. | Turnlock's history grammar, narrative, and policy remain local. | CONSUMER | — |
| ADR-09 | Generated ADR index currentness. | Historical rendering candidate; Turnlock's generated index is derived from canonical frontmatter. | Ruu also validates its generated index against its own corpus. | Projection Integrity. | Required generated-projection currentness is already owned generically. | COVERED | — |
| ADR-10 | Generated ADR index rendering and presentation. | Turnlock renderer owns local display names, Markdown structure, relation presentation, wording, and output path. | Ruu's renderer has distinct presentation and legacy-relation behavior. | Consumer repositories. | Presentation policy is consumer-owned. | CONSUMER | — |
| ADR-11 | ADR CLI, orchestration, diagnostics, and validation membership/order. | Turnlock profile commands, local wrapper, and repository-integrity membership. | Ruu has distinct commands, diagnostics, and validation membership. | Consumer repositories; Repository Integrity evaluates consumer-owned membership but does not own it. | Repository-local command behavior and validation composition remain local bindings. | CONSUMER | — |
| FORMAL-01 | Generic separation among repository semantics, projections, integrity, and evidence. | Turnlock AGENTS, ADR-041, formal documentation, and local provider bindings. | Ruu AGENTS, Projection Integrity binding, and qualification boundary preserve the same generic separation. | Shared Governance Provider; Projection Integrity; Repository Integrity. | Existing contracts already keep semantics local, deny projection authority, separate integrity from assurance/qualification, and deny broader truth from evidence presence. A second formal-specific contract is unnecessary. | COVERED | — |
| FORMAL-02 | Formal-assurance graph/metamodel for claims, normative coverage, domains/modalities, realizations, residual assurance, reviews, and evidence contracts. | ADR-041; `formal/verification.yaml`; `formal/verification.schema.json`; no realizations currently exist. | Ruu uses materially different qualification and state-space structures. | No current shared owner. | Turnlock has no executable canonical formal model, and the graph has not reached steady-state use across canonical semantics and real verification evidence. | DEFER_UNSTABLE | Gate B promotes the first canonical formal semantic representation and the graph is exercised against real formal realizations under it, or explicit later authority replaces the architecture. |
| FORMAL-03 | Gate A, Gate B, and Gate C readiness architecture. | ADR-041; formal documentation; generated mapping reports Gate A blocked and later gates not applicable. | Ruu does not use this staged Gate architecture. | No current shared owner. | The sequence has not been exercised end to end. | DEFER_UNSTABLE | A real campaign completes Gate A, Gate B promotes canonical formal semantics, and Gate C semantics are exercised against concrete verification evidence. |
| FORMAL-04 | Canonical formal semantic representation and backend choice. | ADR-041 selects future integrated Turnlock-specific TLA+ semantics and TLC for bounded checking. | Ruu uses different state-space and qualification machinery. | Consumer repositories. | No universal formal representation or backend architecture is justified. | CONSUMER | — |
| FORMAL-05 | Generic formal realization, coverage, and reverse-traceability model. | `formal/verification.yaml`, schema, checker, and generated mapping; `formal_realizations` is empty. | Ruu's qualification traceability does not instantiate the same model. | No current shared owner. | Steady-state realization, property binding, coverage, and reverse impact have not been exercised against a canonical model. | DEFER_UNSTABLE | The first canonical formal semantic representation is promoted and actual accepted formal realizations are used by the steady-state forward and reverse traceability process. |
| EVID-01 | Exact evidence binding, currentness, and evidence-class non-substitution: evidence must bind the exact subject/state and interpretation-relevant context; stale, mismatched, cross-class, missing, or undetermined binding cannot authorize current success; evidence does not acquire subject authority. | ADR-041 separates review from checker evidence; ADR-043 binds exact reviewed packets and refutation challenges; ADR-044 requires one subject without precedence fallback; ADR-045 binds subject and protocol identity plus sealed evidence; ADR-046 binds challenge execution inputs; ADR-048 and ADR-049 make interpretation/evidence identity explicit and versioned; `formal/tlc-result.schema.json` and formal policy require exact execution identity. | ADR-030 retains broader exact-state/TOCTOU binding; qualification lineage and manifests hash exact artifacts; historical, post-baseline, and Git smoke replay bind admitted executables, outputs, and state; `qualification/README.md` rejects preserved PASS text as current claim proof. | Authorized future proto-ring extraction bounded by Issue #20. | The property does not depend on a finished Turnlock model. Ruu exercises exact evidence/state binding concretely, while accepted Turnlock authority repeats exact subject/context binding across distinct evidence classes. Extraction need not adopt Turnlock Gate policy, Ruu formats, a universal proof format, or a universal backend. | EXTRACT | [Issue #20 — Extract Exact Evidence Binding from Turnlock and Ruu](https://github.com/fanilosendrison/proto-ring/issues/20) |
| EVID-02 | Evidence custody or presence versus historical/qualification claim validity. | Turnlock separates repository integrity, review evidence, and mechanical results. | Ruu qualification explicitly separates artifact preservation from replay and claim validity. | Repository Integrity. | Repository Integrity already distinguishes current custody obligations from historical claim validity and rejects preserved PASS text as proof merely because the artifact exists. | COVERED | — |
| EVID-03 | Hostile-review operational independence and reviewer/model qualification. | ADR-042, ADR-045, review schemas, and current protocols; no real campaign has executed and all current reviewer-profile registries are empty. | Ruu hostile audits do not demonstrate the same reviewer/execution protocol. | No current shared owner. | The protocol is designed but unexercised. | DEFER_UNSTABLE | The first real Gate A hostile-review campaign completes under accepted current authority and exercises reviewer/execution qualification against actual evidence. |
| EVID-04 | Content-addressed review protocol and meta-schema evolution, excluding the exact binding property assigned to Exact Evidence Binding. | ADR-045, ADR-048, immutable meta-schemas, and the protocol v4-to-v1 predecessor chain. | Ruu does not demonstrate the same protocol-bundle architecture. | No current shared owner. | The complete campaign interpretation lifecycle has not been exercised. | DEFER_UNSTABLE | A real campaign is interpreted and retained through the current content-addressed protocol/meta-schema chain, or accepted later authority replaces that chain after execution experience. |
| EVID-05 | Finding normalization, materiality, adjudication, refutation, challenge, and retry policy. | ADR-042 through ADR-049 and current review protocol/schema machinery. | Ruu qualification does not exercise this Gate A finding lifecycle. | No current shared owner. | These may contain generic parts, but they currently belong to an unexecuted Turnlock Gate A protocol. | DEFER_UNSTABLE | The first real Gate A campaign reaches normalization, adjudication, refutation/challenge, and retry handling under accepted current authority. |
| EVID-06 | Ruu retained lineage, exact replay, and recorded-output mechanism. | Turnlock uses a different prospective formal-review and TLC evidence architecture. | Ruu's immutable ADR-080 snapshot, retained lineage, post-baseline registration, version continuity, exact stdout replay, and repository layout are concrete qualification mechanisms. | Ruu. | Generic custody/currentness/claim-validity distinctions are covered elsewhere; the Ruu qualification system remains local. | CONSUMER | — |
| CHECK-01 | Schema loading, exact hashing, and basic path/structured validation primitives inside the formal checker. | Historical formal-checker candidate; current checker uses these bounded helpers. | Ruu tooling independently uses corresponding shared ADR primitives where applicable. | ADR Metadata Primitives, where applicable. | The generic primitive layer is already shared; this does not classify the whole formal checker as covered. | COVERED | — |
| CHECK-02 | Coverage consistency and reverse formal traceability. | `scripts/check-formal-traceability.py`, manifest schema, and generated mapping; no actual formal realization exists. | Ruu's qualification coverage does not instantiate the same traceability model. | No current shared owner. | This responsibility depends on the unexercised realization model. | DEFER_UNSTABLE | The formal-realization trigger in the matrix is satisfied and real canonical formal realizations are present. |
| CHECK-03 | Formal review-evidence validation beyond exact subject/context/currentness binding. | Turnlock traceability checker, meta-schemas, protocol bundles, receipts, raw-output, and challenge schemas. | Ruu does not exercise the same review protocol validator. | No current shared owner; exact binding is isolated in the sole extraction candidate. | Remaining validation depends on the unexecuted hostile-review system. | DEFER_UNSTABLE | The first real Gate A campaign exercises current review evidence validation end to end, including finding lifecycle handling. |
| CHECK-04 | Readiness derivation from formal and review evidence. | Turnlock checker `derive_gate_a`, complete validation, and generated readiness projection. | Ruu has no corresponding Gate readiness derivation. | No current shared owner. | No actual Gate A readiness has been derived from real hostile-review evidence, and later-gate evidence does not exist. | DEFER_UNSTABLE | Actual Gate A readiness is derived from real campaign evidence and later gates have real formal/evidence state where applicable. |
| CHECK-05 | Gate A assurance-decomposition subject construction. | ADR-041 through ADR-045 and `build_gate_a_review_subject` in the Turnlock checker. | Ruu has no Gate A subject. | Turnlock. | The subject dependency set and selector are Turnlock assurance-policy semantics. | CONSUMER | — |
| CHECK-06 | Formal realization integrity. | Turnlock manifest schema and checker reject realizations without the expected model; none currently exist. | Ruu does not use the same realization contract. | No current shared owner. | Integrity semantics cannot be stabilized before a canonical representation and accepted realizations exist. | DEFER_UNSTABLE | A canonical formal representation and actual accepted formal realizations exist. |
| VERIFY-01 | Current TLC evidence contract in `formal/tlc-result.schema.json` and `formal/results/`. | Historical TLC candidate; current schema is present, canonical model is absent, and results contain no concrete run records. | Ruu demonstrates concrete qualification evidence, not the same TLC backend contract. | No current shared owner. | The steady-state TLC evidence contract is not demonstrated. No universal backend abstraction is justified. | DEFER_UNSTABLE | Gate C admits the first concrete bounded mechanical verification evidence under accepted canonical Turnlock formal semantics. |
| MAP-01 | Projection ownership and currentness of the generated invariant mapping. | Turnlock Projection Integrity binding, renderer, and generated `docs/formal/invariant-mapping.md`. | Ruu's projection binding demonstrates the same generic owner/currentness rule for different artifacts. | Projection Integrity. | Generic projection ownership/currentness is already accepted. | COVERED | — |
| MAP-02 | Turnlock formal mapping renderer and presentation, including claim/invariant IDs, paths, vocabulary, layout, and wording. | `scripts/render-formal-mapping.py` and generated Turnlock mapping. | Ruu has different qualification presentations and vocabulary. | Turnlock. | The renderer's presentation and formal vocabulary are consumer-owned. | CONSUMER | — |
| MAP-03 | Generic formal mapping semantics beyond projection currentness. | Current mapping projects an evolving graph with no formal realizations. | Ruu does not project the same formal architecture. | No current shared owner. | A reusable boundary cannot be demonstrated before mappings contain real canonical realizations. | DEFER_UNSTABLE | The formal-realization trigger in the matrix is satisfied and the mapping projects real canonical realizations rather than only evolving architecture. |
| NEW-01 | Shared Governance Provider introduced after the historical audit. | ADR-050, ADR-051, and Turnlock's local provider binding. | Ruu has its own accepted Shared Governance Provider binding. | Shared Governance Provider. | Existing proto-ring authority already owns provider selection and consumer-local authority preservation. | COVERED | — |
| NEW-02 | Authoritative Ref Monotonicity and GitHub effective-rule live conformance. | Turnlock's local authoritative-ref binding and provider coordinates. | The responsibility is provider-neutral and already extracted; consumer coordinates remain local. | Authoritative Ref Monotonicity; GitHub Authoritative Ref Monotonicity live-conformance provider. | Existing contract and provider mechanism own the generic responsibility. | COVERED | — |
| NEW-03 | Canonical Governance Routing. | Turnlock AGENTS frontmatter routes ADR and Shared Governance Provider authority. | Ruu uses corresponding local routing with different targets. | Canonical Governance Routing. | Exact-route traversal and containment are already shared while vocabulary remains local. | COVERED | — |
| NEW-04 | Generic task-worktree isolation, lifecycle, publication safety, reconciliation, and cleanup. | Turnlock worktree policy explicitly specializes enclosing global implementation rules. | The same workspace operational authority applies outside any one consumer. | Global operational implementation and `code-implementation.md` worktree rules. | The responsibility already has external operational authority and must not be duplicated in proto-ring. | EXTERNAL | — |
| NEW-05 | Turnlock worktree paths, base ref, detached mode, publication target, synchronization, and topology specializations. | `turnlock-rust-worktree-management.md` and Turnlock AGENTS. | Ruu has its own repository execution boundaries. | Turnlock. | These are consumer-specific repository coordinates and publication choices. | CONSUMER | — |
| NEW-06 | Gate A campaign runner construction and recovery NIB machinery. | Active construction plan, NIB-S, campaign-state NIB-M briefs, and recovery/operator NIB-M briefs. | Ruu has no corresponding campaign-runner construction system. | No current shared owner. | The artifacts explicitly govern implementation construction for a hostile-review system that has not executed a real campaign; they are not steady-state generic governance. | DEFER_UNSTABLE | The runner exists as an accepted realization and executes a real Gate A campaign through persistence, mutation, recovery, and reconciliation under steady-state authority. |
| NEW-07 | Repository Integrity local wrapper and tri-state fail-closed semantics. | Turnlock's consumer-owned validation profile composes the shared substrate. | Ruu composes a different profile with qualification remaining separate. | Repository Integrity. | The shared contract already owns tri-state, fail-closed, exact-state, and purity semantics; validation membership and diagnostics remain local. | COVERED | — |

## Matrix completeness

The 48 candidates have these terminal counts:

```text
COVERED: 12
EXTERNAL: 7
CONSUMER: 15
DEFER_UNSTABLE: 13
EXTRACT: 1
OBSOLETE: 0
```

Exactly one residual responsibility is authorized for follow-up extraction:
Exact Evidence Binding, tracked by
[fanilosendrison/proto-ring#20](https://github.com/fanilosendrison/proto-ring/issues/20).

## Bounded conclusion

For the exact audited baselines, every currently identified residual Turnlock
repository-governance candidate has been adjudicated.

Exactly one residual responsibility is currently justified for new proto-ring
extraction:

```text
Exact Evidence Binding
```

No other additional extraction is justified at the current Turnlock maturity
state.

Candidates whose possible reusable boundary depends on Turnlock's unfinished
formal-assurance or hostile-review method are retained as `DEFER_UNSTABLE` with
explicit re-audit triggers.

This is not a claim that Turnlock's methodology is complete and not a permanent
exhaustion claim. Turnlock's method and specification remain evolving. Future
accepted Turnlock evolution may stabilize deferred candidates, invalidate a
previous candidate boundary, or create new shared governance.
