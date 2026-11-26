//! Integer and tritwise operations, as pure functions of word values.
//! Each implements one rule of docs/isa.md section 7 and returns a value
//! already in word range. Ported from the original Python tools.

use crate::trits::{WORD_MAX, WORD_MOD, WORD_TRITS, field, low, pow3, split_high, wrap, wrap_wide};

/// (high, low) words of the 54-trit product: a·b = hi·3²⁷ + lo.
pub fn mul_parts(a: i64, b: i64) -> (i64, i64) {
    let p = a as i128 * b as i128;
    let lo = wrap_wide(p);
    (((p - lo as i128) / WORD_MOD as i128) as i64, lo)
}

/// C-style division: quotient toward zero, remainder signed like a.
/// Division by zero gives (0, a).
pub fn div_trunc(a: i64, b: i64) -> (i64, i64) {
    if b == 0 {
        return (0, a);
    }
    let q = a / b;
    (wrap(q), a - q * b)
}

/// Division rounding to nearest, ties toward zero; |rem| ≤ |b|/2.
pub fn div_round(a: i64, b: i64) -> (i64, i64) {
    if b == 0 {
        return (0, a);
    }
    let (mut q, mut r) = div_trunc(a, b);
    if 2 * r.abs() > b.abs() {
        q += r.signum() * b.signum();
        r = a - q * b;
    }
    (wrap(q), r)
}

/// Shift left by n trits; a negative n shifts right, rounding to nearest.
pub fn shift(x: i64, n: i64) -> i64 {
    let w = WORD_TRITS as i64;
    if n >= w || n <= -w {
        0
    } else if n >= 0 {
        wrap_wide(x as i128 * pow3(n as u32) as i128)
    } else {
        split_high(x, (-n) as u32)
    }
}

/// Rotate left by n trits, n taken modulo 27.
pub fn rotate(x: i64, n: i64) -> i64 {
    let n = n.rem_euclid(WORD_TRITS as i64) as u32;
    if n == 0 {
        return x;
    }
    let keep = WORD_TRITS - n;
    low(x, keep) * pow3(n) + split_high(x, keep)
}

pub fn trit_insert(dest: i64, src: i64, pos: u32, len: u32) -> i64 {
    dest + (low(src, len) - field(dest, pos, len)) * pow3(pos)
}

pub fn leading_zero_trits(x: i64) -> i64 {
    (0..WORD_TRITS).rev().take_while(|&i| field(x, i, 1) == 0).count() as i64
}

pub fn trailing_zero_trits(mut x: i64) -> i64 {
    if x == 0 {
        return WORD_TRITS as i64;
    }
    let mut n = 0;
    while x % 3 == 0 {
        x /= 3;
        n += 1;
    }
    n
}

pub fn nonzero_trits(x: i64) -> i64 {
    (0..WORD_TRITS).filter(|&i| field(x, i, 1) != 0).count() as i64
}

/// Apply a 9-trit truth table to each trit pair of a and b.
pub fn tlut(a: i64, b: i64, table: i64) -> i64 {
    // The table's trits, read once, plus one: entry 3(a+1) + (b+1).
    let mut entry = [0u64; 9];
    let mut t = table;
    for e in &mut entry {
        let trit = low(t, 1);
        *e = (trit + 1) as u64;
        t = (t - trit) / 3;
    }
    // Each operand's trits plus one are its digits in plain base 3
    // once WORD_MAX is added; the result is built the same way.
    let (mut a, mut b) = ((a + WORD_MAX) as u64, (b + WORD_MAX) as u64);
    let (mut out, mut place) = (0u64, 1u64);
    for _ in 0..WORD_TRITS {
        out += entry[(3 * (a % 3) + b % 3) as usize] * place;
        a /= 3;
        b /= 3;
        place *= 3;
    }
    out as i64 - WORD_MAX
}

/// 2ⁿ modulo 3²⁷, for a left binary shift by a large n.
fn pow2_mod(mut n: i64) -> i128 {
    let m = WORD_MOD as i128;
    let (mut base, mut out) = (2i128, 1i128);
    while n > 0 {
        if n & 1 == 1 {
            out = out * base % m;
        }
        base = base * base % m;
        n >>= 1;
    }
    out
}

/// Shift the two's-complement form left by n bits; negative n shifts
/// right. A left shift by a huge n is still defined after wrapping,
/// since wrap(x·2ⁿ) only needs 2ⁿ modulo 3²⁷.
pub fn binary_shift(x: i64, n: i64) -> i64 {
    if n >= 0 {
        wrap_wide(x as i128 * pow2_mod(n))
    } else {
        // Any shift of 63 or more leaves only the sign, as in Python.
        wrap(x >> (-n).min(63))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trits::WORD_MAX;

    #[test]
    fn division_rounds_as_specified() {
        assert_eq!(div_trunc(-7, 2), (-3, -1));
        assert_eq!(div_round(8, 3), (3, -1));
        assert_eq!(div_round(-8, 3), (-3, 1));
        assert_eq!(div_trunc(5, 0), (0, 5));
    }

    #[test]
    fn mulh_is_the_high_word() {
        let (hi, lo) = mul_parts(WORD_MAX, WORD_MAX - 5);
        assert_eq!(hi, 1_906_399_371_244);
        assert_eq!(
            hi as i128 * WORD_MOD as i128 + lo as i128,
            WORD_MAX as i128 * (WORD_MAX - 5) as i128
        );
    }

    #[test]
    fn shifts_and_rotations() {
        assert_eq!(shift(1, 3), 27);
        assert_eq!(shift(41, -1), 14);
        assert_eq!(shift(5, 27), 0);
        assert_eq!(rotate(1, -1), pow3(26)); // trit 0 comes round to trit 26
        assert_eq!(rotate(rotate(12345, 5), -5), 12345);
    }

    #[test]
    fn trit_counts() {
        assert_eq!(trailing_zero_trits(54), 3);
        assert_eq!(nonzero_trits(81 - 9 + 1), 3);
        assert_eq!(leading_zero_trits(1), 26);
        assert_eq!(leading_zero_trits(0), 27);
    }

    #[test]
    fn tlut_matches_its_definition() {
        // Trit by trit, as docs/isa.md defines it.
        fn reference(a: i64, b: i64, table: i64) -> i64 {
            (0..WORD_TRITS)
                .map(|i| {
                    let (ta, tb) = (field(a, i, 1), field(b, i, 1));
                    field(table, (3 * (ta + 1) + tb + 1) as u32, 1) * pow3(i)
                })
                .sum()
        }
        let mut x: i64 = 1958;
        let mut next = || {
            x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (x >> 20) % WORD_MAX
        };
        for _ in 0..20_000 {
            let (a, b, t) = (next(), next(), low(next(), 9));
            assert_eq!(tlut(a, b, t), reference(a, b, t), "{a} {b} {t}");
        }
        for (a, b) in [(WORD_MAX, -WORD_MAX), (-WORD_MAX, WORD_MAX), (0, 0)] {
            for t in [-9841, 9841, 5792, 0] {
                assert_eq!(tlut(a, b, t), reference(a, b, t));
            }
        }
    }

    #[test]
    fn tlut_applies_tmin() {
        let tmin = 5792; // docs/isa.md 7.5
        // 0t+0-+ and 0t+++- give 0t+0--
        assert_eq!(tlut(25, 38, tmin), 23);
    }

    #[test]
    fn binary_shift_matches_twos_complement() {
        assert_eq!(binary_shift(3, 2), 12);
        assert_eq!(binary_shift(-8, -2), -2);
        assert_eq!(binary_shift(-1, -100), -1);
    }
}
