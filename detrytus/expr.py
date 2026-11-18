"""Assembler expressions: integers, symbols, and + - * / % with parentheses.

Number forms: decimal (42), balanced ternary (0t+0-), hexadecimal
(0x2a) and character literals ('a'). '.' is the current address.
"""

import re

from .trits import parse_trits


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


def _tokens(text: str) -> list[tuple[str, str]]:
    out = []
    pos = 0
    text = text.rstrip()
    while pos < len(text):
        m = _TOKEN.match(text, pos)
        if not m or m.end() == pos:
            raise ExprError(f"cannot parse {text[pos:]!r}")
        kind = m.lastgroup
        assert kind is not None
        out.append((kind, m.group(kind)))
        pos = m.end()
    return out


def evaluate(text: str, symbols: dict[str, int], here: int | None = None) -> int:
    toks = _tokens(text)
    if not toks:
        raise ExprError("empty expression")
    pos = 0

    def peek() -> tuple[str, str] | None:
        return toks[pos] if pos < len(toks) else None

    def take() -> tuple[str, str]:
        nonlocal pos
        tok = toks[pos]
        pos += 1
        return tok

    def atom() -> int:
        tok = peek()
        if tok is None:
            raise ExprError(f"unexpected end of {text!r}")
        kind, value = take()
        if kind == "op" and value == "(":
            v = sum_()
            if peek() != ("op", ")"):
                raise ExprError(f"missing ')' in {text!r}")
            take()
            return v
        if kind == "op" and value in "+-":
            v = atom()
            return -v if value == "-" else v
        if kind == "tern":
            return parse_trits(value[2:])
        if kind == "hex":
            return int(value, 16)
        if kind == "dec":
            return int(value)
        if kind == "char":
            return ord(_ESCAPES.get(value[1], value[1]) if value[0] == "\\" else value)
        if kind == "name":
            if value == ".":
                if here is None:
                    raise Undefined("'.' has no address here")
                return here
            if value not in symbols:
                raise Undefined(f"undefined symbol {value!r}")
            return symbols[value]
        raise ExprError(f"unexpected {value!r} in {text!r}")

    def product() -> int:
        v = atom()
        while (tok := peek()) is not None and tok[0] == "op" and tok[1] in "*/%":
            op = take()[1]
            rhs = atom()
            if op == "*":
                v *= rhs
            else:
                if rhs == 0:
                    raise ExprError(f"division by zero in {text!r}")
                q = abs(v) // abs(rhs) * (1 if (v < 0) == (rhs < 0) else -1)
                v = q if op == "/" else v - q * rhs
        return v

    def sum_() -> int:
        v = product()
        while (tok := peek()) is not None and tok[0] == "op" and tok[1] in "+-":
            op = take()[1]
            rhs = product()
            v = v + rhs if op == "+" else v - rhs
        return v

    v = sum_()
    if pos != len(toks):
        raise ExprError(f"trailing input in {text!r}")
    return v
