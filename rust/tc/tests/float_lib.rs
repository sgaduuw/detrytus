//! The floating-point functions detrytus ll adds to a program
//! (src/ll/lower/float.ll), run on the bare machine against the host's
//! answers for edge cases and pseudo-random inputs. A NaN may be any
//! NaN; everything else must have the same bits.

use std::fmt::Write;
use std::path::Path;

use detrytus_core::{Machine, Outcome};

/// A small xorshift generator, so that the cases are the same each run.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    /// A double of any bits, or a short one near the integers.
    fn f64(&mut self) -> f64 {
        let r = self.next();
        match r % 4 {
            0 => f64::from_bits(self.next()),
            1 => (self.next() as i64 >> 40) as f64 / 8.0,
            2 => (self.next() as i64 >> 10) as f64 / 1024.0,
            _ => f64::from_bits(self.next() >> 2) * if r & 16 == 0 { 1.0 } else { -1.0 },
        }
    }
    fn f32(&mut self) -> f32 {
        let r = self.next();
        match r % 4 {
            0 => f32::from_bits(self.next() as u32),
            1 => (self.next() as i32 >> 20) as f32 / 8.0,
            _ => self.f64() as f32,
        }
    }
    fn i128(&mut self) -> i128 {
        let v = ((self.next() as u128) << 64 | self.next() as u128) as i128;
        v >> (self.next() % 127)
    }
}

const F64_EDGES: &[f64] = &[
    0.0,
    -0.0,
    0.5,
    -0.5,
    1.5,
    -1.5,
    2.5,
    -2.5,
    0.49999999999999994,
    1.0,
    -1.0,
    4503599627370495.5,
    4503599627370496.0,
    -4503599627370497.0,
    9007199254740993.0,
    f64::MIN_POSITIVE,
    5e-324,
    -5e-324,
    f64::MAX,
    f64::MIN,
    f64::INFINITY,
    f64::NEG_INFINITY,
    f64::NAN,
    1.7e38,
    -1.7e38,
    3.4e38,
    1e19,
    -1e19,
];
const F32_EDGES: &[f32] = &[
    0.0,
    -0.0,
    0.5,
    -0.5,
    1.5,
    -2.5,
    8388607.5,
    8388608.0,
    1e-45,
    f32::MIN_POSITIVE,
    f32::MAX,
    f32::MIN,
    f32::INFINITY,
    f32::NEG_INFINITY,
    f32::NAN,
    1.7e38,
    -1.7e38,
];

/// A value as the IR type it is passed as, and its bits as a word.
#[derive(Clone, Copy)]
enum V {
    D(f64),
    F(f32),
    I(i128),
}

impl V {
    fn ty(self) -> &'static str {
        match self {
            V::D(_) => "double",
            V::F(_) => "float",
            V::I(_) => "i128",
        }
    }
    fn carrier(self) -> &'static str {
        match self {
            V::D(_) => "i64",
            V::F(_) => "i32",
            V::I(_) => "i128",
        }
    }
    fn bits(self) -> String {
        match self {
            V::D(x) => (x.to_bits() as i64).to_string(),
            V::F(x) => (x.to_bits() as i32).to_string(),
            V::I(x) => x.to_string(),
        }
    }
    fn is_nan(self) -> bool {
        match self {
            V::D(x) => x.is_nan(),
            V::F(x) => x.is_nan(),
            V::I(_) => false,
        }
    }
}

/// One function over a table of cases: (arguments, expected answer).
struct Table {
    name: &'static str,
    cases: Vec<(Vec<V>, V)>,
}

fn global(out: &mut String, name: &str, vals: impl Iterator<Item = V>, carrier: &str) {
    let vals: Vec<String> = vals.map(|v| format!("{carrier} {}", v.bits())).collect();
    writeln!(out, "@{name} = private constant [{} x {carrier}] [{}]", vals.len(), vals.join(", "))
        .unwrap();
}

