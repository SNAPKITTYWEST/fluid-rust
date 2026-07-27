//! Effect Transitions: Model algebraic effects as state transitions
//!
//! Effects are not primitives; they are composable handlers.
//! This module tracks effect requests and verifies they follow a valid ordering.

use std::collections::HashMap;

/// Represents an effect operation before it is handled by the runtime.
#[derive(Debug, Clone)]
pub struct EffectRequest {
    pub kind: String, // "IO", "State", "Async", "Region", "GC", "Exception", "FFI", "Concurrency"
    pub payload: String,
}

/// Represents a state change caused by handling an effect.
#[derive(Debug, Clone)]
pub struct StateChange {
    pub before: String,
    pub after: String,
}

/// Models the lifecycle of an effect from emission to completion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectPhase {
    /// Effect has been emitted but not yet handled
    Pending,
    /// Handler is processing the effect
    InProgress,
    /// Handler has completed, state change is final
    Complete,
    /// Error during handling
    Failed,
}

/// Verifies effect ordering: effects must be handled in a consistent order.
pub struct EffectTracker {
    effects: Vec<(EffectRequest, EffectPhase)>,
    effect_index: HashMap<String, usize>,
}

impl EffectTracker {
    pub fn new() -> Self {
        EffectTracker {
            effects: Vec::new(),
            effect_index: HashMap::new(),
        }
    }

    pub fn emit_effect(&mut self, effect: EffectRequest) -> usize {
        let id = self.effects.len();
        self.effect_index
            .insert(format!("{}_{}", effect.kind, id), id);
        self.effects.push((effect, EffectPhase::Pending));
        id
    }

    pub fn effect_precondition(&self, effect_id: usize) -> Option<String> {
        // Return the precondition that must be verified before this effect can be handled.
        // This is effect-specific.
        if let Some((effect, _)) = self.effects.get(effect_id) {
            match effect.kind.as_str() {
                "IO" => Some("file handle must be valid and open".to_string()),
                "State" => Some("mutable reference must be valid".to_string()),
                "Async" => Some("scheduler must be available".to_string()),
                "Region" => Some("region must be in correct state".to_string()),
                "GC" => Some("heap must be consistent".to_string()),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn mark_in_progress(&mut self, effect_id: usize) -> Result<(), String> {
        if let Some((_, phase)) = self.effects.get_mut(effect_id) {
            if *phase == EffectPhase::Pending {
                *phase = EffectPhase::InProgress;
                Ok(())
            } else {
                Err(format!(
                    "Effect {} already in progress or complete",
                    effect_id
                ))
            }
        } else {
            Err(format!("Effect {} not found", effect_id))
        }
    }

    pub fn mark_complete(&mut self, effect_id: usize) -> Result<(), String> {
        if let Some((_, phase)) = self.effects.get_mut(effect_id) {
            if *phase == EffectPhase::InProgress {
                *phase = EffectPhase::Complete;
                Ok(())
            } else {
                Err(format!("Effect {} not in progress", effect_id))
            }
        } else {
            Err(format!("Effect {} not found", effect_id))
        }
    }

    pub fn verify_no_unhandled_effects(&self) -> Result<(), Vec<usize>> {
        let unhandled: Vec<usize> = self
            .effects
            .iter()
            .enumerate()
            .filter(|(_, (_, phase))| {
                *phase == EffectPhase::Pending || *phase == EffectPhase::InProgress
            })
            .map(|(i, _)| i)
            .collect();

        if unhandled.is_empty() {
            Ok(())
        } else {
            Err(unhandled)
        }
    }

    pub fn all_effects(&self) -> &[(EffectRequest, EffectPhase)] {
        &self.effects
    }
}

// TODO: Implement effect dependency analysis (which effects can run in parallel)
// TODO: Implement effect isolation verification (effects from different regions)
// TODO: Implement effect ordering constraints from the type system
// TODO: Implement proof obligation extraction for effect preconditions
