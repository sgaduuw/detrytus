"""The two passes: lay out statements and bind labels, then encode.

A statement's size must not change between the passes, so anything whose
size depends on a value (``li``) decides its size in pass one, taking the
long form when the value is not yet known.
"""

import re
from collections.abc import Callable
from dataclasses import dataclass, field
from pathlib import Path

from detrytus import isa
from detrytus.asm import pseudo
from detrytus.asm.errors import AsmError
from detrytus.asm.operands import OperandError, Operands, encode_instruction
from detrytus.asm.source import Statement, parse
from detrytus.disasm import disassemble
from detrytus.expr import ExprError, evaluate
from detrytus.trits import TRYTE_MAX, TRYTE_TRITS, low, split_high

TRYTE_DIRECTIVES = (".tryte", ".string", ".ascii", ".zero")
STRING_ESCAPES = {"n": "\n", "t": "\t", "0": "\0"}


@dataclass
class Listing:
    addr: int
    word: int | None
    line: int
    source: str
    text: str = ""  # disassembly of the emitted word
    file: str | None = None  # where the line came from, for included files
    load: int | None = None  # where it sits in the image, if not at addr


@dataclass
class Image:
    trytes: dict[int, int] = field(default_factory=dict)
    symbols: dict[str, int] = field(default_factory=dict)
    listing: list[Listing] = field(default_factory=list)
    labels: dict[str, int] = field(default_factory=dict)  # addresses, not .equ


