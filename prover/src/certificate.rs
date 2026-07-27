//! Proof Certificate Generation and Serialization
//!
//! The proof certificate is a JSON artifact that bundles:
//! - RMIR bytecode
//! - ASP facts + answer set
//! - SMT assertions + model
//! - Digital signature (Ed25519)
//!
//! The certificate can be verified offline by the tiny verifier (~200 lines).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A proof certificate: a complete proof that a program is safe.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProofCertificate {
    pub metadata: CertificateMetadata,
    pub facts: ProofFacts,
    pub asp_result: AspProofResult,
    pub smt_result: SmtProofResult,
    pub signature: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CertificateMetadata {
    pub program_hash: String,          // SHA256 of RMIR bytecode
    pub program_name: String,
    pub timestamp: String,             // ISO 8601
    pub verifier_version: String,
    pub compiler_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProofFacts {
    pub rmir_instructions: Vec<RmirInstructionRecord>,
    pub ownership_facts: Vec<String>,  // ASP format
    pub region_facts: Vec<String>,
    pub capability_facts: Vec<String>,
    pub effect_facts: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RmirInstructionRecord {
    pub id: u32,
    pub opcode: String,
    pub args: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AspProofResult {
    pub satisfiable: bool,
    pub answer_set: Option<String>,
    pub diagnosis: Option<String>, // For unsatisfiable: unsat core or contradiction
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SmtProofResult {
    pub satisfiable: bool,
    pub model: Option<HashMap<String, String>>,
    pub diagnosis: Option<String>,
}

impl ProofCertificate {
    pub fn new(
        program_hash: String,
        program_name: String,
        facts: ProofFacts,
    ) -> Self {
        ProofCertificate {
            metadata: CertificateMetadata {
                program_hash,
                program_name,
                timestamp: "2026-07-26T12:34:56Z".to_string(),
                verifier_version: "0.1.0".to_string(),
                compiler_version: "0.1.0".to_string(),
            },
            facts,
            asp_result: AspProofResult {
                satisfiable: false,
                answer_set: None,
                diagnosis: None,
            },
            smt_result: SmtProofResult {
                satisfiable: false,
                model: None,
                diagnosis: None,
            },
            signature: String::new(),
        }
    }

    /// Check if the entire proof is valid (both ASP and SMT satisfied).
    pub fn is_valid(&self) -> bool {
        self.asp_result.satisfiable && self.smt_result.satisfiable
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_certificate_serialization() {
        let facts = ProofFacts {
            rmir_instructions: vec![],
            ownership_facts: vec!["owns(42, 0, 100).".to_string()],
            region_facts: vec![],
            capability_facts: vec![],
            effect_facts: vec![],
        };

        let cert = ProofCertificate::new(
            "abc123".to_string(),
            "example.rs".to_string(),
            facts,
        );

        let json = cert.to_json().unwrap();
        assert!(json.contains("abc123"));

        let cert2 = ProofCertificate::from_json(&json).unwrap();
        assert_eq!(cert2.metadata.program_hash, "abc123");
    }
}
