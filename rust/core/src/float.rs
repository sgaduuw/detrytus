//! The ternary floating point unit (docs/isa.md 7.11), as pure functions
//! of word values that also report the `fcsr` flags they raise.
//!
//! A float is `E = x[26:21]`, `M = x[20:0]`, worth `M · 3^(E − 20)`, so
//! every finite value is an integer significand times a power of three.
//! Each operation works out its exact result as such a pair, or as one
//! wide enough that the trits it cannot hold only decide the direction
//! of rounding, and [`round`] rounds that once. Round to nearest is
//! truncation of balanced trits; the directed modes step one place from
//! there when the dropped part points their way.

use crate::trits::{WORD_MAX, field, low, pow3};

/// Significand trits.
const M_TRITS: u32 = 21;
/// The largest significand, (3²¹ − 1) / 2.
const M_MAX: i128 = (3i128.pow(M_TRITS) - 1) / 2;
/// The smallest normalised significand, (3²⁰ + 1) / 2: `M[20] ≠ 0`.
const M_NORMAL: i128 = (3i128.pow(M_TRITS - 1) + 1) / 2;
/// The exponent field's reserved ends.
const E_SPECIAL: i64 = 364;
const E_MAX: i64 = 363;
/// The weight of the lowest significand trit of the smallest floats:
/// subnormals count in units of 3^(−363 − 20).
const Q_MIN: i64 = -E_MAX - 20;
const INF_M: i64 = pow3(M_TRITS - 1);

/// The default NaN, which every operation that makes a NaN returns:
/// `E = +364`, `M = 0`.
pub const NAN: i64 = E_SPECIAL * pow3(M_TRITS);
pub const INF: i64 = E_SPECIAL * pow3(M_TRITS) + INF_M;
pub const NEG_INF: i64 = E_SPECIAL * pow3(M_TRITS) - INF_M;

/// `fcsr` flags, by trit (8.2).
pub const INVALID: u8 = 1 << 4;
pub const DIV_ZERO: u8 = 1 << 3;
pub const OVERFLOW: u8 = 1 << 2;
pub const UNDERFLOW: u8 = 1 << 1;
pub const INEXACT: u8 = 1;

/// A rounding mode, as `fn[1:0]` and `fcsr` trits 6..5 hold it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rounding {
    Nearest,
    Up,
    Down,
    TowardZero,
}

/// The `fn[1:0]` value that asks for the dynamic mode in `fcsr`.
pub const DYNAMIC: i64 = -3;

impl Rounding {
    /// A static mode from its encoding; None for dynamic or reserved.
    pub fn from_code(code: i64) -> Option<Rounding> {
        Some(match code {
            0 => Rounding::Nearest,
            1 => Rounding::Up,
            -1 => Rounding::Down,
            3 => Rounding::TowardZero,
            _ => return None,
        })
    }
}

/// A float word, taken apart.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum F {
    Nan,
    Inf(i64),
    /// Worth `m · 3^e`.
    Fin(i128, i64),
}

fn unpack(x: i64) -> F {
    let (e, m) = (field(x, M_TRITS, 6), low(x, M_TRITS));
    match e {
        E_SPECIAL if m.abs() == INF_M => F::Inf(m.signum()),
        E_SPECIAL => F::Nan,
        _ if e == -E_SPECIAL => F::Fin(m as i128, Q_MIN),
        _ => F::Fin(m as i128, e - 20),
    }
}

/// The word for `m · 3^q`, where m is normalised at q (its top trit is
/// trit 20) or q is [`Q_MIN`]. Too large is infinity.
fn pack(m: i128, q: i64) -> i64 {
    if m == 0 {
        return 0;
    }
    let e = if q == Q_MIN && m.abs() < M_NORMAL { -E_SPECIAL } else { q + 20 };
    if e > E_MAX {
        return if m > 0 { INF } else { NEG_INF };
    }
    e * pow3(M_TRITS) + m as i64
}

fn pow3w(n: u32) -> i128 {
    3i128.pow(n)
}

/// The low n trits of x, balanced.
fn low_w(x: i128, n: u32) -> i128 {
    let m = pow3w(n);
    (x + (m - 1) / 2).rem_euclid(m) - (m - 1) / 2
}

/// How many trits x needs: 0 for zero.
fn len(x: i128) -> u32 {
    let mut n = 0;
    while x.abs() > (pow3w(n) - 1) / 2 {
        n += 1;
    }
    n
}

/// The exact value `s · 3^e`, plus, when `sticky` is not 0, a part
/// below the lowest trit of s with that sign: less than half a unit of
/// it, or exactly half when `half` is set.
#[derive(Clone, Copy, Debug)]
struct Exact {
    s: i128,
    e: i64,
    sticky: i128,
    half: bool,
}

impl Exact {
    fn of(s: i128, e: i64) -> Exact {
        Exact { s, e, sticky: 0, half: false }
    }
}

/// The next float above `m · 3^q` (normalised, or at [`Q_MIN`]), as a
/// pair; an exponent past +363 stands for +∞.
fn step_up(m: i128, q: i64) -> (i128, i64) {
    if m >= 0 {
        if m < M_MAX { (m + 1, q) } else { (M_NORMAL, q + 1) }
    } else if q > Q_MIN && m + 1 == -(M_NORMAL - 1) {
        // Leaving the smallest normalised magnitude at q: the next
        // value up is the largest magnitude one place finer.
        (-M_MAX, q - 1)
    } else {
        (m + 1, q)
    }
}

