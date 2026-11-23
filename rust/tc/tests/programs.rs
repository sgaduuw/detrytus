//! Trice programs compiled, assembled and run on the bare machine
//! (tests/bare.s), and programs the compiler must refuse.

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use detrytus_core::{Machine, Outcome};

static COUNT: AtomicUsize = AtomicUsize::new(0);

/// A program's file: `bare.s` imported and `putc` declared ahead of `src`.
fn write(src: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "detrytus-tc-{}-{}",
        std::process::id(),
        COUNT.fetch_add(1, Ordering::SeqCst)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let bare = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/bare.s");
    let path = dir.join("main.tc");
    let text = format!("import \"{bare}\";\nextern void putc(int c);\n{src}");
    std::fs::write(&path, text).unwrap();
    path
}

fn compile(src: &str) -> Result<detrytus_tc::Output, detrytus_tc::Error> {
    let path = write(src);
    detrytus_tc::compile(&path, path.parent().unwrap())
}

/// The exit code and console output of running `src`.
fn run(src: &str) -> (i64, String) {
    let path = write(src);
    let out = detrytus_tc::compile(&path, path.parent().unwrap())
        .unwrap_or_else(|e| panic!("does not compile: {e}"));
    let shown = path.with_extension("s").display().to_string();
    let image = detrytus_asm::assemble(&out.assembly, Some(&shown))
        .unwrap_or_else(|e| panic!("does not assemble: {e}\n{}", out.assembly));
    let mut machine = Machine::with_harts(10, 1, 1);
    for (addr, tryte) in image.trytes {
        machine.bus.poke(addr, tryte);
    }
    match machine.run(2_000_000) {
        Outcome::Exit { code, .. } => (code, machine.bus.console.text()),
        other => panic!("{other:?}\n{}", out.assembly),
    }
}

fn exit(src: &str) -> i64 {
    run(src).0
}

fn refused(src: &str) -> String {
    match compile(src) {
        Ok(_) => panic!("compiled, but should not have"),
        Err(e) => e.message,
    }
}

#[test]
fn main_returns_an_exit_code() {
    assert_eq!(exit("int main() { return 3; }"), 3);
    assert_eq!(exit("int main() { return -7; }"), -7);
}

#[test]
fn arithmetic_is_on_words_and_wraps() {
    assert_eq!(exit("int main() { return 2 + 3 * 4 - 20 / 3; }"), 8);
    assert_eq!(exit("int main() { int a = -7; return a % 3; }"), -1);
    assert_eq!(exit("int main() { int a = 5; int b = 0; return a / b; }"), 0);
    // H27 + 1 wraps to -H27.
    assert_eq!(
        exit(
            "int main() { int a = 3812798742493; a = a + 1; return a == -3812798742493 ? 1 : 0; }"
        ),
        1
    );
}

#[test]
fn shifts_are_by_trits() {
    assert_eq!(exit("int main() { int x = 5; return x << 2; }"), 45);
    assert_eq!(exit("int main() { int x = 50; return x >> 2; }"), 6); // 50 / 9 rounds to 6
    assert_eq!(exit("int main() { int x = 50; int n = 2; return x >> n; }"), 6);
    assert_eq!(exit("int main() { int x = 1; int n = 3; return x << n; }"), 27);
}

#[test]
fn trit_wise_operators() {
    // tmin and tmax trit by trit: 0t+0- & 0t0++ is 0t00-; | is 0t+++.
    assert_eq!(exit("int main() { return 0t+0- & 0t0++; }"), -1);
    assert_eq!(exit("int main() { int a = 0t+0-; return a | 0t0++; }"), 13);
    assert_eq!(exit("int main() { int a = 4; return -a; }"), -4);
}

