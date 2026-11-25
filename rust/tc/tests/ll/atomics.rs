//! Atomics: loads, stores, read-modify-write, compare and exchange, on
//! 8- and 32-bit values the compiler cannot see through. Exits with
//! the number of checks that failed, naming each.
#![no_std]

use core::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, AtomicU8, Ordering::*, fence};

static FLAG: AtomicBool = AtomicBool::new(false);
static N: AtomicU32 = AtomicU32::new(40);
static S: AtomicI32 = AtomicI32::new(-5);
static B: AtomicU8 = AtomicU8::new(0xF0);
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

#[unsafe(no_mangle)]
pub extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    check(b"load", N.load(Acquire) as u64, 40);
    N.store(41, Release);
    check(b"store", N.load(Relaxed) as u64, 41);
    check(b"fetch_add", N.fetch_add(2, SeqCst) as u64, 41);
    check(b"after add", N.load(SeqCst) as u64, 43);
    check(b"fetch_sub", N.fetch_sub(50, AcqRel) as u64, 43);
    check(b"after sub", N.load(SeqCst) as u64, 43u32.wrapping_sub(50) as u64);
    check(b"swap", N.swap(7, SeqCst) as u64, 43u32.wrapping_sub(50) as u64);
    check(b"cas ok", N.compare_exchange(7, 9, SeqCst, Relaxed).is_ok() as u64, 1);
    check(b"cas fail", N.compare_exchange(7, 11, SeqCst, Relaxed).unwrap_err() as u64, 9);
    check(b"fetch_max", S.fetch_max(3, SeqCst) as u32 as u64, -5i32 as u32 as u64);
    check(b"fetch_min", S.fetch_min(-8, SeqCst) as u64, 3);
    check(b"after min", S.load(SeqCst) as u32 as u64, -8i32 as u32 as u64);
    check(b"fetch_umax", N.fetch_max(100, SeqCst) as u64, 9);
    check(b"fetch_and", B.fetch_and(0x3C, SeqCst) as u64, 0xF0);
    check(b"fetch_or", B.fetch_or(0x01, SeqCst) as u64, 0x30);
    check(b"fetch_xor", B.fetch_xor(0xFF, SeqCst) as u64, 0x31);
    check(b"fetch_nand", B.fetch_nand(0x0F, SeqCst) as u64, 0xCE);
    check(b"after nand", B.load(SeqCst) as u64, !(0xCEu8 & 0x0F) as u64);
    check(b"flag", FLAG.swap(true, SeqCst) as u64, 0);
    check(b"flag set", FLAG.load(SeqCst) as u64, 1);
    fence(SeqCst);
    let mut v = N.load(Relaxed);
    loop {
        match N.compare_exchange_weak(v, v * 3, SeqCst, Relaxed) {
            Ok(_) => break,
            Err(now) => v = now,
        }
    }
    check(b"cas loop", N.load(SeqCst) as u64, 300);
    unsafe { FAILS }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
