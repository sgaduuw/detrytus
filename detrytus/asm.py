"""Two-pass assembler. Syntax is described in docs/assembler.md.

Pass one lays out every statement and binds labels; pass two evaluates
operands and encodes. A statement's size must not change between the
passes, so anything whose size depends on a value (``li``) decides its
size in pass one, taking the long form when the value is not yet known.
"""

import re
from dataclasses import dataclass, field

from detrytus import isa as I
from detrytus.disasm import disassemble
from detrytus.expr import ExprError, Undefined, evaluate
from detrytus.trits import TRYTE_MAX, WORD_MAX, half, low, split_high


class AsmError(Exception):
    def __init__(self, message: str, line: int | None = None, source: str = "") -> None:
        where = f"line {line}: " if line is not None else ""
        super().__init__(
            f"{where}{message}" + (f"\n    {source.strip()}" if source else "")
        )
        self.line = line


@dataclass
class Listing:
    addr: int
    word: int | None
    line: int
    source: str
    text: str = ""  # disassembly of the emitted word


@dataclass
class Image:
    trytes: dict[int, int] = field(default_factory=dict)
    symbols: dict[str, int] = field(default_factory=dict)
    listing: list[Listing] = field(default_factory=list)


@dataclass
class Statement:
    line: int
    source: str
    labels: list[str]
    mnemonic: str  # '' for a label-only line
    operands: list[str]
    addr: int = 0
    size: int = 0  # words, or trytes for tryte data
    kind: str = "word"  # 'word', 'tryte', or 'none'


# --- lexing ------------------------------------------------------------------

_LABEL = re.compile(r"\s*([A-Za-z_.][A-Za-z0-9_.$]*)\s*:(?!=)")


def _strip_comment(line: str) -> str:
    quote = None
    for i, c in enumerate(line):
        if quote:
            if c == "\\":
                continue
            if c == quote and line[i - 1] != "\\":
                quote = None
        elif c in "\"'":
            quote = c
        elif c in "#;":
            return line[:i]
    return line


def _split_operands(text: str) -> list[str]:
    out, depth, quote, cur = [], 0, None, []
    for i, c in enumerate(text):
        if quote:
            cur.append(c)
            if c == quote and text[i - 1] != "\\":
                quote = None
            continue
        if c in "\"'":
            quote = c
        elif c == "(":
            depth += 1
        elif c == ")":
            depth -= 1
        elif c == "," and depth == 0:
            out.append("".join(cur).strip())
            cur = []
            continue
        cur.append(c)
    if "".join(cur).strip():
        out.append("".join(cur).strip())
    return out


def parse(source: str) -> list[Statement]:
    stmts = []
    for n, raw in enumerate(source.splitlines(), start=1):
        text = _strip_comment(raw)
        labels = []
        while m := _LABEL.match(text):
            labels.append(m.group(1))
            text = text[m.end() :]
        text = text.strip()
        if not text:
            if labels:
                stmts.append(Statement(n, raw, labels, "", []))
            continue
        mnemonic, *rest = text.split(None, 1)
        operands = _split_operands(rest[0]) if rest else []
        stmts.append(Statement(n, raw, labels, mnemonic.lower(), operands))
    return stmts


# --- pseudo-instructions and suffixes ----------------------------------------

BRANCH_ZERO = {
    "beqz": "eq",
    "bnez": "ne",
    "bltz": "lt",
    "bgtz": "gt",
    "blez": "le",
    "bgez": "ge",
}
LOGIC_PSEUDOS = {"tmin", "tmax", "tmul", "tsum", "tcons", "tany"}
HINTS_B = {"likely": 1, "unlikely": -1}
HINTS_T = {"n": -1, "z": 0, "p": 1}
ORDERINGS = {"aq": 1, "rl": 3, "aqrl": 4}
FENCE_SETS = {"r": 0, "w": 1, "i": 2, "o": 3}


def _split_suffix(mnemonic: str) -> tuple[str, str]:
    if mnemonic in I.OPS:
        return mnemonic, ""
    base, dot, suffix = mnemonic.rpartition(".")
    if dot and base:
        return base, suffix
    return mnemonic, ""


# --- the assembler -----------------------------------------------------------


