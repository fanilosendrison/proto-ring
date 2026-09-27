"""Consumer-independent structural and heuristic terminology governance.
Lexical discovery supports review; it never proves semantic equivalence or completeness.
"""

from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass
import hashlib
import re

from .normative_terminology_matching import definition_concepts as _definition_concepts


@dataclass(frozen=True)
class RegistryFormat:
    start_marker: str
    end_marker: str
    headers: tuple[str, ...]
    key_pattern: str
    anchor_link_pattern: str
    anchor_pattern: str

    def __post_init__(self) -> None:
        if not self.start_marker or not self.end_marker:
            raise ValueError("registry boundary markers must be non-empty")
        if self.start_marker == self.end_marker:
            raise ValueError("registry boundary markers must be distinct")
        if len(self.headers) != 6 or any(not value for value in self.headers):
            raise ValueError("registry format must define six non-empty headers")
        try:
            re.compile(self.key_pattern)
            anchor_link = re.compile(self.anchor_link_pattern)
            anchor = re.compile(self.anchor_pattern)
        except re.error as error:
            raise ValueError(f"registry format contains an invalid regex: {error}") from error
        if anchor_link.groups < 1 or anchor.groups < 1:
            raise ValueError("anchor patterns must capture the canonical destination")


@dataclass(frozen=True)
class RegistryEntry:
    key: str
    canonical_term: str
    anchor: str
    aliases: tuple[str, ...]
    deprecated: tuple[str, ...]
    term_structure: str

    @property
    def expressions(self) -> tuple[str, ...]:
        return (self.canonical_term, *self.aliases, *self.deprecated)


@dataclass(frozen=True)
class MarkdownBlock:
    line: int
    section: str
    heading: str
    text: str


@dataclass(frozen=True)
class Occurrence:
    concepts: tuple[str, ...]
    section: str
    heading: str
    fingerprint: str
    line: int
    canonical: bool

    def __post_init__(self) -> None:
        object.__setattr__(self, "concepts", tuple(sorted(self.concepts)))

    @property
    def signature(self) -> tuple[tuple[str, ...], str, str, str]:
        return (self.concepts, self.section, self.heading, self.fingerprint)


@dataclass(frozen=True)
class TerminologyPolicy:
    registry: RegistryFormat
    section_from_heading: Callable[[str], str]
    canonical_location: Callable[[RegistryEntry, MarkdownBlock], bool]
    role_for: Callable[[Occurrence], str]
    allowed_roles: frozenset[str]

    def __post_init__(self) -> None:
        callbacks = (self.section_from_heading, self.canonical_location, self.role_for)
        if not all(callable(callback) for callback in callbacks):
            raise ValueError("terminology policy callbacks must be callable")
        if not isinstance(self.allowed_roles, frozenset) or not self.allowed_roles:
            raise ValueError("terminology policy must define a non-empty frozen role set")
        if any(not isinstance(role, str) or not role for role in self.allowed_roles):
            raise ValueError("terminology policy roles must be non-empty strings")


def normalize_text(text: str) -> str:
    return " ".join(text.split())


def fingerprint_text(text: str) -> str:
    return hashlib.sha256(normalize_text(text).encode("utf-8")).hexdigest()


def _table_cells(line: str) -> list[str]:
    return [cell.strip() for cell in line.strip().strip("|").split("|")]

def _valid_backtick_delimiters(value: str) -> bool:
    value = value.strip()
    count = value.count("`")
    return count == 0 or (count == 2 and value.startswith("`") and value.endswith("`"))

def _plain_cell(value: str) -> str:
    value = value.strip()
    if value.startswith("`") and value.endswith("`"):
        value = value[1:-1].strip()
    return value

def _is_em_dash_form(value: str) -> bool:
    decoded = _plain_cell(value)
    return decoded.strip("*_~\"'“”‘’").strip() == "—"

def _term_list(value: str) -> tuple[str, ...]:
    if value.strip() in {"", "—"}:
        return ()
    decoded = (
        _plain_cell(item) for item in value.split(";")
        if item.strip() and not _is_em_dash_form(item)
    )
    return tuple(item for item in decoded if item)

def _has_invalid_em_dash(value: str) -> bool:
    if value.strip() in {"", "—"}:
        return False
    return any(_is_em_dash_form(item) for item in value.split(";") if item.strip())

def _has_empty_term(value: str) -> bool:
    if value.strip() in {"", "—"}:
        return False
    return any(
        not item.strip() or not _plain_cell(item) for item in value.split(";")
    )

def _registry_bounds(
    document: str, registry_format: RegistryFormat
) -> tuple[list[str], tuple[int, int] | None, str | None]:
    lines = document.splitlines()
    starts = [i for i, line in enumerate(lines) if line.strip() == registry_format.start_marker]
    ends = [i for i, line in enumerate(lines) if line.strip() == registry_format.end_marker]
    if len(starts) != 1 or len(ends) != 1:
        return lines, None, "document must contain exactly one terminology registry"
    if ends[0] <= starts[0]:
        return lines, None, "terminology registry end marker must follow its start marker"
    return lines, (starts[0], ends[0]), None


