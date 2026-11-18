"""What each instruction does: one handler per mnemonic, in one table.

The table is explicit rather than built from method names, so renaming an
instruction or mistyping a handler fails at import (see the completeness
check at the bottom) and type checkers can follow every call.
"""

from __future__ import annotations

from collections.abc import Callable
from typing import TYPE_CHECKING

from detrytus import alu, isa
from detrytus.trap import Trap
from detrytus.trits import WORD_TRITS, field, low, set_trit, sign, wrap

if TYPE_CHECKING:
    from detrytus.cpu import Hart

Handler = Callable[["Hart", isa.Decoded], None]
Binary = Callable[[int, int], int]

# --- register-to-register operations -----------------------------------------

# rd = f(rs1, rs2): the shape most R-format instructions share.
BINARY: dict[str, Binary] = {
    "add": lambda a, b: wrap(a + b),
    "sub": lambda a, b: wrap(a - b),
    "mul": lambda a, b: alu.mul_parts(a, b)[1],
    "mulh": lambda a, b: alu.mul_parts(a, b)[0],
    "div": lambda a, b: alu.div_trunc(a, b)[0],
    "rem": lambda a, b: alu.div_trunc(a, b)[1],
    "divr": lambda a, b: alu.div_round(a, b)[0],
    "remr": lambda a, b: alu.div_round(a, b)[1],
    "cmp": lambda a, b: sign(a - b),
    "min": min,
    "max": max,
    "sh": alu.shift,
    "rot": alu.rotate,
    "band": lambda a, b: wrap(a & b),
    "bor": lambda a, b: wrap(a | b),
    "bxor": lambda a, b: wrap(a ^ b),
    "bsh": alu.binary_shift,
}

# rd = f(rs1)
UNARY: dict[str, Callable[[int], int]] = {
    "abs": abs,
    "lzt": alu.leading_zero_trits,
    "tzt": alu.trailing_zero_trits,
    "nzt": alu.nonzero_trits,
}

# rd = f(rs1, imm)
WITH_IMMEDIATE: dict[str, Binary] = {
    "addi": lambda a, imm: wrap(a + imm),
    "cmpi": lambda a, imm: sign(a - imm),
    "shi": alu.shift,
    "roti": alu.rotate,
}

# mem = f(old, rs2), rd = old
AMO: dict[str, Binary] = {
    "amoswap": lambda _old, src: src,
    "amoadd": lambda old, src: wrap(old + src),
    "amomin": min,
    "amomax": max,
}


def _binary(fn: Binary) -> Handler:
    def run(h: Hart, d: isa.Decoded) -> None:
        h.set_x(d.rd, fn(h.x(d.rs1), h.x(d.rs2)))

    return run


def _unary(fn: Callable[[int], int]) -> Handler:
    def run(h: Hart, d: isa.Decoded) -> None:
        h.set_x(d.rd, fn(h.x(d.rs1)))

    return run


def _with_immediate(fn: Binary) -> Handler:
    def run(h: Hart, d: isa.Decoded) -> None:
        h.set_x(d.rd, fn(h.x(d.rs1), d.imm))

    return run


def sel(h: Hart, d: isa.Decoded) -> None:
    c = h.x(d.rs1)
    h.set_x(d.rd, h.x(d.rs2) if c < 0 else h.x(d.rs3) if c > 0 else 0)


def tlut(h: Hart, d: isa.Decoded) -> None:
    h.set_x(d.rd, alu.tlut(h.x(d.rs1), h.x(d.rs2), d.fn))


def _field_args(d: isa.Decoded) -> tuple[int, int]:
    pos, length = low(d.imm, 4), field(d.imm, 4, 4)
    if pos < 0 or length < 1 or pos + length > WORD_TRITS:
        raise Trap(isa.ILLEGAL, d.word)
    return pos, length


def tex(h: Hart, d: isa.Decoded) -> None:
    pos, length = _field_args(d)
    h.set_x(d.rd, alu.trit_extract(h.x(d.rs1), pos, length))


def tins(h: Hart, d: isa.Decoded) -> None:
    pos, length = _field_args(d)
    h.set_x(d.rd, alu.trit_insert(h.x(d.rd), h.x(d.rs1), pos, length))


def lui(h: Hart, d: isa.Decoded) -> None:
    h.set_x(d.rd, d.imm * 3**8)


def auipc(h: Hart, d: isa.Decoded) -> None:
    h.set_x(d.rd, wrap(h.pc + d.imm * 3**8))


# --- memory ------------------------------------------------------------------


def lt(h: Hart, d: isa.Decoded) -> None:
    h.set_x(d.rd, h.load(wrap(h.x(d.rs1) + d.imm), 1))


def lw(h: Hart, d: isa.Decoded) -> None:
    h.set_x(d.rd, h.load(wrap(h.x(d.rs1) + d.imm), 3))


def st(h: Hart, d: isa.Decoded) -> None:
    h.store(wrap(h.x(d.rs1) + d.imm), 1, low(h.x(d.rs2), 9))


def sw(h: Hart, d: isa.Decoded) -> None:
    h.store(wrap(h.x(d.rs1) + d.imm), 3, h.x(d.rs2))


def _amo_addr(h: Hart, d: isa.Decoded) -> int:
    addr = h.x(d.rs1)
    if low(addr, 1) != 0:
        raise Trap(isa.MISALIGNED, addr)
    return addr


def _amo(update: Binary) -> Handler:
    def run(h: Hart, d: isa.Decoded) -> None:
        addr = _amo_addr(h, d)
        old = h.load(addr, 3)
        h.store(addr, 3, update(old, h.x(d.rs2)))
        h.set_x(d.rd, old)

    return run


