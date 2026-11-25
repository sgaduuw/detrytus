//! core::fmt: write! into a sink that calls write, with integers,
//! strings, padding and Debug.
#![no_std]

use core::fmt::Write;

unsafe extern "C" {
    fn write(fd: i32, buf: *const u8, n: usize) -> isize;
}

struct Out;

impl Write for Out {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        unsafe { write(1, s.as_ptr(), s.len()) };
        Ok(())
    }
}

#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

#[unsafe(no_mangle)]
pub extern "C" fn main(argc: i32, _argv: *const *const u8) -> i32 {
    let mut o = Out;
    let big: u64 = 12345678901234567890;
    let neg: i64 = -9876543210;
    let _ = writeln!(o, "{} {} {}", argc, -42i32, 4000000000u32);
    let _ = writeln!(o, "{big} {neg} {:x} {:#b}", 0xdead_beefu32, 5u8);
    let _ = writeln!(o, "[{:>6}] [{:<4}] [{:^7}] [{:05}]", "ab", 7, "mid", -3);
    let _ = writeln!(o, "{:?} {:?} {:?}", Point { x: 1, y: -2 }, "q\"s", Some(3u16));
    0
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
