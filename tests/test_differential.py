"""The Python and Rust cores, step by step: same program, same state.

The conformance suite checks what a program ends with; this checks every
step on the way, across all the architectural state both cores keep:
registers, pc, ring, the trap banks, stored CSRs, the reservation,
single-step state, counters, the console, and memory (every
MEMORY_EVERY steps, and at the end). It runs the conformance programs
and random ones, built from legal encodings so most of them execute
rather than trap. Skipped unless the optional detrytus-rs package is
installed (docs/rust-plan.md, phase 4).
"""

import contextlib
import random
from pathlib import Path
from typing import Any

import pytest

from detrytus import conformance, isa, rust
from detrytus.asm import Image, assemble
from detrytus.devices import Halt
from detrytus.machine import Machine
from detrytus.trap import UnhandledTrap
from detrytus.trits import TRYTE_TRITS, field, half, low, split_high
from tests.backends import needs_rust

pytestmark = needs_rust

ROOT = Path(__file__).parent.parent / "conformance"
MEMORY_EVERY = 50
CASE_STEPS = 20_000  # per conformance program
RANDOM_PROGRAMS = 300
RANDOM_STEPS = 300
RANDOM_WORDS = 120
CODE = 3**9  # random code starts at the centre of RAM page 1


# --- running both ------------------------------------------------------------


def _py_state(m: Machine) -> dict[str, Any]:
    h = m.hart
    return {
        "regs": list(h.regs),
        "pc": h.pc,
        "ring": h.ring,
        "instret": h.instret,
        "time": h.time,
        "bank+1": list(h.banks[1]),
        "bank0": list(h.banks[0]),
        "csrs": sorted(h.csrs.items()),
        "reservation": h.reservation,
        "step_ring": h.step_ring,
        "console": m.console.text,
    }


def _rs_state(core: Any) -> dict[str, Any]:
    return {
        "regs": core.regs(),
        "pc": core.pc,
        "ring": core.ring,
        "instret": core.instret,
        "time": core.time,
        "bank+1": core.bank(1),
        "bank0": core.bank(0),
        "csrs": core.stored_csrs(),
        "reservation": core.reservation,
        "step_ring": core.step_ring,
        "console": core.console_text(),
    }


def _py_memory(m: Machine) -> list[tuple[int, int]]:
    return sorted((a, t) for a, t in m.bus.mem.items() if t)


def _py_step(m: Machine) -> tuple[Any, ...]:
    try:
        m.hart.step()
    except Halt as e:
        return ("exit", e.code)
    except UnhandledTrap as e:
        return ("unhandled", e.cause, e.tval, e.pc, e.ring)
    except NotImplementedError:
        return ("unsupported",)
    return ("running",)


def _rs_step(core: Any) -> tuple[Any, ...]:
    kind, numbers, _message = core.step()
    if kind == "exit":
        return ("exit", numbers[0])
    if kind == "unhandled":
        return ("unhandled", *numbers)
    return (kind,)


def _diff(a: dict[str, Any], b: dict[str, Any]) -> str:
    return "; ".join(f"{k}: python {a[k]} rust {b[k]}" for k in a if a[k] != b[k])


def compare(image: Image, steps: int, ram_pages: int = 729, text: str = "") -> int:
    """Step both cores through image; fail at the first difference.
    Returns the number of steps taken."""
    py = Machine(ram_pages=ram_pages, seed=1)
    rs = rust.RustMachine(ram_pages=ram_pages, seed=1)
    for m in (py, rs):
        m.console.feed(text)
        m.load(image)
    core = rs.core
    for n in range(1, steps + 1):
        outcomes = (_py_step(py), _rs_step(core))
        assert outcomes[0] == outcomes[1], f"step {n}: outcomes {outcomes}"
        a, b = _py_state(py), _rs_state(core)
        assert a == b, f"step {n}: {_diff(a, b)}"
        if n % MEMORY_EVERY == 0 or outcomes[0][0] != "running":
            assert _py_memory(py) == core.trytes(), f"step {n}: memory differs"
        if outcomes[0][0] != "running":
            return n
    assert _py_memory(py) == core.trytes(), "memory differs at the end"
    return steps


# --- the conformance programs ------------------------------------------------

CASES = conformance.load_all(ROOT)


@pytest.mark.parametrize("case", CASES, ids=[c.id for c in CASES])
def test_conformance_programs_step_alike(case):
    image = conformance.build(case)
    compare(image, min(case.max_steps, CASE_STEPS), case.ram_pages, case.input)


# --- random programs ---------------------------------------------------------

# Firmware: a handler in each bank that skips the instruction that
# trapped (or was interrupted), registers seeded with values that are
# often RAM addresses, then a jump to the random code.
HANDLERS = """
        .org  6000
mhandler:
        csrr  t6, mtpc
        addi  t6, t6, 3
        csrw  mtpc, t6
        tret
shandler:
        csrr  t6, stpc
        addi  t6, t6, 3
        csrw  stpc, t6
        tret
"""


