"""The instruction set as data: opcodes, formats, fields, registers, CSRs.

The assembler, the disassembler and the emulator all read this module,
so there is exactly one place that decides what an encoding means. The
table mirrors docs/isa.md section 7; tests/test_spec_sync.py checks the
two agree.
"""

from dataclasses import dataclass

from .trits import field, half, low

# --- registers ---------------------------------------------------------------

REG_NAMES: dict[str, int] = {f"r{i}": i for i in range(-13, 14)}
REG_NAMES.update(
    {
        "zero": 0,
        **{f"a{i}": 1 + i for i in range(6)},
        **{f"t{i}": 7 + i for i in range(7)},
        **{f"s{i}": -1 - i for i in range(9)},
        "fp": -1,
        "gp": -10,
        "tp": -11,
        "ra": -12,
        "sp": -13,
    }
)

ABI_NAMES: dict[int, str] = {
    0: "zero",
    **{1 + i: f"a{i}" for i in range(6)},
    **{7 + i: f"t{i}" for i in range(7)},
    **{-1 - i: f"s{i}" for i in range(9)},
    -10: "gp",
    -11: "tp",
    -12: "ra",
    -13: "sp",
}

# --- fields ------------------------------------------------------------------

# (lowest trit, width). Register fields sit at the same place in every
# format that has them, so they are listed once.
OP = (0, 5)
RD = (5, 3)
RS1 = (8, 3)
RS2 = (11, 3)
RS3 = (14, 3)
FN = (17, 10)
IMM_I = (11, 16)
COND = (5, 2)
HINT_B = (7, 1)
OFF_B = (14, 13)
HINT_T = (5, 1)
ZERO_T = (6, 2)
OFFN = (11, 8)
OFFP = (19, 8)
IMM_U = (8, 19)

FORMAT_FIELDS: dict[str, dict[str, tuple[int, int]]] = {
    "R": {"rd": RD, "rs1": RS1, "rs2": RS2, "rs3": RS3, "fn": FN},
    "I": {"rd": RD, "rs1": RS1, "imm": IMM_I},
    "S": {"rs2": RD, "rs1": RS1, "imm": IMM_I},
    "B": {"cond": COND, "hint": HINT_B, "rs1": RS1, "rs2": RS2, "off": OFF_B},
    "T": {"hint": HINT_T, "zero": ZERO_T, "rs1": RS1, "offn": OFFN, "offp": OFFP},
    "U": {"rd": RD, "imm": IMM_U},
    "J": {"rd": RD, "imm": IMM_U},
}

# --- branch conditions -------------------------------------------------------

CONDS: dict[str, int] = {"eq": 0, "ne": 3, "gt": 1, "lt": -1, "ge": 2, "le": -2}
COND_NAMES = {v: k for k, v in CONDS.items()}
# For each cond, the cmp outcomes that take the branch.
COND_TAKEN: dict[int, frozenset[int]] = {
    0: frozenset({0}),
    3: frozenset({-1, 1}),
    1: frozenset({1}),
    -1: frozenset({-1}),
    2: frozenset({0, 1}),
    -2: frozenset({-1, 0}),
}

# --- instructions ------------------------------------------------------------

# Operand syntax letters, used by the assembler and disassembler:
#   d rd   s rs1   t rs2   u rs3   i immediate   m imm(rs1)   a (rs1)
#   L branch/jump target   N/P br3 targets   x tlut table   c CSR
#   pos/len trit-field position and length   F fence sets


@dataclass(frozen=True)
class Op:
    name: str
    opcode: int
    fmt: str
    syntax: tuple[str, ...]

    @property
    def atomic(self) -> bool:
        return self.opcode in ATOMIC_OPCODES


ATOMIC_OPCODES = frozenset(range(-27, -19))

