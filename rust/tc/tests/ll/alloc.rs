//! The alloc crate: Vec, String, Box and format! over a global
//! allocator, here a bump allocator in a static array; and the
//! intrinsics alloc reaches: multiplication and addition that report
//! overflow, bit and byte reversal; and sorting Strings, whose
//! comparison tests the sign bit of memcmp's result.
#![no_std]

extern crate alloc;

use alloc::{boxed::Box, format, string::String, vec::Vec};
use core::alloc::{GlobalAlloc, Layout};
use core::fmt::Write;
use core::ptr::{addr_of, addr_of_mut, read_volatile as get};

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

const SIZE: usize = 30000;
static mut HEAP: [u8; SIZE] = [0; SIZE];
static mut BASE: *mut u8 = addr_of_mut!(HEAP) as *mut u8;
static mut USED: usize = 0;

struct Bump;

unsafe impl GlobalAlloc for Bump {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        unsafe {
            let base = get(addr_of!(BASE)) as usize;
            let start = (base + USED + l.align() - 1) & !(l.align() - 1);
            if start + l.size() > base + SIZE {
                return core::ptr::null_mut();
            }
            USED = start + l.size() - base;
            start as *mut u8
        }
    }
    unsafe fn dealloc(&self, _p: *mut u8, _l: Layout) {}
}

#[global_allocator]
static ALLOCATOR: Bump = Bump;

static mut X: u32 = 0x1234_5678;
static mut Y: u32 = 70_000;
static mut Z: i32 = 2_000_000_000;

#[unsafe(no_mangle)]
pub extern "C" fn main(argc: i32, _argv: *const *const u8) -> i32 {
    let mut o = Out;
    let mut v: Vec<u32> = Vec::new();
    for i in 0..30 {
        v.push(i * i);
    }
    let words: Vec<String> = ["one", "two", "three"].iter().map(|w| format!("<{w}>")).collect();
    let mut s = String::from("joined:");
    for w in &words {
        s.push(' ');
        s.push_str(w);
    }
    let b = Box::new(argc as u64 * 1_000_000_007);
    let _ = writeln!(o, "{} {:?} {} {}", v.iter().sum::<u32>(), &v[..4], s, b);
    let (x, y, z) = unsafe { (get(addr_of!(X)), get(addr_of!(Y)), get(addr_of!(Z))) };
    let _ = writeln!(
        o,
        "{:?} {:?} {:?} {:?} {:x} {:x} {:?}",
        y.checked_mul(y),
        y.checked_mul(60_000),
        z.checked_add(z),
        z.checked_sub(-z),
        x.reverse_bits(),
        x.swap_bytes(),
        (z as i32).overflowing_add(147_483_648),
    );
    let mut x = x;
    let mut lines: Vec<String> = Vec::new();
    for _ in 0..200 {
        x = x.wrapping_mul(1103515245).wrapping_add(12345);
        lines.push(format!("line {}", (x >> 16) % 1000));
    }
    lines.sort_unstable();
    let ok = lines.windows(2).all(|w| w[0] <= w[1]);
    let _ = writeln!(o, "sorted {} {} {}", ok, lines[0], lines[199]);
    0
}
