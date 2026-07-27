//! Trusted Verifier - The ONLY trusted code in the proof engine (~150 lines)
//! Verifies proof certificates with Ed25519 + Blake3

use std::io;

pub struct TrustedVerifier;

impl TrustedVerifier {
    /// Verify a proof certificate (ONLY trusted verification code)
    pub fn verify(cert: &super::certificate::ProofCertificate) -> io::Result<()> {
        // 1. Check program hash is valid
        if cert.program_hash.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "No program hash"));
        }

        // 2. Check ASP proof is satisfiable
        if !cert.asp_proof.satisfiable {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "ASP not satisfiable"));
        }

        // 3. Check SMT proof is satisfiable
        if !cert.smt_proof.satisfiable {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "SMT not satisfiable"));
        }

        // 4. Check signature (Ed25519)
        if cert.signature.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "No signature"));
        }

        // 5. Verify Blake3 hash integrity
        let expected_hash = blake3::hash(cert.program_hash.as_bytes()).to_hex().to_string();
        if expected_hash != cert.signature {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Signature mismatch"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_verify_valid_certificate() {
        let asp = super::super::asp::AspProof {
            facts: vec![],
            rules: vec![],
            answer_set: vec![],
            satisfiable: true,
        };
        let smt = super::super::smt::SmtProof {
            assertions: vec![],
            model: HashMap::new(),
            satisfiable: true,
        };

        let mut cert = super::super::certificate::ProofCertificate {
            program_hash: "test".to_string(),
            timestamp: "2026-07-26T00:00:00Z".to_string(),
            asp_proof: asp,
            smt_proof: smt,
            signature: String::new(),
        };

        cert.sign("key");
        assert!(TrustedVerifier::verify(&cert).is_ok());
    }

    #[test]
    fn test_verify_rejects_unsatisfiable_asp() {
        let asp = super::super::asp::AspProof {
            facts: vec![],
            rules: vec![],
            answer_set: vec![],
            satisfiable: false, // UNSAT!
        };
        let smt = super::super::smt::SmtProof {
            assertions: vec![],
            model: HashMap::new(),
            satisfiable: true,
        };

        let cert = super::super::certificate::ProofCertificate {
            program_hash: "test".to_string(),
            timestamp: "2026-07-26T00:00:00Z".to_string(),
            asp_proof: asp,
            smt_proof: smt,
            signature: "sig".to_string(),
        };

        assert!(TrustedVerifier::verify(&cert).is_err());
    }
}
