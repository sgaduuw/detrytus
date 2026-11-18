import pytest

from detrytus.trits import (
    WORD_MAX,
    field,
    format_trits,
    from_trits,
    low,
    parse_trits,
    split_high,
    to_trits,
    wrap,
)


@pytest.mark.parametrize("x", [0, 1, -1, 8, -8, 9841, -9841, WORD_MAX, -WORD_MAX])
def test_trits_round_trip(x):
    assert from_trits(to_trits(x)) == x
    assert parse_trits(format_trits(x)) == x


def test_literal_reads_most_significant_first():
    assert parse_trits("+0-") == 8
    assert format_trits(8, 3) == "+0-"


def test_wrap_is_symmetric_and_modular():
    assert wrap(WORD_MAX + 1) == -WORD_MAX
    assert wrap(-WORD_MAX - 1) == WORD_MAX
    assert wrap(3**27) == 0


def test_split_is_exact_and_carry_free():
    # Any integer is hi*3**n + lo with lo balanced: no fix-up needed.
    for x in (-1000, -14, -13, 13, 14, 1000, 123456789):
        lo = low(x, 3)
        assert abs(lo) <= 13
        assert split_high(x, 3) * 27 + lo == x


def test_field_reads_signed_middle_trits():
    x = parse_trits("+-0+0")  # trits 4..0
    assert field(x, 1, 3) == parse_trits("-0+")
