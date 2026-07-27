//! RMIR: Refinement MIR (Proof-Carrying Intermediate Representation)
//!
//! RMIR is Rust's MIR augmented with:
//! - Ownership facts
//! - Region state machine tracking
//! - Effect annotations
//! - Proof obligations embedded in the instruction stream
//!
//! Each RMIR instruction represents a concrete computation step that may:
//! - Change the state of a region (enter, allocate, deallocate, exit)
//! - Transfer ownership (move, borrow, consume)
//! - Emit an effect (IO, State, Async, etc.)
//! - Generate a proof obligation (lemma to discharge)

pub mod capability;
pub mod effect;
pub mod ir;
pub mod state;

// TODO: Implement RMIR instruction scheduler
// TODO: Implement RMIR verifier (tiny checker for proof certificates)
// TODO: Implement RMIR serialization (bytecode format)
// TODO: Implement RMIR pretty-printer for debugging