_TABLE = [
    # 7.1 integer arithmetic
    ("add", 1, "R", "d,s,t"),
    ("sub", 2, "R", "d,s,t"),
    ("mul", 3, "R", "d,s,t"),
    ("mulh", 4, "R", "d,s,t"),
    ("div", 5, "R", "d,s,t"),
    ("rem", 6, "R", "d,s,t"),
    ("divr", 7, "R", "d,s,t"),
    ("remr", 8, "R", "d,s,t"),
    # 7.2 compare, select, min/max
    ("cmp", 9, "R", "d,s,t"),
    ("min", 10, "R", "d,s,t"),
    ("max", 11, "R", "d,s,t"),
    ("abs", 12, "R", "d,s"),
    ("sel", 13, "R", "d,s,t,u"),
    # 7.3 shifts and trit fields
    ("sh", 14, "R", "d,s,t"),
    ("rot", 15, "R", "d,s,t"),
    ("shi", 30, "I", "d,s,i"),
    ("roti", 31, "I", "d,s,i"),
    ("tex", 32, "I", "d,s,pos,len"),
    ("tins", 33, "I", "d,s,pos,len"),
    # 7.4 trit counting
    ("lzt", 16, "R", "d,s"),
    ("tzt", 17, "R", "d,s"),
    ("nzt", 18, "R", "d,s"),
    # 7.5 tritwise logic
    ("tlut", 19, "R", "d,s,t,x"),
    # 7.6 binary operations
    ("band", 20, "R", "d,s,t"),
    ("bor", 21, "R", "d,s,t"),
    ("bxor", 22, "R", "d,s,t"),
    ("bsh", 23, "R", "d,s,t"),
    # 7.7 immediates and pc-relative
    ("addi", 34, "I", "d,s,i"),
    ("cmpi", 35, "I", "d,s,i"),
    ("lui", 36, "U", "d,i"),
    ("auipc", 37, "U", "d,i"),
    # 7.8 loads and stores
    ("lt", -1, "I", "d,m"),
    ("lw", -2, "I", "d,m"),
    ("st", -3, "S", "t,m"),
    ("sw", -4, "S", "t,m"),
    # 7.9 atomics
    ("lr", -20, "R", "d,a"),
    ("sc", -21, "R", "d,t,a"),
    ("amoswap", -22, "R", "d,t,a"),
    ("amoadd", -23, "R", "d,t,a"),
    ("amomin", -24, "R", "d,t,a"),
    ("amomax", -25, "R", "d,t,a"),
    ("amotlut", -26, "R", "d,t,a,x"),
    ("cas", -27, "R", "d,t,a"),
    # 7.10 control transfer
    ("jal", -10, "J", "d,L"),
    ("jalr", -11, "I", "d,m"),
    ("b", -12, "B", "s,t,L"),
    ("br3", -13, "T", "s,N,P"),
    # 7.12 system
    ("sys", -30, "I", ""),
    ("tret", -31, "I", ""),
    ("csrrw", -32, "I", "d,c,s"),
    ("csrrtmax", -33, "I", "d,c,s"),
    ("csrrtmin", -34, "I", "d,c,s"),
    ("fence", -35, "I", "F"),
    ("fence.i", -36, "I", ""),
    ("fence.vm", -37, "R", "s,t"),
    ("wfi", -38, "I", ""),
    ("brk", -39, "I", ""),
    ("cclean", -40, "R", "s"),
    ("cflush", -41, "R", "s"),
    ("cinval", -42, "R", "s"),
]

OPS: dict[str, Op] = {
    name: Op(name, code, fmt, tuple(syn.split(",")) if syn else ())
    for name, code, fmt, syn in _TABLE
}
BY_OPCODE: dict[int, Op] = {op.opcode: op for op in OPS.values()}
assert len(BY_OPCODE) == len(OPS), "duplicate opcode"

BUNDLE_OPCODE = 121  # 6.1; recognised but not yet executed

# --- tritwise truth tables (7.5) ---------------------------------------------


