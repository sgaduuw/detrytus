//! What an accelerator, such as a JIT compiler, plugs into: the hart's
//! side of running compiled code (CONTEXT.md, "A JIT on Cranelift",
//! says why it is shaped so).
//!
//! The core stays free of dependencies: it names the trait and gives
//! compiled code the narrow access it needs, and the compiler lives in
//! its own crate. Compiled code must behave exactly as the interpreter
//! would, one round and one instruction at a time, so the machine only
//! hands it runs of rounds in which nothing outside the hart can change
//! (see `Machine::fast_rounds`) and no interrupt can become pending
//! ([`Hart::quiet_steps`]). Anything else, or anything compiled code
//! is unsure of, it leaves to the interpreter by stopping before it.

use crate::bus::Bus;
use crate::hart::Hart;
use crate::isa::{Decoded, Kind};

/// Runs compiled code for a hart.
pub trait Accelerator {
    /// Run the hart from its pc for at most `budget` instructions,
    /// leaving pc at the next one, and say how many ran (0: the
    /// interpreter should run the next one). The machine counts the
    /// rounds and the instructions retired.
    fn run(&mut self, hart: &mut Hart, bus: &mut Bus, budget: u64) -> u64;
}

/// How compiled code may treat an instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    /// Reads and writes only the hart's registers: [`Hart::jit_compute`].
    Compute,
    /// A load of 1 or 3 trytes: [`Hart::jit_load`].
    Load(u32),
    /// A store of 1 or 3 trytes: [`Hart::jit_store`].
    Store(u32),
    /// A jump or branch: ends a block.
    Control,
    /// Anything else (atomics, system instructions): left to the
    /// interpreter, so a block ends before it.
    Other,
}

pub fn class(d: &Decoded) -> Class {
    match d.op.kind {
        Kind::Lt => Class::Load(1),
        Kind::Lw => Class::Load(3),
        Kind::St => Class::Store(1),
        Kind::Sw => Class::Store(3),
        Kind::Jal | Kind::Jalr | Kind::B | Kind::Br3 => Class::Control,
        Kind::Amoswap
        | Kind::Amoadd
        | Kind::Amomin
        | Kind::Amomax
        | Kind::Amotlut
        | Kind::Lr
        | Kind::Sc
        | Kind::Wrs
        | Kind::Cas
        | Kind::Sys
        | Kind::Brk
        | Kind::Fence
        | Kind::FenceI
        | Kind::Cclean
        | Kind::Cflush
        | Kind::Cinval
        | Kind::Tret
        | Kind::FenceVm
        | Kind::Csrrw
        | Kind::Csrrtmax
        | Kind::Csrrtmin
        | Kind::Wfi => Class::Other,
        _ => Class::Compute,
    }
}

/// An instruction compiled code may run: decoded, and how far pc moves
/// past it.
#[derive(Clone, Copy, Debug)]
pub struct Insn {
    pub d: Decoded,
    pub advance: i64,
}
