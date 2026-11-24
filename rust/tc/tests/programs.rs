//! Trice programs compiled, assembled and run on the bare machine
//! (tests/bare.s, and again with a global pointer, tests/bare_gp.s),
//! and programs the compiler must refuse.

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use detrytus_core::{Machine, Outcome};

static COUNT: AtomicUsize = AtomicUsize::new(0);

/// A program's file: `bare.s` imported and `putc` declared ahead of `src`.
fn write(src: &str) -> PathBuf {
    write_with(src, "bare.s")
}

fn write_with(src: &str, start: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "detrytus-tc-{}-{}",
        std::process::id(),
        COUNT.fetch_add(1, Ordering::SeqCst)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let bare = format!("{}/tests/{start}", env!("CARGO_MANIFEST_DIR"));
    let path = dir.join("main.tc");
    let text = format!("import \"{bare}\";\nextern void putc(int c);\n{src}");
    std::fs::write(&path, text).unwrap();
    path
}

fn compile(src: &str) -> Result<detrytus_tc::Output, detrytus_tc::Error> {
    let path = write(src);
    detrytus_tc::compile(&path, path.parent().unwrap())
}

/// The exit code and console output of running `src`, the same with
/// and without a global pointer.
fn run(src: &str) -> (i64, String) {
    let plain = run_with(src, "bare.s");
    assert_eq!(run_with(src, "bare_gp.s"), plain, "with .gp");
    plain
}

