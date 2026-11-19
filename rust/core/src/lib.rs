//! The detrytus machine in Rust: a balanced-ternary hart, its bus and
//! devices, with no dependencies. This crate is the part of the
//! implementation meant to survive the move from Python
//! (docs/rust-plan.md); the Python core stays the reference until the
//! two agree on everything.
//!
//! Phase 2 status: the unprivileged instructions. Traps without a
//! handler stop the machine; CSRs, rings and paging come in phase 3.

pub mod alu;
pub mod bus;
pub mod hart;
pub mod image;
pub mod isa;
pub mod machine;
pub mod trits;

pub use machine::{Machine, Outcome};
