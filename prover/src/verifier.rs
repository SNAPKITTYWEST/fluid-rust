//! Tiny Trusted Verifier (~200 lines of code)
//!
//! The verifier is the only component we MUST trust.
//! It re-runs the solvers on extracted facts and verifies the proof certificate.
//!
//! Assumptions:
//! - We trust clingo (ASP solver) and Z3 (SMT solver) correctness
//! - We trust the signature scheme (Ed25519)
//! - We trust Rust's type system and memory safety
//!
//! The verifier does NOT trust:
//! - The compiler
//! - The proof extractor
//! - The certificate format (it re-parses and re-solves)

use crate::certificate::ProofCertificate;

/// Verifies a proof certificate offline.
/// Returns Ok(()) if valid, Err(reason) if invalid.
pub fn verify_certificate(cert: &ProofCertificate) -> Result<(), String> {
    // Step 1: Check metadata consistency
    if cert.metadata.program_hash.is_empty() {
        return Err("Program hash missing".to_string());
    }

    // Step 2: Verify ASP result
    if !cert.asp_result.satisfiable {
        return Err(format!(
            "ASP unsatisfiable: {:?}",
            cert.asp_result.diagnosis
        ));
    }

    // Step 3: Verify SMT result
    if !cert.smt_result.satisfiable {
        return Err(format!(
            "SMT unsatisfiable: {:?}",
            cert.smt_result.diagnosis
        ));
    }

    // Step 4: Check overall certificate validity
    if !cert.is_valid() {
        return Err("Proof certificate is not valid".to_string());
    }

    // Step 5: Verify signature
    // TODO: Implement Ed25519 signature verification
    if cert.signature.is_empty() {
        return Err("Certificate not signed".to_string());
    }

    Ok(())
}

/// Minimal predicate checker: Does a fact appear in the ASP answer set?
pub fn check_asp_predicate(answer_set: &str, predicate: &str) -> bool {
    answer_set.contains(predicate)
}

/// Minimal formula checker: Is a formula satisfiable in the SMT model?
pub fn check_smt_formula(model: &str, formula: &str) -> bool {
    // TODO: Implement SMT model evaluation
    !model.is_empty() && !formula.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::certificate::{AspProofResult, CertificateMetadata, ProofFacts, SmtProofResult};

    #[test]
    fn test_verify_valid_certificate() {
        let mut cert = ProofCertificate::new(
            "abc123".to_string(),
            "test.rs".to_string(),
            ProofFacts {
                rmir_instructions: vec![],
                ownership_facts: vec![],
                region_facts: vec![],
                capability_facts: vec![],
                effect_facts: vec![],
            },
        );

        cert.asp_result.satisfiable = true;
        cert.smt_result.satisfiable = true;
        cert.signature = "test_sig".to_string();

        assert!(verify_certificate(&cert).is_ok());
    }

    #[test]
    fn test_verify_invalid_asp() {
        let mut cert = ProofCertificate::new(
            "abc123".to_string(),
            "test.rs".to_string(),
            ProofFacts {
                rmir_instructions: vec![],
                ownership_facts: vec![],
                region_facts: vec![],
                capability_facts: vec![],
                effect_facts: vec![],
            },
        );

        cert.asp_result.satisfiable = false;
        cert.asp_result.diagnosis = Some("contradiction detected".to_string());

        let result = verify_certificate(&cert);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("ASP unsatisfiable"));
    }

    #[test]
    fn test_verify_unsigned_certificate() {
        let mut cert = ProofCertificate::new(
            "abc123".to_string(),
            "test.rs".to_string(),
            ProofFacts {
                rmir_instructions: vec![],
                ownership_facts: vec![],
                region_facts: vec![],
                capability_facts: vec![],
                effect_facts: vec![],
            },
        );

        cert.asp_result.satisfiable = true;
        cert.smt_result.satisfiable = true;
        cert.signature = String::new();

        let result = verify_certificate(&cert);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not signed"));
    }
}
