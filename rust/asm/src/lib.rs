//! The detrytus assembler in Rust (docs/assembler.md). A port of the
//! Python assembler it replaced, held to the same output: the
//! same image bytes and the same error messages for the same source
//! (docs/rust-plan.md, phase 5).

pub mod assembler;
pub mod disasm;
pub mod error;
pub mod expr;
pub mod gp;
pub mod link;
pub mod object;
pub mod operands;
pub mod pseudo;
pub mod pyrepr;
pub mod slots;
pub mod source;
pub mod text;

pub use assembler::{Image, Listing, assemble, assemble_object};
pub use error::AsmError;