#[test]
fn three_way_compare_and_which() {
    let src = "
        int sign(int a, int b) {
            which (a <=> b) {
                < : return -1;
                = : return 0;
                > : return 1;
            }
            return 9;
        }
        int main() { return 100 * sign(1, 2) + 10 * sign(5, 5) + sign(7, 3); }";
    assert_eq!(exit(src), -99);
    assert_eq!(exit("int main() { trit t = 5 <=> 9; return t; }"), -1);
}

#[test]
fn bool_has_three_values_and_if_takes_only_true() {
    let src = "
        int test(bool b) {
            if (b) return 1; else return 2;
        }
        int main() { return 100 * test(true) + 10 * test(unknown) + test(false); }";
    assert_eq!(exit(src), 122);
    let src = "
        int show(bool b) {
            which (b) { true: return 1; unknown: return 0; false: return -1; }
            return 9;
        }
        int main() {
            bool u = unknown;
            return 100 * show(u && true) + 10 * show(u && false) + show(u || true);
        }";
    // unknown && true is unknown (0), unknown && false is false, unknown || true is true.
    assert_eq!(exit(src), -10 + 1);
    assert_eq!(exit("int main() { bool b = !unknown; return (int) b; }"), 0);
    assert_eq!(exit("int main() { bool b = !true; return (int) b; }"), -1);
    // A fresh global flag is unknown.
    assert_eq!(exit("bool done; int main() { return (int) done; }"), 0);
}

#[test]
fn comparisons_as_values() {
    let src = "
        int t(bool b) { return (int) b; }
        int main() {
            int a = 3; int b = 5;
            return t(a < b) + 3 * t(a <= b) + 9 * t(a > b) + 27 * t(a >= b)
                 + 81 * t(a == b) + 243 * t(a != b);
        }";
    assert_eq!(exit(src), 1 + 3 - 9 - 27 - 81 + 243);
    let same = src.replace("int b = 5", "int b = 3");
    assert_eq!(exit(&same), -1 + 3 - 9 + 27 + 81 - 243);
    assert_eq!(exit("int main() { int a = 4; bool e = a == 4; return (int) e; }"), 1);
}

#[test]
fn and_or_short_circuit() {
    let src = "
        int calls;
        bool yes() { calls++; return true; }
        int main() {
            bool b = false && yes();
            b = true || yes();
            b = true && yes();
            if (false && yes()) calls = 100;
            if (true || yes()) calls += 10;
            return calls;
        }";
    assert_eq!(exit(src), 11);
}

#[test]
fn loops_break_and_continue() {
    let src = "
        int main() {
            int sum = 0;
            for (int i = 0; i < 10; i++) {
                if (i == 3) continue;
                if (i == 8) break;
                sum += i;
            }
            int n = 0;
            while (true) { n++; if (n >= 5) break; }
            return sum * 10 + n;
        }";
    assert_eq!(exit(src), (28 - 3) * 10 + 5);
}

#[test]
fn recursion() {
    let src = "
        int fib(int n) { if (n < 2) return n; return fib(n - 1) + fib(n - 2); }
        int main() { return fib(15); }";
    assert_eq!(exit(src), 610);
}

#[test]
fn calls_keep_the_values_around_them() {
    let src = "
        int id(int x) { return x; }
        int six(int a, int b, int c, int d, int e, int f) { return a + 2*b + 3*c + 4*d + 5*e + 6*f; }
        int main() { return 1000 * id(1) + 100 * id(2) + six(1, id(1), 1, 1, id(1), 1); }";
    assert_eq!(exit(src), 1000 + 200 + 21);
}

#[test]
fn arrays_and_pointers() {
    let src = "
        int g[5];
        tryte buf[7];
        int main() {
            int a[4];
            for (int i = 0; i < 4; i++) a[i] = i * i;
            int *p = a;
            p++;
            for (int i = 0; i < 5; i++) g[i] = 10 + i;
            buf[6] = 42;
            tryte *q = &buf[6];
            int *e = &a[3];
            return a[3] + *p * 100 + g[4] * 1000 + *q * 100000 + (e - p) * 10000000;
        }";
    assert_eq!(exit(src), 9 + 100 + 14_000 + 4_200_000 + 20_000_000);
}

