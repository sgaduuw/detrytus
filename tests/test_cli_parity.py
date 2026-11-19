"""The Rust `detrytus` command against the Python one: `asm` and `run`
on the same input give the same stdout, stderr, exit status and image
bytes (docs/rust-plan.md, phase 5).

Inline conformance programs go in on stdin, from the case's directory
so their includes resolve; file sources go in by path. The Rust binary
is found through DETRYTUS_RUST_CLI, or at target/debug/detrytus once
`cargo build` has run; without it the tests skip, unless
DETRYTUS_REQUIRE_RUST is set, as in CI.
"""

import os
import subprocess
import sys
from pathlib import Path

import pytest

from detrytus import conformance

ROOT = Path(__file__).parent.parent
CONFORMANCE = ROOT / "conformance"
BINARY = Path(
    os.environ.get("DETRYTUS_RUST_CLI", ROOT / "target" / "debug" / "detrytus")
)

pytestmark = pytest.mark.skipif(
    not BINARY.exists() and not os.environ.get("DETRYTUS_REQUIRE_RUST"),
    reason="the Rust command is not built (cargo build)",
)

CASES = conformance.load_all(CONFORMANCE)
FILES = sorted(
    [*(ROOT / "examples").glob("*.s"), *(CONFORMANCE / "paging").glob("*.s")]
)

Result = tuple[int, str, str]


def _run(
    command: list[str], stdin: str | None = None, cwd: Path | None = None
) -> Result:
    done = subprocess.run(
        command, input=stdin, capture_output=True, text=True, timeout=120, cwd=cwd
    )
    return done.returncode, done.stdout, done.stderr


def _both(
    *args: str, stdin: str | None = None, cwd: Path | None = None
) -> tuple[Result, Result]:
    python = _run([sys.executable, "-m", "detrytus.cli", *args], stdin, cwd)
    rust = _run([str(BINARY), *args], stdin, cwd)
    return python, rust


def _input(case: conformance.Case) -> tuple[list[str], str | None, Path]:
    """The files argument, stdin and directory that give a case's program."""
    if case.text is not None:
        return ["-"], case.text, case.file.parent
    return [str(p) for p in case.sources], None, case.file.parent


# Console input: neither command feeds the program anything.
RUNNABLE = [c for c in CASES if not c.input and "asm_error" not in c.expect]


@pytest.mark.parametrize("case", RUNNABLE, ids=[c.id for c in RUNNABLE])
def test_run_matches_the_python_command(case):
    files, stdin, cwd = _input(case)
    args = ["--ram-pages", str(case.ram_pages), "--max-steps", str(case.max_steps)]
    python, rust = _both("run", "--stats", *args, *files, stdin=stdin, cwd=cwd)
    assert rust == python


SINGLE = [c for c in CASES if c.text is not None or len(c.sources) == 1]


@pytest.mark.parametrize("case", SINGLE, ids=[c.id for c in SINGLE])
def test_asm_matches_the_python_command(case, tmp_path):
    # The same image bytes, listing and errors; for an expected assembly
    # error, the same message.
    files, stdin, cwd = _input(case)
    results = []
    for name, command in (
        ("py", [sys.executable, "-m", "detrytus.cli"]),
        ("rs", [str(BINARY)]),
    ):
        out = tmp_path / f"{name}.dti"
        status, stdout, stderr = _run(
            [*command, "asm", *files, "-o", str(out), "--listing"], stdin, cwd
        )
        results.append(
            (status, stdout, stderr, out.read_bytes() if out.exists() else None)
        )
    assert results[1] == results[0]


@pytest.mark.parametrize("path", FILES, ids=[p.name for p in FILES])
def test_asm_of_a_file_matches(path, tmp_path):
    results = []
    for name, command in (
        ("py", [sys.executable, "-m", "detrytus.cli"]),
        ("rs", [str(BINARY)]),
    ):
        out = tmp_path / f"{name}.dti"
        status, stdout, stderr = _run(
            [*command, "asm", str(path), "-o", str(out), "--listing"]
        )
        results.append(
            (status, stdout, stderr, out.read_bytes() if out.exists() else None)
        )
    assert results[1] == results[0]


# Broken source, at least one snippet for every error the assembler
# raises, and some near misses that are fine.
BROKEN = [
    "bogus a0",
    "add a0, a1",
    "add a0, a1, q9",
    "li a0",
    "li a0, 3812798742494",
    "lw a0, 3(r14)",
    "lw a0, 3",
    "amoadd a0, a1, 3(a2)",
    "j 1",
    "beq a0, a1, 4",
    ".org 1",
    ".equ X, 1\n.equ X, 2",
    "x: nop\nx: nop",
    ".section foo",
    ".section text, 3\n.section text, 6",
    ".section a, 1",
    ".section a, 3, 4",
    ".section",
    ".word 1\n.org 0\n.word 2",
    ".tryte 9842",
    ".string abc",
    ".string",
    ".words -1",
    ".zero",
    "li a0, (1",
    "li a0, 1 +",
    "li a0, 1 2",
    "li a0, 1 / 0",
    "li a0, @",
    "li a0, undefined_sym + 1",
    "tex a0, a1, 26, 2",
    "tex a0, a1, -1, 2",
    "fence rx, w",
    "fence r",
    "add.aq a0, a1, a2",
    "beq.p a0, a1, 0",
    "br3.likely a0, 0, 0",
    "lr.aq a0, (a1)",
    "sc.foo a0, a1, (a2)",
    "j 1f",
    "1: j 2b",
    ".include",
    '.include "nope.s"',
    ".include nope.s",
    "addi a0, a1, 99999999",
    "lui a0, 9999999999",
    "csrw nosuchcsr, a0",
    "tlut a0, a1, a2, tnope",
    "mv a0",
    "nop a0",
    "ret 1",
    "tmap a0, a1",
    "beqz a0",
    ".align -1",
    ".align 28",
    ".align 2\nnop",
    ".equ",
    ".org",
    "li a0, '",
    "li a0, 'ab'",
    "li a0, 0t",
    "li a0, 0x",
    "li a0, -",
    "li a0, ()",
    "  label  :  bogus",
    "ADD A0, A1, A2\nbogus",
    "li a0, .",
    ".equ Y, .",
    "la a0",
    "li a0, 3**2",
    "sw a0, 0(sp",
    "sw a0, sp)",
    "jalr a0, (zero) extra",
    ".word x\nx:",
    "li a0, 'é'\nbogus",
    "cmp a0, a1, a2, a3",
    "sel a0, a1, a2",
    "fence.vm a0",
    "csrrw a0, mtvec",
    "add a0, a1, a2 # fine\n\tbogus 'x;'",
    '.ascii "a\\"b',
    '\t.string "a" "b"',
    "1: li a0, '1b'",
]


@pytest.mark.parametrize("source", BROKEN)
def test_errors_match_the_python_command(source):
    python, rust = _both("asm", "-", stdin=source)
    assert rust == python


def test_overlapping_images_are_refused_alike():
    hello = str(ROOT / "examples" / "hello.s")
    python, rust = _both("run", hello, hello)
    assert python[0] == 65
    assert rust == python


def test_a_missing_file_is_reported_alike(tmp_path):
    # The OS error's wording differs between the languages; the status
    # and the file named do not.
    missing = str(tmp_path / "missing.s")
    for command in ("asm", "run"):
        (py_status, _, py_err), (rs_status, _, rs_err) = _both(command, missing)
        assert py_status == rs_status == 66
        assert missing in py_err
        assert rs_err.startswith(f"detrytus: {missing}: ")
