//! The detrytus machine in Rust: a balanced-ternary hart, its bus and
//! devices, with no dependencies. This crate is the part of the
//! implementation meant to survive the move from Python
//! (docs/rust-plan.md); the Python core stays the reference until the
//! two agree on everything.
//!
//! Status: everything the Python core implements: the integer core,
//! traps, delegation, interrupts, the timer, single step and paged
//! virtual memory. Floating point, compressed bundles and debug triggers
//! are specified but implemented in neither core yet.

pub mod alu;
pub mod bus;
pub mod entropy;
pub mod hart;
pub mod image;
pub mod isa;
pub mod machine;
pub mod mmu;
mod privileged;
pub mod trits;

pub use machine::{Machine, Outcome};
