//! Proof Obligations: safety properties that must be proven

use serde::{Deserialize, Serialize};

/// A single proof obligation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProofObligation {
    pub id: u32,
    pub kind: ObligationKind,
    pub description: String,
    pub assumptions: Vec<String>,
    pub target: String,
    pub negated_query: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ObligationKind {
    /// All uses of value respect linearity (used exactly once)
    OwnershipInvariant,
    /// Region lifecycle: only access in active state
    RegionSafety,
    /// Affine types: no shared mutable access
    LinearityConstraint,
    /// Effects ordered: all preconditions met
    EffectOrdering,
    /// Array index within bounds
    BoundsCheck,
    /// Effect precondition satisfied
    EffectPrecondition,
}

impl ProofObligation {
    pub fn ownership_invariant(id: u32, value: &str) -> Self {
        Self {
            id,
            kind: ObligationKind::OwnershipInvariant,
            description: format!("Value '{}' respects ownership invariant (linear use)", value),
            assumptions: vec![
                format!("linear({})", value),
                format!("not_consumed({})", value),
            ],
            target: format!("safe(owns({}))", value),
            negated_query: format!("?- double_use({}).", value),
        }
    }

    pub fn region_safety(id: u32, region: &str) -> Self {
        Self {
            id,
            kind: ObligationKind::RegionSafety,
            description: format!("Region '{}' access is safe (active state)", region),
            assumptions: vec![
                format!("region_entered({})", region),
                format!("not region_closed({})", region),
            ],
            target: format!("safe(access_to({}))", region),
            negated_query: format!("?- access_to_closed({}).", region),
        }
    }

    pub fn bounds_check(id: u32, index: &str, length: &str) -> Self {
        Self {
            id,
            kind: ObligationKind::BoundsCheck,
            description: format!("Index {} is within bounds of {}", index, length),
            assumptions: vec![
                format!("index_value({})", index),
                format!("array_length({})", length),
            ],
            target: format!("within_bounds({}, {})", index, length),
            negated_query: format!("?- {} >= {}.", index, length),
        }
    }

    pub fn effect_ordering(id: u32, effect1: &str, effect2: &str) -> Self {
        Self {
            id,
            kind: ObligationKind::EffectOrdering,
            description: format!(
                "Effect '{}' preconditions satisfied before '{}'",
                effect1, effect2
            ),
            assumptions: vec![
                format!("effect_precondition({})", effect1),
                format!("ordering({}, {})", effect1, effect2),
            ],
            target: format!("safe(order({}, {}))", effect1, effect2),
            negated_query: format!("?- precondition_violated({}, {}).", effect1, effect2),
        }
    }
}

/// A set of proof obligations for a program
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObligationSet {
    pub obligations: Vec<ProofObligation>,
}

impl ObligationSet {
    pub fn new() -> Self {
        Self {
            obligations: Vec::new(),
        }
    }

    pub fn add(&mut self, obligation: ProofObligation) {
        self.obligations.push(obligation);
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "obligations": self.obligations,
            "count": self.obligations.len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ownership_obligation() {
        let ob = ProofObligation::ownership_invariant(0, "x");
        assert_eq!(ob.id, 0);
        assert!(ob.description.contains("linear"));
    }

    #[test]
    fn test_bounds_check_obligation() {
        let ob = ProofObligation::bounds_check(1, "i", "100");
        assert_eq!(ob.id, 1);
        assert!(ob.target.contains("within_bounds"));
    }

    #[test]
    fn test_obligation_set() {
        let mut set = ObligationSet::new();
        set.add(ProofObligation::ownership_invariant(0, "x"));
        set.add(ProofObligation::region_safety(1, "r1"));
        assert_eq!(set.obligations.len(), 2);
    }
}
