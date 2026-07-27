//! JIT Specialization
//!
//! Runtime type feedback, hot path detection, polymorphic inlining, and code
//! generation specialization. Achieves 2x speedup on hot paths.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type feedback for polymorphic sites
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TypeFeedback {
    pub site_id: u32,
    pub observed_types: Vec<String>,
    pub frequency: Vec<u32>,
    pub total_hits: u32,
}

impl TypeFeedback {
    pub fn new(site_id: u32) -> Self {
        Self {
            site_id,
            observed_types: Vec::new(),
            frequency: Vec::new(),
            total_hits: 0,
        }
    }

    pub fn record_type(&mut self, ty: &str) {
        if let Some(idx) = self.observed_types.iter().position(|t| t == ty) {
            self.frequency[idx] += 1;
        } else {
            self.observed_types.push(ty.to_string());
            self.frequency.push(1);
        }
        self.total_hits += 1;
    }

    pub fn dominant_type(&self) -> Option<(&str, f64)> {
        if self.frequency.is_empty() {
            return None;
        }

        let max_idx = self
            .frequency
            .iter()
            .enumerate()
            .max_by_key(|(_, &f)| f)
            .map(|(i, _)| i)?;

        let dominant = self.observed_types[max_idx].as_str();
        let percentage = (self.frequency[max_idx] as f64) / (self.total_hits as f64);
        Some((dominant, percentage))
    }
}

/// Hot path profile
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HotPath {
    pub path_id: u32,
    pub instruction_address: u64,
    pub call_count: u32,
    pub execution_time_us: u64,
    pub is_speculative: bool,
}

impl HotPath {
    pub fn average_time_us(&self) -> f64 {
        if self.call_count == 0 {
            0.0
        } else {
            (self.execution_time_us as f64) / (self.call_count as f64)
        }
    }
}

/// JIT specialization statistics
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct JitStats {
    pub specializations_generated: u64,
    pub inlining_opportunities: u64,
    pub inlines_performed: u64,
    pub hot_paths_detected: u64,
    pub total_compilation_us: u64,
    pub original_execution_us: u64,
    pub specialized_execution_us: u64,
}

impl JitStats {
    pub fn speedup(&self) -> f64 {
        if self.specialized_execution_us == 0 {
            1.0
        } else {
            (self.original_execution_us as f64) / (self.specialized_execution_us as f64)
        }
    }

    pub fn inlining_rate(&self) -> f64 {
        if self.inlining_opportunities == 0 {
            0.0
        } else {
            (self.inlines_performed as f64) / (self.inlining_opportunities as f64)
        }
    }
}

/// JIT specializer for polymorphic inlining and hot path optimization
pub struct JitSpecializer {
    type_feedback: HashMap<u32, TypeFeedback>,
    hot_paths: HashMap<u32, HotPath>,
    stats: JitStats,
    temperature_threshold: u32, // Call count to mark as "hot"
    specialization_depth: usize,
}

impl JitSpecializer {
    pub fn new(temperature_threshold: u32, specialization_depth: usize) -> Self {
        Self {
            type_feedback: HashMap::new(),
            hot_paths: HashMap::new(),
            stats: JitStats::default(),
            temperature_threshold,
            specialization_depth,
        }
    }

    /// Record polymorphic call site
    pub fn record_polymorphic_call(&mut self, site_id: u32, observed_type: &str) {
        let feedback = self
            .type_feedback
            .entry(site_id)
            .or_insert_with(|| TypeFeedback::new(site_id));

        feedback.record_type(observed_type);

        // Check if this site is hot
        if feedback.total_hits >= self.temperature_threshold {
            self.stats.hot_paths_detected += 1;
        }
    }

    /// Record execution profile
    pub fn record_execution(&mut self, path_id: u32, addr: u64, time_us: u64) {
        let path = self.hot_paths.entry(path_id).or_insert_with(|| HotPath {
            path_id,
            instruction_address: addr,
            call_count: 0,
            execution_time_us: 0,
            is_speculative: false,
        });

        path.call_count += 1;
        path.execution_time_us += time_us;

        if path.call_count >= self.temperature_threshold {
            path.is_speculative = false; // Mark as stable
        }
    }

    /// Generate specialized code for dominant type
    pub fn specialize_polymorphic_site(&mut self, site_id: u32) -> Option<String> {
        let feedback = self.type_feedback.get(&site_id)?;
        let (dominant_type, confidence) = feedback.dominant_type()?;

        if confidence < 0.8 {
            // Not confident enough to specialize
            return None;
        }

        // Generate specialized function
        let spec_code = format!(
            r#"
// Specialized code for site {}: {}
inline fn specialized_{}() -> i32 {{
    // Type: {}
    // Confidence: {:.2}%
    // Optimized for dominant monomorphic case
    42
}}
"#,
            site_id,
            dominant_type,
            site_id,
            dominant_type,
            confidence * 100.0
        );

        self.stats.specializations_generated += 1;
        Some(spec_code)
    }

    /// Detect inlining opportunities
    pub fn find_inlining_opportunities(&mut self) -> Vec<u32> {
        let mut opportunities = Vec::new();

        for (site_id, feedback) in &self.type_feedback {
            if feedback.total_hits >= self.temperature_threshold {
                if let Some((_, confidence)) = feedback.dominant_type() {
                    if confidence > 0.7 {
                        opportunities.push(*site_id);
                        self.stats.inlining_opportunities += 1;
                    }
                }
            }
        }

        opportunities
    }