def parse_registry(
    document: str, registry_format: RegistryFormat
) -> tuple[list[RegistryEntry], list[str]]:
    errors: list[str] = []
    lines, bounds, boundary_error = _registry_bounds(document, registry_format)
    if boundary_error:
        return [], [boundary_error]
    assert bounds is not None
    start, end = bounds
    registry_lines = [line.strip() for line in lines[start + 1 : end] if line.strip()]
    valid_outer_pipes = all(
        line.startswith("|") and line.endswith("|")
        and not line.startswith("||") and not line.endswith("||")
        for line in registry_lines
    )
    if not valid_outer_pipes:
        return [], ["terminology registry rows require exactly one outer pipe at each edge"]
    rows = [_table_cells(line) for line in registry_lines]
    if len(rows) < 3 or tuple(rows[0]) != registry_format.headers:
        return [], ["terminology registry has missing or unexpected table headers"]
    separator_valid = len(rows[1]) == len(registry_format.headers) and all(
        re.fullmatch(r":?-{3,}:?", cell) for cell in rows[1]
    )
    if not separator_valid:
        return [], ["terminology registry has a missing or malformed GFM separator row"]
    entries: list[RegistryEntry] = []
    for row_number, row in enumerate(rows[2:], start=1):
        if len(row) != len(registry_format.headers):
            errors.append(f"terminology registry row {row_number} has {len(row)} cells; expected 6")
            continue
        plain_cells = (row[0], row[1], row[5])
        term_cells = tuple(
            item for cell in (row[3], row[4]) for item in cell.split(";")
            if item.strip() and item.strip() != "—"
        )
        if any(not _valid_backtick_delimiters(value) for value in (*plain_cells, *term_cells)):
            errors.append(f"terminology registry row {row_number} has malformed backtick delimiters")
            continue
        key, canonical_term = _plain_cell(row[0]), _plain_cell(row[1])
        anchor_match = re.fullmatch(registry_format.anchor_link_pattern, row[2])
        if not re.fullmatch(registry_format.key_pattern, key):
            errors.append(f"invalid terminology concept key: {key!r}")
        if not canonical_term:
            errors.append(f"{key or f'row {row_number}'} has no canonical term")
        if not anchor_match:
            errors.append(f"{key or f'row {row_number}'} has an invalid canonical anchor link")
            continue
        anchor = anchor_match.group(1)
        if not isinstance(anchor, str) or not anchor:
            errors.append(f"{key or f'row {row_number}'} has no non-empty canonical destination")
            continue
        aliases, deprecated = _term_list(row[3]), _term_list(row[4])
        if _has_invalid_em_dash(row[3]) or _has_invalid_em_dash(row[4]):
            errors.append(f"{key} uses the em dash outside its sole empty-list form")
        if _has_empty_term(row[3]):
            errors.append(f"{key} has an empty alias")
        if _has_empty_term(row[4]):
            errors.append(f"{key} has empty deprecated wording")
        normalized = [value.casefold() for value in (canonical_term, *aliases, *deprecated)]
        if len(normalized) != len(set(normalized)):
            errors.append(f"{key} repeats a canonical, alias, or deprecated term")
        term_structure = _plain_cell(row[5])
        if not term_structure:
            errors.append(f"{key} has no term-structure classification")
        if canonical_term and term_structure:
            entries.append(
                RegistryEntry(
                    key, canonical_term, anchor, aliases, deprecated, term_structure,
                )
            )
    keys, anchors = [entry.key for entry in entries], [entry.anchor for entry in entries]
    if len(keys) != len(set(keys)):
        errors.append("terminology registry contains duplicate concept keys")
    if len(anchors) != len(set(anchors)):
        errors.append("terminology registry contains duplicate canonical destinations")
    return entries, errors


