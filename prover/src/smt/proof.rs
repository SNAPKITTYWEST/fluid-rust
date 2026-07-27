//! SMT Proof Result

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SmtProof {
    pub assertions: Vec<String>,
    pub satisfiable: bool,
    pub model: HashMap<String, String>,
}

impl Default for SmtProof {
    fn default() -> Self {
        Self {
            assertions: Vec::new(),
            satisfiable: false,
            model: HashMap::new(),
        }
    }
}
