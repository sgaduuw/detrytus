//! The binary floating point coprocessor (docs/platform.md, The binary
//! floating point coprocessor): IEEE 754 binary32 and binary64 beside
//! the ternary core, for programs whose floats are binary, such as
//! Rust's `f32` and `f64`.
//!
//! Each hart has its own bank: operands A, B and C, and the result R,
//! each two words of 32 bits (low, high). Writing OP computes at once.
//! The host's IEEE arithmetic does the work, rounding to nearest, ties
//! to even; NaN results are the canonical quiet NaN. Conversions to
//! integers truncate and saturate, NaN giving 0, as Rust's `as` does.

use std::collections::HashMap;

use crate::bus::PAGE;

pub const FPU_PAGE: i64 = -15;
const CENTRE: i64 = FPU_PAGE * PAGE;

// Registers, as offsets from the page's centre.
const PRESENT: i64 = -12;
const A_LO: i64 = -9;
const A_HI: i64 = -6;
const B_LO: i64 = -3;
const B_HI: i64 = 0;
const C_LO: i64 = 3;
const C_HI: i64 = 6;
const OP: i64 = 9;
const R_LO: i64 = 12;
const R_HI: i64 = 15;

/// What PRESENT reads: the coprocessor's version.
pub const FPU_VERSION: i64 = 1;

/// Operations. The sign is the precision: + binary64, − binary32.
pub const FPU_ADD: i64 = 1;
pub const FPU_SUB: i64 = 2;
pub const FPU_MUL: i64 = 3;
pub const FPU_DIV: i64 = 4;
pub const FPU_SQRT: i64 = 5;
pub const FPU_FMA: i64 = 6;
pub const FPU_MIN: i64 = 7;
pub const FPU_MAX: i64 = 8;
pub const FPU_CMP: i64 = 9;
pub const FPU_CVT: i64 = 10;
pub const FPU_TO_I32: i64 = 11;
pub const FPU_TO_U32: i64 = 12;
pub const FPU_TO_I64: i64 = 13;
pub const FPU_TO_U64: i64 = 14;
pub const FPU_FROM_I32: i64 = 15;
pub const FPU_FROM_U32: i64 = 16;
pub const FPU_FROM_I64: i64 = 17;
pub const FPU_FROM_U64: i64 = 18;

/// What FPU_CMP gives when an operand is a NaN.
pub const FPU_UNORDERED: i64 = 2;

const NAN32: u32 = 0x7fc0_0000;
const NAN64: u64 = 0x7ff8_0000_0000_0000;

/// The address of a register: for tests and tools.
pub fn fpu_addr(offset: i64) -> i64 {
    CENTRE + offset
}

#[derive(Clone, Copy, Default)]
struct Bank {
    a: [i64; 2],
    b: [i64; 2],
    c: [i64; 2],
    r: [i64; 2],
}

#[derive(Default)]
pub struct Fpu {
    banks: HashMap<i64, Bank>,
}

/// A word read as 32 bits: its value modulo 2³².
fn bits32(w: i64) -> u32 {
    w.rem_euclid(1 << 32) as u32
}

fn bits64(pair: [i64; 2]) -> u64 {
    bits32(pair[0]) as u64 | (bits32(pair[1]) as u64) << 32
}

fn words64(x: u64) -> [i64; 2] {
    [(x & 0xffff_ffff) as i64, (x >> 32) as i64]
}

fn words32(x: u32) -> [i64; 2] {
    [x as i64, 0]
}

fn f32_out(x: f32) -> [i64; 2] {
    words32(if x.is_nan() { NAN32 } else { x.to_bits() })
}

fn f64_out(x: f64) -> [i64; 2] {
    words64(if x.is_nan() { NAN64 } else { x.to_bits() })
}

/// IEEE minimumNumber and maximumNumber: a NaN operand is ignored, and
/// −0 is below +0. None when both are NaN.
fn pick<T: Copy + PartialOrd>(
    a: T,
    b: T,
    nan: [bool; 2],
    negative: [bool; 2],
    max: bool,
) -> Option<T> {
    match nan {
        [true, true] => None,
        [true, false] => Some(b),
        [false, true] => Some(a),
        // Equal but for the sign: zeros of both signs.
        _ if a == b && negative[0] != negative[1] => Some(if negative[0] != max { a } else { b }),
        _ => Some(if (b < a) == max { a } else { b }),
    }
}

fn compare<T: PartialOrd>(a: T, b: T) -> i64 {
    match a.partial_cmp(&b) {
        Some(o) => o as i64,
        None => FPU_UNORDERED,
    }
}