def parse_blocks(
    document: str, policy: TerminologyPolicy
) -> tuple[list[MarkdownBlock], dict[str, list[tuple[int, str, str]]]]:
    blocks: list[MarkdownBlock] = []
    anchors: dict[str, list[tuple[int, str, str]]] = {}
    paragraph: list[str] = []
    paragraph_line = 0
    section = heading = ""
    lines, registry_bounds, _boundary_error = _registry_bounds(document, policy.registry)
    fence: list[str] | None = None
    fence_line = 0
    fence_token = ""

    def flush_paragraph() -> None:
        nonlocal paragraph, paragraph_line
        if paragraph:
            blocks.append(MarkdownBlock(paragraph_line, section, heading, "\n".join(paragraph)))
            paragraph, paragraph_line = [], 0
    for line_number, line in enumerate(lines, start=1):
        line_index = line_number - 1
        if registry_bounds and registry_bounds[0] <= line_index <= registry_bounds[1]:
            if line_index == registry_bounds[0]:
                flush_paragraph()
            continue
        if fence is not None:
            fence.append(line)
            if line.strip().startswith(fence_token):
                blocks.append(MarkdownBlock(fence_line, section, heading, "\n".join(fence)))
                fence = None
            continue
        fence_match = re.match(r"^\s*(```|~~~)", line)
        if fence_match:
            flush_paragraph()
            fence, fence_line, fence_token = [line], line_number, fence_match.group(1)
            continue
        heading_match = re.match(r"^ {0,3}#{1,6}(?:[ \t]+(.*?)|[ \t]*)$", line)
        if heading_match:
            flush_paragraph()
            heading = heading_match.group(1) or ""
            heading = re.sub(r"(?:[ \t]+|^)#+[ \t]*$", "", heading).strip()
            section = policy.section_from_heading(heading)
            if not isinstance(section, str):
                raise ValueError("section policy must return a string")
            continue
        anchor_match = re.fullmatch(policy.registry.anchor_pattern, line.strip())
        if anchor_match:
            flush_paragraph()
            anchor = anchor_match.group(1)
            if not isinstance(anchor, str) or not anchor:
                raise ValueError("anchor policy captured an empty canonical destination")
            anchors.setdefault(anchor, []).append((line_number, section, heading))
            continue
        if not line.strip() or line.lstrip().startswith("<!--"):
            flush_paragraph()
            continue
        if not paragraph:
            paragraph_line = line_number
        paragraph.append(line)
    flush_paragraph()
    return blocks, anchors


def discover_occurrences(
    document: str, entries: list[RegistryEntry], policy: TerminologyPolicy
) -> tuple[list[Occurrence], list[str]]:
    parsed_entries, registry_errors = parse_registry(document, policy.registry)
    if registry_errors:
        return [], registry_errors
    if entries != parsed_entries:
        return [], ["provided registry entries do not match the document registry"]
    try:
        blocks, anchors = parse_blocks(document, policy)
    except ValueError as error:
        return [], [str(error)]
    errors: list[str] = []
    registered = {entry.anchor for entry in entries}
    for anchor in sorted(set(anchors) - registered):
        errors.append(f"canonical terminology anchor is not registered: {anchor}")
    canonical_by_block: dict[int, set[str]] = {}
    occurrences: list[Occurrence] = []
    for entry in entries:
        anchor_locations = anchors.get(entry.anchor)
        if not anchor_locations:
            errors.append(f"canonical anchor is missing: {entry.anchor}")
            continue
        if len(anchor_locations) != 1:
            errors.append(f"canonical anchor occurs more than once: {entry.anchor}")
            continue
        anchor_line, section, _heading = anchor_locations[0]
        block_index = next(
            (index for index, block in enumerate(blocks) if block.line > anchor_line), None
        )
        if block_index is None or blocks[block_index].section != section:
            errors.append(f"canonical anchor {entry.anchor} is not followed by a definition block")
            continue
        block = blocks[block_index]
        canonical_location = policy.canonical_location(entry, block)
        if not isinstance(canonical_location, bool):
            raise ValueError("canonical-location policy must return a boolean")
        if not canonical_location:
            errors.append(
                f"canonical anchor {entry.anchor} is outside the consumer-defined canonical location"
            )
            continue
        intervening = any(
            anchor_line < other_line < block.line
            for locations in anchors.values()
            for other_line, _, _ in locations
        )
        if intervening:
            errors.append(
                f"canonical anchor {entry.anchor} is not immediately followed by its definition block"
            )
            continue
        if entry.key not in _definition_concepts(block, [entry]):
            errors.append(
                f"canonical anchor {entry.anchor} does not lead to a definition of {entry.canonical_term}"
            )
            continue
        canonical_by_block.setdefault(block_index, set()).add(entry.key)
        occurrences.append(
            Occurrence(
                (entry.key,), section, block.heading, fingerprint_text(block.text),
                block.line, True,
            )
        )
    for index, block in enumerate(blocks):
        concepts = _definition_concepts(block, entries) - canonical_by_block.get(index, set())
        if concepts:
            occurrences.append(
                Occurrence(
                    tuple(sorted(concepts)), block.section, block.heading,
                    fingerprint_text(block.text), block.line, False,
                )
            )
    return occurrences, errors


def occurrence_record(
    occurrence: Occurrence, policy: TerminologyPolicy
) -> dict[str, object]:
    from .normative_terminology_inventory import occurrence_record as render

    return render(occurrence, policy)


def reconcile_inventory(
    entries: list[RegistryEntry],
    occurrences: list[Occurrence],
    inventory_occurrences: object,
    policy: TerminologyPolicy,
) -> list[str]:
    from .normative_terminology_inventory import reconcile_inventory as reconcile

    return reconcile(entries, occurrences, inventory_occurrences, policy)
