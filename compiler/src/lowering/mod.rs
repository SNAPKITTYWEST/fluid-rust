//! Lowering: RMIR → Machine-independent lowered form
//!
//! Lowering transforms RMIR (with ownership facts and proof obligations)
//! into a form that can be compiled to multiple backends (native, managed, hybrid).
//!
//! Key transformation: ownership invariants become explicit guards in code.

pub mod normal_mir;

// TODO: Implement RMIR to lowered form transformation
// TODO: Implement proof obligation forwarding
// TODO: Implement capability splitting/joining lowering
// TODO: Implement effect handler integration points
