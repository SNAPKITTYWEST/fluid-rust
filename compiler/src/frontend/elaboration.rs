//! Elaboration: Rust HIR → RMIR AST with refinement types
//!
//! Elaboration is the process of:
//! 1. Taking Rust's abstract syntax tree (parsed by syn)
//! 2. Annotating it with refinement types (liquid types)
//! 3. Inserting region and effect markers
//! 4. Building proof obligations
//!
//! Output: ElaboratedFunction (ready for RMIR generation)

use std::collections::HashMap as StdHashMap;

/// A refined type annotation extracted from source code.
/// Example: `x: i32{v | v > 0}` or `buf: &mut [u8]{lifetime < 'a}`
#[derive(Debug, Clone)]
pub struct RefinedType {
    pub base_type: String,      // "i32", "&mut [u8]", etc.
    pub refinement: Option<String>, // "v > 0", "lifetime < 'a", etc.
    pub region: Option<String>,  // Region constraint, if any
}

/// An elaborated function with ownership facts and proof obligations.
#[derive(Debug)]
pub struct ElaboratedFunction {
    pub name: String,
    pub parameters: Vec<(String, RefinedType)>,
    pub return_type: RefinedType,
    pub body: Vec<ElaboratedStatement>,
    pub effects: Vec<String>, // ["IO", "State", "Async", ...]
    pub proof_obligations: Vec<ProofObligation>,
}

/// A statement in the elaborated AST.
#[derive(Debug)]
pub enum ElaboratedStatement {
    RegionEnter { name: String },
    Allocate { region: String, ptr_name: String, size: usize },
    Deallocate { region: String, ptr_name: String },
    RegionExit { name: String },
    EffectEmit { effect: String, payload: String },
    Consume { value: String },
    Borrow { value: String, borrow_name: String, mode: String, lifetime: String },
    Assert { predicate: String },
}

/// A proof obligation: a lemma the discrete prover must verify.
#[derive(Debug, Clone)]
pub struct ProofObligation {
    pub id: u32,
    pub kind: String, // "no_use_after_consume", "region_wellformed", etc.
    pub description: String,
}

/// Context for elaboration: tracks regions, capabilities, and effect state.
#[derive(Debug)]
pub struct ElaborationContext {
    active_regions: StdHashMap<String, RegionState>,
    owned_values: StdHashMap<String, ValueState>,
    current_effects: Vec<String>,
    proof_counter: u32,
}

#[derive(Debug)]
struct RegionState {
    status: String, // "unentered", "active", "closed"
    allocations: Vec<String>, // ["ptr_0", "ptr_1", ...]
}

#[derive(Debug)]
struct ValueState {
    kind: String,     // "unique", "borrowed_shared", "borrowed_mut"
    lifetime: Option<String>,
    region: Option<String>,
}

impl ElaborationContext {
    pub fn new() -> Self {
        ElaborationContext {
            active_regions: StdHashMap::new(),
            owned_values: StdHashMap::new(),
            current_effects: Vec::new(),
            proof_counter: 0,
        }
    }

    pub fn enter_region(&mut self, name: String) {
        self.active_regions.insert(name, RegionState {
            status: "active".to_string(),
            allocations: Vec::new(),
        });
    }

    pub fn exit_region(&mut self, name: String) -> Result<(), String> {
        if let Some(region) = self.active_regions.get(&name) {
            if !region.allocations.is_empty() {
                return Err(format!("Cannot exit region {} with active allocations", name));
            }
        }
        self.active_regions.remove(&name);
        Ok(())
    }

    pub fn allocate(&mut self, region: String, ptr: String, _size: usize) -> Result<(), String> {
        if let Some(region_state) = self.active_regions.get_mut(&region) {
            region_state.allocations.push(ptr);
            Ok(())
        } else {
            Err(format!("Region {} not active", region))
        }
    }

    pub fn next_proof_id(&mut self) -> u32 {
        let id = self.proof_counter;
        self.proof_counter += 1;
        id
    }
}

// TODO: Implement elaborate_function() to consume parsed Rust function and produce ElaboratedFunction
// TODO: Implement ownership inference: track moves, borrows, consumes
// TODO: Implement region state machine checking
// TODO: Implement effect ordering verification
// TODO: Implement proof obligation generation for each statement
// TODO: Add diagnostic error messages for ownership violations
