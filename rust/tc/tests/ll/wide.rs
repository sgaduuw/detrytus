//! 64-bit integers, two words each: arithmetic on values the compiler
//! cannot see (read through volatile loads), checked against what rustc
//! worked out at compile time; calls with more than six argument words;
//! counting leading zeros. Exits with the number of checks that failed,
//! naming each.
#![no_std]

use core::ptr::read_volatile as get;

static mut A: u64 = 0xDEAD_BEEF_0123_4567;
static mut B: u64 = 0x0000_0001_FFFF_FFFF;
static mut C: i64 = -9_876_543_210;
static mut X: u32 = 0x8000_0001;
static mut S: u32 = 7;
static mut Z: u32 = 0x0000_1000;
static mut FAILS: i32 = 0;

unsafe extern "C" {
    fn write(fd: i32, buf: *const u8, n: usize) -> isize;
}

#[inline(never)]
fn check(name: &[u8], got: u64, want: u64) {
    if got != want {
        unsafe {
            write(1, name.as_ptr(), name.len());
            write(1, b"\n".as_ptr(), 1);
            FAILS += 1;
        }
    }
}

/// Eight argument words: two go on the stack.
#[inline(never)]
fn many(a: u64, b: u64, c: u64, d: u64) -> u64 {
    a.wrapping_sub(b) ^ c.wrapping_add(d)
}

#[unsafe(no_mangle)]
pub extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    unsafe {
        let a = get(&raw const A);
        let b = get(&raw const B);
        let c = get(&raw const C);
        let x = get(&raw const X);
        let s = get(&raw const S);
        let z = get(&raw const Z);
        const KA: u64 = 0xDEAD_BEEF_0123_4567;
        const KB: u64 = 0x0000_0001_FFFF_FFFF;
        const KC: i64 = -9_876_543_210;
        check(b"add", a.wrapping_add(b), KA.wrapping_add(KB));
        check(b"sub", b.wrapping_sub(a), KB.wrapping_sub(KA));
        check(b"mul", a.wrapping_mul(b), KA.wrapping_mul(KB));
        check(b"mul const", a.wrapping_mul(10000), KA.wrapping_mul(10000));
        check(b"udiv", a / 10000, KA / 10000);
        check(b"urem", a % 10000, KA % 10000);
        check(b"udiv 7", b / 7, KB / 7);
        check(b"shl 4", a << 4, KA << 4);
        check(b"shl 40", a << 40, KA << 40);
        check(b"lshr 4", a >> 4, KA >> 4);
        check(b"lshr 36", a >> 36, KA >> 36);
        check(b"ashr 4", (c >> 4) as u64, (KC >> 4) as u64);
        check(b"ashr 40", (c >> 40) as u64, (KC >> 40) as u64);
        check(b"udiv any", a / b, KA / KB);
        check(b"urem any", a % b, KA % KB);
        check(b"udiv top", a / (a - 1), KA / (KA - 1));
        check(b"sdiv", (c / (s as i64)) as u64, (KC / 7) as u64);
        check(b"srem", (c % (s as i64)) as u64, (KC % 7) as u64);
        check(b"shl var", a << s, KA << 7);
        check(b"shl var 39", a << (s + 32), KA << 39);
        check(b"lshr var", a >> (s + 30), KA >> 37);
        check(b"ashr var", (c >> s) as u64, (KC >> 7) as u64);
        check(b"ashr var 40", (c >> (s + 33)) as u64, (KC >> 40) as u64);
        check(b"and", a & b, KA & KB);
        check(b"or", a | b, KA | KB);
        check(b"xor", a ^ b, KA ^ KB);
        check(b"neg", c.wrapping_neg() as u64, 9_876_543_210);
        check(b"abs", c.unsigned_abs(), 9_876_543_210);
        check(b"zext", x as u64, 0x8000_0001);
        check(b"sext", x as i32 as i64 as u64, 0x8000_0001u32 as i32 as i64 as u64);
        check(b"trunc", (a as u32) as u64, KA as u32 as u64);
        check(b"wide mul", (x as u64) * (x as u64), 0x8000_0001u64 * 0x8000_0001);
        check(b"ult", (b < a) as u64, 1);
        check(b"ugt", (b > a) as u64, 0);
        check(b"slt", (c < 0) as u64, 1);
        check(b"sgt", (c > -1) as u64, 0);
        check(b"eq", (a == KA) as u64, 1);
        check(b"ne", (b != KB) as u64, 0);
        check(b"many", many(a, b, a, b), KA.wrapping_sub(KB) ^ KA.wrapping_add(KB));
        check(b"ctlz", x.leading_zeros() as u64, 0);
        check(b"ctlz 2", z.leading_zeros() as u64, 19);
        check(b"ctlz 3", (s as u8).leading_zeros() as u64, 5);
        FAILS
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
