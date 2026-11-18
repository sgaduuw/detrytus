"""Operand text to instruction fields, and the encoding of one instruction."""

from collections.abc import Callable

from detrytus import isa
from detrytus.expr import evaluate
from detrytus.trits import WORD_TRITS


class OperandError(ValueError):
    """An operand that is malformed or does not suit its instruction."""


HINTS_B = {"likely": 1, "unlikely": -1}
HINTS_T = {"n": -1, "z": 0, "p": 1}
ORDERINGS = {"aq": 1, "rl": 3, "aqrl": 4}
FENCE_SETS = {"r": 0, "w": 1, "i": 2, "o": 3}
FENCE_ALL = 40  # all four set trits: 1 + 3 + 9 + 27
FIELD_SPLIT = 81  # pos and len, and pred and succ, are two 4-trit halves


class Operands:
    """Reads operand text against the symbol table."""

    def __init__(self, symbols: dict[str, int]) -> None:
        self.symbols = symbols

    def value(self, text: str, here: int) -> int:
        return evaluate(text, self.symbols, here)

    @staticmethod
    def reg(text: str) -> int:
        name = text.strip().lower()
        if name not in isa.REG_NAMES:
            raise OperandError(f"not a register: {text!r}")
        return isa.REG_NAMES[name]

    def mem(self, text: str, here: int) -> tuple[int, int]:
        """Parse 'imm(reg)' or '(reg)' into (imm, reg)."""
        text = text.strip()
        if not text.endswith(")") or "(" not in text:
            raise OperandError(f"expected imm(reg), got {text!r}")
        open_at = text.rindex("(")
        imm_text = text[:open_at].strip()
        imm = self.value(imm_text, here) if imm_text else 0
        return imm, self.reg(text[open_at + 1 : -1])

    def offset(self, text: str, pc: int) -> int:
        """A branch target as a word offset from pc."""
        target = self.value(text, pc)
        delta = target - pc
        if delta % 3:
            raise OperandError(f"branch target {target} is not word aligned")
        return delta // 3

    def named(self, table: dict[str, int], text: str, here: int) -> int:
        """A name from table (a tlut table, a CSR), or any expression."""
        name = text.strip().lower()
        return table[name] if name in table else self.value(text, here)

    @staticmethod
    def fence(ops: list[str]) -> int:
        if not ops:
            return FENCE_ALL + FIELD_SPLIT * FENCE_ALL
        if len(ops) != 2:
            raise OperandError("fence takes no operands, or pred, succ")

        def trits(text: str) -> int:
            letters = text.strip().lower()
            if not letters or any(c not in FENCE_SETS for c in letters):
                raise OperandError(f"fence set {text!r}: use letters r, w, i, o")
            return sum(3 ** FENCE_SETS[c] for c in set(letters))

        return trits(ops[0]) + FIELD_SPLIT * trits(ops[1])


# One function per operand letter (see isa.py). Each reads its text and
# stores the field(s) it sets.
Setter = Callable[[Operands, isa.Op, str, int, dict[str, int]], None]


def _register(name: str) -> Setter:
    def put(ops: Operands, _op: isa.Op, text: str, _pc: int, f: dict[str, int]) -> None:
        f[name] = ops.reg(text)

    return put


def _imm(ops: Operands, _op: isa.Op, text: str, pc: int, f: dict[str, int]) -> None:
    f["imm"] = ops.value(text, pc)


def _mem(ops: Operands, _op: isa.Op, text: str, pc: int, f: dict[str, int]) -> None:
    f["imm"], f["rs1"] = ops.mem(text, pc)


def _amo(ops: Operands, _op: isa.Op, text: str, pc: int, f: dict[str, int]) -> None:
    imm, f["rs1"] = ops.mem(text, pc)
    if imm:
        raise OperandError("atomics take no offset: write (reg)")


def _target(ops: Operands, op: isa.Op, text: str, pc: int, f: dict[str, int]) -> None:
    f["off" if op.fmt == "B" else "imm"] = ops.offset(text, pc)


def _offset_field(name: str) -> Setter:
    def put(ops: Operands, _op: isa.Op, text: str, pc: int, f: dict[str, int]) -> None:
        f[name] = ops.offset(text, pc)

    return put


def _table(ops: Operands, _op: isa.Op, text: str, pc: int, f: dict[str, int]) -> None:
    f["fn"] = ops.named(isa.TABLES, text, pc)


def _csr(ops: Operands, _op: isa.Op, text: str, pc: int, f: dict[str, int]) -> None:
    f["imm"] = ops.named(isa.CSRS, text, pc)


def _field_part(name: str) -> Setter:
    # pos and len are combined into imm once both are known.
    def put(ops: Operands, _op: isa.Op, text: str, pc: int, f: dict[str, int]) -> None:
        f[name] = ops.value(text, pc)

    return put


SETTERS: dict[str, Setter] = {
    "d": _register("rd"),
    "s": _register("rs1"),
    "t": _register("rs2"),
    "u": _register("rs3"),
    "i": _imm,
    "m": _mem,
    "a": _amo,
    "L": _target,
    "N": _offset_field("offn"),
    "P": _offset_field("offp"),
    "x": _table,
    "c": _csr,
    "pos": _field_part("pos"),
    "len": _field_part("len"),
}


def _combine_trit_field(fields: dict[str, int]) -> None:
    if "pos" not in fields:
        return
    pos, length = fields.pop("pos"), fields.pop("len")
    if pos < 0 or length < 1 or pos + length > WORD_TRITS:
        raise OperandError(f"trit field {pos}+{length} is outside the word")
    fields["imm"] = pos + FIELD_SPLIT * length


def _apply_suffix(op: isa.Op, suffix: str, fields: dict[str, int]) -> None:
    if not suffix:
        return
    if op.atomic and suffix in ORDERINGS:
        fields["rs3"] = ORDERINGS[suffix]
    elif op.fmt == "T" and suffix in HINTS_T:
        fields["hint"] = HINTS_T[suffix]
    elif op.fmt == "B" and suffix in HINTS_B:
        fields["hint"] = HINTS_B[suffix]
    else:
        raise OperandError(f"{op.name} takes no suffix .{suffix}")


def encode_instruction(
    ops: Operands,
    op: isa.Op,
    suffix: str,
    texts: list[str],
    pc: int,
    extra: dict[str, int] | None = None,
) -> int:
    """Encode one machine instruction from its operand texts."""
    fields = dict(extra or {})
    syntax = list(op.syntax)
    if syntax == ["F"]:
        fields["imm"] = ops.fence(texts)
        syntax, texts = [], []
    if len(texts) != len(syntax):
        raise OperandError(
            f"{op.name} takes {len(syntax)} operand(s), got {len(texts)}"
        )
    for kind, text in zip(syntax, texts, strict=True):
        SETTERS[kind](ops, op, text, pc, fields)
    _combine_trit_field(fields)
    _apply_suffix(op, suffix, fields)
    return isa.encode(op, **fields)