fn step_down(m: i128, q: i64) -> (i128, i64) {
    let (m, q) = step_up(-m, q);
    (-m, q)
}

/// Round an exact result to a float once, raising the flags it earns.
fn round(x: Exact, mode: Rounding, flags: &mut u8) -> i64 {
    let Exact { s, e, sticky, half } = x;
    if s == 0 {
        debug_assert!(sticky == 0, "a rounded part needs trits above it");
        return 0;
    }
    let top = e + len(s) as i64 - 1;
    // Keep 21 trits, or as many as reach the subnormal floor.
    let q = (top - 20).max(Q_MIN);
    let cut = q - e;
    let mut q = q;
    // Truncate: t is the nearest place on the grid of 3^q, and dir the
    // side of it the exact value lies on.
    let (mut m, dir) = if cut <= 0 {
        debug_assert!(sticky == 0, "a rounded part needs trits above it");
        (s * pow3w((-cut) as u32), 0)
    } else if cut > len(s) as i64 {
        // Everything is below the last place kept: under half of it.
        (0, s.signum())
    } else {
        let cut = cut as u32;
        let r = low_w(s, cut);
        let t = (s - r) / pow3w(cut);
        let dir = if r != 0 { r.signum() } else { sticky };
        // At the bottom of an exponent's range the grid below is three
        // times finer, and its largest value is 2·3^(q−1) under t's:
        // nearer once the value is a third of a place under t, and on a
        // tie, being nearer zero, chosen.
        let bottom = t.abs() == M_NORMAL && q > Q_MIN && dir == -t.signum();
        let third = pow3w(cut - 1);
        if bottom && (r.abs() > third || (r.abs() == third && sticky != -dir)) {
            q -= 1;
            (t.signum() * M_MAX, t.signum())
        } else if half && r == sticky * (pow3w(cut) - 1) / 2 {
            // Exactly between t and t + sticky: ties go toward zero.
            if t == 0 || t.signum() == sticky { (t, sticky) } else { (t + sticky, -sticky) }
        } else {
            (t, dir)
        }
    };
    // Tiny: below the smallest normal float, before rounding.
    let tiny = q == Q_MIN && (m.abs() < M_NORMAL || (m.abs() == M_NORMAL && dir * m < 0));
    if dir != 0 {
        *flags |= INEXACT;
        if tiny {
            *flags |= UNDERFLOW;
        }
    }
    if q + 20 > E_MAX {
        *flags |= OVERFLOW | INEXACT;
        let away = match mode {
            Rounding::Nearest => true,
            Rounding::Up => m > 0,
            Rounding::Down => m < 0,
            Rounding::TowardZero => false,
        };
        return match (away, m > 0) {
            (true, true) => INF,
            (true, false) => NEG_INF,
            (false, true) => pack(M_MAX, E_MAX - 20),
            (false, false) => pack(-M_MAX, E_MAX - 20),
        };
    }
    let up = match mode {
        Rounding::Nearest => false,
        Rounding::Up => dir > 0,
        Rounding::Down => false,
        Rounding::TowardZero => dir > 0 && m < 0,
    };
    let down = match mode {
        Rounding::Down => dir < 0,
        Rounding::TowardZero => dir < 0 && m > 0,
        _ => false,
    };
    if up {
        (m, q) = step_up(m, q);
    } else if down {
        (m, q) = step_down(m, q);
    }
    if q + 20 > E_MAX {
        *flags |= OVERFLOW;
    }
    pack(m, q)
}

/// `a · 3^ea + b · 3^eb`, exactly or with the far smaller one reduced
/// to the trits that matter and a sticky sign.
fn add_exact(a: i128, ea: i64, b: i128, eb: i64) -> Exact {
    if a == 0 {
        return Exact::of(b, eb);
    }
    if b == 0 {
        return Exact::of(a, ea);
    }
    let (ta, tb) = (ea + len(a) as i64 - 1, eb + len(b) as i64 - 1);
    let m = ea.min(eb);
    // 3⁷⁸ / 2 and the carry fit an i128 with room to spare.
    if ta.max(tb) - m + 2 <= 78 {
        return Exact::of(a * pow3w((ea - m) as u32) + b * pow3w((eb - m) as u32), m);
    }
    // Far apart: the larger keeps 50 trits, enough that the smaller
    // (at most 43 trits, ending more than 76 below the larger's top)
    // only settles which way to round.
    let ((big, ebig, tbig), (small, esmall, tsmall)) =
        if ta >= tb { ((a, ea, ta), (b, eb, tb)) } else { ((b, eb, tb), (a, ea, ta)) };
    let e = tbig - 50;
    let scaled = big * pow3w((ebig - e) as u32);
    if tsmall < e {
        return Exact { s: scaled, e, sticky: small.signum(), half: false };
    }
    let cut = (e - esmall) as u32;
    let r = low_w(small, cut);
    Exact { s: scaled + (small - r) / pow3w(cut), e, sticky: r.signum(), half: false }
}

