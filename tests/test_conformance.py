"""Runs the conformance suite (conformance/) on the Python core."""

from pathlib import Path

import pytest

from detrytus import conformance
from detrytus.asm import AsmError

ROOT = Path(__file__).parent.parent / "conformance"
CASES = conformance.load_all(ROOT)


@pytest.mark.parametrize("case", CASES, ids=[c.id for c in CASES])
def test_case(case):
    failures = conformance.run(case).failures
    assert not failures, "\n".join(failures)


# --- the runner itself ---------------------------------------------------------

HALT_WITH_4 = """
[[case]]
name = "four"
program = "li a0, 4\\nli t6, -39366\\nsw a0, 0(t6)"
"""


def _cases(tmp_path, text):
    path = tmp_path / "cases.toml"
    path.write_text(text)
    return conformance.load_file(path)


def _failures(tmp_path, expect):
    (case,) = _cases(tmp_path, HALT_WITH_4 + expect)
    return conformance.run(case).failures


def test_a_matching_case_has_no_failures(tmp_path):
    assert (
        _failures(tmp_path, "expect.exit_code = 4\nexpect.registers = { a0 = 4 }") == []
    )


@pytest.mark.parametrize(
    ("expect", "message"),
    [
        ("expect.exit_code = 5", "outcome: wanted exit 5, got exit 4"),
        ("expect.step_limit = true", "outcome: wanted step_limit"),
        ("expect.unhandled_cause = 5", "outcome: wanted unhandled 5"),
        ('expect.exit_code = 4\nexpect.console = "x"', "console"),
        ("expect.exit_code = 4\nexpect.ring = 0", "ring"),
        ("expect.exit_code = 4\nexpect.registers = { a0 = 3 }", "register a0"),
        ("expect.exit_code = 4\nexpect.csrs = { mtcause = 1 }", "csr mtcause"),
        ('expect.exit_code = 4\nexpect.words = { "19683" = 1 }', "word at 19683"),
        ('expect.exit_code = 4\nexpect.trytes = { "19683" = 1 }', "tryte at 19683"),
    ],
)
def test_every_expectation_is_checked(tmp_path, expect, message):
    failures = _failures(tmp_path, expect)
    assert len(failures) == 1
    assert message in failures[0]


@pytest.mark.parametrize(
    ("text", "message"),
    [
        (HALT_WITH_4 + "expect.exit_code = 4\nspeed = 1", "unknown keys ['speed']"),
        (HALT_WITH_4 + "expect.exit_cod = 4", "unknown keys ['exit_cod']"),
        (HALT_WITH_4, "exactly one of"),
        (
            HALT_WITH_4 + "expect.exit_code = 4\nexpect.step_limit = true",
            "exactly one of",
        ),
        (HALT_WITH_4 + 'expect.exit_code = 4\nsource = "x.s"', "program, source"),
        ('[[case]]\nprogram = "nop"\nexpect.exit_code = 0', "needs a name"),
        ("[defaults]\nram_pages = 3", "no cases"),
        (
            '[defaults]\nname = "x"\n' + HALT_WITH_4 + "expect.exit_code = 4",
            "[defaults]",
        ),
        (
            HALT_WITH_4
            + "expect.exit_code = 4\n"
            + HALT_WITH_4
            + "expect.exit_code = 4",
            "repeat",
        ),
    ],
)
def test_malformed_case_files_are_refused(tmp_path, text, message):
    with pytest.raises(conformance.CaseError) as e:
        _cases(tmp_path, text)
    assert message in str(e.value)


def test_defaults_apply_and_cases_override_them(tmp_path):
    first, second = _cases(
        tmp_path,
        """
[defaults]
max_steps = 7
postlude = "li t6, -39366\\nsw a0, 0(t6)"

[[case]]
name = "uses the defaults"
program = "li a0, 1"
expect.exit_code = 1

[[case]]
name = "overrides them"
max_steps = 9
program = "li a0, 2"
expect.exit_code = 2
""",
    )
    assert (first.max_steps, second.max_steps) == (7, 9)
    assert conformance.run(first).failures == []
    assert conformance.run(second).failures == []


def test_assembly_errors_name_the_case(tmp_path):
    (case,) = _cases(
        tmp_path,
        '[[case]]\nname = "broken"\nprogram = "bogus a0"\nexpect.exit_code = 0',
    )
    with pytest.raises(AsmError, match=r"cases\.toml \[broken\]:1"):
        conformance.run(case)


def test_console_input_reaches_the_program(tmp_path):
    (case,) = _cases(
        tmp_path,
        """
[[case]]
name = "echo"
input = "Z"
program = '''
        li    t0, -19683
        lw    a0, 3(t0)          # console input
        li    t6, -39366
        sw    a0, 0(t6)
'''
expect.exit_code = 90
""",
    )
    assert conformance.run(case).failures == []
