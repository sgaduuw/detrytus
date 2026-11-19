//! The emulated platform: a bus with its devices and one hart.

use crate::bus::Bus;
use crate::hart::{Hart, Stop};
use crate::image::Segment;

/// How a run ended.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The program powered off. `steps` counts the step that did it.
    Exit { code: i64, steps: u64 },
    /// Still running after the step limit.
    StepLimit,
    /// A trap reached a ring with no handler.
    Unhandled { cause: i64, tval: i64, pc: i64, ring: i64 },
    /// The program needed something this core does not implement yet.
    Unsupported(String),
}

pub struct Machine {
    pub bus: Bus,
    pub hart: Hart,
}

impl Machine {
    /// A machine with `ram_pages` pages of RAM; `seed` seeds the entropy CSR.
    pub fn new(ram_pages: i64, seed: u32) -> Self {
        Machine { bus: Bus::new(ram_pages), hart: Hart::new(seed) }
    }

    pub fn load(&mut self, segments: &[Segment]) {
        for s in segments {
            for (addr, &tryte) in (s.start..).zip(&s.trytes) {
                self.bus.poke(addr, tryte);
            }
        }
    }

    /// Run until the program powers off, stops, or reaches `max_steps`.
    pub fn run(&mut self, max_steps: u64) -> Outcome {
        for steps in 0..max_steps {
            match self.hart.step(&mut self.bus) {
                Ok(()) => {}
                Err(Stop::Halt(code)) => return Outcome::Exit { code, steps: steps + 1 },
                Err(Stop::Unhandled { cause, tval, pc, ring }) => {
                    return Outcome::Unhandled { cause, tval, pc, ring };
                }
                Err(Stop::Unsupported(what)) => return Outcome::Unsupported(what),
            }
        }
        Outcome::StepLimit
    }
}
