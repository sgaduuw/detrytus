"""The conformance suite: machine behaviour as data, runnable by any core.

Cases live in TOML files under conformance/ (format in
conformance/README.md). Each one is an assembly program and what running
it must produce. The format is plain data so the cases outlive this
Python implementation (docs/rust-plan.md).
"""

import tomllib
from collections.abc import Callable
from dataclasses import dataclass, field
from pathlib import Path
from typing import TYPE_CHECKING, Any

from detrytus import image as images
from detrytus.asm import AsmError, Image, assemble
from detrytus.machine import Machine, StepLimit, create
from detrytus.trap import UnhandledTrap

if TYPE_CHECKING:
    from detrytus.rust import RustMachine

CASE_KEYS = {
    "name",
    "program",
    "prelude",
    "postlude",
    "source",
    "sources",
    "ram_pages",
    "max_steps",
    "input",
}
DEFAULT_KEYS = CASE_KEYS - {"name"}
OUTCOMES = {"exit_code", "step_limit", "unhandled_cause", "asm_error"}
CHECKS = {"console", "registers", "csrs", "ring", "words", "trytes"}
# Only with asm_error: the line the error is reported at.
ASM_ERROR_KEYS = {"asm_error", "asm_error_line"}
DEFAULT_RAM_PAGES = 729
DEFAULT_MAX_STEPS = 100_000


class CaseError(ValueError):
    """A case file that does not follow the format."""


@dataclass
class Case:
    file: Path
    name: str
    text: str | None  # assembly to assemble, or None when sources are given
    sources: list[Path]
    ram_pages: int
    max_steps: int
    input: str
    expect: dict[str, Any]

    @property
    def id(self) -> str:
        return f"{self.file.parent.name}/{self.file.stem}::{self.name}"


@dataclass
class Outcome:
    kind: str  # 'exit', 'step_limit', 'unhandled' or 'asm_error'
    value: int | str | None  # the exit code, cause or error message
    machine: "Machine | RustMachine | None"  # None if assembly failed
    line: int | None = None  # where an assembly error was reported
    failures: list[str] = field(default_factory=list)


def _check_keys(where: str, table: dict[str, Any], allowed: set[str]) -> None:
    unknown = set(table) - allowed
    if unknown:
        raise CaseError(f"{where}: unknown keys {sorted(unknown)}")


def _case(path: Path, raw: dict[str, Any], defaults: dict[str, Any]) -> Case:
    expect = raw.get("expect", {})
    case_raw = {k: v for k, v in raw.items() if k != "expect"}
    where = f"{path}: case {raw.get('name', '?')!r}"
    _check_keys(where, case_raw, CASE_KEYS)
    allowed = ASM_ERROR_KEYS if "asm_error" in expect else OUTCOMES | CHECKS
    _check_keys(f"{where} [expect]", expect, allowed)
    if "name" not in raw or "/" in raw["name"]:
        raise CaseError(f"{path}: every case needs a name, without '/'")
    outcomes = OUTCOMES & set(expect)
    if len(outcomes) != 1:
        raise CaseError(f"{where}: give exactly one of {sorted(OUTCOMES)}")
    merged = {**defaults, **case_raw}
    given = [k for k in ("program", "source", "sources") if k in merged]
    if len(given) != 1:
        raise CaseError(f"{where}: give exactly one of program, source, sources")
    text = None
    sources: list[Path] = []
    if "program" in merged:
        parts = (
            merged.get("prelude", ""),
            merged["program"],
            merged.get("postlude", ""),
        )
        text = "\n".join(p for p in parts if p)
    else:
        names = [merged["source"]] if "source" in merged else merged["sources"]
        sources = [(path.parent / n).resolve() for n in names]
    return Case(
        file=path,
        name=raw["name"],
        text=text,
        sources=sources,
        ram_pages=merged.get("ram_pages", DEFAULT_RAM_PAGES),
        max_steps=merged.get("max_steps", DEFAULT_MAX_STEPS),
        input=merged.get("input", ""),
        expect=expect,
    )


