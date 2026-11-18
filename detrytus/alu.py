"""Integer and tritwise operations, as pure functions of word values.

Each function implements one rule of docs/isa.md section 7 and returns
a value already reduced into word range.
"""

from detrytus.trits import (
    WORD_MOD,
    WORD_TRITS,
    field,
    from_trits,
    low,
    pow3,
    sign,
    split_high,
    to_trits,
    wrap,
)


def mul_parts(a: int, b: int) -> tuple[int, int]:
    """(high, low) words of the 54-trit product: a*b = hi*3**27 + lo."""
    p = a * b
    lo = wrap(p)
    return (p - lo) // WORD_MOD, lo


def div_trunc(a: int, b: int) -> tuple[int, int]:
    """C-style division: quotient toward zero, remainder signed like a."""
    if b == 0:
        return 0, a
    q = abs(a) // abs(b) * sign(a) * sign(b)
    return wrap(q), a - q * b


def div_round(a: int, b: int) -> tuple[int, int]:
    """Division rounding to nearest, ties toward zero; |rem| <= |b|/2."""
    if b == 0:
        return 0, a
    q, r = div_trunc(a, b)
    if 2 * abs(r) > abs(b):
        q += sign(r) * sign(b)
        r = a - q * b
    return wrap(q), r


def shift(x: int, n: int) -> int:
    """Shift left by n trits; negative n shifts right, rounding to nearest."""
    if n >= WORD_TRITS or n <= -WORD_TRITS:
        return 0
    if n >= 0:
        return wrap(x * pow3(n))
    return split_high(x, -n)


def rotate(x: int, n: int) -> int:
    """Rotate left by n trits, n taken modulo 27."""
    n %= WORD_TRITS
    if n == 0:
        return x
    keep = WORD_TRITS - n
    return low(x, keep) * pow3(n) + split_high(x, keep)


def trit_extract(x: int, pos: int, length: int) -> int:
    return field(x, pos, length)


def trit_insert(dest: int, src: int, pos: int, length: int) -> int:
    old = field(dest, pos, length)
    return dest + (low(src, length) - old) * pow3(pos)


def leading_zero_trits(x: int) -> int:
    trits = to_trits(x)
    n = 0
    for t in reversed(trits):
        if t:
            break
        n += 1
    return n


def trailing_zero_trits(x: int) -> int:
    if x == 0:
        return WORD_TRITS
    n = 0
    while x % 3 == 0:
        x //= 3
        n += 1
    return n


def nonzero_trits(x: int) -> int:
    return sum(1 for t in to_trits(x) if t)


def tlut(a: int, b: int, table: int) -> int:
    """Apply a 9-trit truth table to each trit pair of a and b."""
    entries = to_trits(table, 9)
    return from_trits(
        [
            entries[3 * (ta + 1) + (tb + 1)]
            for ta, tb in zip(to_trits(a), to_trits(b), strict=True)
        ]
    )


def binary_shift(x: int, n: int) -> int:
    """Shift the two's-complement form left by n bits; negative shifts right.

    A left shift by a huge n is still well defined after wrapping, since
    wrap(x * 2**n) only needs 2**n modulo 3**27.
    """
    if n >= 0:
        return wrap(x * pow(2, n, WORD_MOD))
    return wrap(x >> min(-n, 64))
