//! The emulated platform: a bus with its devices and the harts, run in
//! lock step (docs/platform.md, Time).
//!
//! Each step is a round: the clock advances by one, then every running
//! hart takes one step, in hart-number order. A hart waiting in `wfi`
//! sits a round out until an interrupt is pending for it; when every
//! running hart is waiting, the clock skips ahead to the earliest moment
//! one could wake. With one hart this is exactly the single-hart
//! machine: one instruction per step, and `wfi` skipping to the timer.

use std::collections::BTreeMap;

use crate::bus::Bus;
use crate::entropy::Entropy;
use crate::hart::{Event, Hart, Stop};
use crate::harts::{self, RUNNING};
use crate::image::Segment;

/// How a run ended.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The program powered off. `steps` counts the step that did it.
    Exit { code: i64, steps: u64 },
    /// Still running after the step limit.
    StepLimit,
    /// A trap reached a ring with no handler, on hart `hart`.
    Unhandled { hart: i64, cause: i64, tval: i64, pc: i64, ring: i64 },
    /// The program needed something this core does not implement yet.
    Unsupported(String),
}

/// A hot-plug change the platform makes at a set time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hotplug {
    Add(i64),
    Remove(i64),
}

pub struct Machine {
    pub bus: Bus,
    /// Every hart that has run, by number, including stopped ones, whose
    /// last state stays here for tools to read.
    harts: BTreeMap<i64, Hart>,
    /// The shared clock.
    pub time: i64,
    /// Seeds the entropy source of each hart started after reset.
    seeds: Entropy,
    /// Hot-plug changes still to come, by time.
    schedule: Vec<(i64, Hotplug)>,
    /// The harts to step this round, kept to save allocating each step.
    order: Vec<i64>,
}

/// Whether hart `h`, waiting, may run: an interrupt pending and enabled
/// ends `wfi` and `wrs`; a lost reservation or a deadline reached also
/// ends `wrs`.
fn can_wake(hart: &Hart, bus: &Bus, h: i64) -> bool {
    hart.wakeable()
        || hart.watching
            && (!bus.holds_reservation(h) || hart.deadline.is_some_and(|d| hart.time >= d))
}

/// Stop waiting, as the hart runs again.
fn wake(hart: &mut Hart) {
    hart.waiting = false;
    hart.watching = false;
    hart.deadline = None;
}

/// The first `n` hart numbers outward from 0: 0, −1, +1, −2, +2, ...
pub fn hart_numbers(n: i64) -> Vec<i64> {
    (0..n).map(|i| if i % 2 == 1 { -(i + 1) / 2 } else { i / 2 }).collect()
}

impl Machine {
    /// A machine with one hart and `ram_pages` pages of RAM; `seed`
    /// seeds the entropy CSR.
    pub fn new(ram_pages: i64, seed: u32) -> Self {
        Self::with_harts(ram_pages, seed, 1)
    }

    /// A machine with `n` harts present (1 to 729), numbered outward
    /// from 0. Hart 0 runs; the others wait to be started.
    pub fn with_harts(ram_pages: i64, seed: u32, n: i64) -> Self {
        assert!((1..=harts::HARTS).contains(&n), "a machine has 1 to 729 harts");
        let mut harts = BTreeMap::new();
        harts.insert(0, Hart::new(seed));
        Machine {
            bus: Bus::with_harts(ram_pages, &hart_numbers(n)),
            harts,
            time: 0,
            seeds: Entropy::new(seed.wrapping_add(1)),
            schedule: Vec::new(),
            order: Vec::new(),
        }
    }

    pub fn load(&mut self, segments: &[Segment]) {
        for s in segments {
            for (addr, &tryte) in (s.start..).zip(&s.trytes) {
                self.bus.poke(addr, tryte);
            }
        }
    }

    /// Hart `h`, if it has ever run.
    pub fn hart(&self, h: i64) -> Option<&Hart> {
        self.harts.get(&h)
    }

    pub fn hart_mut(&mut self, h: i64) -> Option<&mut Hart> {
        self.harts.get_mut(&h)
    }

    /// Hart 0, which is always there.
    pub fn boot_hart(&self) -> &Hart {
        &self.harts[&0]
    }