fn single(op: i64, bank: &Bank) -> Option<[i64; 2]> {
    let f = |w: [i64; 2]| f32::from_bits(bits32(w[0]));
    let (a, b, c) = (f(bank.a), f(bank.b), f(bank.c));
    Some(match op {
        FPU_ADD => f32_out(a + b),
        FPU_SUB => f32_out(a - b),
        FPU_MUL => f32_out(a * b),
        FPU_DIV => f32_out(a / b),
        FPU_SQRT => f32_out(a.sqrt()),
        FPU_FMA => f32_out(a.mul_add(b, c)),
        FPU_MIN | FPU_MAX => {
            let (nan, neg) =
                ([a.is_nan(), b.is_nan()], [a.is_sign_negative(), b.is_sign_negative()]);
            let r = pick(a, b, nan, neg, op == FPU_MAX);
            f32_out(r.unwrap_or(f32::NAN))
        }
        FPU_CMP => [compare(a, b), 0],
        FPU_CVT => f64_out(a as f64),
        FPU_TO_I32 => words32(a as i32 as u32),
        FPU_TO_U32 => words32(a as u32),
        FPU_TO_I64 => words64(a as i64 as u64),
        FPU_TO_U64 => words64(a as u64),
        FPU_FROM_I32 => f32_out(bits32(bank.a[0]) as i32 as f32),
        FPU_FROM_U32 => f32_out(bits32(bank.a[0]) as f32),
        FPU_FROM_I64 => f32_out(bits64(bank.a) as i64 as f32),
        FPU_FROM_U64 => f32_out(bits64(bank.a) as f32),
        _ => return None,
    })
}

fn double(op: i64, bank: &Bank) -> Option<[i64; 2]> {
    let f = |w: [i64; 2]| f64::from_bits(bits64(w));
    let (a, b, c) = (f(bank.a), f(bank.b), f(bank.c));
    Some(match op {
        FPU_ADD => f64_out(a + b),
        FPU_SUB => f64_out(a - b),
        FPU_MUL => f64_out(a * b),
        FPU_DIV => f64_out(a / b),
        FPU_SQRT => f64_out(a.sqrt()),
        FPU_FMA => f64_out(a.mul_add(b, c)),
        FPU_MIN | FPU_MAX => {
            let (nan, neg) =
                ([a.is_nan(), b.is_nan()], [a.is_sign_negative(), b.is_sign_negative()]);
            let r = pick(a, b, nan, neg, op == FPU_MAX);
            f64_out(r.unwrap_or(f64::NAN))
        }
        FPU_CMP => [compare(a, b), 0],
        FPU_CVT => f32_out(a as f32),
        FPU_TO_I32 => words32(a as i32 as u32),
        FPU_TO_U32 => words32(a as u32),
        FPU_TO_I64 => words64(a as i64 as u64),
        FPU_TO_U64 => words64(a as u64),
        FPU_FROM_I32 => f64_out(bits32(bank.a[0]) as i32 as f64),
        FPU_FROM_U32 => f64_out(bits32(bank.a[0]) as f64),
        FPU_FROM_I64 => f64_out(bits64(bank.a) as i64 as f64),
        FPU_FROM_U64 => f64_out(bits64(bank.a) as f64),
        _ => return None,
    })
}

impl Fpu {
    pub fn read(&self, addr: i64, hart: i64) -> i64 {
        let bank = self.banks.get(&hart).copied().unwrap_or_default();
        match addr - CENTRE {
            PRESENT => FPU_VERSION,
            A_LO => bank.a[0],
            A_HI => bank.a[1],
            B_LO => bank.b[0],
            B_HI => bank.b[1],
            C_LO => bank.c[0],
            C_HI => bank.c[1],
            R_LO => bank.r[0],
            R_HI => bank.r[1],
            _ => 0,
        }
    }