#[test]
fn strings_and_the_console() {
    let src = "
        void puts(tryte *s) { while (*s != 0) putc(*s++); }
        int strlen(tryte *s) { int n = 0; while (s[n] != 0) n++; return n; }
        int main() { puts(\"hello, \\\"Setun\\\"\\n\"); return strlen(\"four\"); }";
    assert_eq!(run(src), (4, "hello, \"Setun\"\n".to_string()));
}

#[test]
fn trytes_narrow_when_stored() {
    let src = "int main() { tryte t = 9841; t = t + 1; return t; }";
    assert_eq!(exit(src), -9841);
    assert_eq!(exit("int main() { int x = 9842; return (tryte) x; }"), -9841);
    assert_eq!(exit("int main() { int x = 2; return (trit) x; }"), -1);
}

#[test]
fn constants_and_sizeof() {
    let src = "
        const int N = 3 * 4;
        int main() { tryte b[N]; int w[N]; return sizeof b + sizeof(int) * 100 + sizeof w * 1000; }";
    assert_eq!(exit(src), 12 + 300 + 36_000);
}

#[test]
fn constants_come_from_imported_assembly() {
    // bare.s defines CONSOLE_OUT.
    assert_eq!(exit("int main() { return CONSOLE_OUT; }"), -19683);
}

#[test]
fn builtins() {
    assert_eq!(exit("int main() { return min(4, -2) + 10 * max(4, -2) + 100 * abs(-3); }"), 338);
    assert_eq!(exit("int main() { return divr(7, 2); }"), 3);
    assert_eq!(exit("int main() { return tzt(81); }"), 4);
    assert_eq!(exit("int main() { return band(12, 10); }"), 8);
}

#[test]
fn inline_assembly() {
    assert_eq!(exit("int main() { asm {\n li a0, 17\n j main$ret\n} return 0; }"), 17);
}

#[test]
fn errors() {
    assert_eq!(
        refused("int main() { int n = 1; if (n) return 1; return 0; }"),
        "a condition must be a bool, not an int: compare it, as in x != 0"
    );
    assert_eq!(refused("int main() { return x; }"), "unknown name x");
    assert_eq!(
        refused("int f(int a) { return a; } int main() { return f(); }"),
        "f takes 1 arguments, not 0"
    );
    assert_eq!(
        refused("int main() { int a = 1; return ~a; }"),
        "Trice has no ~: -x already flips every trit"
    );
    assert_eq!(refused("int main() { int a0 = 1; return a0; }"), "a0 is a register name");
    assert_eq!(refused("int main() { bool b = 1; return 0; }"), "cannot use an int as a bool");
    assert_eq!(
        refused("int main() { int x = 1; bool b = (bool) x; return 0; }"),
        "cannot cast an int to a bool: write a comparison, as in x != 0"
    );
    assert_eq!(refused("void f() { return 1; }"), "f returns nothing");
    assert_eq!(refused("int main() { break; }"), "break and continue belong in a loop");
    assert_eq!(
        refused("int CONSOLE_OUT; int main() { return 0; }"),
        "CONSOLE_OUT is already a constant from imported assembly"
    );
}

#[test]
fn errors_name_the_file_and_line() {
    let e = compile("int main() {\n  return y;\n}").unwrap_err();
    assert!(e.file.ends_with("main.tc"));
    assert_eq!(e.line, 4); // two lines of preamble come first
}

#[test]
fn comparing_with_minus_one_warns() {
    let out = compile("int main() { int x = 1; if (x == -1) return 1; return 0; }").unwrap();
    assert_eq!(out.warnings.len(), 1);
    assert!(out.warnings[0].contains("warning: -1 is an ordinary value"));
    let out = compile(
        "const int NONE = -1; int main() { int x = 1; if (x == NONE) return 1; return 0; }",
    )
    .unwrap();
    assert!(out.warnings.is_empty());
}
