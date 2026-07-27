//! RMIR Instructions and Core Types
//!
//! The instruction set for proof-carrying MIR.
//! Each instruction is a primitive operation on the execution state machine.

use std::collections::HashMap as StdHashMap;

/// A unique identifier for values, regions, pointers, etc.
pub type Id = u32;

/// A region identifier uniquely naming a memory region (stack, heap, etc).
pub type RegionId = u32;

/// An opaque value at runtime (pointer, integer, reference, etc).
#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Pointer(Id),
    Reference(Id, String), // "shared" or "mut"
    Boolean(bool),
    // TODO: Add unit, struct, enum, array, etc.
}

/// The execution state after each instruction.
/// Captures all invariant facts needed for proof verification.
#[derive(Debug, Clone)]
pub struct ExecutionState {
    /// Current values (SSA form): value_id -> Value
    pub values: StdHashMap<Id, Value>,
    /// Region states: region_id -> RegionStatus
    pub regions: StdHashMap<RegionId, RegionStatus>,
    /// Linear capabilities: (region_id, ptr_id) -> CapabilityKind
    pub capabilities: StdHashMap<(RegionId, Id), CapabilityKind>,
    /// Active effects in this execution
    pub effects: Vec<Effect>,
    /// Proof obligations generated so far
    pub proof_obligations: Vec<ProofObligation>,
    /// Current program counter
    pub pc: u32,
}

/// The status of a region (see ARCHITECTURE.md for state machine diagram).
#[derive(Debug, Clone, PartialEq)]
pub enum RegionStatus {
    /// Region not yet entered; cannot allocate or access
    Unentered,
    /// Region is active; allocations are valid
    Active { allocations: Vec<Id> },
    /// Region has been exited; all allocations must be deallocated
    Closed,
}

/// A linear capability: permission to perform an action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityKind {
    /// Permission to read from a location
    Read,
    /// Permission to write to a location
    Write,
    /// Permission to deallocate a location
    Deallocate,
}

/// An effect: a runtime side effect (IO, State, Async, etc).
#[derive(Debug, Clone)]
pub struct Effect {
    pub kind: String, // "IO", "State", "Async", "Region", "GC", "Exception", "FFI", "Concurrency"
    pub request: String, // Effect-specific payload
}

/// A proof obligation: a lemma the discrete prover must discharge.
#[derive(Debug, Clone)]
pub struct ProofObligation {
    pub id: u32,
    pub kind: String, // "no_use_after_consume", "region_wellformed", "effect_precondition", etc.
    pub description: String,
}

/// RMIR Instruction: one step of execution.
/// Each instruction transitions the ExecutionState to a new state.
#[derive(Debug, Clone)]
pub enum RmirInstruction {
    /// Assign a value to an identifier.
    /// Precondition: none
    /// Postcondition: values[id] = v
    Assign { id: Id, value: Value },

    /// Move a value from one identifier to another (transfer ownership).
    /// Precondition: values[src] exists, capability(src, move) held
    /// Postcondition: values[dst] = values[src], values[src] invalidated
    Move { src: Id, dst: Id },

    /// Borrow a value (create a reference with limited lifetime).
    /// Precondition: values[src] exists, capability(src, borrow) held
    /// Postcondition: values[borrow_id] = Reference(src, mode), borrow valid for 'lifetime
    Borrow {
        src: Id,
        borrow_id: Id,
        mode: String,  // "shared" or "mut"
        lifetime: u32, // Program point where borrow ends
    },

    /// Consume a value (invalidate it, making further use a compile error).
    /// Precondition: values[id] exists
    /// Postcondition: values[id] = consumed, generates proof obligation
    Consume { id: Id },

    /// Enter a region (begin its lifetime, prepare for allocations).
    /// Precondition: regions[region_id] = Unentered
    /// Postcondition: regions[region_id] = Active({})
    RegionEnter { region_id: RegionId },

    /// Allocate memory within a region.
    /// Precondition: regions[region_id] = Active, size > 0
    /// Postcondition: regions[region_id].allocations += ptr_id, capabilities[(region_id, ptr_id)] = Write
    Allocate {
        region_id: RegionId,
        size: u32,
        ptr_id: Id,
    },

    /// Deallocate memory within a region.
    /// Precondition: regions[region_id] = Active, ptr_id in allocations
    /// Postcondition: regions[region_id].allocations -= ptr_id
    Deallocate { region_id: RegionId, ptr_id: Id },

    /// Exit a region (end its lifetime, validate all allocations deallocated).
    /// Precondition: regions[region_id] = Active, allocations.is_empty()
    /// Postcondition: regions[region_id] = Closed, generates proof obligation
    RegionExit { region_id: RegionId },

    /// Emit an effect (request a side effect from the runtime).
    /// Precondition: none (effect may have preconditions checked by prover)
    /// Postcondition: effects += eff, generates proof obligation for preconditions
    EffectEmit { effect: Effect },

    /// Assert a predicate must hold (generates proof obligation).
    /// Precondition: none (prover must verify predicate is true)
    /// Postcondition: generates proof obligation for predicate
    Assert { predicate: String },
}

/// A function in RMIR form: a list of instructions with initial state.
#[derive(Debug)]
pub struct RmirFunction {
    pub name: String,
    pub instructions: Vec<RmirInstruction>,
    pub initial_state: ExecutionState,
    pub final_state: ExecutionState,
}

// TODO: Implement instruction executor (state_machine.rs)
// TODO: Implement proof obligation extractor
// TODO: Implement RMIR to bytecode serializer
// TODO: Implement RMIR bytecode deserializer (proof certificate validator)
// TODO: Add debug printing for execution traces