class Assembler:
    def __init__(self) -> None:
        self.symbols: dict[str, int] = {}
        self.image = Image()

    # pass one ---------------------------------------------------------------

    def layout(self, stmts: list[Statement]) -> None:
        loc = -1  # next free tryte; the first word lands centred on 0
        pending: list[Statement] = []

        def bind(addr: int) -> None:
            for s in pending:
                for label in s.labels:
                    self._define(label, addr, s)
            pending.clear()

        for s in stmts:
            if s.labels:
                pending.append(s)
            m = s.mnemonic
            if not m:
                s.kind = "none"
                continue
            try:
                if m in (".equ", ".set"):
                    s.kind = "none"
                    name, value = self._expect(s, 2)
                    self._define(name, evaluate(value, self.symbols, None), s)
                elif m == ".org":
                    (value,) = self._expect(s, 1)
                    target = evaluate(value, self.symbols, None)
                    if low(target, 1) != 0:
                        raise AsmError(
                            ".org needs a word-aligned address (a multiple of 3)"
                        )
                    s.kind = "none"
                    loc = target - 1
                elif m == ".align":
                    (value,) = self._expect(s, 1)
                    k = evaluate(value, self.symbols, None)
                    unit = 3**k
                    centre = -(-(loc + 1) // unit) * unit
                    s.kind = "none"
                    loc = centre - 1
                elif m in (".tryte", ".string", ".ascii", ".zero"):
                    s.kind = "tryte"
                    s.addr = loc
                    s.size = self._tryte_count(s)
                    bind(loc)
                    loc += s.size
                else:
                    s.kind = "word"
                    s.size = self._word_count(s)
                    centre = -(-(loc + 1) // 3) * 3
                    s.addr = centre
                    bind(centre)
                    loc = centre + 3 * s.size - 1
            except (ExprError, ValueError) as e:
                raise AsmError(str(e), s.line, s.source) from None
        bind(loc)

    def _define(self, name: str, value: int, s: Statement) -> None:
        if name in self.symbols:
            raise AsmError(f"symbol {name!r} defined twice", s.line, s.source)
        self.symbols[name] = value

    def _expect(self, s: Statement, n: int) -> list[str]:
        if len(s.operands) != n:
            raise AsmError(f"{s.mnemonic} takes {n} operand(s)", s.line, s.source)
        return s.operands

    def _tryte_count(self, s: Statement) -> int:
        if s.mnemonic == ".zero":
            (n,) = self._expect(s, 1)
            return evaluate(n, self.symbols, None)
        if s.mnemonic == ".tryte":
            return len(s.operands)
        text = self._string(s)
        return len(text) + (1 if s.mnemonic == ".string" else 0)

    def _string(self, s: Statement) -> str:
        (lit,) = self._expect(s, 1)
        if len(lit) < 2 or lit[0] != '"' or lit[-1] != '"':
            raise AsmError("expected a double-quoted string", s.line, s.source)
        body = lit[1:-1]
        return re.sub(
            r"\\(.)", lambda m: {"n": "\n", "t": "\t", "0": "\0"}.get(m[1], m[1]), body
        )

    def _word_count(self, s: Statement) -> int:
        m = s.mnemonic
        if m == ".word":
            return len(s.operands)
        if m == "li" and len(s.operands) == 2:
            try:
                value = evaluate(s.operands[1], self.symbols, None)
            except Undefined:
                return 2
            return 1 if abs(value) <= half(16) else 2
        if m == "la":
            return 2
        return 1

    # pass two ---------------------------------------------------------------

    def emit(self, stmts: list[Statement]) -> None:
        for s in stmts:
            if s.kind == "none":
                continue
            try:
                if s.kind == "tryte":
                    self._emit_trytes(s)
                else:
                    words = self._words(s)
                    assert len(words) == s.size, (s.mnemonic, words, s.size)
                    for i, w in enumerate(words):
                        self._put_word(s.addr + 3 * i, w, s)
            except (ExprError, I.EncodingError, ValueError, KeyError) as e:
                raise AsmError(str(e), s.line, s.source) from None

    def _put_word(self, addr: int, word: int, s: Statement) -> None:
        lo = low(word, 9)
        rest = split_high(word, 9)
        mid = low(rest, 9)
        for a, t in ((addr - 1, lo), (addr, mid), (addr + 1, split_high(rest, 9))):
            if a in self.image.trytes:
                raise AsmError(f"address {a} assembled twice", s.line, s.source)
            self.image.trytes[a] = t
        self.image.listing.append(Listing(addr, word, s.line, s.source))

    def _emit_trytes(self, s: Statement) -> None:
        m = s.mnemonic
        if m == ".zero":
            values = [0] * s.size
        elif m == ".tryte":
            values = [self._value(op, s.addr) for op in s.operands]
        else:
            values = [ord(c) for c in self._string(s)] + ([0] if m == ".string" else [])
        for i, v in enumerate(values):
            if abs(v) > TRYTE_MAX:
                raise AsmError(f"{v} does not fit a tryte", s.line, s.source)
            a = s.addr + i
            if a in self.image.trytes:
                raise AsmError(f"address {a} assembled twice", s.line, s.source)
            self.image.trytes[a] = v
        self.image.listing.append(Listing(s.addr, None, s.line, s.source))

    def _value(self, text: str, here: int) -> int:
        return evaluate(text, self.symbols, here)

    def _reg(self, text: str) -> int:
        name = text.strip().lower()
        if name not in I.REG_NAMES:
            raise ValueError(f"not a register: {text!r}")
        return I.REG_NAMES[name]

    def _mem(self, text: str, here: int) -> tuple[int, int]:
        """Parse 'imm(reg)' or '(reg)' into (imm, reg)."""
        text = text.strip()
        if not text.endswith(")") or "(" not in text:
            raise ValueError(f"expected imm(reg), got {text!r}")
        open_at = text.rindex("(")
        imm_text = text[:open_at].strip()
        return (self._value(imm_text, here) if imm_text else 0), self._reg(
            text[open_at + 1 : -1]
        )

    def _offset(self, text: str, pc: int) -> int:
        target = self._value(text, pc)
        delta = target - pc
        if delta % 3:
            raise ValueError(f"branch target {target} is not word aligned")
        return delta // 3

    def _table(self, text: str, here: int) -> int:
        name = text.strip().lower()
        return I.TABLES[name] if name in I.TABLES else self._value(text, here)

    def _csr(self, text: str, here: int) -> int:
        name = text.strip().lower()
        return I.CSRS[name] if name in I.CSRS else self._value(text, here)

    def _words(self, s: Statement) -> list[int]:
        m, ops, pc = s.mnemonic, s.operands, s.addr
        if m == ".word":
            return [self._value(op, pc) for op in ops]
        base, suffix = _split_suffix(m)
        pseudo = self._pseudo(base, suffix, ops, pc, s)
        if pseudo is not None:
            return pseudo
        if base not in I.OPS:
            raise ValueError(f"unknown instruction {m!r}")
        return [self._encode(I.OPS[base], suffix, ops, pc, {})]

    def _encode(
        self, op: I.Op, suffix: str, ops: list[str], pc: int, extra: dict
    ) -> int:
        syntax = [k for k in op.syntax if k != "F"]
        fields: dict[str, int] = dict(extra)
        if "F" in op.syntax:
            fields["imm"] = self._fence(ops)
            ops = []
        if len(ops) != len(syntax):
            raise ValueError(
                f"{op.name} takes {len(syntax)} operand(s), got {len(ops)}"
            )
        pos = length = None
        for kind, text in zip(syntax, ops, strict=True):
            if kind == "d":
                fields["rd"] = self._reg(text)
            elif kind == "s":
                fields["rs1"] = self._reg(text)
            elif kind == "t":
                fields["rs2"] = self._reg(text)
            elif kind == "u":
                fields["rs3"] = self._reg(text)
            elif kind == "i":
                fields["imm"] = self._value(text, pc)
            elif kind == "m":
                fields["imm"], fields["rs1"] = self._mem(text, pc)
            elif kind == "a":
                imm, fields["rs1"] = self._mem(text, pc)
                if imm:
                    raise ValueError("atomics take no offset: write (reg)")
            elif kind == "L":
                fields["off" if op.fmt == "B" else "imm"] = self._offset(text, pc)
            elif kind == "N":
                fields["offn"] = self._offset(text, pc)
            elif kind == "P":
                fields["offp"] = self._offset(text, pc)
            elif kind == "x":
                fields["fn"] = self._table(text, pc)
            elif kind == "c":
                fields["imm"] = self._csr(text, pc)
            elif kind == "pos":
                pos = self._value(text, pc)
            elif kind == "len":
                length = self._value(text, pc)
        if pos is not None and length is not None:
            if pos < 0 or length < 1 or pos + length > 27:
                raise ValueError(f"trit field {pos}+{length} is outside the word")
            fields["imm"] = pos + 81 * length
        if suffix:
            if op.atomic and suffix in ORDERINGS:
                fields["rs3"] = ORDERINGS[suffix]
            elif op.fmt == "T" and suffix in HINTS_T:
                fields["hint"] = HINTS_T[suffix]
            elif op.fmt == "B" and suffix in HINTS_B:
                fields["hint"] = HINTS_B[suffix]
            else:
                raise ValueError(f"{op.name} takes no suffix .{suffix}")
        return I.encode(op, **fields)

    def _fence(self, ops: list[str]) -> int:
        if not ops:
            return 40 + 81 * 40  # every set: four + trits in pred and succ
        if len(ops) != 2:
            raise ValueError("fence takes no operands, or pred, succ")

        def trits(text: str) -> int:
            letters = text.strip().lower()
            if not letters or any(c not in FENCE_SETS for c in letters):
                raise ValueError(f"fence set {text!r}: use letters r, w, i, o")
            return sum(3 ** FENCE_SETS[c] for c in set(letters))

        return trits(ops[0]) + 81 * trits(ops[1])

    def _pseudo(
        self, m: str, suffix: str, ops: list[str], pc: int, s: Statement
    ) -> list[int] | None:
        enc = self._encode
        OPS = I.OPS
        if m.startswith("b") and m[1:] in I.CONDS:
            return [enc(OPS["b"], suffix, ops, pc, {"cond": I.CONDS[m[1:]]})]
        if m in BRANCH_ZERO:
            if len(ops) != 2:
                raise ValueError(f"{m} takes reg, label")
            return [
                enc(
                    OPS["b"],
                    suffix,
                    [ops[0], "zero", ops[1]],
                    pc,
                    {"cond": I.CONDS[BRANCH_ZERO[m]]},
                )
            ]
        if suffix:
            return None
        if m == "nop":
            return [enc(OPS["addi"], "", ["zero", "zero", "0"], pc, {})]
        if m == "mv":
            return [enc(OPS["addi"], "", [*ops, "0"], pc, {})]
        if m in ("neg", "not"):
            return [enc(OPS["sub"], "", [ops[0], "zero", ops[1]], pc, {})]
        if m == "sgn":
            return [enc(OPS["cmp"], "", [*ops, "zero"], pc, {})]
        if m == "j":
            return [enc(OPS["jal"], "", ["zero", *ops], pc, {})]
        if m == "call":
            return [enc(OPS["jal"], "", ["ra", *ops], pc, {})]
        if m == "ret":
            if ops:
                raise ValueError("ret takes no operands")
            return [enc(OPS["jalr"], "", ["zero", "0(ra)"], pc, {})]
        if m == "csrr":
            return [enc(OPS["csrrw"], "", [ops[0], ops[1], "zero"], pc, {})]
        if m == "csrw":
            return [enc(OPS["csrrw"], "", ["zero", *ops], pc, {})]
        if m in LOGIC_PSEUDOS:
            return [enc(OPS["tlut"], "", [*ops, m], pc, {})]
        if m == "tabs":
            return [enc(OPS["tlut"], "", [*ops, "zero", "tabs"], pc, {})]
        if m == "tmap":
            if len(ops) != 3:
                raise ValueError("tmap takes rd, rs, f")
            f = self._value(ops[2], pc)
            outs = [low(split_high(f, i), 1) for i in range(3)]  # f(-), f(0), f(+)
            table = I.make_table(lambda a, b: outs[a + 1])
            return [enc(OPS["tlut"], "", [ops[0], ops[1], "zero", str(table)], pc, {})]
        if m == "li":
            if len(ops) != 2:
                raise ValueError("li takes rd, value")
            value = self._value(ops[1], pc)
            if abs(value) > WORD_MAX:
                raise ValueError(f"{value} does not fit a word")
            if s.size == 1:
                return [enc(OPS["addi"], "", [ops[0], "zero", str(value)], pc, {})]
            lo = low(value, 8)
            hi = (value - lo) // 3**8
            return [
                enc(OPS["lui"], "", [ops[0], str(hi)], pc, {}),
                enc(OPS["addi"], "", [ops[0], ops[0], str(lo)], pc, {}),
            ]
        if m == "la":
            if len(ops) != 2:
                raise ValueError("la takes rd, symbol")
            delta = self._value(ops[1], pc) - pc
            lo = low(delta, 8)
            hi = (delta - lo) // 3**8
            return [
                enc(OPS["auipc"], "", [ops[0], str(hi)], pc, {}),
                enc(OPS["addi"], "", [ops[0], ops[0], str(lo)], pc, {}),
            ]
        return None


def assemble(source: str) -> Image:
    asm = Assembler()
    stmts = parse(source)
    asm.layout(stmts)
    asm.emit(stmts)
    asm.image.symbols = dict(asm.symbols)
    for entry in asm.image.listing:
        if entry.word is not None:
            entry.text = disassemble(entry.word, entry.addr)
    return asm.image
