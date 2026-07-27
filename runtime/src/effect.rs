//! Effect Tracking: Manage effect lifecycle and ordering
//!
//! This module mirrors the compiler's effect tracking but operates at runtime.

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

// TODO: Implement effect tracker at runtime
// TODO: Implement effect ordering verification
// TODO: Implement effect precondition checking
