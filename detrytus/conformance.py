"""The conformance suite: machine behaviour as data, runnable by any core.

Cases live in TOML files under conformance/ (format in
conformance/README.md). Each one is an assembly program and what running
it must produce. The format is plain data so the cases outlive this
Python implementation (docs/rust-plan.md).

Until there is a Rust assembler, other cores run the cases from images
this module assembles into conformance/build/: run it as
`python -m detrytus.conformance` after changing a case.
"""

import re
import sys
import tomllib
from collections.abc import Callable
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from detrytus import image as images
from detrytus import isa
from detrytus.asm import Image, assemble
from detrytus.machine import Machine, StepLimit
from detrytus.trap import UnhandledTrap

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
OUTCOMES = {"exit_code", "step_limit", "unhandled_cause"}
CHECKS = {"console", "registers", "csrs", "ring", "words", "trytes"}
DEFAULT_RAM_PAGES = 729
DEFAULT_MAX_STEPS = 100_000
BUILD_DIR = "build"


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
    kind: str  # 'exit', 'step_limit' or 'unhandled'
    value: int | None
    machine: Machine
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
    _check_keys(f"{where} [expect]", expect, OUTCOMES | CHECKS)
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
    slugs = [slug(c.name) for c in cases]
    if len(slugs) != len(set(slugs)) or "" in slugs:
        raise CaseError(f"{path}: case names repeat, or are empty, once slugged")
    return cases


def load_all(root: Path) -> list[Case]:
    return [case for p in sorted(root.rglob("*.toml")) for case in load_file(p)]


def slug(name: str) -> str:
    """A case name as a file name: lower case, runs of other characters
    as one '-'. Other runners derive image paths the same way."""
    return re.sub(r"[^a-z0-9]+", "-", name.lower()).strip("-")


def image_path(root: Path, case: Case) -> Path:
    """Where a case's image lives: build/<case file without .toml>/<slug>.dti."""
    relative = case.file.relative_to(root).with_suffix("")
    return root / BUILD_DIR / relative / f"{slug(case.name)}{images.SUFFIX}"


def build_images(root: Path) -> dict[Path, bytes]:
    """Every case's image, keyed by the path it belongs at."""
    return {
        image_path(root, case): images.dumps(build(case)) for case in load_all(root)
    }


def write_images(root: Path) -> list[Path]:
    """Bring conformance/build/ up to date; return the paths changed."""
    wanted = build_images(root)
    changed = []
    for path in sorted((root / BUILD_DIR).rglob("*" + images.SUFFIX)):
        if path not in wanted:
            path.unlink()
            changed.append(path)
    for path, data in wanted.items():
        if not path.exists() or path.read_bytes() != data:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
            changed.append(path)
    return changed


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


def run(case: Case) -> Outcome:
    m = Machine(ram_pages=case.ram_pages, seed=1)
    m.console.feed(case.input)
    m.load(build(case))
    try:
        result = m.run(case.max_steps)
        outcome = Outcome("exit", result.exit_code, m)
    except StepLimit:
        outcome = Outcome("step_limit", None, m)
    except UnhandledTrap as e:
        outcome = Outcome("unhandled", e.cause, m)
    outcome.failures = _compare(case, outcome)
    return outcome


def _read_csr(m: Machine, name: str) -> int:
    hart = m.hart
    ring, hart.ring = hart.ring, 1  # read as ring +1, then restore
    try:
        return hart.read_csr(isa.CSRS[name])
    finally:
        hart.ring = ring


# Checks on named things: expectation key -> (label, read one by name).
NAMED: dict[str, tuple[str, Callable[[Machine, str], int]]] = {
    "registers": ("register {}", lambda m, name: m.hart.reg(name)),
    "csrs": ("csr {}", _read_csr),
    "words": ("word at {}", lambda m, addr: m.bus.read(int(addr), 3, "walk")),
    "trytes": ("tryte at {}", lambda m, addr: m.bus.read(int(addr), 1, "walk")),
}


def _outcome(e: dict[str, Any], out: Outcome) -> list[str]:
    if "exit_code" in e:
        wanted = ("exit", e["exit_code"])
    elif "step_limit" in e:
        wanted = ("step_limit", None)
    else:
        wanted = ("unhandled", e["unhandled_cause"])
    if (out.kind, out.value) == wanted:
        return []
    return [f"outcome: wanted {wanted[0]} {wanted[1]}, got {out.kind} {out.value}"]


def _compare(case: Case, out: Outcome) -> list[str]:
    e, m = case.expect, out.machine
    failures = _outcome(e, out)
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


def main() -> None:
    root = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("conformance")
    for path in write_images(root.resolve()):
        print(path)


if __name__ == "__main__":
    main()
