//! Compiled code must do exactly what the interpreter does: the same
//! output, exit, step count and registers, for every example and for
//! code that rewrites itself.

use std::cell::Cell;
use std::rc::Rc;

use detrytus_core::accel::Accelerator;
use detrytus_core::bus::Bus;
use detrytus_core::hart::Hart;
use detrytus_core::{Machine, Outcome};
use detrytus_jit::Jit;

/// The JIT, counting the instructions it ran.
struct Counting(Jit, Rc<Cell<u64>>);

impl Accelerator for Counting {
    fn run(&mut self, hart: &mut Hart, bus: &mut Bus, budget: u64) -> u64 {
        let ran = self.0.run(hart, bus, budget);
        self.1.set(self.1.get() + ran);
        ran
    }
}

thread_local! {
    static COMPILED: Rc<Cell<u64>> = Rc::new(Cell::new(0));
}

const SELF_MODIFYING: &str = r#"
        li    t0, 19683
        li    t5, 0
again:  li    t2, first
        lw    t1, 0(t2)
        sw    t1, 0(t0)
        lw    t1, 3(t2)
        sw    t1, 3(t0)
        li    t3, 50
warm:   jalr  ra, 0(t0)          # hot: compiled
        addi  t3, t3, -1
        bne   t3, r0, warm
        add   t5, t5, a0
        li    t2, second
        lw    t1, 0(t2)
        sw    t1, 0(t0)          # rewrite the compiled code
        fence.i
        jalr  ra, 0(t0)
        add   a0, a0, t5
        li    t6, -39366
        sw    a0, 0(t6)
first:  addi  a0, r0, 1
        jalr  r0, 0(ra)
second: addi  a0, r0, 20
"#;

const LOOP: &str = r#"
        li    t0, 39366          # sum memory words in a loop, with calls
        li    t1, 0
        li    t2, 300
fill:   sw    t2, 0(t0)
        addi  t0, t0, 3
        addi  t2, t2, -1
        bne   t2, r0, fill
        li    t0, 39366
        li    t2, 300
        li    a0, 0
sum:    lw    t3, 0(t0)
        jal   ra, add3
        addi  t0, t0, 3
        addi  t2, t2, -1
        bgt   t2, r0, sum
        mul   a0, a0, a0
        li    t6, -39366
        sw    a0, 0(t6)
add3:   add   a0, a0, t3
        cmp   t4, a0, t3
        sel   t4, t4, a0, t3
        jalr  r0, 0(ra)
"#;

/// Loads and stores of trytes and words, with values too wide for a
/// tryte, ending in a misaligned load that traps; and the arithmetic
/// compiled inline, at the edges of the word range. (Stores to decoded
/// code: SELF_MODIFYING.)
const MEMORY: &str = r#"
        li    t0, 59049          # a data page
        li    t1, 3800000000000  # beyond a tryte, near the word's top
        li    t2, 40
        li    a0, 0
fill:   sw    t1, 0(t0)
        st    t1, 3(t0)          # keeps the low tryte
        st    t2, 4(t0)
        lw    t3, 0(t0)
        lt    t4, 3(t0)
        lt    t5, 4(t0)
        add   a0, a0, t3
        add   a0, a0, t4
        sub   a0, a0, t5
        mul   t3, t1, t3         # wraps
        mul   t4, t3, t2
        add   a0, a0, t4
        div   t4, t3, t2
        rem   t5, t3, t2
        add   a0, a0, t4
        add   a0, a0, t5
        div   t4, t3, r0         # by zero: 0
        rem   t5, t3, r0         # by zero: the dividend
        add   a0, a0, t4
        sub   a0, a0, t5
        band  t4, t3, t1
        bor   t5, t3, t2
        add   a0, a0, t4
        add   a0, a0, t5
        sub   t6, r0, t3
        bxor  t4, t6, t1
        add   a0, a0, t4
        addi  t0, t0, 6
        addi  t1, t1, -12345678
        addi  t2, t2, -1
        bne   t2, r0, fill
        li    t6, 59049
        lw    t4, -2(t6)         # misaligned: left to the interpreter
"#;

fn run_file(
    source: &str,
    file: Option<&str>,
    input: &str,
    jit: bool,
) -> (Outcome, String, [i64; 27]) {
    let image = detrytus_asm::assembler::assemble(source, file).expect("assembles");
    let mut m = Machine::new(729, 1);
    for (&addr, &tryte) in &image.trytes {
        m.bus.poke(addr, tryte);
    }
    m.bus.console.feed(input);
    m.bus.console.closed = true;
    if jit {
        let count = COMPILED.with(Rc::clone);
        m.set_accelerator(Some(Box::new(Counting(Jit::with_threshold(1), count))));
    }
    let outcome = m.run(10_000_000);
    (outcome, m.bus.console.text(), m.boot_hart().regs())
}

fn run(source: &str, input: &str, jit: bool) -> (Outcome, String, [i64; 27]) {
    run_file(source, None, input, jit)
}

fn same(name: &str, source: &str, file: Option<&str>, input: &str) {
    let plain = run_file(source, file, input, false);
    let compiled = run_file(source, file, input, true);
    assert_eq!(plain, compiled, "{name}");
}

#[test]
fn compiled_code_matches_the_interpreter() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples");
    for name in ["hello.s", "bsearch.s", "echo.s", "paging.s", "sections.s"] {
        let path = format!("{root}/{name}");
        let source = std::fs::read_to_string(&path).unwrap();
        same(name, &source, Some(&path), "Setun, 1958\n");
    }
    let before = COMPILED.with(|c| c.get());
    same("loop", LOOP, None, "");
    let ran = COMPILED.with(|c| c.get()) - before;
    assert!(ran > 2000, "the loop ran compiled ({ran} instructions)");
}

#[test]
fn rewritten_code_runs_as_rewritten() {
    let (outcome, _, _) = run(SELF_MODIFYING, "", true);
    assert_eq!(outcome, run(SELF_MODIFYING, "", false).0);
    assert!(matches!(outcome, Outcome::Exit { code: 21, .. }), "{outcome:?}");
}

#[test]
fn memory_and_arithmetic_match_the_interpreter() {
    let (outcome, _, regs) = run(MEMORY, "", true);
    assert!(matches!(outcome, Outcome::Unhandled { cause: 2, .. }), "{outcome:?}");
    assert_eq!((outcome, regs), {
        let (o, _, r) = run(MEMORY, "", false);
        (o, r)
    });
}
