//! Trait objects: calls through function pointers in vtables, which
//! live in static data as addresses.
#![no_std]

unsafe extern "C" {
    fn write(fd: i32, buf: *const u8, n: usize) -> isize;
}

fn out(s: &[u8]) {
    unsafe { write(1, s.as_ptr(), s.len()) };
}

trait Shape {
    fn area(&self) -> u32;
    fn name(&self) -> &[u8];
}

struct Square(u32);
struct Rect(u32, u32);

impl Shape for Square {
    fn area(&self) -> u32 {
        self.0 * self.0
    }
    fn name(&self) -> &[u8] {
        b"square"
    }
}

impl Shape for Rect {
    fn area(&self) -> u32 {
        self.0 * self.1
    }
    fn name(&self) -> &[u8] {
        b"rect"
    }
}

#[inline(never)]
fn apply(f: fn(u32) -> u32, x: u32) -> u32 {
    f(x)
}

fn double(x: u32) -> u32 {
    2 * x
}

fn square(x: u32) -> u32 {
    x * x
}

#[unsafe(no_mangle)]
pub extern "C" fn main(argc: i32, _argv: *const *const u8) -> i32 {
    let n = argc as u32;
    let shapes: [&dyn Shape; 3] = [&Square(n + 5), &Rect(3, n), &Square(n)];
    let mut total = 0;
    for s in shapes {
        out(s.name());
        out(b" ");
        total += s.area();
    }
    out(b"\n");
    let fs: [fn(u32) -> u32; 2] = [double, square];
    total += apply(fs[(n & 1) as usize], 10) + apply(fs[1 - (n & 1) as usize], 3);
    // 49 + 6 + 4, then double(10) = 20 and square(3) = 9 when argc is 2.
    total as i32 - 88
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