/// A program whose main runs every table, printing the name and the
/// case number of each mismatch and returning how many there were.
fn program(tables: &[Table]) -> String {
    let mut g = String::new();
    let mut body =
        String::from("define i32 @main(i32 %argc, ptr %argv) {\nstart:\n  br label %t0\n");
    for (k, t) in tables.iter().enumerate() {
        let (args, ret) = &t.cases[0];
        let n = t.cases.len();
        for j in 0..args.len() {
            global(&mut g, &format!("a{k}_{j}"), t.cases.iter().map(|c| c.0[j]), args[j].carrier());
        }
        global(&mut g, &format!("r{k}"), t.cases.iter().map(|c| c.1), ret.carrier());
        let nan: Vec<String> =
            t.cases.iter().map(|c| format!("i8 {}", c.1.is_nan() as u8)).collect();
        writeln!(g, "@n{k} = private constant [{n} x i8] [{}]", nan.join(", ")).unwrap();
        let label = format!("{}#", t.name);
        writeln!(g, "@s{k} = private constant [{} x i8] c\"{label}\"", label.len()).unwrap();
        let fails_in = if k == 0 { "0".to_string() } else { format!("%f{}", k - 1) };
        let mut call = Vec::new();
        let mut loads = String::new();
        for (j, a) in args.iter().enumerate() {
            let c = a.carrier();
            write!(
                loads,
                "  %p{k}_{j} = getelementptr inbounds {c}, ptr @a{k}_{j}, i32 %i{k}\n\
                 \x20 %w{k}_{j} = load {c}, ptr %p{k}_{j}\n"
            )
            .unwrap();
            if a.ty() == c {
                call.push(format!("{c} %w{k}_{j}"));
            } else {
                writeln!(loads, "  %v{k}_{j} = bitcast {c} %w{k}_{j} to {}", a.ty()).unwrap();
                call.push(format!("{} %v{k}_{j}", a.ty()));
            }
        }
        let (rt, rc) = (ret.ty(), ret.carrier());
        let got = if rt == rc { format!("%x{k}") } else { format!("%y{k}") };
        write!(
            body,
            "t{k}:\n  br label %l{k}\n\
             l{k}:\n\
             \x20 %i{k} = phi i32 [ 0, %t{k} ], [ %i{k}n, %c{k} ]\n\
             \x20 %e{k} = phi i32 [ {fails_in}, %t{k} ], [ %e{k}n, %c{k} ]\n\
             {loads}\
             \x20 %x{k} = call {rt} @{}({})\n",
            t.name,
            call.join(", ")
        )
        .unwrap();
        if rt != rc {
            writeln!(body, "  %y{k} = bitcast {rt} %x{k} to {rc}").unwrap();
        }
        write!(
            body,
            "  %q{k} = getelementptr inbounds {rc}, ptr @r{k}, i32 %i{k}\n\
             \x20 %z{k} = load {rc}, ptr %q{k}\n\
             \x20 %same{k} = icmp eq {rc} {got}, %z{k}\n\
             \x20 %np{k} = getelementptr inbounds i8, ptr @n{k}, i32 %i{k}\n\
             \x20 %nb{k} = load i8, ptr %np{k}\n\
             \x20 %wantnan{k} = icmp ne i8 %nb{k}, 0\n"
        )
        .unwrap();
        if rt == rc {
            writeln!(body, "  %isnan{k} = icmp eq i32 0, 1").unwrap();
        } else {
            writeln!(body, "  %isnan{k} = fcmp uno {rt} %x{k}, %x{k}").unwrap();
        }
        write!(
            body,
            "  %bothnan{k} = and i1 %wantnan{k}, %isnan{k}\n\
             \x20 %ok{k} = or i1 %same{k}, %bothnan{k}\n\
             \x20 br i1 %ok{k}, label %c{k}, label %b{k}\n\
             b{k}:\n\
             \x20 call i32 @write(i32 1, ptr @s{k}, i32 {})\n\
             \x20 call void @num(i32 %i{k})\n\
             \x20 br label %c{k}\n\
             c{k}:\n\
             \x20 %bad{k} = xor i1 %ok{k}, 1\n\
             \x20 %inc{k} = zext i1 %bad{k} to i32\n\
             \x20 %e{k}n = add i32 %e{k}, %inc{k}\n\
             \x20 %i{k}n = add i32 %i{k}, 1\n\
             \x20 %more{k} = icmp ult i32 %i{k}n, {n}\n\
             \x20 br i1 %more{k}, label %l{k}, label %d{k}\n\
             d{k}:\n\
             \x20 %f{k} = add i32 %e{k}n, 0\n\
             \x20 br label %t{}\n",
            label.len(),
            k + 1
        )
        .unwrap();
    }
    writeln!(body, "t{}:\n  ret i32 %f{}\n}}", tables.len(), tables.len() - 1).unwrap();
    // num(i): the case number in decimal and a newline.
    let num = "@digits = private global [12 x i8] zeroinitializer\n\
        define void @num(i32 %v) {\nstart:\n  br label %loop\n\
        loop:\n  %n = phi i32 [ %v, %start ], [ %q, %loop ]\n\
        \x20 %k = phi i32 [ 11, %start ], [ %k1, %loop ]\n\
        \x20 %q = udiv i32 %n, 10\n  %r = urem i32 %n, 10\n\
        \x20 %c = add i32 %r, 48\n  %b = trunc i32 %c to i8\n\
        \x20 %k1 = sub i32 %k, 1\n\
        \x20 %p = getelementptr inbounds i8, ptr @digits, i32 %k1\n\
        \x20 store i8 %b, ptr %p\n  %more = icmp ne i32 %q, 0\n\
        \x20 br i1 %more, label %loop, label %done\n\
        done:\n  %nl = getelementptr inbounds i8, ptr @digits, i32 11\n\
        \x20 store i8 10, ptr %nl\n  %len = sub i32 12, %k1\n\
        \x20 %s = getelementptr inbounds i8, ptr @digits, i32 %k1\n\
        \x20 call i32 @write(i32 1, ptr %s, i32 %len)\n  ret void\n}\n\
        declare i32 @write(i32, ptr, i32)\n";
    format!("{g}{body}\n{num}")
}