def amotlut(h: Hart, d: isa.Decoded) -> None:
    _amo(lambda old, src: alu.tlut(old, src, d.fn))(h, d)


def lr(h: Hart, d: isa.Decoded) -> None:
    addr = _amo_addr(h, d)
    h.set_x(d.rd, h.load(addr, 3))
    h.reservation = addr


def sc(h: Hart, d: isa.Decoded) -> None:
    addr = _amo_addr(h, d)
    ok = h.reservation == addr
    h.reservation = None
    if ok:
        h.store(addr, 3, h.x(d.rs2))
    h.set_x(d.rd, 0 if ok else 1)


def cas(h: Hart, d: isa.Decoded) -> None:
    addr = _amo_addr(h, d)
    old = h.load(addr, 3)
    if old == h.x(d.rd):
        h.store(addr, 3, h.x(d.rs2))
    h.set_x(d.rd, old)


# --- control transfer --------------------------------------------------------


def jal(h: Hart, d: isa.Decoded) -> None:
    h.set_x(d.rd, h.next_pc)
    h.next_pc = wrap(h.pc + 3 * d.imm)


def jalr(h: Hart, d: isa.Decoded) -> None:
    target = wrap(h.x(d.rs1) + d.imm)
    if low(target, 1) == -1:
        raise Trap(isa.MISALIGNED, target)
    h.set_x(d.rd, h.next_pc)
    h.next_pc = target


def branch(h: Hart, d: isa.Decoded) -> None:
    if sign(h.x(d.rs1) - h.x(d.rs2)) in isa.COND_TAKEN[d.cond]:
        h.next_pc = wrap(h.pc + 3 * d.off)


def br3(h: Hart, d: isa.Decoded) -> None:
    s = sign(h.x(d.rs1))
    if s:
        h.next_pc = wrap(h.pc + 3 * (d.offn if s < 0 else d.offp))


# --- system ------------------------------------------------------------------


def sys(_h: Hart, _d: isa.Decoded) -> None:
    raise Trap(isa.SYSCALL)


def brk(h: Hart, _d: isa.Decoded) -> None:
    raise Trap(isa.BREAKPOINT, h.pc)


def _privileged(h: Hart, _d: isa.Decoded) -> None:
    if h.ring < 0:
        raise Trap(isa.ILLEGAL)


def _nothing(_h: Hart, _d: isa.Decoded) -> None:
    """For instructions a lone hart with no caches already satisfies:
    fences (nothing is reordered), fence.i (the decode cache is keyed by
    word value, so never stale) and cache maintenance."""


def tret(h: Hart, d: isa.Decoded) -> None:
    _privileged(h, d)
    bank = h.banks[h.ring]
    status = bank[isa.TSTATUS]
    back = field(status, isa.ST_PR, 1)
    if back > h.ring:
        raise Trap(isa.ILLEGAL, d.word)
    bank[isa.TSTATUS] = set_trit(status, isa.ST_IE, field(status, isa.ST_PIE, 1))
    if field(status, isa.ST_SS, 1) == 1:
        h.step_ring = h.ring
    h.ring = back
    h.next_pc = bank[isa.TPC]


def _csr(combine: Binary | None) -> Handler:
    def run(h: Hart, d: isa.Decoded) -> None:
        number = low(d.imm, 9)
        old = h.read_csr(number)
        if d.rs1 != 0:
            src = h.x(d.rs1)
            h.write_csr(number, src if combine is None else combine(old, src))
        h.set_x(d.rd, old)

    return run


def wfi(h: Hart, _d: isa.Decoded) -> None:
    # Nothing else can happen in a lone hart, so skip ahead to the
    # earliest enabled timer instead of spinning.
    deadlines = [
        b[isa.TIMECMP]
        for b in h.banks.values()
        if field(b[isa.IE], 1, 1) == 1 and b[isa.TIMECMP] > h.time
    ]
    if deadlines:
        h.time = min(deadlines)


# --- the table ---------------------------------------------------------------

HANDLERS: dict[str, Handler] = {
    **{name: _binary(fn) for name, fn in BINARY.items()},
    **{name: _unary(fn) for name, fn in UNARY.items()},
    **{name: _with_immediate(fn) for name, fn in WITH_IMMEDIATE.items()},
    **{name: _amo(fn) for name, fn in AMO.items()},
    "sel": sel,
    "tlut": tlut,
    "tex": tex,
    "tins": tins,
    "lui": lui,
    "auipc": auipc,
    "lt": lt,
    "lw": lw,
    "st": st,
    "sw": sw,
    "lr": lr,
    "sc": sc,
    "amotlut": amotlut,
    "cas": cas,
    "jal": jal,
    "jalr": jalr,
    "b": branch,
    "br3": br3,
    "sys": sys,
    "brk": brk,
    "tret": tret,
    "csrrw": _csr(None),
    "csrrtmax": _csr(lambda old, src: alu.tlut(old, src, isa.TABLES["tmax"])),
    "csrrtmin": _csr(lambda old, src: alu.tlut(old, src, isa.TABLES["tmin"])),
    "fence": _nothing,
    "fence.i": _nothing,
    "fence.vm": _privileged,
    "wfi": wfi,
    "cclean": _nothing,
    "cflush": _nothing,
    "cinval": _privileged,
}

if HANDLERS.keys() != isa.OPS.keys():
    raise RuntimeError(
        "instructions without handlers: "
        f"{sorted(isa.OPS.keys() - HANDLERS.keys())}; "
        f"handlers without instructions: {sorted(HANDLERS.keys() - isa.OPS.keys())}"
    )

BY_OPCODE: dict[int, Handler] = {
    isa.OPS[name].opcode: fn for name, fn in HANDLERS.items()
}
