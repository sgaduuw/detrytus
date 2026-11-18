"""Balanced-ternary arithmetic on Python integers.

A value is kept as a plain int in its balanced range rather than as a
list of trits: Python ints already have exact arithmetic, and every
field operation reduces to modular arithmetic on them. Trit lists are
built only where an operation is genuinely tritwise.
"""

WORD_TRITS = 27
TRYTE_TRITS = 9


def half(n: int) -> int:
    """Largest value an n-trit field holds, (3**n - 1) / 2."""
    return (3**n - 1) // 2


WORD_MAX = half(WORD_TRITS)
TRYTE_MAX = half(TRYTE_TRITS)
WORD_MOD = 3**WORD_TRITS


def low(x: int, n: int) -> int:
    """The low n trits of x, read as a balanced value.

    This is also wrap-around to an n-trit field, since keeping the low
    trits is reduction modulo 3**n onto the balanced representative.
    """
    m = 3**n
    h = (m - 1) // 2
    return (x + h) % m - h


def wrap(x: int) -> int:
    """Reduce x into the 27-trit word range."""
    return low(x, WORD_TRITS)


def field(x: int, lo: int, n: int) -> int:
    """Trits lo .. lo+n-1 of x, read as a balanced value."""
    return low(split_high(x, lo), n)


def split_high(x: int, n: int) -> int:
    """x with its low n trits cut off: the exact q in x = q*3**n + low(x, n)."""
    return (x - low(x, n)) // 3**n


def sign(x: int) -> int:
    return (x > 0) - (x < 0)


def to_trits(x: int, n: int = WORD_TRITS) -> list[int]:
    """The low n trits of x, least significant first."""
    out = []
    for _ in range(n):
        t = low(x, 1)
        out.append(t)
        x = (x - t) // 3
    return out


def from_trits(trits: list[int]) -> int:
    """Inverse of to_trits: trits least significant first."""
    v = 0
    for t in reversed(trits):
        v = v * 3 + t
    return v


_CHARS = {-1: "-", 0: "0", 1: "+"}
_VALUES = {"-": -1, "0": 0, "+": 1}


def format_trits(x: int, n: int = WORD_TRITS) -> str:
    """x as an n-trit literal body, most significant first: 8 -> '0...+0-'."""
    return "".join(_CHARS[t] for t in reversed(to_trits(x, n)))


def parse_trits(digits: str) -> int:
    """Value of a trit string such as '+0-', most significant first."""
    if not digits or any(c not in _VALUES for c in digits):
        raise ValueError(f"not a balanced-ternary literal: {digits!r}")
    return from_trits([_VALUES[c] for c in reversed(digits)])
