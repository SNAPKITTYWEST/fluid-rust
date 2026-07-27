//! Execution State Machine: Execute RMIR instructions and track state
//!
//! This module implements the state machine that:
//! 1. Takes an RMIR instruction
//! 2. Checks preconditions
//! 3. Updates the execution state
//! 4. Generates proof obligations
//! 5. Produces the next state

use crate::rmir::ir::*;

/// Executes a single RMIR instruction on a state, producing the next state and any proof obligations.
pub fn execute_instruction(
    instr: &RmirInstruction,
    state: &ExecutionState,
) -> Result<(ExecutionState, Vec<ProofObligation>), String> {
    let mut next_state = state.clone();
    let mut obligations = Vec::new();

    next_state.pc += 1;

    match instr {
        RmirInstruction::Assign { id, value } => {
            next_state.values.insert(*id, value.clone());
        }

        RmirInstruction::Move { src, dst } => {
            // Precondition: src exists
            let val = state.values.get(src).ok_or(format!("Value {} not found", src))?;

            // Postcondition: dst takes the value, src is invalidated
            next_state.values.insert(*dst, val.clone());
            next_state.values.remove(src); // src is now consumed

            // Proof obligation: no use-after-move on src after this point
            obligations.push(ProofObligation {
                id: next_state.proof_obligations.len() as u32,
                kind: "no_use_after_move".to_string(),
                description: format!("Value {} moved to {}", src, dst),
            });
        }

        RmirInstruction::Borrow {
            src,
            borrow_id,
            mode,
            lifetime,
        } => {
            // Precondition: src exists
            let _val = state.values.get(src).ok_or(format!("Value {} not found", src))?;

            // Postcondition: create a reference
            next_state
                .values
                .insert(*borrow_id, Value::Reference(*src, mode.clone()));

            // Proof obligation: borrow doesn't outlive src
            obligations.push(ProofObligation {
                id: next_state.proof_obligations.len() as u32,
                kind: "borrow_lifetime_valid".to_string(),
                description: format!(
                    "Borrow {} of {} valid until program point {}",
                    borrow_id, src, lifetime
                ),
            });
        }

        RmirInstruction::Consume { id } => {
            // Precondition: id exists
            if !state.values.contains_key(id) {
                return Err(format!("Cannot consume non-existent value {}", id));
            }

            next_state.values.remove(id);

            // Proof obligation: no use-after-consume
            obligations.push(ProofObligation {
                id: next_state.proof_obligations.len() as u32,
                kind: "no_use_after_consume".to_string(),
                description: format!("Value {} consumed here", id),
            });
        }

        RmirInstruction::RegionEnter { region_id } => {
            // Precondition: region must be unentered
            let region_status = state.regions.get(region_id).cloned().unwrap_or(RegionStatus::Unentered);
            if region_status != RegionStatus::Unentered {
                return Err(format!("Region {} already entered", region_id));
            }

            // Postcondition: mark region as active
            next_state
                .regions
                .insert(*region_id, RegionStatus::Active { allocations: vec![] });

            obligations.push(ProofObligation {
                id: next_state.proof_obligations.len() as u32,
                kind: "region_lifecycle".to_string(),
                description: format!("Region {} entered", region_id),
            });
        }

        RmirInstruction::Allocate {
            region_id,
            size: _size,
            ptr_id,
        } => {
            // Precondition: region must be active
            match state.regions.get(region_id) {
                Some(RegionStatus::Active { allocations }) => {
                    // Postcondition: add pointer to allocations
                    let mut new_allocs = allocations.clone();
                    new_allocs.push(*ptr_id);
                    next_state.regions.insert(
                        *region_id,
                        RegionStatus::Active {
                            allocations: new_allocs,
                        },
                    );

                    // Add write capability
                    next_state
                        .capabilities
                        .insert((*region_id, *ptr_id), CapabilityKind::Write);

                    obligations.push(ProofObligation {
                        id: next_state.proof_obligations.len() as u32,
                        kind: "allocation_wellformed".to_string(),
                        description: format!("Allocated {} in region {}", ptr_id, region_id),
                    });
                }
                _ => return Err(format!("Region {} not active", region_id)),
            }
        }

        RmirInstruction::Deallocate {
            region_id,
            ptr_id,
        } => {
            // Precondition: region must be active and contain ptr_id
            match state.regions.get(region_id) {
                Some(RegionStatus::Active { allocations }) => {
                    if !allocations.contains(ptr_id) {
                        return Err(format!("Pointer {} not allocated in region {}", ptr_id, region_id));
                    }

                    // Postcondition: remove pointer from allocations
                    let mut new_allocs = allocations.clone();
                    new_allocs.retain(|&p| p != *ptr_id);
                    next_state.regions.insert(
                        *region_id,
                        RegionStatus::Active {
                            allocations: new_allocs,
                        },
                    );

                    // Remove capability
                    next_state.capabilities.remove(&(*region_id, *ptr_id));
                }
                _ => return Err(format!("Region {} not active", region_id)),
            }
        }

        RmirInstruction::RegionExit { region_id } => {
            // Precondition: region must be active with no allocations
            match state.regions.get(region_id) {
                Some(RegionStatus::Active { allocations }) => {
                    if !allocations.is_empty() {
                        return Err(format!(
                            "Cannot exit region {} with active allocations: {:?}",
                            region_id, allocations
                        ));
                    }

                    // Postcondition: mark region as closed
                    next_state.regions.insert(*region_id, RegionStatus::Closed);

                    obligations.push(ProofObligation {
                        id: next_state.proof_obligations.len() as u32,
                        kind: "region_lifecycle".to_string(),
                        description: format!("Region {} exited", region_id),
                    });
                }
                _ => return Err(format!("Region {} not active", region_id)),
            }
        }

        RmirInstruction::EffectEmit { effect } => {
            next_state.effects.push(effect.clone());

            obligations.push(ProofObligation {
                id: next_state.proof_obligations.len() as u32,
                kind: "effect_precondition".to_string(),
                description: format!("Effect {} emitted, preconditions must be verified", effect.kind),
            });
        }

        RmirInstruction::Assert { predicate } => {
            obligations.push(ProofObligation {
                id: next_state.proof_obligations.len() as u32,
                kind: "assertion".to_string(),
                description: format!("Assert: {}", predicate),
            });
        }
    }

    next_state.proof_obligations.extend(obligations.clone());
    Ok((next_state, obligations))
}

// TODO: Implement bulk execution of instruction sequences
// TODO: Implement execution trace recording
// TODO: Implement state snapshots for debugging
// TODO: Implement counterexample generation when invariant violated