def make_table(fn) -> int:
    """Encode a two-input trit function as the 9-trit tlut table."""
    return sum(fn(k // 3 - 1, k % 3 - 1) * 3**k for k in range(9))


def _sum3(a: int, b: int) -> int:
    return low(a + b, 1)


TABLES: dict[str, int] = {
    "tmin": make_table(min),
    "tmax": make_table(max),
    "tmul": make_table(lambda a, b: a * b),
    "tsum": make_table(_sum3),
    "tcons": make_table(lambda a, b: a if a == b else 0),
    "tany": make_table(lambda a, b: (a + b > 0) - (a + b < 0)),
    "tabs": make_table(lambda a, b: abs(a)),
}

# --- CSRs (8.2) --------------------------------------------------------------


def csr_number(ring: int, access: int, index: int) -> int:
    return 6561 * ring + 2187 * access + index


BANK = [
    "tstatus",
    "tvec",
    "tpc",
    "tcause",
    "tval",
    "scratch",
    "ie",
    "ip",
    "timecmp",
]

CSRS: dict[str, int] = {}
for _prefix, _ring in (("m", 1), ("s", 0)):
    for _i, _name in enumerate(BANK):
        CSRS[_prefix + _name] = csr_number(_ring, 1, _i)
CSRS.update(
    {
        "edeleg": csr_number(1, 1, 20),
        "ideleg": csr_number(1, 1, 21),
        "hartid": csr_number(1, -1, 0),
        "ptbr": csr_number(0, 1, 10),
        "entropy": csr_number(0, -1, 0),
        "cycle": csr_number(-1, -1, 0),
        "time": csr_number(-1, -1, 1),
        "instret": csr_number(-1, -1, 2),
        "fcsr": csr_number(-1, 1, 0),
        **{f"tdaddr{i}": csr_number(0, 1, 20 + i) for i in range(9)},
        **{f"tdctl{i}": csr_number(0, 1, 30 + i) for i in range(9)},
    }
)
CSR_NAMES = {v: k for k, v in CSRS.items()}


def csr_ring(number: int) -> int:
    """Lowest ring allowed to access a CSR: trit 8 of its number."""
    return field(number, 8, 1)


def csr_writable(number: int) -> bool:
    return field(number, 7, 1) == 1


# --- trap causes (8.5) -------------------------------------------------------

ILLEGAL = 1
MISALIGNED = 2
ACCESS_FAULT = 3
SYSCALL = 4
BREAKPOINT = 5
INSTR_PAGE_FAULT = 6
LOAD_PAGE_FAULT = 7
STORE_PAGE_FAULT = 8
SINGLE_STEP = 9
IRQ_TIMER = -1
IRQ_EXTERNAL = -2
IRQ_SOFTWARE = -3

# --- encode / decode ---------------------------------------------------------


class EncodingError(ValueError):
    pass


def put(value: int, spec: tuple[int, int], name: str) -> int:
    lo, width = spec
    if abs(value) > half(width):
        raise EncodingError(
            f"{name} = {value} does not fit {width} trits (±{half(width)})"
        )
    return value * 3**lo


def encode(op: Op, **fields: int) -> int:
    """Build an instruction word from named field values."""
    layout = FORMAT_FIELDS[op.fmt]
    word = op.opcode
    for name, value in fields.items():
        if name not in layout:
            raise EncodingError(f"{op.name}: format {op.fmt} has no field {name}")
        word += put(value, layout[name], name)
    return word


class Decoded:
    """One decoded instruction: the Op plus every field of its format."""

    __slots__ = (
        "cond",
        "fn",
        "hint",
        "imm",
        "off",
        "offn",
        "offp",
        "op",
        "rd",
        "rs1",
        "rs2",
        "rs3",
        "word",
    )

    def __init__(self, op: Op, word: int) -> None:
        self.op = op
        self.word = word
        self.rd = self.rs1 = self.rs2 = self.rs3 = 0
        self.fn = self.imm = self.cond = self.hint = 0
        self.off = self.offn = self.offp = 0
        for name, (lo, width) in FORMAT_FIELDS[op.fmt].items():
            setattr(self, name if name != "zero" else "off", field(word, lo, width))


class IllegalInstruction(Exception):
    pass


def decode(word: int) -> Decoded:
    """Decode a word, raising IllegalInstruction for any invalid encoding.

    Fields an instruction does not use must be zero (section 6), so they
    stay free for extensions; that is enforced here rather than in the
    executor so the disassembler applies the same rule.
    """
    opcode = low(word, 5)
    op = BY_OPCODE.get(opcode)
    if op is None:
        raise IllegalInstruction(f"opcode {opcode}")
    d = Decoded(op, word)
    syn = set(op.syntax)
    if op.fmt == "R":
        _zero(d.rd, "d" in syn, "rd")
        _zero(d.rs1, bool(syn & {"s", "a"}), "rs1")
        _zero(d.rs2, "t" in syn, "rs2")
        if op.atomic:
            # rs3 carries the ordering: trit 0 aq, trit 1 rl, trit 2 zero.
            if any(t not in (0, 1) for t in (field(d.rs3, 0, 1), field(d.rs3, 1, 1))):
                raise IllegalInstruction("ordering trits must be 0 or +")
            _zero(field(d.rs3, 2, 1), False, "rs3")
        else:
            _zero(d.rs3, "u" in syn, "rs3")
        if "x" in syn:
            _zero(field(d.fn, 9, 1), False, "fn[9]")
        else:
            _zero(d.fn, False, "fn")
    elif op.fmt == "I":
        _zero(d.rd, "d" in syn, "rd")
        _zero(d.rs1, bool(syn & {"s", "m"}), "rs1")
        if "c" in syn:
            _zero(d.imm - low(d.imm, 9), False, "imm above the CSR number")
        elif "pos" in syn or "F" in syn:
            _zero(d.imm - low(d.imm, 8), False, "imm above trit 7")
        else:
            _zero(d.imm, bool(syn & {"i", "m"}), "imm")
    elif op.fmt == "B":
        if d.cond not in COND_TAKEN:
            raise IllegalInstruction(f"reserved branch condition {d.cond}")
    elif op.fmt == "T":
        _zero(d.off, False, "trits 7..6")
    return d


def _zero(value: int, used: bool, name: str) -> None:
    if not used and value != 0:
        raise IllegalInstruction(f"{name} must be zero")
