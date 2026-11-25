//! Memory, aggregates and control flow: structs and arrays in memory,
//! slices passed and returned, a match, recursion, pointers in static
//! data. Prints what it finds, and exits with the number of checks
//! that failed.
#![no_std]

use core::ptr::read_volatile as get;

unsafe extern "C" {
    fn write(fd: i32, buf: *const u8, n: usize) -> isize;
}

fn out(s: &[u8]) {
    unsafe { write(1, s.as_ptr(), s.len()) };
}

static mut FAILS: i32 = 0;

#[inline(never)]
fn check(name: &[u8], ok: bool) {
    if !ok {
        out(name);
        out(b" failed\n");
        unsafe { FAILS += 1 };
    }
}

#[derive(Clone, Copy)]
struct Mixed {
    a: u8,
    b: u32,
    c: u16,
    d: i8,
}

static mut M: Mixed = Mixed { a: 7, b: 0xCAFE_F00D, c: 0xBEEF, d: -3 };
static WORDS: [&[u8]; 3] = [b"zero", b"one", b"two"];

/// Decimal digits of n, which may be negative.
#[inline(never)]
fn decimal(n: i32, buf: &mut [u8; 12]) -> &[u8] {
    let mut i = buf.len();
    let mut m = n.unsigned_abs();
    loop {
        i -= 1;
        buf[i] = b'0' + (m % 10) as u8;
        m /= 10;
        if m == 0 {
            break;
        }
    }
    if n < 0 {
        i -= 1;
        buf[i] = b'-';
    }
    &buf[i..]
}

#[inline(never)]
fn trim(s: &[u8]) -> &[u8] {
    let mut a = 0;
    let mut b = s.len();
    while a < b && s[a] == b' ' {
        a += 1;
    }
    while b > a && s[b - 1] == b' ' {
        b -= 1;
    }
    &s[a..b]
}

#[inline(never)]
fn classify(c: u8) -> u32 {
    match c {
        b'a' | b'e' | b'i' | b'o' | b'u' => 1,
        b'0'..=b'9' => 2,
        b' ' => 3,
        b'\n' => 4,
        _ => 0,
    }
}

#[inline(never)]
fn find(s: &[u8], c: u8) -> Option<usize> {
    s.iter().position(|&x| x == c)
}

#[inline(never)]
fn fib(n: u32) -> u32 {
    if n < 2 { n } else { fib(n - 1) + fib(n - 2) }
}

#[unsafe(no_mangle)]
pub extern "C" fn main(argc: i32, argv: *const *const u8) -> i32 {
    let mut buf = [0u8; 12];
    unsafe {
        // Field by field: a volatile read of the whole 8-byte struct is
        // one i64 load to LLVM 1.100's rustc, and 64 bits are not
        // supported yet.
        let p = &raw const M;
        let m = Mixed {
            a: get(&raw const (*p).a),
            b: get(&raw const (*p).b),
            c: get(&raw const (*p).c),
            d: get(&raw const (*p).d),
        };
        check(b"struct u8", m.a == 7);
        check(b"struct u32", m.b == 0xCAFE_F00D);
        check(b"struct u16", m.c == 0xBEEF);
        check(b"struct i8", m.d == -3);
        let p = &raw mut M;
        (*p).b = m.b.wrapping_add(m.c as u32);
        core::ptr::write_volatile(&raw mut (*p).d, m.d - 1);
        let (b2, d2) = (get(&raw const (*p).b), get(&raw const (*p).d));
        check(b"struct store", b2 == 0xCAFE_F00Du32.wrapping_add(0xBEEF) && d2 == -4);
    }
    let args = unsafe { core::slice::from_raw_parts(argv, argc as usize) };
    let arg = unsafe { core::ffi::CStr::from_ptr(args[1] as *const core::ffi::c_char) }.to_bytes();
    out(b"arg: [");
    out(trim(arg));
    out(b"]\n");
    let word = u32::from_le_bytes([arg[0], arg[1], arg[2], arg[3]]);
    check(b"from_le_bytes", word == u32::from_le_bytes(*b" hi "));
    let mut back = [0u8; 4];
    for (b, w) in back.iter_mut().zip(word.to_le_bytes()) {
        *b = w;
    }
    check(b"to_le_bytes", &back == b" hi ");
    let mut counts = [0u32; 5];
    for &c in arg {
        counts[classify(c) as usize] += 1;
    }
    check(b"classify", counts == [4, 3, 1, 5, 0]);
    check(b"find", find(arg, b't') == Some(4) && find(arg, b'z').is_none());
    let mut rev = [0u8; 16];
    let n = arg.len();
    for (i, &c) in arg.iter().enumerate() {
        rev[n - 1 - i] = c;
    }
    out(&rev[..n]);
    out(b"\n");
    let mut squares = [0u16; 10];
    for (i, s) in squares.iter_mut().enumerate() {
        *s = (i * i * 700) as u16;
    }
    let sum: u32 = squares.iter().map(|&s| s as u32).sum();
    check(b"u16 array", sum == (0..10u32).map(|i| (i * i * 700) as u16 as u32).sum());
    check(b"fib", fib(15) == 610);
    out(WORDS[argc as usize]);
    out(b" ");
    out(decimal(-1234 * argc, &mut buf));
    out(b" ");
    out(decimal(fib(20) as i32, &mut buf));
    out(b"\n");
    unsafe { FAILS }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
