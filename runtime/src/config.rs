//! Production Configuration
//!
//! Tunable parameters (memory limits, timeouts, cache sizes), runtime feature flags,
//! determinism control, and audit trail configuration.

use serde::{Deserialize, Serialize};
use std::io;

/// Feature flags for runtime behavior
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub proof_caching_enabled: bool,
    pub effect_batching_enabled: bool,
    pub jit_compilation_enabled: bool,
    pub profiling_enabled: bool,
    pub determinism_mode: bool,
    pub checkpoint_recovery_enabled: bool,
    pub deadline_enforcement_enabled: bool,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            proof_caching_enabled: true,
            effect_batching_enabled: true,
            jit_compilation_enabled: true,
            profiling_enabled: true,
            determinism_mode: false,
            checkpoint_recovery_enabled: true,
            deadline_enforcement_enabled: true,
        }
    }
}

/// Memory configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub max_heap_size: usize,
    pub max_stack_size: usize,
    pub gc_trigger_threshold: usize,
    pub max_proof_cache_size: usize,
    pub region_alloc_limit: usize,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_heap_size: 1024 * 1024 * 1024,    // 1GB
            max_stack_size: 8 * 1024 * 1024,      // 8MB
            gc_trigger_threshold: 512 * 1024 * 1024, // 512MB
            max_proof_cache_size: 128 * 1024 * 1024, // 128MB
            region_alloc_limit: 256 * 1024 * 1024,   // 256MB per region
        }
    }
}

/// Timeout configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimeoutConfig {
    pub default_task_timeout_ms: u64,
    pub default_effect_timeout_ms: u64,
    pub gc_timeout_ms: u64,
    pub proof_verification_timeout_ms: u64,
    pub checkpoint_save_timeout_ms: u64,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            default_task_timeout_ms: 30000,      // 30 seconds
            default_effect_timeout_ms: 5000,     // 5 seconds
            gc_timeout_ms: 10000,                // 10 seconds
            proof_verification_timeout_ms: 60000, // 60 seconds
            checkpoint_save_timeout_ms: 5000,    // 5 seconds
        }
    }
}

/// Determinism configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeterminismConfig {
    pub strict_determinism: bool,
    pub seed: u64,
    pub verify_determinism: bool,
    pub determinism_check_interval: u32,
}

impl Default for DeterminismConfig {
    fn default() -> Self {
        Self {
            strict_determinism: false,
            seed: 42,
            verify_determinism: false,
            determinism_check_interval: 1000,
        }
    }
}

/// Audit trail configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditConfig {
    pub audit_enabled: bool,
    pub log_all_effects: bool,
    pub log_all_proofs: bool,
    pub log_gc_events: bool,
    pub audit_file_path: String,
    pub audit_buffer_size: usize,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            audit_enabled: true,
            log_all_effects: false,
            log_all_proofs: true,
            log_gc_events: true,
            audit_file_path: "audit.jsonl".to_string(),
            audit_buffer_size: 10000,
        }
    }
}

/// Profiling configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProfilingConfig {
    pub enable_latency_profiling: bool,
    pub enable_memory_profiling: bool,
    pub enable_cache_profiling: bool,
    pub sample_rate: f64, // 0.0 to 1.0
    pub export_interval_ms: u64,
}

impl Default for ProfilingConfig {
    fn default() -> Self {
        Self {
            enable_latency_profiling: true,
            enable_memory_profiling: true,
            enable_cache_profiling: true,
            sample_rate: 0.1, // Sample 10% of events
            export_interval_ms: 10000, // Export every 10 seconds
        }
    }
}

/// Production configuration (all tunable parameters)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProductionConfig {
    pub features: FeatureFlags,
    pub memory: MemoryConfig,
    pub timeouts: TimeoutConfig,
    pub determinism: DeterminismConfig,
    pub audit: AuditConfig,
    pub profiling: ProfilingConfig,
}

impl ProductionConfig {
    /// Create default production configuration
    pub fn new() -> Self {
        Self {
            features: FeatureFlags::default(),
            memory: MemoryConfig::default(),
            timeouts: TimeoutConfig::default(),
            determinism: DeterminismConfig::default(),
            audit: AuditConfig::default(),
            profiling: ProfilingConfig::default(),
        }
    }

    /// Create conservative configuration (low resource usage)
    pub fn conservative() -> Self {
        let mut config = Self::new();
        config.memory.max_heap_size = 256 * 1024 * 1024; // 256MB
        config.memory.max_proof_cache_size = 32 * 1024 * 1024; // 32MB
        config.features.effect_batching_enabled = true;
        config.profiling.sample_rate = 0.01; // Sample 1% of events
        config
    }