def _firmware(rng: random.Random) -> str:
    lines = [
        "la t0, mhandler",
        "csrw mtvec, t0",
        "la t0, shandler",
        "csrw stvec, t0",
    ]
    for r in range(-13, 14):
        if r == 0:
            continue
        if rng.random() < 0.6:
            value = 3 * rng.randint(3**9 // 3, 5 * 3**9 // 3)  # in RAM, aligned
        else:
            value = rng.randint(-1000, 1000)
        lines.append(f"li r{r}, {value}")
    lines.append(f"li t6, {CODE}")
    lines.append("jalr zero, 0(t6)")
    return "\n".join(lines) + HANDLERS


def _reg(rng: random.Random) -> int:
    return rng.randint(-13, 13)


def _small(rng: random.Random, n: int = 30) -> int:
    return rng.randint(-n, n)


def _immediate(rng: random.Random, op: isa.Op) -> int:
    syn = set(op.syntax)
    if "c" in syn:  # mostly real CSRs
        if rng.random() < 0.9:
            return rng.choice(list(isa.CSRS.values()))
        return rng.randint(-half(9), half(9))
    if "pos" in syn or "F" in syn:
        return rng.randint(-half(8), half(8))
    if "m" in syn:  # mostly aligned offsets
        return 3 * _small(rng, 10) + (rng.random() < 0.1)
    if "i" in syn:
        width = 19 if op.fmt == "U" else 16
        return (
            _small(rng, 40)
            if rng.random() < 0.8
            else rng.randint(-half(width), half(width))
        )
    return 0


def _fields(rng: random.Random, op: isa.Op) -> dict[str, int]:
    syn = set(op.syntax)
    f: dict[str, int] = {}
    if op.fmt == "R":
        f = {
            "rd": _reg(rng) if "d" in syn else 0,
            "rs1": _reg(rng) if syn & {"s", "a"} else 0,
            "rs2": _reg(rng) if "t" in syn else 0,
        }
        if op.atomic:
            f["rs3"] = rng.choice([0, 1, 3, 4])  # aq and rl trits
        elif "u" in syn:
            f["rs3"] = _reg(rng)
        if "x" in syn:
            f["fn"] = rng.randint(-half(9), half(9))
    elif op.fmt in ("I", "S"):
        target = "rd" if op.fmt == "I" else "rs2"
        f = {
            target: _reg(rng) if syn & {"d", "t"} else 0,
            "rs1": _reg(rng) if syn & {"s", "m"} else 0,
            "imm": _immediate(rng, op),
        }
    elif op.fmt == "B":
        f = {"cond": rng.choice(list(isa.CONDS.values())), "rs1": _reg(rng)}
        f |= {"rs2": _reg(rng), "off": _small(rng, 20)}
    elif op.fmt == "T":
        f = {"rs1": _reg(rng), "offn": _small(rng, 20), "offp": _small(rng, 20)}
    elif op.fmt == "J":
        f = {"rd": _reg(rng), "imm": _small(rng, 20)}
    elif op.fmt == "U":
        f = {"rd": _reg(rng), "imm": _immediate(rng, op)}
    return f


def _word(rng: random.Random) -> int:
    if rng.random() < 0.05:  # anything at all, mostly illegal
        return rng.randint(-half(27), half(27))
    op = rng.choice(list(isa.OPS.values()))
    return isa.encode(op, **_fields(rng, op))


def random_image(seed: int) -> Image:
    rng = random.Random(seed)
    image = assemble(_firmware(rng))
    for i in range(RANDOM_WORDS):
        word = _word(rng)
        addr = CODE + 3 * i
        rest = split_high(word, TRYTE_TRITS)
        image.trytes[addr - 1] = low(word, TRYTE_TRITS)
        image.trytes[addr] = low(rest, TRYTE_TRITS)
        image.trytes[addr + 1] = field(word, 2 * TRYTE_TRITS, TRYTE_TRITS)
    return image


def test_random_programs_step_alike():
    steps = 0
    for seed in range(RANDOM_PROGRAMS):
        try:
            steps += compare(random_image(seed), RANDOM_STEPS)
        except AssertionError as e:
            raise AssertionError(f"random program {seed}: {e}") from None
    # Preconditions: the programs ran, rather than all stopping at once.
    assert steps > RANDOM_PROGRAMS * RANDOM_STEPS // 2, steps


def test_random_programs_use_most_instructions():
    rng = random.Random(0)
    seen = set()
    for _ in range(RANDOM_PROGRAMS * RANDOM_WORDS // 10):
        with contextlib.suppress(isa.IllegalInstruction):
            seen.add(isa.decode(_word(rng)).op.name)
    assert seen == set(isa.OPS)


def test_a_difference_is_reported():
    # The comparison itself must be able to fail: a core that ran a
    # different image would differ at the first step.
    image = assemble("li a0, 1\nli t6, -39366\nsw a0, 0(t6)")
    py = Machine(seed=1)
    py.load(image)
    _py_step(py)
    rs = rust.RustMachine(seed=1)
    rs.load(assemble("li a0, 2\nli t6, -39366\nsw a0, 0(t6)"))
    _rs_step(rs.core)
    assert _diff(_py_state(py), _rs_state(rs.core)).startswith("regs:")