fn nan(flags: &mut u8, invalid: bool) -> i64 {
    if invalid {
        *flags |= INVALID;
    }
    NAN
}

pub fn add(a: i64, b: i64, mode: Rounding, flags: &mut u8) -> i64 {
    match (unpack(a), unpack(b)) {
        (F::Nan, _) | (_, F::Nan) => nan(flags, false),
        (F::Inf(x), F::Inf(y)) if x != y => nan(flags, true),
        (F::Inf(_), _) => a,
        (_, F::Inf(_)) => b,
        (F::Fin(ma, ea), F::Fin(mb, eb)) => round(add_exact(ma, ea, mb, eb), mode, flags),
    }
}

/// The negation of a float: its significand negated.
pub fn neg(x: i64) -> i64 {
    let e = field(x, M_TRITS, 6);
    e * pow3(M_TRITS) - low(x, M_TRITS)
}

pub fn sub(a: i64, b: i64, mode: Rounding, flags: &mut u8) -> i64 {
    if unpack(b) == F::Nan {
        return nan(flags, false);
    }
    add(a, neg(b), mode, flags)
}

pub fn mul(a: i64, b: i64, mode: Rounding, flags: &mut u8) -> i64 {
    match (unpack(a), unpack(b)) {
        (F::Nan, _) | (_, F::Nan) => nan(flags, false),
        (F::Inf(_), F::Fin(0, _)) | (F::Fin(0, _), F::Inf(_)) => nan(flags, true),
        (F::Inf(x), F::Inf(y)) => inf(x * y),
        (F::Inf(x), F::Fin(m, _)) | (F::Fin(m, _), F::Inf(x)) => inf(x * m.signum() as i64),
        (F::Fin(ma, ea), F::Fin(mb, eb)) => round(Exact::of(ma * mb, ea + eb), mode, flags),
    }
}

fn inf(sign: i64) -> i64 {
    if sign > 0 { INF } else { NEG_INF }
}

/// `a × b + c`, rounded once.
pub fn fma(a: i64, b: i64, c: i64, mode: Rounding, flags: &mut u8) -> i64 {
    let (fa, fb, fc) = (unpack(a), unpack(b), unpack(c));
    if [fa, fb, fc].contains(&F::Nan) {
        // ∞ × 0 is invalid even when the addend is a NaN.
        let invalid = matches!((fa, fb), (F::Inf(_), F::Fin(0, _)) | (F::Fin(0, _), F::Inf(_)));
        return nan(flags, invalid);
    }
    let product = match (fa, fb) {
        (F::Inf(_), F::Fin(0, _)) | (F::Fin(0, _), F::Inf(_)) => return nan(flags, true),
        (F::Inf(x), F::Inf(y)) => F::Inf(x * y),
        (F::Inf(x), F::Fin(m, _)) | (F::Fin(m, _), F::Inf(x)) => F::Inf(x * m.signum() as i64),
        (F::Fin(ma, ea), F::Fin(mb, eb)) => F::Fin(ma * mb, ea + eb),
        _ => unreachable!("NaNs are handled above"),
    };
    match (product, fc) {
        (F::Inf(x), F::Inf(y)) if x != y => nan(flags, true),
        (F::Inf(x), _) => inf(x),
        (_, F::Inf(_)) => c,
        (F::Fin(mp, ep), F::Fin(mc, ec)) => round(add_exact(mp, ep, mc, ec), mode, flags),
        _ => unreachable!("NaNs are handled above"),
    }
}

pub fn div(a: i64, b: i64, mode: Rounding, flags: &mut u8) -> i64 {
    match (unpack(a), unpack(b)) {
        (F::Nan, _) | (_, F::Nan) => nan(flags, false),
        (F::Inf(_), F::Inf(_)) | (F::Fin(0, _), F::Fin(0, _)) => nan(flags, true),
        (F::Inf(x), F::Fin(m, _)) => inf(if m == 0 { x } else { x * m.signum() as i64 }),
        (F::Fin(_, _), F::Inf(_)) => 0,
        (F::Fin(ma, _), F::Fin(0, _)) => {
            *flags |= DIV_ZERO;
            inf(ma.signum() as i64)
        }
        (F::Fin(ma, ea), F::Fin(mb, eb)) => {
            // A quotient of 26 or so trits, rounded to nearest with ties
            // toward zero; the remainder says what is left.
            let p = 26 + len(mb) as i64 - len(ma) as i64;
            let n = ma * pow3w(p as u32);
            let mut quot = n / mb;
            let mut rem = n - quot * mb;
            if 2 * rem.abs() > mb.abs() {
                quot += rem.signum() * mb.signum();
                rem = n - quot * mb;
            }
            let x = Exact {
                s: quot,
                e: ea - eb - p,
                sticky: rem.signum() * mb.signum(),
                half: 2 * rem.abs() == mb.abs(),
            };
            round(x, mode, flags)
        }
    }
}

/// ⌊√x⌋.
fn isqrt(x: i128) -> i128 {
    let mut r = (x as f64).sqrt() as i128;
    while r * r > x {
        r -= 1;
    }
    while (r + 1) * (r + 1) <= x {
        r += 1;
    }
    r
}

