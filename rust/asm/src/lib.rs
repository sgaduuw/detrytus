//! The detrytus assembler in Rust (docs/assembler.md). A port of the
//! Python assembler in `detrytus/asm/`, held to the same output: the
//! same image bytes and the same error messages for the same source
//! (docs/rust-plan.md, phase 5).

pub mod assembler;
pub mod disasm;
pub mod error;
pub mod expr;
pub mod operands;
pub mod pseudo;
mod pyrepr;
pub mod source;
pub mod text;

pub use assembler::{Image, Listing, assemble};
pub use error::AsmError;
