//! Fluid Rust Compiler
//!
//! The compiler is organized in three major phases:
//!
//! 1. **Frontend** (elaboration, ownership tracking): Rust source → RMIR AST
//! 2. **Lowering**: RMIR AST → machine-independent proof obligations
//! 3. **Backend** (native, managed, hybrid): Proof obligations → machine code / runtime IR / WASM

pub mod frontend;
pub mod rmir;
pub mod lowering;
pub mod backend;

// TODO: Add error handling, diagnostic reporting, and CLI argument parsing
// TODO: Wire up parser, elaborator, and lowering pipeline
// TODO: Implement proof obligation extraction and serialization
