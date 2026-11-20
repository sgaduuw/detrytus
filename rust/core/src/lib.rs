//! The detrytus machine: balanced-ternary harts, their bus and devices,
//! with no dependencies.
//!
//! Status: the integer core, traps, delegation, interrupts, the timer,
//! single step, paged virtual memory, and up to 729 harts with the hart
//! controller. Floating point, compressed bundles and debug triggers are
//! specified but not implemented yet.

pub mod alu;
pub mod bus;
pub mod entropy;
pub mod hart;
pub mod harts;
pub mod image;
pub mod isa;
pub mod machine;
pub mod mmu;
mod privileged;
pub mod trits;

pub use machine::{Hotplug, Machine, Outcome};
