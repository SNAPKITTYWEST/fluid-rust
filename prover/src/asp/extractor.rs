//! ASP Fact Extractor: RMIR → ASP facts
//!
//! Converts RMIR execution state to ASP facts for the solver.
//! Key facts extracted:
//! - Ownership facts: owns(value, thread, timestamp)
//! - Region facts: region_status(region, timestamp, status), allocated_in(ptr, region, timestamp)
//! - Capability facts: capability(value, kind, timestamp)
//! - Effect facts: effect_emitted(effect, timestamp)

use std::io;

/// Main ASP extractor (Phase P1 stub)
pub struct AspExtractor {
    facts: AspFacts,
}

impl AspExtractor {
    pub fn new() -> Self {
        AspExtractor {
            facts: AspFacts::new(),
        }
    }

    pub fn extract_from_rmir(&mut self, _rmir_bytecode: &[u8]) -> io::Result<String> {
        // TODO: Phase P4 — parse RMIR and extract facts
        // For now, return stub facts
        Ok(self.facts.to_asp_program())
    }

    pub fn solve(&mut self) -> io::Result<super::proof::AspProof> {
        // Generate ASP program from facts
        let program = self.facts.to_asp_program();

        // Solve with ASP solver
        let solver = super::solver::AspSolver::new(&program);
        let result = solver.solve()?;

        // Convert solver result to proof
        match result {
            super::solver::SolveResult::Satisfiable { answer_set } => {
                Ok(super::proof::AspProof {
                    facts: self
                        .facts
                        .ownership_facts
                        .iter()
                        .map(|f| f.to_string())
                        .collect(),
                    rules: vec![],
                    satisfiable: true,
                    answer_set: answer_set.lines().map(|s| s.to_string()).collect(),
                })
            }
            super::solver::SolveResult::Unsatisfiable => {
                Ok(super::proof::AspProof {
                    facts: vec![],
                    rules: vec![],
                    satisfiable: false,
                    answer_set: vec![],
                })
            }
            super::solver::SolveResult::Unknown(msg) => {
                Err(io::Error::new(io::ErrorKind::Other, msg))
            }
        }
    }
}

/// Represents a fact in Answer Set Programming.
/// Facts are ground atoms (no variables at this stage).
#[derive(Debug, Clone)]
pub struct AspFact {
    pub predicate: String,
    pub args: Vec<String>,
}

impl AspFact {
    pub fn new(predicate: &str, args: Vec<&str>) -> Self {
        AspFact {
            predicate: predicate.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn to_string(&self) -> String {
        if self.args.is_empty() {
            format!("{}.", self.predicate)
        } else {
            format!("{}({}).", self.predicate, self.args.join(", "))
        }
    }
}

/// A collection of extracted facts, ready for the ASP solver.
pub struct AspFacts {
    pub ownership_facts: Vec<AspFact>,
    pub region_facts: Vec<AspFact>,
    pub capability_facts: Vec<AspFact>,
    pub effect_facts: Vec<AspFact>,
}

impl AspFacts {
    pub fn new() -> Self {
        AspFacts {
            ownership_facts: Vec::new(),
            region_facts: Vec::new(),
            capability_facts: Vec::new(),
            effect_facts: Vec::new(),
        }
    }

    /// Serialize all facts to ASP format (string).
    pub fn to_asp_program(&self) -> String {
        let mut program = String::new();

        program.push_str("% Ownership facts\n");
        for fact in &self.ownership_facts {
            program.push_str(&fact.to_string());
            program.push('\n');
        }

        program.push_str("\n% Region facts\n");
        for fact in &self.region_facts {
            program.push_str(&fact.to_string());
            program.push('\n');
        }

        program.push_str("\n% Capability facts\n");
        for fact in &self.capability_facts {
            program.push_str(&fact.to_string());
            program.push('\n');
        }

        program.push_str("\n% Effect facts\n");
        for fact in &self.effect_facts {
            program.push_str(&fact.to_string());
            program.push('\n');
        }

        program
    }

    /// Add an ownership fact: thread owns a value at a timestamp.
    pub fn add_ownership(&mut self, value_id: u32, thread: u32, timestamp: u32) {
        self.ownership_facts.push(AspFact::new(
            "owns",
            vec![&value_id.to_string(), &thread.to_string(), &timestamp.to_string()],
        ));
    }

    /// Add a region lifecycle fact: region has a status at a timestamp.
    pub fn add_region_status(&mut self, region_id: u32, timestamp: u32, status: &str) {
        self.region_facts.push(AspFact::new(
            "region_status",
            vec![&region_id.to_string(), &timestamp.to_string(), status],
        ));
    }

    /// Add an allocation fact: pointer is allocated in region at timestamp.
    pub fn add_allocated_in(&mut self, ptr_id: u32, region_id: u32, timestamp: u32) {
        self.region_facts.push(AspFact::new(
            "allocated_in",
            vec![&ptr_id.to_string(), &region_id.to_string(), &timestamp.to_string()],
        ));
    }

    /// Add a capability fact: value has a capability at timestamp.
    pub fn add_capability(&mut self, value_id: u32, capability: &str, timestamp: u32) {
        self.capability_facts.push(AspFact::new(
            "capability",
            vec![&value_id.to_string(), capability, &timestamp.to_string()],
        ));
    }

    /// Add an effect fact: effect was emitted at timestamp.
    pub fn add_effect_emitted(&mut self, effect: &str, timestamp: u32) {
        self.effect_facts.push(AspFact::new(
            "effect_emitted",
            vec![effect, &timestamp.to_string()],
        ));
    }
}

// TODO: Implement extraction from RmirFunction to AspFacts
// TODO: Implement timestamp assignment (program point numbering)
// TODO: Implement fact deduplication
// TODO: Implement fact validation (sanity checks before passing to solver)

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asp_fact_serialization() {
        let fact = AspFact::new("owns", vec!["42", "0", "100"]);
        assert_eq!(fact.to_string(), "owns(42, 0, 100).");
    }

    #[test]
    fn test_asp_facts_to_program() {
        let mut facts = AspFacts::new();
        facts.add_ownership(42, 0, 100);
        facts.add_region_status(1, 0, "unentered");

        let program = facts.to_asp_program();
        assert!(program.contains("owns(42, 0, 100)."));
        assert!(program.contains("region_status(1, 0, unentered)."));
    }
}
