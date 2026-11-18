"""Pseudo-instructions: each expands to one or two machine instructions."""

from collections.abc import Callable

from detrytus import isa
from detrytus.asm.operands import OperandError, Operands
from detrytus.expr import Undefined, evaluate
from detrytus.trits import WORD_MAX, half, low, split_high

# (machine mnemonic, operand texts, fields set directly)
Expansion = tuple[str, list[str], dict[str, int]]
Expander = Callable[[Operands, list[str], int, int], list[Expansion]]

BRANCH_ZERO = {
    "beqz": "eq",
    "bnez": "ne",
    "bltz": "lt",
    "bgtz": "gt",
    "blez": "le",
    "bgez": "ge",
}
LOGIC = ("tmin", "tmax", "tmul", "tsum", "tcons", "tany")
SHORT_IMMEDIATE = half(16)  # what one addi can load
LOW_TRITS = 8  # lui/auipc supply everything above the low 8 trits


def _arity(name: str, texts: list[str], n: int) -> None:
    if len(texts) != n:
        raise OperandError(f"{name} takes {n} operand(s), got {len(texts)}")


def _split_constant(value: int) -> tuple[int, int]:
    """value = hi * 3**8 + lo with lo balanced: no carry fix-up needed."""
    lo = low(value, LOW_TRITS)
    return (value - lo) // 3**LOW_TRITS, lo


def _fixed(name: str, op: str, template: list[str], arity: int) -> Expander:
    """An expansion that slots the operands into a template: '{0}' is the
    first operand, '{1}' the second."""

    def expand(
        _ops: Operands, texts: list[str], _pc: int, _size: int
    ) -> list[Expansion]:
        _arity(name, texts, arity)
        return [(op, [t.format(*texts) for t in template], {})]

    return expand


def _li(ops: Operands, texts: list[str], pc: int, size: int) -> list[Expansion]:
    _arity("li", texts, 2)
    rd = texts[0]
    value = ops.value(texts[1], pc)
    if abs(value) > WORD_MAX:
        raise OperandError(f"{value} does not fit a word")
    if size == 1:
        return [("addi", [rd, "zero", str(value)], {})]
    hi, lo = _split_constant(value)
    return [("lui", [rd, str(hi)], {}), ("addi", [rd, rd, str(lo)], {})]


def _la(ops: Operands, texts: list[str], pc: int, _size: int) -> list[Expansion]:
    _arity("la", texts, 2)
    rd = texts[0]
    hi, lo = _split_constant(ops.value(texts[1], pc) - pc)
    return [("auipc", [rd, str(hi)], {}), ("addi", [rd, rd, str(lo)], {})]


def _tmap(ops: Operands, texts: list[str], pc: int, _size: int) -> list[Expansion]:
    _arity("tmap", texts, 3)
    f = ops.value(texts[2], pc)
    outputs = [low(split_high(f, i), 1) for i in range(3)]  # f(-), f(0), f(+)
    table = isa.make_table(lambda a, _b: outputs[a + 1])
    return [("tlut", [texts[0], texts[1], "zero", str(table)], {})]


def _logic(name: str) -> Expander:
    def expand(
        _ops: Operands, texts: list[str], _pc: int, _size: int
    ) -> list[Expansion]:
        _arity(name, texts, 3)
        return [("tlut", [*texts, name], {})]

    return expand


def _branch(cond: str) -> Expander:
    def expand(
        _ops: Operands, texts: list[str], _pc: int, _size: int
    ) -> list[Expansion]:
        return [("b", texts, {"cond": isa.CONDS[cond]})]

    return expand


def _branch_zero(name: str, cond: str) -> Expander:
    def expand(
        _ops: Operands, texts: list[str], _pc: int, _size: int
    ) -> list[Expansion]:
        _arity(name, texts, 2)
        return [("b", [texts[0], "zero", texts[1]], {"cond": isa.CONDS[cond]})]

    return expand


PSEUDOS: dict[str, Expander] = {
    "nop": _fixed("nop", "addi", ["zero", "zero", "0"], 0),
    "mv": _fixed("mv", "addi", ["{0}", "{1}", "0"], 2),
    "neg": _fixed("neg", "sub", ["{0}", "zero", "{1}"], 2),
    "not": _fixed("not", "sub", ["{0}", "zero", "{1}"], 2),
    "sgn": _fixed("sgn", "cmp", ["{0}", "{1}", "zero"], 2),
    "j": _fixed("j", "jal", ["zero", "{0}"], 1),
    "call": _fixed("call", "jal", ["ra", "{0}"], 1),
    "ret": _fixed("ret", "jalr", ["zero", "0(ra)"], 0),
    "csrr": _fixed("csrr", "csrrw", ["{0}", "{1}", "zero"], 2),
    "csrw": _fixed("csrw", "csrrw", ["zero", "{0}", "{1}"], 2),
    "tabs": _fixed("tabs", "tlut", ["{0}", "{1}", "zero", "tabs"], 2),
    "tmap": _tmap,
    "li": _li,
    "la": _la,
    **{name: _logic(name) for name in LOGIC},
    **{"b" + cond: _branch(cond) for cond in isa.CONDS},
    **{name: _branch_zero(name, cond) for name, cond in BRANCH_ZERO.items()},
}


def size(name: str, texts: list[str], symbols: dict[str, int]) -> int:
    """Words a pseudo needs, decided in pass one and kept in pass two."""
    if name == "la":
        return 2
    if name == "li" and len(texts) == 2:
        try:
            value = evaluate(texts[1], symbols, None)
        except Undefined:
            return 2  # not known yet: reserve the long form
        return 1 if abs(value) <= SHORT_IMMEDIATE else 2
    return 1
