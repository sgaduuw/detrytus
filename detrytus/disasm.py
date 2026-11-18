"""Disassembler: one instruction word back to assembly text."""

from collections.abc import Callable

from detrytus import isa
from detrytus.trits import field, low

_HINT_B = {1: ".likely", -1: ".unlikely", 0: ""}
_HINT_T = {-1: ".n", 0: ".z", 1: ".p"}
_ORDER = {0: "", 1: ".aq", 3: ".rl", 4: ".aqrl"}
_FENCE = "rwio"
_FENCE_ALL = 40


def reg(r: int) -> str:
    return isa.ABI_NAMES.get(r, f"r{r}")


def _target(pc: int | None, off: int) -> str:
    return str(pc + 3 * off) if pc is not None else f".{3 * off:+d}"


def _table(fn: int) -> str:
    names = {value: name for name, value in isa.TABLES.items()}
    return names.get(fn, str(fn))


def _fence_set(trits: int) -> str:
    return "".join(c for i, c in enumerate(_FENCE) if field(trits, i, 1) == 1)


def _fence(d: isa.Decoded) -> list[str]:
    pred, succ = low(d.imm, 4), field(d.imm, 4, 4)
    if (pred, succ) == (_FENCE_ALL, _FENCE_ALL):
        return []
    return [_fence_set(pred), _fence_set(succ)]


def _csr(d: isa.Decoded) -> str:
    number = low(d.imm, 9)
    return isa.CSR_NAMES.get(number, str(number))


# One renderer per operand letter (see isa.py): decoded fields to text.
Renderer = Callable[[isa.Decoded, int | None], list[str]]

RENDERERS: dict[str, Renderer] = {
    "d": lambda d, _pc: [reg(d.rd)],
    "s": lambda d, _pc: [reg(d.rs1)],
    "t": lambda d, _pc: [reg(d.rs2)],
    "u": lambda d, _pc: [reg(d.rs3)],
    "i": lambda d, _pc: [str(d.imm)],
    "m": lambda d, _pc: [f"{d.imm}({reg(d.rs1)})"],
    "a": lambda d, _pc: [f"({reg(d.rs1)})"],
    "L": lambda d, pc: [_target(pc, d.off if d.op.fmt == "B" else d.imm)],
    "N": lambda d, pc: [_target(pc, d.offn)],
    "P": lambda d, pc: [_target(pc, d.offp)],
    "x": lambda d, _pc: [_table(d.fn)],
    "c": lambda d, _pc: [_csr(d)],
    "pos": lambda d, _pc: [str(low(d.imm, 4))],
    "len": lambda d, _pc: [str(field(d.imm, 4, 4))],
    "F": lambda d, _pc: _fence(d),
}


def _mnemonic(d: isa.Decoded) -> str:
    op = d.op
    if op.fmt == "B":
        return "b" + isa.COND_NAMES[d.cond] + _HINT_B[d.hint]
    if op.fmt == "T":
        return op.name + _HINT_T[d.hint]
    if op.atomic:
        return op.name + _ORDER.get(d.rs3, f".?{d.rs3}")
    return op.name


def disassemble(word: int, pc: int | None = None) -> str:
    """Render a word; pc, when given, turns branch offsets into addresses."""
    try:
        d = isa.decode(word)
    except isa.IllegalInstruction as e:
        return f".word {word}  # illegal: {e}"
    parts = [text for kind in d.op.syntax for text in RENDERERS[kind](d, pc)]
    name = _mnemonic(d)
    return f"{name} {', '.join(parts)}" if parts else name
