//! Z3 Solver Bridge: Interface to Z3
//!
//! This module wraps the Z3 SMT solver backend.
//! Mock implementation for Phase P3; real Z3 integration deferred to Phase P4+.

use std::collections::HashMap as StdHashMap;

/// Represents the result of SMT solving.
#[derive(Debug, PartialEq, Eq)]
pub enum SolveResult {
    /// Constraints are satisfiable
    Satisfiable { model: StdHashMap<String, String> },
    /// Constraints are unsatisfiable (provably false)
    Unsatisfiable { unsat_core: Vec<String> },
    /// Solver encountered unknown issue
    Unknown(String),
}

pub struct Z3Solver {
    assertions: String,
}

impl Z3Solver {
    pub fn new(assertions: &str) -> Self {
        Z3Solver {
            assertions: assertions.to_string(),
        }
    }

    /// Solve a set of SMT-LIB2 format constraints.
    ///
    /// PHASE P3: Mock implementation
    /// - If assertions contain "unsat" keyword, returns UNSAT
    /// - Otherwise returns SAT with simple model assignment
    ///
    /// PHASE P4+: Real Z3 integration
    pub fn solve(&self) -> SolveResult {
        // Mock: detect unsatisfiable patterns
        if self.assertions.contains("unsat")
            || self.assertions.contains("false")
            || self.assertions.contains(">= 1000000")
            || self.assertions.contains("< 0")
        {
            return SolveResult::Unsatisfiable {
                unsat_core: vec!["constraint".to_string()],
            };
        }

        // Otherwise, return SAT with mock model
        let mut model = StdHashMap::new();
        model.insert("n".to_string(), "512".to_string());
        model.insert("i".to_string(), "10".to_string());

        SolveResult::Satisfiable { model }
    }

    /// Check if a formula is satisfiable.
    pub fn is_satisfiable(&self) -> bool {
        matches!(self.solve(), SolveResult::Satisfiable { .. })
    }

    /// Get a model (variable assignments) that satisfies the constraints.
    pub fn get_model(&self) -> Option<StdHashMap<String, String>> {
        match self.solve() {
            SolveResult::Satisfiable { model } => Some(model),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_z3_solver_satisfiable() {
        let solver = Z3Solver::new("(> n 0) (<= n 1024)");
        assert!(solver.is_satisfiable());
    }

    #[test]
    fn test_z3_solver_unsatisfiable() {
        let solver = Z3Solver::new("unsat constraint: (< n 0)");
        match solver.solve() {
            SolveResult::Unsatisfiable { .. } => {}
            _ => panic!("Expected UNSAT"),
        }
    }

    #[test]
    fn test_z3_get_model() {
        let solver = Z3Solver::new("(> n 0)");
        let model = solver.get_model();
        assert!(model.is_some());
        assert!(model.unwrap().contains_key("n"));
    }
}
