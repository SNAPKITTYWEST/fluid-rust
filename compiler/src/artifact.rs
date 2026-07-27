//! Compilation Artifact — complete output of the compiler pipeline

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The complete output of a successful compilation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompilationArtifact {
    /// Hash of source code (SHA256)
    pub source_hash: String,
    /// Hash of typed IR (SHA256)
    pub typed_ir_hash: String,
    /// Hash of RMIR bytecode (Blake3)
    pub rmir_hash: String,
    /// Hash of proof obligations (Blake3)
    pub obligation_hash: String,
    /// Hash of generated backend artifact
    pub artifact_hash: Option<String>,
    /// Compiler version
    pub compiler_version: String,
    /// Backend selection: "native", "managed", "wasm", or "proof-only"
    pub backend: String,
    /// Proof mode: "generate", "verify", or "none"
    pub proof_mode: String,
    /// Source file path
    pub source_file: String,
    /// Generated RMIR bytecode (binary)
    pub rmir_bytecode: Vec<u8>,
    /// Proof obligations (JSON)
    pub proof_obligations: serde_json::Value,
    /// Backend-specific outputs
    pub backend_outputs: HashMap<String, Vec<u8>>,
    /// Compilation diagnostics
    pub diagnostics: Vec<Diagnostic>,
    /// Timestamp (ISO 8601)
    pub timestamp: String,
}

/// A compilation diagnostic (error, warning, or info)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub source_location: Option<SourceLocation>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Info,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

impl CompilationArtifact {
    pub fn new(
        source_hash: String,
        compiler_version: String,
        source_file: String,
        rmir_bytecode: Vec<u8>,
        backend: String,
    ) -> Self {
        let rmir_hash = blake3::hash(&rmir_bytecode).to_hex().to_string();

        Self {
            source_hash,
            typed_ir_hash: String::new(), // TODO: compute from typed IR
            rmir_hash,
            obligation_hash: String::new(), // TODO: compute from obligations
            artifact_hash: None,
            compiler_version,
            backend,
            proof_mode: "generate".to_string(),
            source_file,
            rmir_bytecode,
            proof_obligations: serde_json::json!({}),
            backend_outputs: HashMap::new(),
            diagnostics: Vec::new(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn add_diagnostic(&mut self, level: DiagnosticLevel, message: String) {
        self.diagnostics.push(Diagnostic {
            level,
            message,
            source_location: None,
        });
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| matches!(d.level, DiagnosticLevel::Error))
    }
}
