//! Fluid Rust Prover: Discrete Proof Engine
//!
//! The prover integrates two solvers:
//!
//! 1. **ASP Solver** (clingo): Proves ownership and region invariants using logic programming
//! 2. **SMT Solver** (Z3): Proves numeric constraints and effect preconditions
//!
//! Together, they prove that all proof obligations are satisfiable.

pub mod asp;
pub mod smt;
pub mod certificate;
pub mod verifier;

// TODO: Add error handling, diagnostics, and certificate generation
// TODO: Wire up solver backends and result merging
// TODO: Implement proof certificate serialization
