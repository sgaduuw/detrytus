"""docs/isa.md and detrytus.isa must describe the same instruction set.

The opcode tables in the spec are prose that a reader trusts, and the
table in isa.py is what runs. If either changes alone they drift, so the
test parses the spec's tables and compares them with the code.
"""

import re
from pathlib import Path

from detrytus import isa

MINUS = "\u2212"  # the spec writes negative numbers with a true minus sign
SPEC = (Path(__file__).parent.parent / "docs" / "isa.md").read_text(encoding="utf-8")

# Rows like "| 34 | `addi rd, rs1, imm` | ..." or a negative opcode such as -11
ROW = re.compile(r"^\|\s*([\u2212-]?\d+)\s*\|\s*`([a-z][a-z0-9.]*)[ `]", re.M)


def spec_opcodes() -> dict[str, int]:
    start = SPEC.index("## 7. Instructions")
    end = SPEC.index("### 7.13")
    found = {}
    for value, name in ROW.findall(SPEC[start:end]):
        if name[0] == "b" and name[1:] in isa.CONDS:
            continue  # the branch-condition table, not opcodes
        found[name] = int(value.replace(MINUS, "-"))
    return found


def test_every_spec_instruction_matches_the_code():
    spec = spec_opcodes()
    spec.pop("b", None)
    code = {n: op.opcode for n, op in isa.OPS.items()}
    # The branch row is written b<cond>, so the regex cannot see it.
    code.pop("b")
    float_ops = {n for n, v in spec.items() if 40 <= v <= 54 and n.startswith("f")}
    for name in float_ops:
        spec.pop(name)  # floating point is specified but not implemented yet
    assert len(spec) > 50, "the spec parser found too few rows"
    assert spec == code


def test_branch_opcode_matches():
    assert f"| {MINUS}12 | `b<cond> rs1, rs2, label` |" in SPEC
    assert isa.OPS["b"].opcode == -12


def test_tlut_table_values_match_the_spec():
    for name, value in isa.TABLES.items():
        if name == "tabs":
            assert f"`tabs` ({value})" in SPEC
        else:
            assert re.search(
                rf"\| `{name}`\s*\|[^|]*\|\s*{value}\s*\|", SPEC.replace(MINUS, "-")
            ), name


def test_csr_numbers_match_the_spec():
    for name in (
        "edeleg",
        "ideleg",
        "hartid",
        "ptbr",
        "entropy",
        "cycle",
        "time",
        "instret",
        "fcsr",
    ):
        number = str(isa.CSRS[name]).replace("-", MINUS)
        assert re.search(rf"\| `{name}`\s*\|\s*{number}\s*\|", SPEC), name