    pub fn boot_hart_mut(&mut self) -> &mut Hart {
        self.harts.get_mut(&0).unwrap()
    }

    /// Where addr points for hart `h`'s current ring, without side
    /// effects (see [`Hart::probe`]); None if unmapped or no such hart.
    pub fn probe(&mut self, h: i64, addr: i64) -> Option<i64> {
        self.harts.get(&h)?.probe(&mut self.bus, addr)
    }

    /// Every hart that has run, in number order.
    pub fn harts(&self) -> impl Iterator<Item = (i64, &Hart)> {
        self.harts.iter().map(|(&h, hart)| (h, hart))
    }

    /// Record events on every hart, present and future, or stop.
    pub fn record_events(&mut self, on: bool) {
        for hart in self.harts.values_mut() {
            hart.events = on.then(Vec::new);
        }
    }

    /// The events each hart recorded since the last call, by hart.
    pub fn take_events(&mut self) -> Vec<(i64, Event)> {
        let mut out = Vec::new();
        for (&h, hart) in &mut self.harts {
            if let Some(events) = &mut hart.events {
                out.extend(events.drain(..).map(|e| (h, e)));
            }
        }
        out
    }

    /// Plug a hart in, or ask for one to go, when the clock reaches
    /// `time`. Hart 0 cannot be removed.
    pub fn schedule(&mut self, time: i64, change: Hotplug) -> Result<(), String> {
        let h = match change {
            Hotplug::Add(h) | Hotplug::Remove(h) => h,
        };
        if !harts::valid(h) {
            return Err(format!("no hart {h}: harts are numbered -364 to 364"));
        }
        if change == Hotplug::Remove(0) {
            return Err("hart 0 cannot be removed".into());
        }
        let at = self.schedule.partition_point(|(t, _)| *t <= time);
        self.schedule.insert(at, (time, change));
        Ok(())
    }

    /// The harts that will run an instruction in the next step, after
    /// any time skip that step would make: for a debugger to check its
    /// breakpoints against before stepping.
    pub fn prepare_round(&mut self) -> Vec<i64> {
        let order = self.bus.harts.running().to_vec();
        if order.iter().all(|h| self.harts[h].waiting) {
            self.skip_idle_time(&order);
        }
        let time = self.time + 1;
        order
            .into_iter()
            .filter(|&h| {
                let hart = self.harts.get_mut(&h).unwrap();
                let now = std::mem::replace(&mut hart.time, time);
                let runs = !hart.waiting || can_wake(hart, &self.bus, h);
                hart.time = now;
                runs
            })
            .collect()
    }

    /// Run until the program powers off, stops, or reaches `max_steps`.
    pub fn run(&mut self, max_steps: u64) -> Outcome {
        for steps in 0..max_steps {
            if let Some(outcome) = self.round(steps + 1) {
                return outcome;
            }
        }
        Outcome::StepLimit
    }

    /// One step (a round): None while the machine keeps running. A
    /// power-off counts this one step.
    pub fn step(&mut self) -> Option<Outcome> {
        self.round(1)
    }

    fn round(&mut self, steps: u64) -> Option<Outcome> {
        // Harts started during the round first run in the next one.
        let mut order = std::mem::take(&mut self.order);
        order.clear();
        order.extend_from_slice(self.bus.harts.running());
        if order.iter().all(|h| self.harts[h].waiting) {
            self.skip_idle_time(&order);
        }
        self.time += 1;
        self.bus.now = self.time;
        self.bus.advance();
        if self.schedule.first().is_some_and(|(t, _)| *t <= self.time) {
            self.apply_hotplug();
        }
        let outcome = self.run_harts(&order, steps);
        self.order = order;
        outcome
    }

    fn run_harts(&mut self, order: &[i64], steps: u64) -> Option<Outcome> {
        let enabled = self.harts.get(&0).is_some_and(|h| h.events.is_some());
        for &h in order {
            if self.bus.harts.state(h) != RUNNING {
                continue; // stopped earlier in this round
            }
            let hart = self.harts.get_mut(&h).unwrap();
            hart.time = self.time;
            hart.doorbell = self.bus.harts.doorbell(h);
            hart.external = self.bus.external_pending(h);
            if hart.waiting && !can_wake(hart, &self.bus, h) {
                continue;
            }
            wake(hart);
            self.bus.current = h;
            if let Err(stop) = hart.step(&mut self.bus) {
                return Some(Self::outcome(h, stop, steps));
            }
            self.start_and_stop(enabled);
        }
        self.bus.current = 0;
        for (&h, hart) in &mut self.harts {
            hart.doorbell = self.bus.harts.doorbell(h);
            hart.external = self.bus.external_pending(h);
        }
        None
    }

