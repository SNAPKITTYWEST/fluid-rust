//! ASP Solver Bridge: Interface to clingo
//!
//! This module wraps the clingo solver backend.
//! Mock implementation for Phase P3; real clingo integration deferred to Phase P4+.

use std::io;

/// Represents the result of ASP solving.
#[derive(Debug)]
pub enum SolveResult {
    /// Solver found a satisfying answer set
    Satisfiable { answer_set: String },
    /// No satisfying answer set exists
    Unsatisfiable,
    /// Solver timed out or encountered unknown issue
    Unknown(String),
}

pub struct AspSolver {
    program: String,
}

impl AspSolver {
    pub fn new(program: &str) -> Self {
        AspSolver {
            program: program.to_string(),
        }
    }

    /// Solve an ASP program (facts + rules).
    /// Returns satisfiability and answer set if available.
    ///
    /// PHASE P3: Mock implementation
    /// - If program contains "contradiction" or "conflict", returns UNSAT
    /// - Otherwise returns SAT with empty answer set
    ///
    /// PHASE P4+: Real clingo integration
    pub fn solve(&self) -> io::Result<SolveResult> {
        // Mock: detect common unsatisfiable patterns
        if self.program.contains("contradiction")
            || self.program.contains("conflict")
            || self.program.contains("double_use")
            || self.program.contains("access_to_closed") {
            return Ok(SolveResult::Unsatisfiable);
        }

        // Otherwise, assume satisfiable
        Ok(SolveResult::Satisfiable {
            answer_set: format!(
                "Answer: 1\n{}\nSATISFIABLE\n",
                extract_facts(&self.program)
            ),
        })
    }

    /// Check if a particular predicate holds in the answer set.
    pub fn check_predicate(&self, answer_set: &str, predicate: &str) -> bool {
        answer_set.contains(predicate)
    }
}

/// Extract ASP facts from program (simple text parsing)
fn extract_facts(program: &str) -> String {
    program
        .lines()
        .filter(|line| line.ends_with('.') && !line.trim_start().starts_with('%'))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asp_solver_satisfiable() {
        let program = "owns(x, 0, 100).\nlinear(x).";
        let solver = AspSolver::new(program);
        match solver.solve().unwrap() {
            SolveResult::Satisfiable { .. } => {},
            _ => panic!("Expected SAT"),
        }
    }

    #[test]
    fn test_asp_solver_unsatisfiable() {
        let program = "contradiction.\ndouble_use(x).";
        let solver = AspSolver::new(program);
        match solver.solve().unwrap() {
            SolveResult::Unsatisfiable => {},
            _ => panic!("Expected UNSAT"),
        }
    }

    #[test]
    fn test_check_predicate() {
        let program = "owns(x, 0, 100).";
        let solver = AspSolver::new(program);
        if let Ok(SolveResult::Satisfiable { answer_set }) = solver.solve() {
            assert!(solver.check_predicate(&answer_set, "owns"));
        }
    }
}
