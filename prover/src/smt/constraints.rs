//! SMT Constraint Extraction: Proof Obligations → SMT Assertions
//!
//! Converts proof obligations into SMT-LIB2 format assertions.

use std::collections::HashMap as StdHashMap;

/// Represents a constraint on a variable.
#[derive(Debug, Clone)]
pub struct Constraint {
    pub variable: String,
    pub operator: String, // ">", "<", ">=", "<=", "==", "!="
    pub value: i64,
}

impl Constraint {
    pub fn to_smt_lib2(&self) -> String {
        match self.operator.as_str() {
            ">" => format!("(> {} {})", self.variable, self.value),
            "<" => format!("(< {} {})", self.variable, self.value),
            ">=" => format!("(>= {} {})", self.variable, self.value),
            "<=" => format!("(<= {} {})", self.variable, self.value),
            "==" => format!("(= {} {})", self.variable, self.value),
            "!=" => format!("(not (= {} {}))", self.variable, self.value),
            _ => panic!("Unknown operator: {}", self.operator),
        }
    }
}

/// Collection of constraints ready for SMT solver.
pub struct SmtConstraints {
    pub constraints: Vec<Constraint>,
    pub variable_types: StdHashMap<String, String>, // var -> "Int", "Real", etc.
}

impl SmtConstraints {
    pub fn new() -> Self {
        SmtConstraints {
            constraints: Vec::new(),
            variable_types: Default::default(),
        }
    }

    /// Add a numeric constraint.
    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    /// Declare a variable and its type.
    pub fn declare_variable(&mut self, name: &str, type_name: &str) {
        self.variable_types.insert(name.to_string(), type_name.to_string());
    }

    /// Generate SMT-LIB2 format output.
    pub fn to_smt_lib2(&self) -> String {
        let mut output = String::new();

        output.push_str("(set-logic QF_LIA)\n\n");

        output.push_str("; Variable declarations\n");
        for (var, type_name) in &self.variable_types {
            output.push_str(&format!("(declare-const {} {})\n", var, type_name));
        }

        output.push_str("\n; Constraints\n");
        for constraint in &self.constraints {
            output.push_str(&format!("(assert {})\n", constraint.to_smt_lib2()));
        }

        output.push_str("\n(check-sat)\n");
        output
    }

    /// Example: Generate constraints for a region allocation scenario.
    pub fn region_allocation_example() -> Self {
        let mut constraints = SmtConstraints::new();

        constraints.declare_variable("size_stack_0", "Int");
        constraints.declare_variable("used_stack_0", "Int");

        constraints.add_constraint(Constraint {
            variable: "size_stack_0".to_string(),
            operator: ">".to_string(),
            value: 0,
        });

        constraints.add_constraint(Constraint {
            variable: "used_stack_0".to_string(),
            operator: ">=".to_string(),
            value: 0,
        });

        constraints.add_constraint(Constraint {
            variable: "used_stack_0".to_string(),
            operator: "<=".to_string(),
            value: 4096,
        });

        constraints
    }
}

// TODO: Implement constraint extraction from RMIR proof obligations
// TODO: Implement numeric range inference
// TODO: Implement constraint simplification
// TODO: Implement incremental constraint building

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constraint_to_smt_lib2() {
        let c = Constraint {
            variable: "x".to_string(),
            operator: ">".to_string(),
            value: 0,
        };
        assert_eq!(c.to_smt_lib2(), "(> x 0)");
    }

    #[test]
    fn test_constraints_to_smt_lib2() {
        let constraints = SmtConstraints::region_allocation_example();
        let output = constraints.to_smt_lib2();
        assert!(output.contains("(declare-const size_stack_0 Int)"));
        assert!(output.contains("(assert (> size_stack_0 0))"));
        assert!(output.contains("(check-sat)"));
    }
}