pub fn sqrt(a: i64, mode: Rounding, flags: &mut u8) -> i64 {
    match unpack(a) {
        F::Nan => nan(flags, false),
        F::Inf(1) => a,
        F::Inf(_) => nan(flags, true),
        F::Fin(0, _) => 0,
        F::Fin(m, _) if m < 0 => nan(flags, true),
        F::Fin(m, e) => {
            // √(m · 3^p) · 3^((e − p) / 2) with e − p even and about 54
            // trits under the root, so the root has 27.
            let mut p = 54 - len(m) as i64;
            if (e - p).rem_euclid(2) != 0 {
                p += 1;
            }
            let x = m * pow3w(p as u32);
            let mut r = isqrt(x);
            if x - r * r > r {
                r += 1;
            }
            // A root is never exactly half way: (2k + 1)² / 4 is not a
            // whole number.
            round(
                Exact { s: r, e: (e - p) / 2, sticky: (x - r * r).signum(), half: false },
                mode,
                flags,
            )
        }
    }
}

/// Compare two values that are not NaN.
fn order(a: F, b: F) -> std::cmp::Ordering {
    use std::cmp::Ordering::*;
    match (a, b) {
        (F::Inf(x), F::Inf(y)) => x.cmp(&y),
        (F::Inf(x), _) => {
            if x > 0 {
                Greater
            } else {
                Less
            }
        }
        (_, F::Inf(y)) => {
            if y > 0 {
                Less
            } else {
                Greater
            }
        }
        (F::Fin(ma, ea), F::Fin(mb, eb)) => {
            if ma.signum() != mb.signum() || ma == 0 {
                return ma.signum().cmp(&mb.signum());
            }
            // The top trit's place orders magnitudes; on a tie the two
            // are close enough to line up exactly.
            let (ta, tb) = (ea + len(ma) as i64, eb + len(mb) as i64);
            let magnitude = if ta != tb {
                ta.cmp(&tb)
            } else {
                let e = ea.min(eb);
                let (x, y) = (ma * pow3w((ea - e) as u32), mb * pow3w((eb - e) as u32));
                x.abs().cmp(&y.abs())
            };
            if ma > 0 { magnitude } else { magnitude.reverse() }
        }
        _ => unreachable!("NaNs are not ordered here"),
    }
}

pub fn min(a: i64, b: i64) -> i64 {
    pick(a, b, std::cmp::Ordering::Less)
}

pub fn max(a: i64, b: i64) -> i64 {
    pick(a, b, std::cmp::Ordering::Greater)
}

fn pick(a: i64, b: i64, want: std::cmp::Ordering) -> i64 {
    match (unpack(a), unpack(b)) {
        (F::Nan, F::Nan) => NAN,
        (F::Nan, _) => b,
        (_, F::Nan) => a,
        (x, y) => {
            if order(y, x) == want {
                b
            } else {
                a
            }
        }
    }
}

/// The total order: NaNs with a negative significand below −∞, the
/// others above +∞, NaNs among themselves by significand.
pub fn cmp(a: i64, b: i64) -> i64 {
    let rank = |x: i64| match unpack(x) {
        F::Nan if low(x, M_TRITS) < 0 => -1,
        F::Nan => 1,
        _ => 0,
    };
    let (ra, rb) = (rank(a), rank(b));
    let o = if ra != 0 && ra == rb {
        low(a, M_TRITS).cmp(&low(b, M_TRITS))
    } else if ra != rb {
        ra.cmp(&rb)
    } else {
        order(unpack(a), unpack(b))
    };
    o as i64
}

/// The class code (7.11): ±4 infinities, ±3 normal, ±2 subnormal, 0
/// zero, +1 NaN.
pub fn class(a: i64) -> i64 {
    match unpack(a) {
        F::Nan => 1,
        F::Inf(x) => 4 * x,
        F::Fin(0, _) => 0,
        F::Fin(m, e) => {
            let sign = m.signum() as i64;
            if e + len(m) as i64 > -E_MAX { 3 * sign } else { 2 * sign }
        }
    }
}

/// A word integer as a float.
pub fn from_int(a: i64, mode: Rounding, flags: &mut u8) -> i64 {
    round(Exact::of(a as i128, 0), mode, flags)
}

/// A float as a word integer: rounded by the mode, saturating out of
/// range, NaN 0; both of those raise invalid and not inexact.
pub fn to_int(a: i64, mode: Rounding, flags: &mut u8) -> i64 {
    let saturate = |sign: i64, flags: &mut u8| {
        *flags |= INVALID;
        sign * WORD_MAX
    };
    let (m, e) = match unpack(a) {
        F::Nan => return nan_int(flags),
        F::Inf(x) => return saturate(x, flags),
        F::Fin(m, e) => (m, e),
    };
    let (t, dir) = if e >= 0 {
        if len(m) as i64 + e > 28 {
            return saturate(m.signum() as i64, flags);
        }
        (m * pow3w(e as u32), 0)
    } else if -e > 40 {
        (0, m.signum())
    } else {
        let cut = (-e) as u32;
        let r = low_w(m, cut);
        ((m - r) / pow3w(cut), r.signum())
    };
    let t = match mode {
        Rounding::Nearest => t,
        Rounding::Up if dir > 0 => t + 1,
        Rounding::Down if dir < 0 => t - 1,
        Rounding::TowardZero if t != 0 && dir == -t.signum() => t + dir,
        _ => t,
    };
    if t.abs() > WORD_MAX as i128 {
        return saturate(t.signum() as i64, flags);
    }
    if dir != 0 {
        *flags |= INEXACT;
    }
    t as i64
}

