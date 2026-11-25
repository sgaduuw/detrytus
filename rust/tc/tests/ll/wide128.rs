//! 128-bit integers: arithmetic, shifts, comparisons, casts, division,
//! and formatting with core::fmt, on values the compiler cannot see
//! through. Prints what it formats; exits with the number of failed
//! checks, naming each.
#![no_std]

use core::fmt::Write;

unsafe extern "C" {
    fn write(fd: i32, buf: *const u8, n: usize) -> isize;
}

static mut FAILS: i32 = 0;

#[inline(never)]
fn opaque<T>(x: T) -> T {
    unsafe { core::ptr::read_volatile(&x) }
}

#[inline(never)]
fn check(name: &[u8], ok: bool) {
    if !ok {
        unsafe {
            write(1, name.as_ptr(), name.len());
            write(1, b"\n".as_ptr(), 1);
            FAILS += 1;
        }
    }
}

struct Out;

impl Write for Out {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        unsafe { write(1, s.as_ptr(), s.len()) };
        Ok(())
    }
}

#[inline(never)]
fn mul_div(a: u128, b: u128) -> (u128, u128, u128) {
    (a.wrapping_mul(b), a / b, a % b)
}

#[unsafe(no_mangle)]
pub extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    let a: u128 = opaque(0x0123_4567_89ab_cdef_fedc_ba98_7654_3210);
    let b: u128 = opaque(0xffff_ffff_ffff_ffff_0000_0000_0000_0001);
    let s: i128 = opaque(-170141183460469231731687303715884105728);
    let k: u32 = opaque(77);
    check(b"add", a.wrapping_add(b) == 0x0123_4567_89ab_cdee_fedc_ba98_7654_3211);
    check(b"sub", a.wrapping_sub(b) == 0x0123_4567_89ab_cdf0_fedc_ba98_7654_320f);
    check(b"and or xor", (a & b) ^ (a | b) == a ^ b);
    check(b"not", !a == 0xfedc_ba98_7654_3210_0123_4567_89ab_cdef);
    check(b"shl", a << 4 == 0x1234_5678_9abc_deff_edcb_a987_6543_2100);
    check(b"lshr 70", a >> 70 == 0x4_8d15_9e26_af37);
    check(b"shl 100", a << 100 == 0x6543_2100_0000_0000_0000_0000_0000_0000);
    check(b"shl 32", b << 32 == 0xffff_ffff_0000_0000_0000_0001_0000_0000);
    check(b"lshr 17", b >> 17 == 0x7fff_ffff_ffff_ffff_8000_0000_0000);
    check(b"ashr 33", s >> 33 == -19807040628566084398385987584);
    check(b"ashr 96", (a as i128).wrapping_neg() >> 96 == -0x123_4568);
    check(b"shl var", a << k == a.wrapping_shl(77));
    check(b"lshr var", b >> k == 0x7_ffff_ffff_ffff);
    check(b"ashr var", (s >> k) == -(1i128 << 50));
    check(b"lt", a < b && !(b < a));
    check(b"slt", s < 0 && (s as u128) > a);
    check(b"eq", a == opaque(a) && a != b);
    check(b"zext", (k as u128) << 100 == 77u128 << 100);
    check(b"sext", (opaque(-3i32) as i128) == -3);
    check(b"trunc", (a as u64) == 0xfedc_ba98_7654_3210 && (a as u32) == 0x7654_3210);
    let (p, q, r) = mul_div(a, opaque(1_000_000_007));
    check(b"mul", p == 0x79be_0246_864e_3073_8641_fdb9_796d_fe70);
    check(b"div", q * 1_000_000_007 + r == a && r < 1_000_000_007);
    let (_, q2, r2) = mul_div(b, a);
    check(b"div big", q2 * a + r2 == b && r2 < a);
    check(b"sdiv", opaque(-1_000_000_000_000_000_000_000i128) / 7 == -142_857_142_857_142_857_142);
    check(b"srem", opaque(-1_000_000_000_000_000_000_000i128) % 7 == -6);
    check(b"leading", a.leading_zeros() == 7);
    let _ = writeln!(Out, "{} {} {:x}", a, s, b);
    unsafe { FAILS }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