def _next_centre(loc: int, unit: int) -> int:
    """The first multiple of unit at which a word can be centred without
    covering anything before loc, the next free tryte: the first c with
    c - 1 >= loc."""
    return -(-(loc + 1) // unit) * unit


def _split_word(word: int) -> tuple[int, int, int]:
    """(low, middle, high) trytes of a word."""
    lo = low(word, TRYTE_TRITS)
    rest = split_high(word, TRYTE_TRITS)
    return lo, low(rest, TRYTE_TRITS), split_high(rest, TRYTE_TRITS)


def _split_suffix(mnemonic: str) -> tuple[str, str]:
    if mnemonic in isa.OPS or mnemonic in pseudo.PSEUDOS:
        return mnemonic, ""
    base, dot, suffix = mnemonic.rpartition(".")
    return (base, suffix) if dot and base else (mnemonic, "")


class Assembler:
    def __init__(self) -> None:
        self.symbols: dict[str, int] = {}
        self.operands = Operands(self.symbols)
        self.image = Image()
        self._loc = -1  # next free tryte, so the first word centres on 0
        self._pending: list[Statement] = []
        # The current section, and its load address minus run address.
        self._section = "text"
        self._shift = 0
        # (next free tryte, shift) of each section other than the current one.
        self._sections: dict[str, tuple[int, int]] = {}

    # --- pass one: layout ----------------------------------------------------

    def layout(self, stmts: list[Statement]) -> None:
        directives: dict[str, Callable[[Statement], None]] = {
            ".equ": self._lay_equ,
            ".set": self._lay_equ,
            ".org": self._lay_org,
            ".align": self._lay_align,
            ".section": self._lay_section,
        }
        for s in stmts:
            if s.labels:
                self._pending.append(s)
            try:
                if not s.mnemonic:
                    s.kind = "none"
                elif s.mnemonic in directives:
                    s.kind = "none"
                    directives[s.mnemonic](s)
                elif s.mnemonic in TRYTE_DIRECTIVES:
                    self._lay_trytes(s)
                else:
                    self._lay_words(s)
            except (ExprError, OperandError) as e:
                raise AsmError(str(e), s.line, s.source, s.file) from None
        self._bind(self._loc)

    def _bind(self, addr: int) -> None:
        for s in self._pending:
            for label in s.labels:
                self._define(label, addr, s)
                self.image.labels[label] = addr
        self._pending.clear()

    def _define(self, name: str, value: int, s: Statement) -> None:
        if name in self.symbols:
            raise AsmError(f"symbol {name!r} defined twice", s.line, s.source, s.file)
        self.symbols[name] = value

    def _expect(self, s: Statement, n: int) -> list[str]:
        if len(s.operands) != n:
            raise AsmError(
                f"{s.mnemonic} takes {n} operand(s)", s.line, s.source, s.file
            )
        return s.operands

    def _constant(self, text: str) -> int:
        return evaluate(text, self.symbols, None)

    def _lay_equ(self, s: Statement) -> None:
        name, value = self._expect(s, 2)
        self._define(name, self._constant(value), s)

    def _lay_org(self, s: Statement) -> None:
        target = self._constant(self._expect(s, 1)[0])
        if low(target, 1) != 0:
            raise AsmError(
                ".org needs a word-aligned address (a multiple of 3)",
                s.line,
                s.source,
                s.file,
            )
        self._loc = target - 1

    def _lay_section(self, s: Statement) -> None:
        """Switch sections. A section is placed once, on first use, and
        resumes where it left off after that."""
        if not 1 <= len(s.operands) <= 3:
            raise AsmError(
                ".section takes NAME, or NAME, LOAD[, RUN]", s.line, s.source, s.file
            )
        name = s.operands[0].strip()
        if name == self._section and len(s.operands) == 1:
            return
        known = name == self._section or name in self._sections
        if len(s.operands) > 1:
            if known:
                raise AsmError(
                    f"section {name!r} is already placed", s.line, s.source, s.file
                )
            start, shift = self._place(s)
        elif not known:
            raise AsmError(
                f"section {name!r} needs an address the first time: "
                f".section {name}, ADDR",
                s.line,
                s.source,
                s.file,
            )
        else:
            start, shift = self._sections[name]
        # Labels written before the switch name the end of the old section.
        self._bind(self._loc)
        self._sections[self._section] = (self._loc, self._shift)
        self._section, self._loc, self._shift = name, start, shift
        self._sections.pop(name, None)

    def _place(self, s: Statement) -> tuple[int, int]:
        """(first free tryte, shift) for a new section's LOAD and RUN.

        Layout, labels and pc-relative encoding all use the run address,
        where the code executes; only emission adds the shift. They must
        differ by a multiple of 3 so words are aligned in both places.
        """
        load = self._constant(s.operands[1])
        run = self._constant(s.operands[2]) if len(s.operands) == 3 else load
        if low(run, 1) != 0:
            raise AsmError(
                "a section starts at a word-aligned address (a multiple of 3)",
                s.line,
                s.source,
                s.file,
            )
        if (load - run) % 3:
            raise AsmError(
                "a section's load and run addresses must differ by a multiple of 3",
                s.line,
                s.source,
                s.file,
            )
        return run - 1, load - run

    def _lay_align(self, s: Statement) -> None:
        unit = 3 ** self._constant(self._expect(s, 1)[0])
        self._loc = _next_centre(self._loc, unit) - 1

    def _lay_trytes(self, s: Statement) -> None:
        s.kind = "tryte"
        s.shift = self._shift
        s.addr = self._loc
        s.size = self._tryte_count(s)
        self._bind(s.addr)
        self._loc += s.size

    def _lay_words(self, s: Statement) -> None:
        s.kind = "word"
        s.shift = self._shift
        if s.mnemonic == ".word":
            s.size = len(s.operands)
        elif s.mnemonic == ".words":
            s.size = self._constant(self._expect(s, 1)[0])
            if s.size < 0:
                raise AsmError(
                    ".words needs a count of 0 or more", s.line, s.source, s.file
                )
        else:
            s.size = pseudo.size(_split_suffix(s.mnemonic)[0], s.operands, self.symbols)
        s.addr = _next_centre(self._loc, 3)
        self._bind(s.addr)
        self._loc = s.addr + 3 * s.size - 1

    def _tryte_count(self, s: Statement) -> int:
        if s.mnemonic == ".zero":
            return self._constant(self._expect(s, 1)[0])
        if s.mnemonic == ".tryte":
            return len(s.operands)
        return len(self._string(s)) + (s.mnemonic == ".string")

    def _string(self, s: Statement) -> str:
        (lit,) = self._expect(s, 1)
        if len(lit) < 2 or lit[0] != '"' or lit[-1] != '"':
            raise AsmError("expected a double-quoted string", s.line, s.source, s.file)
        return re.sub(r"\\(.)", lambda m: STRING_ESCAPES.get(m[1], m[1]), lit[1:-1])

    # --- pass two: emit ------------------------------------------------------

    def emit(self, stmts: list[Statement]) -> None:
        for s in stmts:
            try:
                if s.kind == "tryte":
                    self._emit_trytes(s)
                elif s.kind == "word":
                    self._emit_words(s)
            except (ExprError, OperandError, isa.EncodingError) as e:
                raise AsmError(str(e), s.line, s.source, s.file) from None

    def _put(self, addr: int, tryte: int, s: Statement) -> None:
        """Place a tryte at its load address: addr (a run address) + shift."""
        addr += s.shift
        if addr in self.image.trytes:
            raise AsmError(f"address {addr} assembled twice", s.line, s.source, s.file)
        self.image.trytes[addr] = tryte

    def _emit_words(self, s: Statement) -> None:
        words = self._words(s)
        if len(words) != s.size:
            raise AsmError(
                f"internal error: {s.mnemonic} sized {s.size} in pass one "
                f"but {len(words)} in pass two",
                s.line,
                s.source,
            )
        for i, word in enumerate(words):
            addr = s.addr + 3 * i
            for offset, tryte in zip((-1, 0, 1), _split_word(word), strict=True):
                self._put(addr + offset, tryte, s)
            self.image.listing.append(
                Listing(addr, word, s.line, s.source, file=s.file, load=addr + s.shift)
            )

    def _emit_trytes(self, s: Statement) -> None:
        if s.mnemonic == ".zero":
            values = [0] * s.size
        elif s.mnemonic == ".tryte":
            values = [self.operands.value(op, s.addr) for op in s.operands]
        else:
            values = [ord(c) for c in self._string(s)]
            values += [0] if s.mnemonic == ".string" else []
        for i, v in enumerate(values):
            if abs(v) > TRYTE_MAX:
                raise AsmError(f"{v} does not fit a tryte", s.line, s.source, s.file)
            self._put(s.addr + i, v, s)
        self.image.listing.append(
            Listing(s.addr, None, s.line, s.source, file=s.file, load=s.addr + s.shift)
        )

    def _words(self, s: Statement) -> list[int]:
        pc = s.addr
        if s.mnemonic == ".word":
            return [self.operands.value(op, pc) for op in s.operands]
        if s.mnemonic == ".words":
            return [0] * s.size
        name, suffix = _split_suffix(s.mnemonic)
        if name in pseudo.PSEUDOS:
            expansions = pseudo.PSEUDOS[name](self.operands, s.operands, pc, s.size)
        elif name in isa.OPS:
            expansions = [(name, s.operands, {})]
        else:
            raise OperandError(f"unknown instruction {s.mnemonic!r}")
        return [
            encode_instruction(
                self.operands, isa.OPS[op], suffix, texts, pc + 3 * i, extra
            )
            for i, (op, texts, extra) in enumerate(expansions)
        ]


def assemble(source: str, file: str | None = None) -> Image:
    """Assemble source text. file, if given, names it in error messages
    and is where .include paths start from."""
    asm = Assembler()
    stmts = parse(source, file)
    asm.layout(stmts)
    asm.emit(stmts)
    asm.image.symbols = dict(asm.symbols)
    for entry in asm.image.listing:
        if entry.word is not None:
            entry.text = disassemble(entry.word, entry.addr)
    return asm.image


def assemble_file(path: str | Path) -> Image:
    return assemble(Path(path).read_text(encoding="utf-8"), str(path))