fn run_with(src: &str, start: &str) -> (i64, String) {
    let path = write_with(src, start);
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
    // Arguments and results are cut to their types too.
    let src = "
        int t(trit x) { return x; }
        tryte r(int x) { return x; }
        int main() { int two = 2; return t(two) * 100000 + r(9842); }";
    assert_eq!(exit(src), -100_000 - 9841);
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

#[test]
fn functions_nothing_calls_are_left_out() {
    let out = compile("int unused() { return 1; } int main() { return 0; }").unwrap();
    assert!(!out.assembly.contains("unused:"));
    // putc is reached from main, and an asm block may name a function.
    let out = compile(
        "int called() { return named() - 1; } int named() { return 3; }
         int main() { putc(called()); asm {\n call named # not unused2\n} return 0; }
         int unused2() { return 4; }",
    )
    .unwrap();
    assert!(out.assembly.contains("called:") && out.assembly.contains("named:"));
    assert!(!out.assembly.contains("unused2:"));
    // They are still checked.
    assert_eq!(refused("int unused() { return x; } int main() { return 0; }"), "unknown name x");
}

#[test]
fn structs_follow_the_abi_layout() {
    let src = "
        struct example { int a; tryte b; tryte c; int d; };
        struct bytes { tryte x; tryte y; tryte z; };
        int main() { return sizeof(struct example) * 10 + sizeof(struct bytes); }";
    assert_eq!(exit(src), 93);
}

#[test]
fn struct_fields_and_pointers() {
    let src = "
        struct point { int x; tryte tag; int y; };
        struct point g;
        int area(struct point *p) { return p->x * p->y; }
        int main() {
            struct point p;
            p.x = 6;
            p.y = 7;
            p.tag = 'k';
            g.x = 2;
            g.y = p.x;
            struct point *q = &g;
            q->tag = 1;
            return area(&p) * 1000 + area(q) * 10 + g.tag + (p.tag == 'k' ? 100000 : 0);
        }";
    assert_eq!(exit(src), 100_000 + 42_000 + 120 + 1);
}

#[test]
fn structs_nest_and_make_arrays() {
    let src = "
        struct inner { tryte a; int b; };
        struct outer { tryte t; struct inner in; struct inner more[3]; };
        struct outer o[2];
        int main() {
            for (int i = 0; i < 2; i++) {
                o[i].t = i;
                o[i].in.b = 10 * i;
                for (int k = 0; k < 3; k++) o[i].more[k].b = 100 * i + k;
            }
            struct outer *p = o;
            p++;
            struct inner *m = &p->more[2];
            return p->in.b + m->b * 100 + (&o[1] - &o[0]) * 100000;
        }";
    assert_eq!(exit(src), 10 + 10_200 + 100_000);
}

#[test]
fn structs_are_copied_by_assignment() {
    let src = "
        struct small { int a; int b; };
        struct big { int w[20]; tryte last; };
        struct big x;
        int main() {
            struct small s = { 3, 4 };
            struct small t;
            t = s;
            s.a = 9;
            struct big y;
            for (int i = 0; i < 20; i++) x.w[i] = i;
            x.last = 5;
            y = x;
            x.w[19] = 0;
            struct small u = t;
            return u.a * 1000 + t.b * 100 + y.w[19] + y.last * 10 + s.a * 0;
        }";
    assert_eq!(exit(src), 3000 + 400 + 19 + 50);
    let src = "
        struct trytes { tryte a; tryte b; tryte c; tryte d; };
        int main() {
            struct trytes p = { 1, 2, 3 };
            struct trytes q;
            q.d = 9;
            q = p;
            return q.a * 1000 + q.b * 100 + q.c * 10 + q.d;
        }";
    assert_eq!(exit(src), 1230);
}

#[test]
fn global_initialisers() {
    let src = "
        int primes[5] = { 2, 3, 5, 7 };
        tryte word[6] = \"tern\";
        struct entry { tryte kind; int value; tryte *name; };
        struct entry table[2] = { { 'a', -5, \"one\" }, { 'b', 3812798742493 } };
        int *p = &primes[0];
        int main() {
            putc(word[0]);
            putc(*table[0].name);
            return primes[3] * 1000 + primes[4] + word[3] * 0 + table[0].kind
                 + table[0].value * 10 + (table[1].value == 3812798742493 ? 100000 : 0)
                 + (table[1].name == 0 ? 1000000 : 0) + (word[4] == 0 ? 10000000 : 0);
        }";
    assert_eq!(run(src), (7000 + 'a' as i64 - 50 + 100_000 + 1_000_000 + 10_000_000, "to".into()));
}

#[test]
fn local_initialisers() {
    let src = "
        struct pair { tryte k; int v; };
        int main() {
            int a[4] = { 1, 2 };
            tryte s[5] = \"hey\";
            struct pair ps[2] = { { 'x', 7 } };
            int n = 0;
            while (s[n] != 0) n++;
            return a[0] * 1000 + a[1] * 100 + a[2] * 10 + a[3] + n * 10000
                 + ps[0].v * 100000 + ps[1].v + ps[1].k;
        }";
    assert_eq!(exit(src), 1200 + 30_000 + 700_000);
}

#[test]
fn calls_with_more_than_six_arguments() {
    let src = "
        int nine(int a, int b, int c, int d, int e, int f, tryte g, int h, tryte i) {
            return a + 2*b + 3*c + 4*d + 5*e + 6*f + 7*g + 8*h + 9*i;
        }
        int id(int x) { return x; }
        int main() {
            int one = 1;
            return 1000 * id(1) + nine(1, 1, 1, 1, id(1), 1, one, id(nine(0, 0, 0, 0, 0, 0, 0, 0, 1) - 8), 1);
        }";
    assert_eq!(exit(src), 1000 + 45);
}

#[test]
fn struct_errors() {
    assert_eq!(refused("struct s x; int main() { return 0; }"), "struct s is not defined");
    assert_eq!(
        refused("struct s { int a; }; int main() { struct s v; return v.b; }"),
        "struct s has no field b"
    );
    assert_eq!(
        refused("struct s { int a; }; int main() { struct s v; struct s *p = &v; return p.a; }"),
        "struct s* is a pointer: use ->a"
    );
    assert_eq!(
        refused("struct s { int a; }; int f(struct s v) { return 0; } int main() { return 0; }"),
        "pass a pointer to the struct: v cannot be one yet"
    );
    assert_eq!(
        refused("int a[2] = { 1, 2, 3 }; int main() { return 0; }"),
        "3 initialisers for 2 elements"
    );
    assert_eq!(
        refused("tryte s[3] = \"abc\"; int main() { return 0; }"),
        "the string needs 4 trytes with its 0"
    );
    assert_eq!(
        refused("struct s { int a; }; int main() { struct s v; return v + 1; }"),
        "+ does not take a struct s and an int"
    );
}

#[test]
fn locals_live_in_registers_and_memory() {
    // Twelve locals: eight fit the saved registers, the rest the frame;
    // one whose address is taken stays in memory.
    let src = "
        void set(int *p) { *p = 5; }
        int main() {
            int a = 1; int b = 2; int c = 3; int d = 4; int e = 5; int f = 6;
            int g = 7; int h = 8; int i = 9; int j = 10; int k = 11; int m = 0;
            set(&m);
            for (int n = 0; n < 3; n++) { int x = n; a += x; }
            { int y = 100; b += y; }
            tryte t = 9841;
            t++;
            trit u = 1;
            u += 1;
            return a + b + c + d + e + f + g + h + i + j + k + m * 1000 + t + u * 10000;
        }";
    assert_eq!(exit(src), 4 + 102 + 3 + 4 + 5 + 6 + 7 + 8 + 9 + 10 + 11 + 5000 - 9841 - 10000);
    // A leaf keeps its parameters and locals in argument registers.
    let src = "
        int sum3(int a, int b, int c) { int s = a + b; s += c; return s; }
        int main() { int x = 4; return sum3(x, x + 1, sum3(1, 1, 1)) + x; }";
    assert_eq!(exit(src), 4 + 5 + 3 + 4);
    let leaf = "int leaf(int a) { int b = a * 2; b = b * b; b = b + a; b = b * a; return b - 3; }";
    let out = compile(&format!("{leaf} int main() {{ return leaf(3) + leaf(4); }}")).unwrap();
    let leaf = out.assembly.split("leaf:").nth(1).unwrap().split("main:").next().unwrap();
    assert!(!leaf.contains("sp"), "a leaf needs no frame:\n{leaf}");
}

#[test]
fn small_functions_are_copied_into_their_callers() {
    let src = "int twice(int a) { return a + a; } int main() { return twice(3) + twice(4); }";
    assert_eq!(exit(src), 14);
    let out = compile(src).unwrap();
    assert!(!out.assembly.contains("twice:"), "{}", out.assembly);
    assert!(!out.assembly.contains("call"), "{}", out.assembly);
    // One called from a single place, whatever its size; labels and
    // registers of each copy are its own.
    let src = "
        int big(int a) { int s = 0; while (a > 0) { s += a * a; a--; } return s; }
        int small(int a) { if (a < 0) return -a; return a; }
        int main() { return big(3) + small(-2) + small(5); }";
    assert_eq!(exit(src), 14 + 2 + 5);
    let out = compile(src).unwrap();
    assert!(!out.assembly.contains("big:") && !out.assembly.contains("small:"));
    // A function that calls is not copied.
    let src = "int fact(int n) { if (n < 2) return 1; return n * fact(n - 1); }
               int main() { return fact(4); }";
    assert_eq!(exit(src), 24);
    assert!(compile(src).unwrap().assembly.contains("fact:"));
}

#[test]
fn reading_a_variable_waits_for_earlier_arguments() {
    let src = "
        int g;
        int bump() { g = g + 10; return 1; }
        int two(int a, int b) { return a * 100 + b; }
        int main() {
            int x = 1;
            int r = two(x++, x);
            g = 5;
            return r * 1000 + two(bump(), g);
        }";
    assert_eq!(exit(src), 102 * 1000 + 115);
}

/// A library in its own file, for compiling separately: a struct, a
/// constant, a global with an initialiser, and functions, one of which
/// nothing in it calls.
const LIBRARY: &str = "
    import \"bare.s\";
    extern void putc(int c);
    struct pair { int a; tryte b; };
    const int SCALE = 10;
    int total = 5;
    struct pair last;
    void add(int n) { total = total + n * SCALE; last.a = n; }
    void say(tryte *s) { while (s[0] != 0) { putc(s[0]); s = s + 1; } }
    int unused(int x) { return x + 1; }
";

/// The program that uses it: the library's declarations come from
/// importing it.
const USER: &str = "
    int main() {
        add(3);
        add(4);
        say(\"linked\\n\");
        return total + last.a;
    }
";

/// Write the library beside a copy of the start code, and the program
/// importing both; the program's path.
fn write_pair(start: &str) -> PathBuf {
    let main = write_with(USER, start);
    let dir = main.parent().unwrap();
    let bare = format!("{}/tests/{start}", env!("CARGO_MANIFEST_DIR"));
    std::fs::copy(bare, dir.join("bare.s")).unwrap();
    std::fs::write(dir.join("lib.tc"), LIBRARY.replace("import \"bare.s\";", "")).unwrap();
    let text = std::fs::read_to_string(&main).unwrap();
    std::fs::write(&main, text.replace("extern void putc", "import \"lib.tc\";\nextern void putc"))
        .unwrap();
    main
}

/// Compile the program and its library each on its own, link them and
/// run the result: its exit code and console output.
fn link_run(start: &str) -> (i64, String) {
    let main = write_pair(start);
    let dir = main.parent().unwrap();
    let mut objects = Vec::new();
    for path in [main.clone(), dir.join("lib.tc")] {
        let out = detrytus_tc::compile_unit(&path, dir)
            .unwrap_or_else(|e| panic!("does not compile: {e}"));
        let shown = path.with_extension("s").display().to_string();
        let (object, _) = detrytus_asm::assemble_object(&out.assembly, Some(&shown))
            .unwrap_or_else(|e| panic!("does not assemble: {e}\n{}", out.assembly));
        objects.push((shown, object));
    }
    let linked = detrytus_asm::link::link(&objects).unwrap_or_else(|e| panic!("{e}"));
    let mut machine = Machine::with_harts(10, 1, 1);
    for (addr, tryte) in linked.image.trytes {
        machine.bus.poke(addr, tryte);
    }
    match machine.run(2_000_000) {
        Outcome::Exit { code, .. } => (code, machine.bus.console.text()),
        other => panic!("{other:?}"),
    }
}

#[test]
fn files_compiled_separately_link_into_the_same_program() {
    let whole = {
        let main = write_pair("bare.s");
        let out = detrytus_tc::compile(&main, main.parent().unwrap()).unwrap();
        let shown = main.with_extension("s").display().to_string();
        let image = detrytus_asm::assemble(&out.assembly, Some(&shown)).unwrap();
        let mut machine = Machine::with_harts(10, 1, 1);
        for (addr, tryte) in image.trytes {
            machine.bus.poke(addr, tryte);
        }
        match machine.run(2_000_000) {
            Outcome::Exit { code, .. } => (code, machine.bus.console.text()),
            other => panic!("{other:?}"),
        }
    };
    assert_eq!(whole, (5 + 30 + 40 + 4, "linked\n".to_string()));
    assert_eq!(link_run("bare.s"), whole);
    // With a global pointer, the library's globals are reached from gp
    // across objects.
    assert_eq!(link_run("bare_gp.s"), whole);
}

#[test]
fn a_file_compiled_on_its_own_exports_everything_it_defines() {
    let main = write_pair("bare.s");
    let lib = main.parent().unwrap().join("lib.tc");
    let out = detrytus_tc::compile_unit(&lib, main.parent().unwrap()).unwrap();
    // Nothing here calls unused, but another file may.
    assert!(out.assembly.contains(".global total, last, add, say, unused"), "{}", out.assembly);
    assert!(out.assembly.contains("unused:"));
    // The program declares what it imports, and defines none of it.
    let out = detrytus_tc::compile_unit(&main, main.parent().unwrap()).unwrap();
    assert!(out.assembly.contains(".global main\n"), "{}", out.assembly);
    assert!(!out.assembly.contains("add:") && !out.assembly.contains("total:"));
}
