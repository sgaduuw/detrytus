//! flat_map: a scan over records and the descriptors in each, as
//! trylobite's kernel first wrote its descriptor scan, against the same
//! scan as nested loops. The flattening iterator keeps its inner range
//! in memory behind a pointer, with 64-bit indices.
#![no_std]

unsafe extern "C" {
    fn write(fd: i32, buf: *const u8, n: usize) -> isize;
}

fn out(s: &[u8]) {
    unsafe { write(1, s.as_ptr(), s.len()) };
}

const NPROC: i64 = 27;
const NDESC: i64 = 9;

static mut TABLE: [[i64; 2 * NDESC as usize]; NPROC as usize] =
    [[0; 2 * NDESC as usize]; NPROC as usize];

#[derive(Clone, Copy)]
struct Proc(*mut i64);

impl Proc {
    fn of(pid: i64) -> Proc {
        Proc((&raw mut TABLE).cast::<i64>().wrapping_offset((pid * 2 * NDESC) as isize))
    }
    fn table() -> impl Iterator<Item = Proc> {
        (0..NPROC).map(Proc::of)
    }
}

#[derive(Clone, Copy)]
struct Desc(*mut i64);

impl Desc {
    fn of(p: Proc, fd: i64) -> Desc {
        Desc(p.0.wrapping_offset((2 * fd) as isize))
    }
    fn all(p: Proc) -> impl Iterator<Item = Desc> {
        (0..NDESC).map(move |fd| Desc::of(p, fd))
    }
    fn server(self) -> i64 {
        unsafe { self.0.read_volatile() }
    }
    fn handle(self) -> i64 {
        unsafe { self.0.add(1).read_volatile() }
    }
    fn set(self, s: i64, h: i64) {
        unsafe {
            self.0.write_volatile(s);
            self.0.add(1).write_volatile(h);
        }
    }
}

#[inline(never)]
fn named_flat(server: i64, handle: i64) -> bool {
    Proc::table().flat_map(Desc::all).any(|d| d.server() == server && d.handle() == handle)
}

#[inline(never)]
fn named_loops(server: i64, handle: i64) -> bool {
    for q in Proc::table() {
        if Desc::all(q).any(|d| d.server() == server && d.handle() == handle) {
            return true;
        }
    }
    false
}

#[inline(never)]
fn count_flat() -> i32 {
    Proc::table().flat_map(Desc::all).count() as i32
}

#[inline(never)]
fn sum_flat() -> i64 {
    Proc::table().flat_map(Desc::all).map(|d| d.server()).sum()
}

fn bit(b: bool) -> u8 {
    if b { b'1' } else { b'0' }
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    Desc::of(Proc::of(3), 0).set(5, 2);
    Desc::of(Proc::of(26), 8).set(7, 1);
    Desc::of(Proc::of(0), 4).set(9, 0);
    let mut s = [b' '; 16];
    let cases = [(5, 2), (7, 1), (9, 0), (5, 1), (0, 0), (8, 3)];
    for (i, &(sv, h)) in cases.iter().enumerate() {
        s[i] = bit(named_flat(sv, h));
        s[i + 7] = bit(named_loops(sv, h));
    }
    s[6] = b'/';
    s[13] = b'\n';
    out(&s[..14]);
    let n = count_flat() as u8;
    let t = sum_flat() as u8;
    let digits = [
        b'0' + n / 100,
        b'0' + n / 10 % 10,
        b'0' + n % 10,
        b' ',
        b'0' + t / 10,
        b'0' + t % 10,
        b'\n',
    ];
    out(&digits);
    0
}