    pub fn write(&mut self, addr: i64, hart: i64, value: i64) {
        let bank = self.banks.entry(hart).or_default();
        match addr - CENTRE {
            A_LO => bank.a[0] = value,
            A_HI => bank.a[1] = value,
            B_LO => bank.b[0] = value,
            B_HI => bank.b[1] = value,
            C_LO => bank.c[0] = value,
            C_HI => bank.c[1] = value,
            // Writable so that a kernel can restore a bank.
            R_LO => bank.r[0] = value,
            R_HI => bank.r[1] = value,
            OP => {
                let r = if value < 0 { single(-value, bank) } else { double(value, bank) };
                if let Some(r) = r {
                    bank.r = r;
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(op: i64, a: [i64; 2], b: [i64; 2], c: [i64; 2]) -> [i64; 2] {
        let mut fpu = Fpu::default();
        for (off, v) in
            [(A_LO, a[0]), (A_HI, a[1]), (B_LO, b[0]), (B_HI, b[1]), (C_LO, c[0]), (C_HI, c[1])]
        {
            fpu.write(fpu_addr(off), 0, v);
        }
        fpu.write(fpu_addr(OP), 0, op);
        [fpu.read(fpu_addr(R_LO), 0), fpu.read(fpu_addr(R_HI), 0)]
    }

    fn s(x: f32) -> [i64; 2] {
        words32(x.to_bits())
    }

    fn d(x: f64) -> [i64; 2] {
        words64(x.to_bits())
    }

    #[test]
    fn single_and_double_arithmetic() {
        let z = [0, 0];
        assert_eq!(run(-FPU_ADD, s(1.5), s(2.25), z), s(3.75));
        assert_eq!(run(FPU_DIV, d(1.0), d(10.0), z), d(0.1));
        assert_eq!(run(FPU_SQRT, d(2.0), z, z), d(2f64.sqrt()));
        assert_eq!(run(-FPU_FMA, s(2.0), s(3.0), s(-1.0)), s(5.0));
        assert_eq!(run(FPU_SUB, d(0.0), d(0.0), z), d(0.0));
        assert_eq!(run(-FPU_DIV, s(0.0), s(0.0), z), words32(NAN32));
        assert_eq!(run(FPU_MUL, d(f64::INFINITY), d(0.0), z), words64(NAN64));
    }

    #[test]
    fn comparisons_and_min_max() {
        let z = [0, 0];
        assert_eq!(run(FPU_CMP, d(1.0), d(2.0), z), [-1, 0]);
        assert_eq!(run(-FPU_CMP, s(2.0), s(2.0), z), [0, 0]);
        assert_eq!(run(FPU_CMP, d(f64::NAN), d(2.0), z), [FPU_UNORDERED, 0]);
        assert_eq!(run(FPU_MIN, d(f64::NAN), d(2.0), z), d(2.0));
        assert_eq!(run(-FPU_MAX, s(-0.0), s(0.0), z), s(0.0));
        assert_eq!(run(-FPU_MIN, s(0.0), s(-0.0), z), s(-0.0));
        assert_eq!(run(FPU_MAX, d(f64::NAN), d(f64::NAN), z), words64(NAN64));
    }

    #[test]
    fn conversions() {
        let z = [0, 0];
        assert_eq!(run(-FPU_CVT, s(1.5), z, z), d(1.5));
        assert_eq!(run(FPU_CVT, d(0.1), z, z), s(0.1));
        assert_eq!(run(FPU_TO_I32, d(-2.9), z, z), words32(-2i32 as u32));
        assert_eq!(run(FPU_TO_I32, d(1e20), z, z), words32(i32::MAX as u32));
        assert_eq!(run(-FPU_TO_U32, s(-5.0), z, z), [0, 0]);
        assert_eq!(run(FPU_TO_U64, d(f64::NAN), z, z), [0, 0]);
        assert_eq!(run(FPU_TO_I64, d(-1.0), z, z), words64(u64::MAX));
        assert_eq!(run(-FPU_FROM_I32, [-7, 0], z, z), s(-7.0));
        assert_eq!(run(FPU_FROM_I32, words32(-7i32 as u32), z, z), d(-7.0));
        assert_eq!(run(FPU_FROM_U64, words64(u64::MAX), z, z), d(u64::MAX as f64));
        assert_eq!(run(FPU_FROM_I64, words64(-3i64 as u64), z, z), d(-3.0));
    }

    #[test]
    fn harts_have_their_own_banks() {
        let mut fpu = Fpu::default();
        fpu.write(fpu_addr(A_LO), 0, 5);
        fpu.write(fpu_addr(A_LO), 1, 6);
        assert_eq!(fpu.read(fpu_addr(A_LO), 0), 5);
        assert_eq!(fpu.read(fpu_addr(A_LO), 1), 6);
        assert_eq!(fpu.read(fpu_addr(PRESENT), 7), FPU_VERSION);
        fpu.write(fpu_addr(R_HI), 0, 77);
        assert_eq!(fpu.read(fpu_addr(R_HI), 0), 77, "R is writable, for a kernel's restore");
        // An unknown operation leaves R alone.
        fpu.write(fpu_addr(OP), 0, -FPU_FROM_I32);
        fpu.write(fpu_addr(OP), 0, 99);
        assert_eq!(fpu.read(fpu_addr(R_LO), 0), s(5.0)[0]);
    }
}