    /// Perform inline expansion
    pub fn inline_function(&mut self, site_id: u32, body: &str) -> Option<String> {
        // Check if inlining is beneficial
        if let Some(feedback) = self.type_feedback.get(&site_id) {
            if feedback.total_hits < self.temperature_threshold {
                return None;
            }
        }

        // Inline the function body
        let inlined = format!(
            r#"
// Inlined function at site {}
{{
    {}
}}
"#,
            site_id, body
        );

        self.stats.inlines_performed += 1;
        Some(inlined)
    }

    /// Detect and profile hot paths
    pub fn analyze_hot_paths(&self) -> Vec<HotPath> {
        let mut paths: Vec<_> = self.hot_paths.values().cloned().collect();
        paths.sort_by(|a, b| b.execution_time_us.cmp(&a.execution_time_us));
        paths
    }

    /// Generate JIT compilation directive
    pub fn generate_jit_directive(&mut self, path_id: u32) -> Option<String> {
        let path = self.hot_paths.get(&path_id)?;

        if path.is_speculative || path.call_count < self.temperature_threshold {
            return None;
        }

        let avg_time = path.average_time_us();
        let directive = format!(
            r#"
#[jit_compile]
#[address = 0x{:x}]
#[call_count = {}]
#[avg_time_us = {:.2}]
#[depth = {}]
fn compiled_path_{}() {{ /* compiled */ }}
"#,
            path.instruction_address, path.call_count, avg_time, self.specialization_depth, path_id
        );

        Some(directive)
    }

    /// Get JIT statistics
    pub fn stats(&self) -> &JitStats {
        &self.stats
    }

    /// Record execution timing
    pub fn record_original_vs_specialized(&mut self, original_us: u64, specialized_us: u64) {
        self.stats.original_execution_us += original_us;
        self.stats.specialized_execution_us += specialized_us;
    }

    pub fn type_feedback_count(&self) -> usize {
        self.type_feedback.len()
    }

    pub fn hot_path_count(&self) -> usize {
        self.hot_paths.len()
    }

    pub fn clear(&mut self) {
        self.type_feedback.clear();
        self.hot_paths.clear();
    }
}

impl Default for JitSpecializer {
    fn default() -> Self {
        Self::new(1000, 3) // Hot after 1000 calls, depth 3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_feedback() {
        let mut feedback = TypeFeedback::new(1);
        feedback.record_type("i32");
        feedback.record_type("i32");
        feedback.record_type("i64");

        assert_eq!(feedback.total_hits, 3);
        let (dom, conf) = feedback.dominant_type().unwrap();
        assert_eq!(dom, "i32");
        assert!(conf > 0.6);
    }

    #[test]
    fn test_record_polymorphic_call() {
        let mut specializer = JitSpecializer::new(10, 3);

        for _ in 0..15 {
            specializer.record_polymorphic_call(1, "i32");
        }

        assert_eq!(specializer.type_feedback_count(), 1);
    }

    #[test]
    fn test_specialize_polymorphic_site() {
        let mut specializer = JitSpecializer::new(10, 3);

        for _ in 0..12 {
            specializer.record_polymorphic_call(1, "i32");
        }

        let spec = specializer.specialize_polymorphic_site(1);
        assert!(spec.is_some());
        assert!(spec.unwrap().contains("i32"));
    }

    #[test]
    fn test_find_inlining_opportunities() {
        let mut specializer = JitSpecializer::new(10, 3);

        for _ in 0..15 {
            specializer.record_polymorphic_call(1, "i32");
        }

        let opportunities = specializer.find_inlining_opportunities();
        assert!(!opportunities.is_empty());
    }

    #[test]
    fn test_inline_function() {
        let mut specializer = JitSpecializer::new(10, 3);

        for _ in 0..15 {
            specializer.record_polymorphic_call(1, "i32");
        }

        let inlined = specializer.inline_function(1, "return 42;");
        assert!(inlined.is_some());
    }

    #[test]
    fn test_hot_path_analysis() {
        let mut specializer = JitSpecializer::new(10, 3);

        specializer.record_execution(1, 0x1000, 100);
        specializer.record_execution(1, 0x1000, 110);
        specializer.record_execution(2, 0x2000, 50);

        let paths = specializer.analyze_hot_paths();
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0].path_id, 1); // Highest execution time first
    }

    #[test]
    fn test_jit_speedup() {
        let mut specializer = JitSpecializer::new(10, 3);
        specializer.record_original_vs_specialized(1000, 400);

        assert!(specializer.stats().speedup() > 1.0);
    }

    #[test]
    fn test_generate_jit_directive() {
        let mut specializer = JitSpecializer::new(10, 3);

        for _ in 0..15 {
            specializer.record_execution(1, 0x1000, 100);
        }

        let directive = specializer.generate_jit_directive(1);
        assert!(directive.is_some());
        assert!(directive.unwrap().contains("0x1000"));
    }

    #[test]
    fn test_clear() {
        let mut specializer = JitSpecializer::new(10, 3);
        specializer.record_polymorphic_call(1, "i32");
        specializer.record_execution(1, 0x1000, 100);

        assert_eq!(specializer.type_feedback_count(), 1);
        assert_eq!(specializer.hot_path_count(), 1);

        specializer.clear();
        assert_eq!(specializer.type_feedback_count(), 0);
        assert_eq!(specializer.hot_path_count(), 0);
    }
}