    /// When every running hart waits, move the clock to just before the
    /// first thing that could wake one: a timer, a hot-plug change or a
    /// block command finishing. If
    /// nothing could, wake them all, as `wfi` is allowed to return early.
    /// (Stopped harts never wait: a hart is replaced when it starts.)
    fn skip_idle_time(&mut self, running: &[i64]) {
        let bus = &self.bus;
        let waiting: Vec<(i64, &mut Hart)> = self
            .harts
            .iter_mut()
            .filter(|(h, _)| running.contains(h))
            .map(|(&h, hart)| (h, hart))
            .collect();
        if !waiting.iter().all(|(_, hart)| hart.waiting)
            || waiting.iter().any(|(h, hart)| can_wake(hart, bus, *h))
        {
            return;
        }
        let now = self.time;
        let timers = waiting.iter().filter_map(|(_, hart)| hart.timer_deadline());
        let deadlines = waiting.iter().filter_map(|(_, hart)| hart.deadline).filter(|&d| d > now);
        let plugs = self.schedule.iter().map(|(t, _)| t - 1).filter(|&t| t > now);
        let disk = self.bus.block.done_at().map(|t| t - 1).filter(|&t| t > now);
        match timers.chain(deadlines).chain(plugs).chain(disk).min() {
            Some(t) => self.time = t,
            None => {
                for (_, hart) in waiting {
                    wake(hart);
                }
            }
        }
    }

    fn apply_hotplug(&mut self) {
        let due = self.schedule.partition_point(|(t, _)| *t <= self.time);
        for (_, change) in self.schedule.drain(..due) {
            match change {
                Hotplug::Add(h) => {
                    self.bus.harts.add(h);
                }
                Hotplug::Remove(h) => {
                    if self.bus.harts.request_removal(h) && self.bus.harts.state(h) == harts::ABSENT
                    {
                        self.harts.remove(&h);
                    }
                }
            }
        }
    }

    /// Carry out the starts and stops the last step asked the controller
    /// for. A started hart begins afresh; a stopped one keeps its last
    /// state for tools, but runs no more and holds no reservation.
    fn start_and_stop(&mut self, events: bool) {
        for start in self.bus.harts.take_starts() {
            let seed = self.seeds.next48() as u32;
            let mut hart = Hart::start(seed, start.hart, start.pc, start.arg);
            hart.time = self.time;
            hart.events = events.then(Vec::new);
            self.harts.insert(start.hart, hart);
        }
        for h in self.bus.harts.take_stops() {
            let current = std::mem::replace(&mut self.bus.current, h);
            self.bus.take_reservation();
            self.bus.current = current;
            if self.bus.harts.state(h) == harts::ABSENT {
                self.harts.remove(&h);
            }
        }
    }

    fn outcome(hart: i64, stop: Stop, steps: u64) -> Outcome {
        match stop {
            Stop::Halt(code) => Outcome::Exit { code, steps },
            Stop::Unhandled { cause, tval, pc, ring } => {
                Outcome::Unhandled { hart, cause, tval, pc, ring }
            }
            Stop::Unsupported(what) => Outcome::Unsupported(what),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn harts_fill_outward_from_zero() {
        assert_eq!(hart_numbers(5), vec![0, -1, 1, -2, 2]);
        let nine = hart_numbers(9);
        assert!((-4..=4).all(|h| nine.contains(&h)));
    }

    #[test]
    fn hart_0_cannot_be_removed() {
        let mut m = Machine::new(1, 1);
        assert!(m.schedule(5, Hotplug::Remove(0)).is_err());
        assert!(m.schedule(5, Hotplug::Add(365)).is_err());
        assert!(m.schedule(5, Hotplug::Add(3)).is_ok());
    }
}