def load_file(path: Path) -> list[Case]:
    with path.open("rb") as f:
        data = tomllib.load(f)
    _check_keys(str(path), data, {"defaults", "case"})
    defaults = data.get("defaults", {})
    _check_keys(f"{path} [defaults]", defaults, DEFAULT_KEYS)
    cases = [_case(path, raw, defaults) for raw in data.get("case", [])]
    if not cases:
        raise CaseError(f"{path}: no cases")
    names = [c.name for c in cases]
    if len(names) != len(set(names)):
        raise CaseError(f"{path}: case names repeat")
    return cases


def load_all(root: Path) -> list[Case]:
    return [case for p in sorted(root.rglob("*.toml")) for case in load_file(p)]


def build(case: Case) -> Image:
    if case.text is not None:
        # Named after the case, in the case file's directory, so errors
        # say which case and .include paths start beside the file.
        return assemble(case.text, f"{case.file} [{case.name}]")
    return images.merge(
        [
            images.read(p)
            if images.is_image(p)
            else assemble(p.read_text(encoding="utf-8"), str(p))
            for p in case.sources
        ]
    )


def run(case: Case, backend: str = "python") -> Outcome:
    """Run a case on the "python" or "rust" core."""
    try:
        image = build(case)
    except AsmError as e:
        outcome = Outcome("asm_error", e.message, None, e.line)
        outcome.failures = _compare(case, outcome)
        return outcome
    m = create(ram_pages=case.ram_pages, seed=1, backend=backend)
    m.console.feed(case.input)
    m.load(image)
    try:
        result = m.run(case.max_steps)
        outcome = Outcome("exit", result.exit_code, m)
    except StepLimit:
        outcome = Outcome("step_limit", None, m)
    except UnhandledTrap as e:
        outcome = Outcome("unhandled", e.cause, m)
    outcome.failures = _compare(case, outcome)
    return outcome


# Checks on named things: expectation key -> (label, read one by name).
NAMED: dict[str, tuple[str, Callable[[Any, str], int]]] = {
    "registers": ("register {}", lambda m, name: m.hart.reg(name)),
    "csrs": ("csr {}", lambda m, name: m.hart.csr(name)),
    "words": ("word at {}", lambda m, addr: m.bus.read(int(addr), 3, "walk")),
    "trytes": ("tryte at {}", lambda m, addr: m.bus.read(int(addr), 1, "walk")),
}


def _outcome(e: dict[str, Any], out: Outcome) -> list[str]:
    if "asm_error" in e:
        wanted: tuple[str, int | str | None] = ("asm_error", e["asm_error"])
    elif "exit_code" in e:
        wanted = ("exit", e["exit_code"])
    elif "step_limit" in e:
        wanted = ("step_limit", None)
    else:
        wanted = ("unhandled", e["unhandled_cause"])
    failures = []
    if (out.kind, out.value) != wanted:
        failures.append(
            f"outcome: wanted {wanted[0]} {wanted[1]!r}, got {out.kind} {out.value!r}"
        )
    if "asm_error_line" in e and out.line != e["asm_error_line"]:
        failures.append(f"asm_error_line: wanted {e['asm_error_line']}, got {out.line}")
    return failures


def _compare(case: Case, out: Outcome) -> list[str]:
    e, m = case.expect, out.machine
    failures = _outcome(e, out)
    if m is None:
        return failures
    if "console" in e and m.console.text != e["console"]:
        failures.append(f"console: wanted {e['console']!r}, got {m.console.text!r}")
    if "ring" in e and m.hart.ring != e["ring"]:
        failures.append(f"ring: wanted {e['ring']}, got {m.hart.ring}")
    for key, (label, read) in NAMED.items():
        for name, value in e.get(key, {}).items():
            got = read(m, name)
            if got != value:
                failures.append(f"{label.format(name)}: wanted {value}, got {got}")
    return failures
