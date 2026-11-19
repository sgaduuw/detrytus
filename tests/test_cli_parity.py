"""The Rust `detrytus run` against the Python one: same images, same
stdout, stderr and exit status (docs/rust-plan.md, phase 5).

Runs every conformance image through both commands. The Rust binary is
found through DETRYTUS_RUST_CLI, or at target/debug/detrytus once
`cargo build` has run; without it the test skips, unless
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

# Console input: neither command feeds the program anything.
CASES = [c for c in conformance.load_all(CONFORMANCE) if not c.input]


def _run(command: list[str]) -> tuple[int, str, str]:
    done = subprocess.run(command, capture_output=True, text=True, timeout=120)
    return done.returncode, done.stdout, done.stderr


def _both(*args: str) -> tuple[tuple[int, str, str], tuple[int, str, str]]:
    python = _run([sys.executable, "-m", "detrytus.cli", *args])
    rust = _run([str(BINARY), *args])
    return python, rust


@pytest.mark.parametrize("case", CASES, ids=[c.id for c in CASES])
def test_run_matches_the_python_command(case):
    image = conformance.image_path(CONFORMANCE, case)
    args = ["--ram-pages", str(case.ram_pages), "--max-steps", str(case.max_steps)]
    python, rust = _both("run", "--stats", *args, str(image))
    assert rust == python


def test_overlapping_images_are_refused_alike():
    hello = str(conformance.image_path(CONFORMANCE, CASES[0]))
    python, rust = _both("run", hello, hello)
    assert python[0] == 65
    assert rust == python


def test_a_missing_file_is_reported_alike(tmp_path):
    # The OS error's wording differs between the languages; the status
    # and the file named do not.
    missing = str(tmp_path / "missing.dti")
    (py_status, _, py_err), (rs_status, _, rs_err) = _both("run", missing)
    assert py_status == rs_status == 66
    assert missing in py_err
    assert rs_err.startswith(f"detrytus: {missing}: ")


def test_the_rust_command_says_it_needs_images(tmp_path):
    source = tmp_path / "prog.s"
    source.write_text("nop\n")
    status, _, err = _run([str(BINARY), "run", str(source)])
    assert status == 65
    assert "detrytus asm -o" in err
