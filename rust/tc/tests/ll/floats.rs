//! Floating point, done in software: arithmetic, comparisons, casts and
//! formatting of f32 and f64 on values the compiler cannot see through.
//! Prints what it formats; exits with the number of failed checks,
//! naming each.
#![no_std]
#![feature(core_float_math)]

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

#[unsafe(no_mangle)]
pub extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    let x: f64 = opaque(1.5);
    let y: f64 = opaque(-0.25);
    let a: f32 = opaque(3.0);
    let b: f32 = opaque(0.1);
    let n: i32 = opaque(-7);
    let big: u64 = opaque(1 << 60);
    check(b"add", x + y == 1.25);
    check(b"sub", x - y == 1.75);
    check(b"mul", x * y == -0.375);
    check(b"div", x / y == -6.0);
    check(b"rem", opaque(7.5f64) % 2.0 == 1.5);
    check(b"neg", -x == -1.5 && (-y).to_bits() == 0.25f64.to_bits());
    check(b"f32", a * b == 0.3f32 + f32::EPSILON / 4.0 || a * b == 0.3f32 || (a * b).to_bits() == 0x3e99999a);
    check(b"f32 div", a / opaque(2.0f32) == 1.5);
    check(b"cmp", y < x && x > y && x >= x && !(x < x) && x != y);
    let nan = opaque(f64::NAN);
    check(b"nan", nan != nan && !(nan < x) && !(nan >= x) && nan.is_nan());
    check(b"inf", (x / opaque(0.0)).is_infinite());
    check(b"to int", (opaque(-2.75f64) as i32) == -2 && (opaque(3.99f32) as u8) == 3);
    check(b"to int sat", (opaque(1e20f64) as i32) == i32::MAX && (nan as i64) == 0 && (opaque(-5.0f32) as u32) == 0);
    check(b"from int", n as f64 == -7.0 && big as f64 == 1152921504606846976.0 && (n as f32) == -7.0);
    check(b"to i64", (opaque(-1e15f64) as i64) == -1_000_000_000_000_000);
    check(b"ext trunc", (b as f64) as f32 == b && (x as f32) == 1.5);
    check(b"sqrt", core::f64::math::sqrt(opaque(2.25)) == 1.5);
    check(b"floor", core::f64::math::floor(opaque(-1.5)) == -2.0 && core::f32::math::ceil(opaque(1.2f32)) == 2.0);
    check(b"abs", opaque(-3.5f64).abs() == 3.5 && opaque(2.0f64).copysign(-1.0) == -2.0);
    check(b"minmax", x.max(y) == x && x.min(y) == y);
    check(b"fma", core::f64::math::mul_add(x, opaque(2.0), y) == 2.75);
    let _ = writeln!(Out, "{} {} {} {:.3} {:e}", x, y, b, core::f64::consts::PI, 6.02e23f64);
    unsafe { FAILS }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