fn run(tables: &[Table]) -> (i64, String) {
    let src = program(tables);
    let bare = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/ll/bare.s");
    let out = detrytus_tc::ll::translate_text(&src, &[bare.display().to_string()])
        .unwrap_or_else(|e| panic!("does not translate: {e}\n{src}"));
    let image = detrytus_asm::assemble(&out.assembly, Some("float_lib.s"))
        .unwrap_or_else(|e| panic!("does not assemble: {e}"));
    let mut machine = Machine::with_harts(10, 1, 1);
    for (addr, tryte) in image.trytes {
        machine.bus.poke(addr, tryte);
    }
    match machine.run(200_000_000) {
        Outcome::Exit { code, .. } => (code, machine.bus.console.text()),
        other => panic!("{other:?}"),
    }
}

fn unary64(name: &'static str, f: fn(f64) -> f64, rng: &mut Rng, n: usize) -> Table {
    let xs = F64_EDGES.iter().copied().chain((0..n).map(|_| rng.f64()));
    Table { name, cases: xs.map(|x| (vec![V::D(x)], V::D(f(x)))).collect() }
}

fn unary32(name: &'static str, f: fn(f32) -> f32, rng: &mut Rng, n: usize) -> Table {
    let xs = F32_EDGES.iter().copied().chain((0..n).map(|_| rng.f32()));
    Table { name, cases: xs.map(|x| (vec![V::F(x)], V::F(f(x)))).collect() }
}

fn minimum64(x: f64, y: f64) -> f64 {
    if x.is_nan() || y.is_nan() {
        f64::NAN
    } else if x < y || (x == y && x.is_sign_negative()) {
        x
    } else {
        y
    }
}

fn maximum64(x: f64, y: f64) -> f64 {
    if x.is_nan() || y.is_nan() {
        f64::NAN
    } else if x > y || (x == y && x.is_sign_positive()) {
        x
    } else {
        y
    }
}

fn binary64(name: &'static str, f: fn(f64, f64) -> f64, rng: &mut Rng, n: usize) -> Table {
    let mut cases = Vec::new();
    for &x in F64_EDGES {
        for &y in F64_EDGES {
            cases.push((vec![V::D(x), V::D(y)], V::D(f(x, y))));
        }
    }
    for _ in 0..n {
        let (x, y) = (rng.f64(), rng.f64());
        cases.push((vec![V::D(x), V::D(y)], V::D(f(x, y))));
    }
    Table { name, cases }
}

