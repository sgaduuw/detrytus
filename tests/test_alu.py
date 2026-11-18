import pytest

from detrytus import alu
from detrytus.trits import WORD_MAX, WORD_MOD, parse_trits


@pytest.mark.parametrize(
    ("a", "b", "q", "r"),
    [(7, 2, 3, 1), (-7, 2, -3, -1), (7, -2, -3, 1), (-7, -2, 3, -1), (5, 0, 0, 5)],
)
def test_div_truncates_toward_zero(a, b, q, r):
    assert alu.div_trunc(a, b) == (q, r)


@pytest.mark.parametrize(
    ("a", "b", "q", "r"),
    [
        (7, 3, 2, 1),
        (8, 3, 3, -1),
        (-8, 3, -3, 1),
        (7, 2, 3, 1),
        (5, 2, 2, 1),
        (5, 0, 0, 5),
    ],
)
def test_divr_rounds_to_nearest_ties_toward_zero(a, b, q, r):
    assert alu.div_round(a, b) == (q, r)
    assert a == q * b + r


def test_division_cannot_overflow():
    # The range is symmetric, so the binary INT_MIN / -1 case has no analogue.
    assert alu.div_trunc(-WORD_MAX, -1) == (WORD_MAX, 0)


def test_mul_parts_reconstruct_the_product():
    a, b = WORD_MAX, -WORD_MAX + 12345
    hi, lo = alu.mul_parts(a, b)
    assert hi * WORD_MOD + lo == a * b
    assert abs(hi) <= WORD_MAX and abs(lo) <= WORD_MAX


def test_right_shift_rounds_to_nearest():
    assert alu.shift(4, -1) == 1  # 4/3 = 1.33
    assert alu.shift(5, -1) == 2  # 5/3 = 1.67
    assert alu.shift(-5, -1) == -2
    assert alu.shift(7, 30) == 0
    assert alu.shift(1, 3) == 27


def test_rotate_moves_top_trits_to_the_bottom():
    x = parse_trits("+" + "0" * 25 + "-")
    assert alu.rotate(x, 1) == parse_trits("0" * 25 + "-+")
    assert alu.rotate(x, 27) == x
    assert alu.rotate(alu.rotate(x, 5), -5) == x


def test_trit_counts():
    assert alu.leading_zero_trits(0) == 27
    assert alu.leading_zero_trits(1) == 26
    assert alu.trailing_zero_trits(54) == 3  # 54 = 2 * 27
    assert alu.trailing_zero_trits(0) == 27
    assert alu.nonzero_trits(parse_trits("+0-0+")) == 3


def test_trit_insert_replaces_only_the_field():
    dest = parse_trits("++++++")
    out = alu.trit_insert(dest, parse_trits("-0"), 2, 2)
    assert out == parse_trits("++-0++")


def test_binary_ops_and_huge_shifts_stay_in_range():
    assert alu.binary_shift(1, 10) == 1024
    assert alu.binary_shift(-1024, -3) == -128
    big = alu.binary_shift(1, 10**12)  # must not try to build 2**(10**12)
    assert big != 0
    assert abs(big) <= WORD_MAX
    assert (big - pow(2, 10**12, WORD_MOD)) % WORD_MOD == 0
