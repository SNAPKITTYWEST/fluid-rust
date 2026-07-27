//! Fluid Rust Compiler
//!
//! The compiler is organized in three major phases:
//!
//! 1. **Frontend** (elaboration, ownership tracking): Rust source → RMIR AST
//! 2. **Lowering**: RMIR AST → machine-independent proof obligations
//! 3. **Backend** (native, managed, hybrid): Proof obligations → machine code / runtime IR / WASM

pub mod artifact;
pub mod backend;
pub mod frontend;
pub mod lowering;
pub mod pipeline;
pub mod rmir;

pub use artifact::CompilationArtifact;
pub use pipeline::compile;
