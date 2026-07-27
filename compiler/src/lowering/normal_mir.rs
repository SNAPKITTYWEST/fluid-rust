//! Normal MIR: Lowered RMIR with explicit guards
//!
//! This is the target of RMIR lowering.
//! It still contains proof obligations, but regions are now explicit guards/asserts.

#[derive(Debug)]
pub struct LoweredInstruction {
    pub id: u32,
    pub kind: String,
    pub proof_obligations: Vec<String>,
}

// TODO: Implement lowering of region lifecycle to asserts
// TODO: Implement lowering of ownership facts to guards
// TODO: Implement lowering of capabilities to memory operations
