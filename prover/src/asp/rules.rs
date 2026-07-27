//! ASP Rules: Logic programming constraints for ownership and regions
//!
//! This module generates the ASP rules that enforce invariants.
//! Rules are logic program clauses (Horn clauses with negation-as-failure).

/// Generates the complete ASP rule set.
pub fn generate_rules() -> String {
    let mut rules = String::new();

    rules.push_str("% ============ OWNERSHIP RULES ============\n\n");
    rules.push_str(ownership_rules());

    rules.push_str("\n% ============ REGION LIFECYCLE RULES ============\n\n");
    rules.push_str(region_rules());

    rules.push_str("\n% ============ CAPABILITY RULES ============\n\n");
    rules.push_str(capability_rules());

    rules.push_str("\n% ============ EFFECT RULES ============\n\n");
    rules.push_str(effect_rules());

    rules
}

fn ownership_rules() -> &'static str {
    r#"
% Invariant: No two threads own the same value at the same time
:- owns(V, T1, TS), owns(V, T2, TS), T1 != T2.

% Invariant: Once consumed, cannot be owned again
% (If a value is in the consumed set, no subsequent owns fact should exist)
% [This requires tracking consumed values in facts, e.g., consumed(V, TS)]
:- owns(V, T, TS1), consumed(V, TS2), TS2 < TS1.
"#
}

fn region_rules() -> &'static str {
    r#"
% Invariant: Region lifecycle is unentered -> active -> closed
% [Enforce via ASP constraints that check state transitions]

% Invariant: All allocations must be deallocated before region closes
:- region_status(R, TS_close, closed),
   allocated_in(P, R, TS_alloc),
   TS_alloc < TS_close,
   not deallocated(P, TS_close).

% Invariant: Cannot allocate in unentered region
:- allocated_in(P, R, TS),
   region_status(R, TS, unentered).

% Invariant: Cannot allocate in closed region
:- allocated_in(P, R, TS),
   region_status(R, TS, closed).
"#
}

fn capability_rules() -> &'static str {
    r#"
% Invariant: At most one thread has write capability for a resource
:- capability(V, write, TS),
   owns(V, T1, TS),
   owns(V, T2, TS),
   T1 != T2.

% Invariant: Write capability is linear (exactly one holder)
% [Enforced implicitly if owns predicate is well-formed]

% Invariant: Read capabilities are shared (multiple threads can own)
% [No explicit constraint needed if we allow multiple owns facts with "shared" mode]
"#
}

fn effect_rules() -> &'static str {
    r#"
% Invariant: Effect preconditions are verified
% [Effect-specific preconditions should be added as facts]
% Example: effect_precondition(io_write, file_open).
:- effect_emitted(E, TS),
   effect_precondition(E, Pre),
   not precondition_satisfied(Pre, TS).

% Invariant: Effects are ordered consistently
% [If two effects conflict, they cannot both occur;
%  this would be checked via a serialization order]
"#
}

/// Returns just the rules (without facts) for incorporation into a larger program.
pub fn get_rule_set() -> String {
    generate_rules()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rules_generation() {
        let rules = generate_rules();
        assert!(rules.contains("OWNERSHIP RULES"));
        assert!(rules.contains("REGION LIFECYCLE RULES"));
        assert!(rules.contains("CAPABILITY RULES"));
        assert!(rules.contains("EFFECT RULES"));
    }

    #[test]
    fn test_ownership_constraint_exists() {
        let rules = ownership_rules();
        assert!(rules.contains("No two threads own the same value"));
    }

    #[test]
    fn test_region_constraint_exists() {
        let rules = region_rules();
        assert!(rules.contains("All allocations must be deallocated"));
    }
}
