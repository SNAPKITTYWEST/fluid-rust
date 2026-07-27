//! ASP Proof Result

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AspProof {
    pub facts: Vec<String>,
    pub rules: Vec<String>,
    pub satisfiable: bool,
    pub answer_set: Vec<String>,
}

impl Default for AspProof {
    fn default() -> Self {
        Self {
            facts: Vec::new(),
            rules: Vec::new(),
            satisfiable: false,
            answer_set: Vec::new(),
        }
    }
}