fn nan_int(flags: &mut u8) -> i64 {
    *flags |= INVALID;
    0
}

/// A predicate's answer: +1 true, −1 false, 0 when a NaN makes it
/// unordered; `signal` raises invalid then.
fn relation(
    a: i64,
    b: i64,
    holds: fn(std::cmp::Ordering) -> bool,
    signal: bool,
    flags: &mut u8,
) -> i64 {
    let (x, y) = (unpack(a), unpack(b));
    if x == F::Nan || y == F::Nan {
        if signal {
            *flags |= INVALID;
        }
        return 0;
    }
    if holds(order(x, y)) { 1 } else { -1 }
}

pub fn eq(a: i64, b: i64, flags: &mut u8) -> i64 {
    relation(a, b, |o| o.is_eq(), false, flags)
}

pub fn lt(a: i64, b: i64, flags: &mut u8) -> i64 {
    relation(a, b, |o| o.is_lt(), true, flags)
}

pub fn le(a: i64, b: i64, flags: &mut u8) -> i64 {
    relation(a, b, |o| o.is_le(), true, flags)
}

/// The float nearest an f64, for tests and tools: exact rounding when
/// the f64 is between about 10^-17 and 10^19, nearly so beyond.
pub fn from_f64(x: f64) -> i64 {
    if x.is_nan() {
        return NAN;
    }
    if x.is_infinite() {
        return inf(x.signum() as i64);
    }
    if x == 0.0 {
        return 0;
    }
    // x = mant · 2^e2 exactly; with 3^e3 near x / 3³⁰, the quotient
    // x / 3^e3 has about 30 trits to round from.
    let bits = x.to_bits();
    let exp = ((bits >> 52) & 0x7ff) as i64;
    let frac = (bits & ((1 << 52) - 1)) as i128;
    let mant = if exp == 0 { frac } else { frac | 1 << 52 };
    let mant = if x < 0.0 { -mant } else { mant };
    let e2 = if exp == 0 { -1074 } else { exp - 1075 };
    let e3 = x.abs().log(3.0).floor() as i64 - 30;
    let mut flags = 0;
    let Some((num, den)) = fraction(mant, e2, e3) else {
        let s = (x / 3f64.powi(e3 as i32)).round() as i128;
        return round(Exact::of(s, e3), Rounding::Nearest, &mut flags);
    };
    let mut q = num / den;
    let mut r = num - q * den;
    if 2 * r.abs() > den {
        q += r.signum();
        r = num - q * den;
    }
    let x = Exact { s: q, e: e3, sticky: r.signum(), half: 2 * r.abs() == den };
    round(x, Rounding::Nearest, &mut flags)
}

/// mant · 2^e2 / 3^e3 as num / den with den > 0, when both fit.
fn fraction(mant: i128, e2: i64, e3: i64) -> Option<(i128, i128)> {
    let (mut num, mut den) = (mant, 1i128);
    let two = |k: i64| 1i128.checked_shl(u32::try_from(k).ok()?).filter(|&v| v > 0);
    let three = |k: i64| 3i128.checked_pow(u32::try_from(k).ok()?);
    if e2 >= 0 {
        num = num.checked_mul(two(e2)?)?;
    } else {
        den = den.checked_mul(two(-e2)?)?;
    }
    if e3 >= 0 {
        den = den.checked_mul(three(e3)?)?;
    } else {
        num = num.checked_mul(three(-e3)?)?;
    }
    Some((num, den))
}

/// The float nearest a decimal literal such as `-1.25e-3`, `inf`,
/// `-inf` or `nan`, rounded once to nearest, as the assembler's
/// `.float` writes it. None when the text is not such a literal.
pub fn from_decimal(text: &str) -> Option<i64> {
    let (sign, body) = match text.strip_prefix('-') {
        Some(rest) => (-1, rest),
        None => (1, text.strip_prefix('+').unwrap_or(text)),
    };
    match body {
        "inf" => return Some(inf(sign)),
        "nan" if sign > 0 => return Some(NAN),
        _ => {}
    }
    let (mantissa, exp) = match body.find(['e', 'E']) {
        Some(i) => (&body[..i], body[i + 1..].parse::<i64>().ok()?),
        None => (body, 0),
    };
    let (whole, frac) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let digits: String = format!("{whole}{frac}");
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let digits = digits.trim_start_matches('0');
    if digits.is_empty() {
        return Some(0);
    }
    // value = digits · 10^k
    let k = exp.checked_sub(frac.len() as i64)?;
    let magnitude = digits.len() as i64 + k;
    if magnitude > 400 {
        return Some(inf(sign));
    }
    if magnitude < -400 {
        return Some(0);
    }
    let mut num = Big::from_digits(digits);
    let mut den = Big::one();
    for _ in 0..k.unsigned_abs() {
        if k > 0 { num.mul(10) } else { den.mul(10) }
    }
    // Scale by 3^p so the quotient has about 30 trits.
    let p = 30 - ((magnitude - 1) as f64 * 10f64.log(3.0)).floor() as i64;
    for _ in 0..p.unsigned_abs() {
        if p > 0 { num.mul(3) } else { den.mul(3) }
    }
    // Long division in base 3.
    let mut shifted = vec![den.clone()];
    while shifted.last().unwrap().le(&num) {
        let mut next = shifted.last().unwrap().clone();
        next.mul(3);
        shifted.push(next);
    }
    let mut q: i128 = 0;
    for d in shifted.iter().rev() {
        let mut count = 0;
        while d.le(&num) {
            num.sub(d);
            count += 1;
        }
        q = q * 3 + count;
    }
    let mut twice = num.clone();
    twice.mul(2);
    let (q, sticky, half) = match twice.cmp(&den) {
        std::cmp::Ordering::Less => (q, if num.is_zero() { 0 } else { 1 }, false),
        std::cmp::Ordering::Equal => (q, 1, true),
        std::cmp::Ordering::Greater => (q + 1, -1, false),
    };
    let s = sign as i128;
    let x = Exact { s: s * q, e: -p, sticky: s * sticky, half };
    Some(round(x, Rounding::Nearest, &mut 0))
}

