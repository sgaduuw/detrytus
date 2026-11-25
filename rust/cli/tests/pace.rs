//! Pacing the clock over idle time (Machine::set_pace), as a live
//! console does so that a sleeping program sleeps in real time.

use detrytus_asm::assemble;
use detrytus_core::{Machine, Outcome};

/// Waits (wrs) until a deadline 100000 ticks away, then powers off with
/// 1 if it was not woken early.
const SLEEPER: &str = "
        .equ  POWER, -39366
        .equ  DATA, 19683
        li    s0, DATA
        lr    t0, (s0)
        csrr  t1, time
        li    t2, 100000
        add   s1, t1, t2
        wrs   s1
        csrr  t1, time
        li    a0, 1
        bge   t1, s1, 3f
        li    a0, 0
3:      li    t6, POWER
        sw    a0, 0(t6)
";

fn machine() -> Machine {
    let mut m = Machine::new(729, 1);
    for (addr, tryte) in assemble(SLEEPER, None).unwrap().trytes {
        m.bus.poke(addr, tryte);
    }
    m
}

/// Runs to the end in chunks, as `run` does: the outcome, and how many
/// chunks ended on a paced skip.
fn run_out(m: &mut Machine) -> (Outcome, u32) {
    let mut paced = 0;
    loop {
        let (outcome, _) = m.run_until_stalled(1000);
        if m.take_paced() {
            paced += 1;
        }
        if !matches!(outcome, Outcome::StepLimit) {
            return (outcome, paced);
        }
        assert!(paced < 1000, "never finished");
    }
}

#[test]
fn without_a_pace_idle_time_is_skipped_at_once() {
    let mut m = machine();
    let (outcome, paced) = run_out(&mut m);
    assert!(matches!(outcome, Outcome::Exit { code: 1, .. }));
    assert_eq!(paced, 0);
}

#[test]
fn a_pace_breaks_a_skip_into_steps_the_caller_can_wait_between() {
    let mut m = machine();
    m.set_pace(Some(10_000));
    let (outcome, paced) = run_out(&mut m);
    assert!(matches!(outcome, Outcome::Exit { code: 1, .. }));
    assert_eq!(paced, 9); // 100000 ticks, 10000 at a time; the last reaches it
}

/// A link from the outside world that never brings anything: frames
/// could arrive at any time, so time keeps moving while the hart waits.
struct Quiet;

impl detrytus_core::net::Link for Quiet {
    fn send(&mut self, _: &[u8]) {}
    fn receive(&mut self) -> Option<Vec<u8>> {
        None
    }
    fn may_receive(&self) -> bool {
        true
    }
}

#[test]
fn short_skips_to_look_at_a_link_add_up_to_a_paced_one() {
    let mut m = Machine::new(729, 1);
    for (addr, tryte) in assemble("1: wfi\n j 1b\n", None).unwrap().trytes {
        m.bus.poke(addr, tryte);
    }
    m.bus.net.link = Some(Box::new(Quiet));
    m.set_pace(Some(10_000));
    let (outcome, _) = m.run_until_stalled(1000);
    assert!(matches!(outcome, Outcome::StepLimit));
    assert!(m.take_paced(), "ten skips of 1,000 ticks make one pace");
    // 10,000 ticks skipped, and a few steps run between the skips.
    assert!((10_000..10_100).contains(&m.time), "time {}", m.time);
    assert!(!m.take_stalled(), "a link that may bring a frame never stalls the machine");
}
