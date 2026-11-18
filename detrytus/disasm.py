"""Disassembler: one instruction word back to assembly text."""

from detrytus import isa as I
from detrytus.trits import field, low

_HINT_B = {1: ".likely", -1: ".unlikely", 0: ""}
_HINT_T = {-1: ".n", 0: ".z", 1: ".p"}
_ORDER = {0: "", 1: ".aq", 3: ".rl", 4: ".aqrl"}
_FENCE = "rwio"


def reg(r: int) -> str:
    return I.ABI_NAMES.get(r, f"r{r}")


def _target(pc: int | None, off: int) -> str:
    return str(pc + 3 * off) if pc is not None else f".{3 * off:+d}"


def _table(fn: int) -> str:
    for name, value in I.TABLES.items():
        if value == fn:
            return name
    return str(fn)


def _fence_set(trits: int) -> str:
    return "".join(c for i, c in enumerate(_FENCE) if field(trits, i, 1) == 1)


def disassemble(word: int, pc: int | None = None) -> str:
    """Render a word; pc, when given, turns branch offsets into addresses."""
    try:
        d = I.decode(word)
    except I.IllegalInstruction as e:
        return f".word {word}  # illegal: {e}"
    op = d.op
    name = op.name
    if op.fmt == "B":
        name = "b" + I.COND_NAMES[d.cond] + _HINT_B[d.hint]
    elif op.fmt == "T":
        name += _HINT_T[d.hint]
    elif op.atomic:
        name += _ORDER.get(d.rs3, f".?{d.rs3}")
    parts = []
    for kind in op.syntax:
        if kind == "d":
            parts.append(reg(d.rd))
        elif kind == "s":
            parts.append(reg(d.rs1))
        elif kind == "t":
            parts.append(reg(d.rs2))
        elif kind == "u":
            parts.append(reg(d.rs3))
        elif kind == "i":
            parts.append(str(d.imm))
        elif kind == "m":
            parts.append(f"{d.imm}({reg(d.rs1)})")
        elif kind == "a":
            parts.append(f"({reg(d.rs1)})")
        elif kind == "L":
            parts.append(_target(pc, d.off if op.fmt == "B" else d.imm))
        elif kind == "N":
            parts.append(_target(pc, d.offn))
        elif kind == "P":
            parts.append(_target(pc, d.offp))
        elif kind == "x":
            parts.append(_table(d.fn))
        elif kind == "c":
            number = low(d.imm, 9)
            parts.append(I.CSR_NAMES.get(number, str(number)))
        elif kind == "pos":
            parts.append(str(low(d.imm, 4)))
        elif kind == "len":
            parts.append(str(field(d.imm, 4, 4)))
        elif kind == "F":
            pred, succ = low(d.imm, 4), field(d.imm, 4, 4)
            if (pred, succ) != (40, 40):
                parts += [_fence_set(pred), _fence_set(succ)]
    return f"{name} {', '.join(parts)}" if parts else name
