"""Assembler expressions: integers, symbols, and + - * / % with parentheses.

Number forms: decimal (42), balanced ternary (0t+0-), hexadecimal
(0x2a) and character literals ('a'). '.' is the current address.
"""

import re
from collections.abc import Callable

from detrytus.trits import parse_trits


class ExprError(ValueError):
    pass


class Undefined(ExprError):
    """A symbol the expression needs is not defined (yet)."""


_TOKEN = re.compile(
    r"""
    \s*(?:
      (?P<tern>0t[-0+]+)
    | (?P<hex>0x[0-9a-fA-F]+)
    | (?P<dec>\d+)
    | '(?P<char>[^'\\]|\\.)'
    | (?P<name>[A-Za-z_.][A-Za-z0-9_.$]*)
    | (?P<op>[-+*/%()])
    )""",
    re.VERBOSE,
)

_ESCAPES = {"n": "\n", "t": "\t", "0": "\0", "\\": "\\", "'": "'", '"': '"'}

Token = tuple[str, str]


def _tokens(text: str) -> list[Token]:
    out = []
    pos = 0
    text = text.rstrip()
    while pos < len(text):
        m = _TOKEN.match(text, pos)
        if not m or m.end() == pos or m.lastgroup is None:
            raise ExprError(f"cannot parse {text[pos:]!r}")
        out.append((m.lastgroup, m.group(m.lastgroup)))
        pos = m.end()
    return out


def _char(value: str) -> int:
    return ord(_ESCAPES.get(value[1], value[1]) if value[0] == "\\" else value)


def _truncating_divide(a: int, b: int) -> int:
    q = abs(a) // abs(b)
    return q if (a < 0) == (b < 0) else -q


_LITERALS: dict[str, Callable[[str], int]] = {
    "tern": lambda v: parse_trits(v[2:]),
    "hex": lambda v: int(v, 16),
    "dec": int,
    "char": _char,
}


class _Parser:
    """Recursive descent: sum := product (('+'|'-') product)*, and so on."""

    def __init__(self, text: str, symbols: dict[str, int], here: int | None) -> None:
        self.text = text
        self.toks = _tokens(text)
        self.pos = 0
        self.symbols = symbols
        self.here = here

    def peek_op(self, ops: str) -> str | None:
        if self.pos < len(self.toks):
            kind, value = self.toks[self.pos]
            if kind == "op" and value in ops:
                return value
        return None

    def take(self) -> Token:
        if self.pos >= len(self.toks):
            raise ExprError(f"unexpected end of {self.text!r}")
        tok = self.toks[self.pos]
        self.pos += 1
        return tok

    def parse(self) -> int:
        if not self.toks:
            raise ExprError("empty expression")
        v = self.sum()
        if self.pos != len(self.toks):
            raise ExprError(f"trailing input in {self.text!r}")
        return v

    def sum(self) -> int:
        v = self.product()
        while op := self.peek_op("+-"):
            self.take()
            rhs = self.product()
            v = v + rhs if op == "+" else v - rhs
        return v

    def product(self) -> int:
        v = self.atom()
        while op := self.peek_op("*/%"):
            self.take()
            rhs = self.atom()
            if op == "*":
                v *= rhs
                continue
            if rhs == 0:
                raise ExprError(f"division by zero in {self.text!r}")
            q = _truncating_divide(v, rhs)
            v = q if op == "/" else v - q * rhs
        return v

    def atom(self) -> int:
        kind, value = self.take()
        if kind in _LITERALS:
            return _LITERALS[kind](value)
        if kind == "name":
            return self.symbol(value)
        if value == "(":
            v = self.sum()
            if self.peek_op(")") is None:
                raise ExprError(f"missing ')' in {self.text!r}")
            self.take()
            return v
        if value in "+-":
            v = self.atom()
            return -v if value == "-" else v
        raise ExprError(f"unexpected {value!r} in {self.text!r}")

    def symbol(self, name: str) -> int:
        if name == ".":
            if self.here is None:
                raise Undefined("'.' has no address here")
            return self.here
        if name not in self.symbols:
            raise Undefined(f"undefined symbol {name!r}")
        return self.symbols[name]


def evaluate(text: str, symbols: dict[str, int], here: int | None = None) -> int:
    return _Parser(text, symbols, here).parse()
