//! Integer arithmetic on values the compiler cannot see (read through
//! volatile loads), checked against what rustc worked out at compile
//! time. Exits with the number of checks that failed, naming each.
#![no_std]

use core::ptr::read_volatile as get;

static mut A: u32 = 0xDEAD_BEEF;
static mut B: u32 = 0x1234_5678;
static mut C: i32 = -7;
static mut D: i32 = 3;
static mut E: u8 = 200;
static mut F: u16 = 65000;
static mut S: u32 = 5;
static mut FAILS: i32 = 0;

unsafe extern "C" {
    fn write(fd: i32, buf: *const u8, n: usize) -> isize;
}

#[inline(never)]
fn check(name: &[u8], got: u32, want: u32) {
    if got != want {
        unsafe {
            write(1, name.as_ptr(), name.len());
            write(1, b"\n".as_ptr(), 1);
            FAILS += 1;
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    unsafe {
        let a = get(&raw const A);
        let b = get(&raw const B);
        let c = get(&raw const C);
        let d = get(&raw const D);
        let e = get(&raw const E);
        let f = get(&raw const F);
        let s = get(&raw const S);
        check(b"add", a.wrapping_add(b), 0xDEAD_BEEFu32.wrapping_add(0x1234_5678));
        check(b"sub", b.wrapping_sub(a), 0x1234_5678u32.wrapping_sub(0xDEAD_BEEF));
        check(b"mul", a.wrapping_mul(b), 0xDEAD_BEEFu32.wrapping_mul(0x1234_5678));
        check(b"mul small", a.wrapping_mul(10), 0xDEAD_BEEFu32.wrapping_mul(10));
        check(b"imul", c.wrapping_mul(b as i32) as u32, (-7i32).wrapping_mul(0x1234_5678) as u32);
        check(b"udiv", a / b, 0xDEAD_BEEF / 0x1234_5678);
        check(b"urem", a % b, 0xDEAD_BEEF % 0x1234_5678);
        check(b"sdiv", (c / d) as u32, (-7i32 / 3) as u32);
        check(b"srem", (c % d) as u32, (-7i32 % 3) as u32);
        check(b"shl", a << s, 0xDEAD_BEEFu32 << 5);
        check(b"shl 20", a << 20, 0xDEAD_BEEFu32 << 20);
        check(b"shl 27", a << (s + 22), 0xDEAD_BEEFu32 << 27);
        check(b"lshr", a >> s, 0xDEAD_BEEFu32 >> 5);
        check(b"ashr", (c >> (s - 4)) as u32, (-7i32 >> 1) as u32);
        check(b"and", a & b, 0xDEAD_BEEF & 0x1234_5678);
        check(b"or", a | b, 0xDEAD_BEEF | 0x1234_5678);
        check(b"xor", a ^ b, 0xDEAD_BEEF ^ 0x1234_5678);
        check(b"not", !a, !0xDEAD_BEEFu32);
        check(b"neg", c.wrapping_neg() as u32, 7);
        check(b"u8 add", e.wrapping_add(100) as u32, 44);
        check(b"u16 mul", f.wrapping_mul(3) as u32, 65000u16.wrapping_mul(3) as u32);
        check(b"sext", e as i8 as i32 as u32, 200u8 as i8 as i32 as u32);
        check(b"slt", (c < d) as u32, 1);
        check(b"ult", (a < b) as u32, 0);
        check(b"sge", (c >= d) as u32, 0);
        check(b"eq", (a == b) as u32, 0);
        check(b"min", a.min(b), 0x1234_5678);
        check(b"smax", c.max(d) as u32, 3);
        check(b"abs", c.unsigned_abs(), 7);
        check(b"sat sub", b.saturating_sub(a), 0);
        check(b"sat add", a.saturating_add(a), u32::MAX);
        check(b"sat add 2", a.saturating_add(b), 0xDEAD_BEEFu32.saturating_add(0x1234_5678));
        check(b"checked", a.checked_add(a).is_none() as u32, 1);
        check(b"checked 2", a.checked_add(b).unwrap_or(0), 0xDEAD_BEEF + 0x1234_5678);
        check(b"overflowing", a.overflowing_sub(b).1 as u32, 0);
        FAILS
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