/// A natural number in base 2³², least significant limb first, for
/// [`from_decimal`].
#[derive(Clone, PartialEq, Eq)]
struct Big(Vec<u32>);

impl Big {
    fn one() -> Big {
        Big(vec![1])
    }

    fn from_digits(digits: &str) -> Big {
        let mut b = Big(vec![0]);
        for d in digits.bytes() {
            b.mul(10);
            b.add((d - b'0') as u32);
        }
        b
    }

    fn is_zero(&self) -> bool {
        self.0.iter().all(|&l| l == 0)
    }

    fn mul(&mut self, k: u32) {
        let mut carry = 0u64;
        for l in &mut self.0 {
            let v = *l as u64 * k as u64 + carry;
            *l = v as u32;
            carry = v >> 32;
        }
        if carry != 0 {
            self.0.push(carry as u32);
        }
    }

    fn add(&mut self, k: u32) {
        let mut carry = k as u64;
        for l in &mut self.0 {
            let v = *l as u64 + carry;
            *l = v as u32;
            carry = v >> 32;
            if carry == 0 {
                return;
            }
        }
        self.0.push(carry as u32);
    }

    /// self − other, which must not be negative.
    fn sub(&mut self, other: &Big) {
        let mut borrow = 0i64;
        for (i, l) in self.0.iter_mut().enumerate() {
            let v = *l as i64 - *other.0.get(i).unwrap_or(&0) as i64 - borrow;
            borrow = (v < 0) as i64;
            *l = v.rem_euclid(1 << 32) as u32;
        }
    }

    fn cmp(&self, other: &Big) -> std::cmp::Ordering {
        let n = self.0.len().max(other.0.len());
        for i in (0..n).rev() {
            let (a, b) = (self.0.get(i).unwrap_or(&0), other.0.get(i).unwrap_or(&0));
            if a != b {
                return a.cmp(b);
            }
        }
        std::cmp::Ordering::Equal
    }

    fn le(&self, other: &Big) -> bool {
        self.cmp(other).is_le()
    }
}