fn binary32(name: &'static str, f: fn(f32, f32) -> f32, rng: &mut Rng, n: usize) -> Table {
    let mut cases = Vec::new();
    for &x in F32_EDGES {
        for &y in F32_EDGES {
            cases.push((vec![V::F(x), V::F(y)], V::F(f(x, y))));
        }
    }
    for _ in 0..n {
        let (x, y) = (rng.f32(), rng.f32());
        cases.push((vec![V::F(x), V::F(y)], V::F(f(x, y))));
    }
    Table { name, cases }
}

#[test]
fn rounding_to_integers() {
    let r = &mut Rng(0x9e3779b97f4a7c15);
    let tables = [
        unary64("trunc", f64::trunc, r, 200),
        unary64("floor", f64::floor, r, 200),
        unary64("ceil", f64::ceil, r, 200),
        unary64("round", f64::round, r, 200),
        unary64("rint", f64::round_ties_even, r, 200),
        unary64("roundeven", f64::round_ties_even, r, 50),
        unary32("truncf", f32::trunc, r, 100),
        unary32("floorf", f32::floor, r, 100),
        unary32("ceilf", f32::ceil, r, 100),
        unary32("roundf", f32::round, r, 100),
        unary32("rintf", f32::round_ties_even, r, 100),
        unary32("roundevenf", f32::round_ties_even, r, 50),
    ];
    assert_eq!(run(&tables), (0, String::new()));
}

#[test]
fn minimum_and_maximum() {
    let r = &mut Rng(0x1234_5678_9abc_def1);
    let tables = [
        binary64("fminimum", minimum64, r, 100),
        binary64("fmaximum", maximum64, r, 100),
        binary32("fminimumf", |x, y| minimum64(x as f64, y as f64) as f32, r, 100),
        binary32("fmaximumf", |x, y| maximum64(x as f64, y as f64) as f32, r, 100),
    ];
    assert_eq!(run(&tables), (0, String::new()));
}

#[test]
fn remainders() {
    let r = &mut Rng(0x0dd_ba11_cafe_f00d);
    let tables = [binary64("fmod", |x, y| x % y, r, 300), binary32("fmodf", |x, y| x % y, r, 300)];
    assert_eq!(run(&tables), (0, String::new()));
}

#[test]
fn conversions_with_128_bit_integers() {
    let r = &mut Rng(0xfeed_face_dead_beef);
    let xs: Vec<f64> = F64_EDGES.iter().copied().chain((0..200).map(|_| r.f64())).collect();
    let ys: Vec<f32> = F32_EDGES.iter().copied().chain((0..200).map(|_| r.f32())).collect();
    let mut ns: Vec<i128> = vec![0, 1, -1, i128::MAX, i128::MIN, 1 << 64, (1 << 64) - 1];
    // Ties and near-ties below the last place kept, each way.
    ns.extend([
        (1i128 << 70) + (1 << 17),
        (1i128 << 70) + (1 << 17) + 1,
        (1i128 << 70) + (3 << 17),
    ]);
    ns.extend((0..300).map(|_| r.i128()));
    let tables = [
        Table {
            name: "__fixdfti",
            cases: xs.iter().map(|&x| (vec![V::D(x)], V::I(x as i128))).collect(),
        },
        Table {
            name: "__fixunsdfti",
            cases: xs.iter().map(|&x| (vec![V::D(x)], V::I(x as u128 as i128))).collect(),
        },
        Table {
            name: "__fixsfti",
            cases: ys.iter().map(|&x| (vec![V::F(x)], V::I(x as i128))).collect(),
        },
        Table {
            name: "__fixunssfti",
            cases: ys.iter().map(|&x| (vec![V::F(x)], V::I(x as u128 as i128))).collect(),
        },
        Table {
            name: "__floattidf",
            cases: ns.iter().map(|&n| (vec![V::I(n)], V::D(n as f64))).collect(),
        },
        Table {
            name: "__floatuntidf",
            cases: ns.iter().map(|&n| (vec![V::I(n)], V::D(n as u128 as f64))).collect(),
        },
        Table {
            name: "__floattisf",
            cases: ns.iter().map(|&n| (vec![V::I(n)], V::F(n as f32))).collect(),
        },
        Table {
            name: "__floatuntisf",
            cases: ns.iter().map(|&n| (vec![V::I(n)], V::F(n as u128 as f32))).collect(),
        },
    ];
    assert_eq!(run(&tables), (0, String::new()));
}
