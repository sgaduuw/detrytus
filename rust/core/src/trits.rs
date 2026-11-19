//! Balanced-ternary arithmetic on `i64`.
//!
//! As in the Python core, a value is a plain integer in its balanced
//! range, not a list of trits: every field operation is modular
//! arithmetic. A word (27 trits, about ±3.8 × 10¹²) fits easily, and a
//! full 54-trit product fits in `i128`.

pub const WORD_TRITS: u32 = 27;
pub const TRYTE_TRITS: u32 = 9;

/// 3ⁿ for 0 ≤ n ≤ 39.
pub const fn pow3(n: u32) -> i64 {
    3i64.pow(n)
}

/// The largest value an n-trit field holds, (3ⁿ − 1) / 2.
pub const fn half(n: u32) -> i64 {
    (pow3(n) - 1) / 2
}

pub const WORD_MAX: i64 = half(WORD_TRITS);
pub const TRYTE_MAX: i64 = half(TRYTE_TRITS);
pub const WORD_MOD: i64 = pow3(WORD_TRITS);
pub const TRYTE_MOD: i64 = pow3(TRYTE_TRITS);

/// The low n trits of x, read as a balanced value. This is also
/// wrap-around to an n-trit field.
pub const fn low(x: i64, n: u32) -> i64 {
    let m = pow3(n);
    let h = (m - 1) / 2;
    (x + h).rem_euclid(m) - h
}

/// x reduced into the 27-trit word range.
pub const fn wrap(x: i64) -> i64 {
    low(x, WORD_TRITS)
}

/// A wide value, such as a product, reduced into the word range.
pub const fn wrap_wide(x: i128) -> i64 {
    let m = WORD_MOD as i128;
    let h = (m - 1) / 2;
    ((x + h).rem_euclid(m) - h) as i64
}

/// x with its low n trits cut off: the exact q in x = q·3ⁿ + low(x, n).
pub const fn split_high(x: i64, n: u32) -> i64 {
    (x - low(x, n)) / pow3(n)
}

/// Trits lo .. lo+n−1 of x, read as a balanced value.
pub const fn field(x: i64, lo: u32, n: u32) -> i64 {
    low(split_high(x, lo), n)
}

/// x with trit i replaced by value.
pub const fn set_trit(x: i64, i: u32, value: i64) -> i64 {
    x + (value - field(x, i, 1)) * pow3(i)
}

/// The low n trits of x, least significant first.
pub fn to_trits(mut x: i64, n: u32) -> Vec<i64> {
    (0..n)
        .map(|_| {
            let t = low(x, 1);
            x = (x - t) / 3;
            t
        })
        .collect()
}

/// Inverse of [`to_trits`].
pub fn from_trits(trits: &[i64]) -> i64 {
    trits.iter().rev().fold(0, |v, &t| v * 3 + t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn low_is_balanced_reduction() {
        assert_eq!(low(5, 1), -1); // 5 = 2·3 − 1
        assert_eq!(low(-5, 1), 1);
        assert_eq!(low(13, 2), 4);
        assert_eq!(low(14, 2), -4);
        assert_eq!(wrap(WORD_MAX + 1), -WORD_MAX);
    }

    #[test]
    fn fields_split_a_value_exactly() {
        let x = 1_936_964_667; // trytes 5, −7, 3 from high to low
        assert_eq!(low(x, 9), 3);
        assert_eq!(field(x, 9, 9), -7);
        assert_eq!(split_high(x, 18), 5);
        assert_eq!(set_trit(0, 3, -1), -27);
    }

    #[test]
    fn wide_values_wrap_like_narrow_ones() {
        let p = WORD_MAX as i128 * (WORD_MAX - 5) as i128;
        let narrow = wrap_wide(p);
        assert_eq!((p - narrow as i128) % WORD_MOD as i128, 0);
        assert!(narrow.abs() <= WORD_MAX);
        assert_eq!(wrap_wide(-7), -7);
    }

    #[test]
    fn trits_round_trip() {
        for x in [-WORD_MAX, -9842, -1, 0, 1, 22, WORD_MAX] {
            assert_eq!(from_trits(&to_trits(x, WORD_TRITS)), x);
        }
        assert_eq!(to_trits(22, 4), vec![1, 1, -1, 1]); // +-++
    }
}