/// A float as the nearest f64, for tests and tools.
pub fn to_f64(x: i64) -> f64 {
    match unpack(x) {
        F::Nan => f64::NAN,
        F::Inf(s) => s as f64 * f64::INFINITY,
        // 3³³ < 2⁵³, so one division is correctly rounded when it
        // reaches.
        F::Fin(m, e) if (-33..0).contains(&e) => m as f64 / 3f64.powi(-e as i32),
        F::Fin(m, e) => m as f64 * 3f64.powi(e as i32),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const N: Rounding = Rounding::Nearest;

    fn f(x: f64) -> i64 {
        from_f64(x)
    }

    fn word(e: i64, m: i64) -> i64 {
        e * pow3(M_TRITS) + m
    }

    #[test]
    fn small_integers_are_exact() {
        let mut fl = 0;
        for i in -100..=100 {
            let x = from_int(i, N, &mut fl);
            assert_eq!(to_int(x, N, &mut fl), i);
            assert_eq!(to_f64(x), i as f64);
        }
        assert_eq!(fl, 0);
        assert_eq!(from_int(0, N, &mut fl), 0);
        // 1 = 3²⁰ / 3²⁰ × 3⁰.
        assert_eq!(from_int(1, N, &mut fl), word(0, pow3(20)));
    }

    #[test]
    fn arithmetic_on_exact_values() {
        let mut fl = 0;
        let (two, three, six) = (f(2.0), f(3.0), f(6.0));
        assert_eq!(add(two, f(4.0), N, &mut fl), six);
        assert_eq!(sub(two, six, N, &mut fl), f(-4.0));
        assert_eq!(mul(two, three, N, &mut fl), six);
        assert_eq!(div(six, three, N, &mut fl), two);
        assert_eq!(sqrt(f(9.0), N, &mut fl), three);
        assert_eq!(fma(two, three, f(-6.0), N, &mut fl), 0);
        assert_eq!(div(f(1.0), f(27.0), N, &mut fl), word(-3, pow3(20)));
        assert_eq!(fl, 0);
    }

    #[test]
    fn inexact_results_round_to_nearest() {
        let mut fl = 0;
        let third = div(f(1.0), three(), N, &mut fl);
        assert_eq!(fl, 0, "1/3 is exact in ternary");
        let tenth = div(f(1.0), f(10.0), N, &mut fl);
        assert_eq!(fl, INEXACT);
        assert!((to_f64(tenth) - 0.1).abs() < 1e-10);
        assert_eq!(tenth, f(0.1));
        let _ = third;
    }

    fn three() -> i64 {
        f(3.0)
    }

    #[test]
    fn directed_rounding_brackets_the_exact_value() {
        let (one, ten) = (f(1.0), f(10.0));
        let mut fl = 0;
        let near = div(one, ten, N, &mut fl);
        let up = div(one, ten, Rounding::Up, &mut fl);
        let down = div(one, ten, Rounding::Down, &mut fl);
        let zero = div(one, ten, Rounding::TowardZero, &mut fl);
        assert!(to_f64(down) < 0.1 && 0.1 < to_f64(up));
        assert!(near == up || near == down);
        assert_eq!(zero, down);
        let neg_zero = div(f(-1.0), ten, Rounding::TowardZero, &mut fl);
        assert_eq!(neg_zero, neg(down));
        assert_eq!(div(f(-1.0), ten, Rounding::Up, &mut fl), neg(down));
    }

    #[test]
    fn stepping_crosses_exponents() {
        let one = f(1.0);
        let (m, q) = (pow3(20) as i128, -20);
        // Just below 1.0 the grid is a third as wide.
        let below = step_down(m, q);
        assert_eq!(pack(below.0, below.1), word(0, pow3(20) - 1));
        let low = (M_NORMAL, -20);
        assert_eq!(step_down(low.0, low.1), (M_MAX, -21));
        assert_eq!(step_up(M_MAX, -21), (M_NORMAL, -20));
        assert_eq!(step_up(-M_NORMAL, -20), (-M_MAX, -21));
        let _ = one;
    }

    #[test]
    fn overflow_and_underflow() {
        let mut fl = 0;
        let big = word(E_MAX, M_MAX as i64);
        assert_eq!(mul(big, f(3.0), N, &mut fl), INF);
        assert_eq!(fl, OVERFLOW | INEXACT);
        fl = 0;
        assert_eq!(mul(big, f(3.0), Rounding::TowardZero, &mut fl), big);
        assert_eq!(mul(neg(big), f(3.0), Rounding::Up, &mut fl), neg(big));
        let tiny = word(-E_SPECIAL, 1);
        fl = 0;
        assert_eq!(div(tiny, f(3.0), N, &mut fl), 0);
        assert_eq!(fl, UNDERFLOW | INEXACT);
        assert_eq!(div(tiny, f(3.0), Rounding::Up, &mut fl), tiny);
        assert_eq!(class(tiny), 2);
        assert_eq!(class(neg(tiny)), -2);
        // Exact subnormal results raise nothing.
        fl = 0;
        assert_eq!(mul(tiny, f(1.0), N, &mut fl), tiny);
        assert_eq!(fl, 0);
    }

    #[test]
    fn specials() {
        let mut fl = 0;
        assert_eq!(add(INF, NEG_INF, N, &mut fl), NAN);
        assert_eq!(fl, INVALID);
        fl = 0;
        assert_eq!(div(f(1.0), 0, N, &mut fl), INF);
        assert_eq!(div(f(-1.0), 0, N, &mut fl), NEG_INF);
        assert_eq!(fl, DIV_ZERO);
        fl = 0;
        assert_eq!(div(0, 0, N, &mut fl), NAN);
        assert_eq!(mul(INF, 0, N, &mut fl), NAN);
        assert_eq!(sqrt(f(-1.0), N, &mut fl), NAN);
        assert_eq!(fl, INVALID);
        assert_eq!(class(INF), 4);
        assert_eq!(class(NEG_INF), -4);
        assert_eq!(class(NAN), 1);
        assert_eq!(class(0), 0);
        assert_eq!(class(f(-2.5)), -3);
    }

    #[test]
    fn comparisons() {
        let mut fl = 0;
        let (a, b) = (f(1.5), f(2.5));
        assert_eq!((lt(a, b, &mut fl), le(b, a, &mut fl), eq(a, a, &mut fl)), (1, -1, 1));
        assert_eq!(fl, 0);
        assert_eq!(eq(a, NAN, &mut fl), 0);
        assert_eq!(fl, 0);
        assert_eq!(lt(a, NAN, &mut fl), 0);
        assert_eq!(fl, INVALID);
        assert_eq!((min(a, b), max(a, b), min(NAN, b), max(a, NAN)), (a, b, b, a));
        assert_eq!(min(NAN, NAN), NAN);
        let neg_nan = word(E_SPECIAL, -5);
        assert_eq!(cmp(neg_nan, NEG_INF), -1);
        assert_eq!(cmp(NAN, INF), 1);
        assert_eq!(cmp(neg_nan, NAN), -1);
        assert_eq!(cmp(a, b), -1);
        assert_eq!(cmp(f(-1e100), f(1e-100)), -1);
        // A subnormal-encoded value equal to a normal one.
        let alias = word(-E_SPECIAL, pow3(20));
        let normal = word(-E_MAX, pow3(20));
        assert_eq!(cmp(alias, normal), 0);
    }

    #[test]
    fn conversions_saturate() {
        let mut fl = 0;
        // 2.5 = 2.111…₃ is stored a hair low, so it rounds down.
        assert_eq!(to_int(f(2.5), N, &mut fl), 2);
        assert_eq!(to_int(f(2.6), N, &mut fl), 3);
        assert_eq!(fl, INEXACT);
        assert_eq!(to_int(f(-2.6), Rounding::TowardZero, &mut fl), -2);
        assert_eq!(to_int(f(2.4), Rounding::Up, &mut fl), 3);
        assert_eq!(to_int(f(-2.4), Rounding::Down, &mut fl), -3);
        fl = 0;
        assert_eq!(to_int(f(1e30), N, &mut fl), WORD_MAX);
        assert_eq!(to_int(NEG_INF, N, &mut fl), -WORD_MAX);
        assert_eq!(to_int(NAN, N, &mut fl), 0);
        assert_eq!(fl, INVALID);
        fl = 0;
        let x = from_int(WORD_MAX, N, &mut fl);
        assert_eq!(fl, INEXACT, "27 trits do not fit 21");
        assert!((to_f64(x) - WORD_MAX as f64).abs() < 1e3);
    }

    #[test]
    fn fma_rounds_once() {
        let mut fl = 0;
        // (1 + ε)(1 − ε) − 1 = −ε²: the product alone rounds to 1.
        let eps = div(f(1.0), from_int(pow3(15), N, &mut fl), N, &mut fl);
        let a = add(f(1.0), eps, N, &mut fl);
        let b = sub(f(1.0), eps, N, &mut fl);
        assert_eq!(fl, 0);
        let r = fma(a, b, f(-1.0), N, &mut fl);
        assert_eq!(r, neg(mul(eps, eps, N, &mut fl)));
        assert_eq!(fl, 0);
        assert_eq!(sub(mul(a, b, N, &mut fl), f(1.0), N, &mut fl), 0);
        // A far smaller addend only decides the direction.
        fl = 0;
        let tiny = word(-300, pow3(20));
        assert_eq!(fma(f(1.0), f(1.0), tiny, Rounding::Up, &mut fl), word(0, pow3(20) + 1));
        assert_eq!(fma(f(1.0), f(1.0), tiny, N, &mut fl), f(1.0));
        assert_eq!(add(f(1.0), neg(tiny), Rounding::Down, &mut fl), word(0, pow3(20) - 1));
        assert_eq!(fl, INEXACT);
    }

    #[test]
    fn division_ties_go_toward_zero() {
        // ½ = 0.111…₃ lies exactly half way between two 21-trit values.
        let mut fl = 0;
        let (one, two) = (f(1.0), f(2.0));
        assert_eq!(div(one, two, N, &mut fl), word(-1, M_MAX as i64));
        assert_eq!(fl, INEXACT);
        assert_eq!(div(neg(one), two, N, &mut fl), word(-1, -M_MAX as i64));
        assert_eq!(div(one, two, Rounding::Up, &mut fl), word(0, M_NORMAL as i64));
        assert_eq!(div(one, two, Rounding::TowardZero, &mut fl), word(-1, M_MAX as i64));
        assert_eq!(f(0.5), word(-1, M_MAX as i64));
    }

    #[test]
    fn decimal_literals() {
        for x in [1.0, -2.5, 0.1, 1e-30, 123456.789, 6.02214076e23, -1.5e-150, 7e170] {
            assert_eq!(from_decimal(&format!("{x:e}")), Some(f(x)), "{x}");
        }
        assert_eq!(from_decimal("0.5"), Some(word(-1, M_MAX as i64)));
        assert_eq!(from_decimal("1e400"), Some(INF));
        assert_eq!(from_decimal("-1e200"), Some(NEG_INF));
        assert_eq!(from_decimal("1e-200"), Some(0));
        assert_eq!(from_decimal("-inf"), Some(NEG_INF));
        assert_eq!(from_decimal("nan"), Some(NAN));
        assert_eq!(from_decimal("0.000"), Some(0));
        assert_eq!(from_decimal("3."), Some(f(3.0)));
        assert_eq!(from_decimal(".5"), Some(f(0.5)));
        for bad in ["", ".", "1e", "x", "1.2.3", "--1", "-nan"] {
            assert_eq!(from_decimal(bad), None, "{bad}");
        }
    }

    #[test]
    fn square_roots() {
        let mut fl = 0;
        let r = sqrt(f(2.0), N, &mut fl);
        assert!((to_f64(r) - 2f64.sqrt()).abs() < 1e-9);
        assert_eq!(fl, INEXACT);
        assert_eq!(sqrt(f(1.0 / 9.0), N, &mut fl), div(f(1.0), f(3.0), N, &mut fl));
        let up = sqrt(f(2.0), Rounding::Up, &mut fl);
        let down = sqrt(f(2.0), Rounding::Down, &mut fl);
        assert!(to_f64(down) < 2f64.sqrt() && 2f64.sqrt() < to_f64(up));
    }
}
