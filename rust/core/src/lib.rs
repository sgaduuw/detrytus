//! The detrytus machine: balanced-ternary harts, their bus and devices,
//! with no dependencies.
//!
//! Status: the integer core, traps, delegation, interrupts, the timer,
//! single step, paged virtual memory, up to 729 harts with the hart
//! controller, the interrupt controller for devices and the block
//! device. Floating point, compressed bundles and debug triggers are
//! specified but not implemented yet.

pub mod alu;
pub mod block;
pub mod bus;
pub mod entropy;
pub mod hart;
pub mod harts;
pub mod image;
pub mod irq;
pub mod isa;
pub mod machine;
pub mod mmu;
mod privileged;
pub mod trits;

pub use machine::{Hotplug, Machine, Outcome};