    /// Create aggressive configuration (high performance)
    pub fn aggressive() -> Self {
        let mut config = Self::new();
        config.memory.max_heap_size = 4 * 1024 * 1024 * 1024; // 4GB
        config.memory.max_proof_cache_size = 512 * 1024 * 1024; // 512MB
        config.features.jit_compilation_enabled = true;
        config.profiling.sample_rate = 0.5; // Sample 50% of events
        config
    }

    /// Create testing configuration
    pub fn testing() -> Self {
        let mut config = Self::new();
        config.memory.max_heap_size = 128 * 1024 * 1024; // 128MB
        config.determinism.strict_determinism = true;
        config.determinism.verify_determinism = true;
        config.audit.log_all_effects = true;
        config.audit.log_all_proofs = true;
        config
    }

    /// Load from JSON file
    pub fn from_json_file(path: &str) -> io::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        serde_json::from_str(&contents)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
    }

    /// Save to JSON file
    pub fn save_to_json_file(&self, path: &str) -> io::Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Validate configuration for internal consistency
    pub fn validate(&self) -> io::Result<()> {
        if self.memory.max_heap_size < 1024 * 1024 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "max_heap_size must be at least 1MB",
            ));
        }

        if self.memory.gc_trigger_threshold > self.memory.max_heap_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "gc_trigger_threshold cannot exceed max_heap_size",
            ));
        }

        if self.profiling.sample_rate < 0.0 || self.profiling.sample_rate > 1.0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "sample_rate must be between 0.0 and 1.0",
            ));
        }

        Ok(())
    }

    /// Export as JSON string
    pub fn to_json_string(&self) -> io::Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
    }
}

impl Default for ProductionConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Runtime configuration builder
pub struct ConfigBuilder {
    config: ProductionConfig,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: ProductionConfig::new(),
        }
    }

    pub fn with_feature(mut self, feature: &str, enabled: bool) -> Self {
        match feature {
            "proof_caching" => self.config.features.proof_caching_enabled = enabled,
            "effect_batching" => self.config.features.effect_batching_enabled = enabled,
            "jit" => self.config.features.jit_compilation_enabled = enabled,
            "profiling" => self.config.features.profiling_enabled = enabled,
            "determinism" => self.config.features.determinism_mode = enabled,
            "checkpoints" => self.config.features.checkpoint_recovery_enabled = enabled,
            "deadlines" => self.config.features.deadline_enforcement_enabled = enabled,
            _ => {}
        }
        self
    }

    pub fn with_max_heap_size(mut self, bytes: usize) -> Self {
        self.config.memory.max_heap_size = bytes;
        self
    }

    pub fn with_default_task_timeout(mut self, ms: u64) -> Self {
        self.config.timeouts.default_task_timeout_ms = ms;
        self
    }

    pub fn with_profiling_enabled(mut self, enabled: bool) -> Self {
        self.config.features.profiling_enabled = enabled;
        self
    }

    pub fn with_strict_determinism(mut self, strict: bool) -> Self {
        self.config.determinism.strict_determinism = strict;
        self
    }

    pub fn build(self) -> io::Result<ProductionConfig> {
        self.config.validate()?;
        Ok(self.config)
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ProductionConfig::new();
        assert!(config.features.proof_caching_enabled);
        assert!(config.memory.max_heap_size > 0);
    }

    #[test]
    fn test_conservative_config() {
        let config = ProductionConfig::conservative();
        assert!(config.memory.max_heap_size < ProductionConfig::new().memory.max_heap_size);
    }

    #[test]
    fn test_aggressive_config() {
        let config = ProductionConfig::aggressive();
        assert!(config.memory.max_heap_size > ProductionConfig::new().memory.max_heap_size);
    }

    #[test]
    fn test_testing_config() {
        let config = ProductionConfig::testing();
        assert!(config.determinism.strict_determinism);
        assert!(config.determinism.verify_determinism);
    }

    #[test]
    fn test_validate_config() {
        let mut config = ProductionConfig::new();
        assert!(config.validate().is_ok());

        config.memory.max_heap_size = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_builder() {
        let config = ConfigBuilder::new()
            .with_feature("proof_caching", false)
            .with_max_heap_size(512 * 1024 * 1024)
            .with_default_task_timeout(60000)
            .build()
            .unwrap();

        assert!(!config.features.proof_caching_enabled);
        assert_eq!(config.memory.max_heap_size, 512 * 1024 * 1024);
        assert_eq!(config.timeouts.default_task_timeout_ms, 60000);
    }

    #[test]
    fn test_config_json_serialization() {
        let config = ProductionConfig::new();
        let json = config.to_json_string().unwrap();
        assert!(!json.is_empty());
        assert!(json.contains("proof_caching_enabled"));
    }

    #[test]
    fn test_sample_rate_validation() {
        let mut config = ProductionConfig::new();
        config.profiling.sample_rate = 1.5;
        assert!(config.validate().is_err());

        config.profiling.sample_rate = 0.5;
        assert!(config.validate().is_ok());
    }
}
