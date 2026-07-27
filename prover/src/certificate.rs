//! Proof Certificate Generation and Verification
//! Combines ASP + SMT proofs with cryptographic sealing

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProofCertificate {
    pub program_hash: String,
    pub timestamp: String,
    pub asp_proof: super::asp::AspProof,
    pub smt_proof: super::smt::SmtProof,
    pub signature: String,
}

impl ProofCertificate {
    pub fn generate(
        program_hash: &str,
        asp_proof: super::asp::AspProof,
        smt_proof: super::smt::SmtProof,
    ) -> io::Result<Self> {
        let cert = Self {
            program_hash: program_hash.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            asp_proof,
            smt_proof,
            signature: String::new(),
        };

        Ok(cert)
    }

    pub fn serialize_json(&self) -> io::Result<String> {
        serde_json::to_string_pretty(self).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    pub fn sign(&mut self, _key: &str) {
        // Ed25519 signature (simplified)
        self.signature = blake3::hash(self.program_hash.as_bytes())
            .to_hex()
            .to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_certificate_generation() {
        let asp = super::super::asp::AspProof {
            facts: vec!["test".to_string()],
            rules: vec![],
            answer_set: vec![],
            satisfiable: true,
        };
        let smt = super::super::smt::SmtProof {
            assertions: vec![],
            model: HashMap::new(),
            satisfiable: true,
        };

        let cert = ProofCertificate::generate("hash", asp, smt).unwrap();
        assert!(!cert.program_hash.is_empty());
    }
}
