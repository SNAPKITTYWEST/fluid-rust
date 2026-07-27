//! Performance Profiler
//!
//! Effect latency tracking, task scheduling statistics, GC pause analysis,
//! and proof cache hit rate metrics.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Histogram bucket for latency distribution
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LatencyBucket {
    pub min_us: u32,
    pub max_us: u32,
    pub count: u64,
}

/// Effect latency metrics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EffectMetrics {
    pub effect_type: String,
    pub total_calls: u64,
    pub total_latency_us: u64,
    pub min_latency_us: u32,
    pub max_latency_us: u32,
    pub latency_distribution: Vec<LatencyBucket>,
}

impl EffectMetrics {
    pub fn new(effect_type: &str) -> Self {
        Self {
            effect_type: effect_type.to_string(),
            total_calls: 0,
            total_latency_us: 0,
            min_latency_us: u32::MAX,
            max_latency_us: 0,
            latency_distribution: vec![
                LatencyBucket {
                    min_us: 0,
                    max_us: 10,
                    count: 0,
                },
                LatencyBucket {
                    min_us: 10,
                    max_us: 100,
                    count: 0,
                },
                LatencyBucket {
                    min_us: 100,
                    max_us: 1000,
                    count: 0,
                },
                LatencyBucket {
                    min_us: 1000,
                    max_us: 10000,
                    count: 0,
                },
                LatencyBucket {
                    min_us: 10000,
                    max_us: u32::MAX,
                    count: 0,
                },
            ],
        }
    }

    pub fn record_latency(&mut self, latency_us: u32) {
        self.total_calls += 1;
        self.total_latency_us += latency_us as u64;
        self.min_latency_us = self.min_latency_us.min(latency_us);
        self.max_latency_us = self.max_latency_us.max(latency_us);

        // Update histogram
        for bucket in &mut self.latency_distribution {
            if latency_us >= bucket.min_us && latency_us < bucket.max_us {
                bucket.count += 1;
                break;
            }
        }
    }

    pub fn average_latency_us(&self) -> f64 {
        if self.total_calls == 0 {
            0.0
        } else {
            (self.total_latency_us as f64) / (self.total_calls as f64)
        }
    }

    pub fn p99_latency_us(&self) -> u32 {
        let target = (self.total_calls * 99) / 100;
        let mut cumulative = 0u64;

        for bucket in &self.latency_distribution {
            cumulative += bucket.count;
            if cumulative >= target {
                return bucket.max_us;
            }
        }

        self.max_latency_us
    }
}

/// Task scheduling statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SchedulingStats {
    pub total_tasks_created: u64,
    pub total_context_switches: u64,
    pub average_queue_length: f64,
    pub max_queue_length: usize,
    pub total_wait_time_us: u64,
}

/// GC pause event
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GcPauseEvent {
    pub timestamp_ms: u64,
    pub pause_duration_us: u32,
    pub freed_bytes: u64,
    pub heap_before: u64,
    pub heap_after: u64,
}

impl GcPauseEvent {
    pub fn new(
        pause_duration_us: u32,
        freed_bytes: u64,
        heap_before: u64,
        heap_after: u64,
    ) -> Self {
        Self {
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            pause_duration_us,
            freed_bytes,
            heap_before,
            heap_after,
        }
    }
}

/// Cache hit rate metrics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CacheHitStats {
    pub cache_name: String,
    pub total_accesses: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl CacheHitStats {
    pub fn new(cache_name: &str) -> Self {
        Self {
            cache_name: cache_name.to_string(),
            total_accesses: 0,
            cache_hits: 0,
            cache_misses: 0,
        }
    }

    pub fn hit_rate(&self) -> f64 {
        if self.total_accesses == 0 {
            0.0
        } else {
            (self.cache_hits as f64) / (self.total_accesses as f64)
        }
    }
}

/// Performance profiler
pub struct Profiler {
    effect_metrics: HashMap<String, EffectMetrics>,
    scheduling_stats: SchedulingStats,
    gc_pauses: Vec<GcPauseEvent>,
    cache_stats: HashMap<String, CacheHitStats>,
    recording: bool,
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            effect_metrics: HashMap::new(),
            scheduling_stats: SchedulingStats {
                total_tasks_created: 0,
                total_context_switches: 0,
                average_queue_length: 0.0,
                max_queue_length: 0,
                total_wait_time_us: 0,
            },
            gc_pauses: Vec::new(),
            cache_stats: HashMap::new(),
            recording: true,
        }
    }

    /// Record effect latency
    pub fn record_effect_latency(&mut self, effect_type: &str, latency_us: u32) {
        if !self.recording {
            return;
        }

        let metrics = self
            .effect_metrics
            .entry(effect_type.to_string())
            .or_insert_with(|| EffectMetrics::new(effect_type));

        metrics.record_latency(latency_us);
    }

    /// Record context switch
    pub fn record_context_switch(&mut self, wait_time_us: u32) {
        if !self.recording {
            return;
        }

        self.scheduling_stats.total_context_switches += 1;
        self.scheduling_stats.total_wait_time_us += wait_time_us as u64;
    }

    /// Record task creation
    pub fn record_task_creation(&mut self) {
        if !self.recording {
            return;
        }

        self.scheduling_stats.total_tasks_created += 1;
    }

    /// Update queue length
    pub fn update_queue_length(&mut self, length: usize) {
        if !self.recording {
            return;
        }

        self.scheduling_stats.max_queue_length = self.scheduling_stats.max_queue_length.max(length);
    }

    /// Record GC pause
    pub fn record_gc_pause(
        &mut self,
        pause_duration_us: u32,
        freed_bytes: u64,
        heap_before: u64,
        heap_after: u64,
    ) {
        if !self.recording {
            return;
        }

        let event = GcPauseEvent::new(pause_duration_us, freed_bytes, heap_before, heap_after);
        self.gc_pauses.push(event);
    }

    /// Record cache access
    pub fn record_cache_access(&mut self, cache_name: &str, hit: bool) {
        if !self.recording {
            return;
        }

        let stats = self
            .cache_stats
            .entry(cache_name.to_string())
            .or_insert_with(|| CacheHitStats::new(cache_name));

        stats.total_accesses += 1;
        if hit {
            stats.cache_hits += 1;
        } else {
            stats.cache_misses += 1;
        }
    }

    /// Get effect metrics
    pub fn get_effect_metrics(&self, effect_type: &str) -> Option<&EffectMetrics> {
        self.effect_metrics.get(effect_type)
    }

    /// Get all effect metrics
    pub fn all_effect_metrics(&self) -> Vec<&EffectMetrics> {
        self.effect_metrics.values().collect()
    }

    /// Get GC statistics
    pub fn gc_statistics(&self) -> (u64, f64, u64) {
        if self.gc_pauses.is_empty() {
            return (0, 0.0, 0);
        }

        let total_pauses = self.gc_pauses.len() as u64;
        let total_pause_time: u64 = self
            .gc_pauses
            .iter()
            .map(|p| p.pause_duration_us as u64)
            .sum();
        let average_pause_us = (total_pause_time as f64) / (total_pauses as f64);
        let total_freed: u64 = self.gc_pauses.iter().map(|p| p.freed_bytes).sum();

        (total_pauses, average_pause_us, total_freed)
    }

    /// Get cache hit rate
    pub fn cache_hit_rate(&self, cache_name: &str) -> Option<f64> {
        self.cache_stats.get(cache_name).map(|s| s.hit_rate())
    }

    /// Export profiling results as JSON
    pub fn export_json(&self) -> Result<String, std::io::Error> {
        #[derive(Serialize)]
        struct ProfileData<'a> {
            effects: Vec<&'a EffectMetrics>,
            scheduling: &'a SchedulingStats,
            gc_pauses: usize,
            caches: Vec<&'a CacheHitStats>,
        }

        let data = ProfileData {
            effects: self.effect_metrics.values().collect(),
            scheduling: &self.scheduling_stats,
            gc_pauses: self.gc_pauses.len(),
            caches: self.cache_stats.values().collect(),
        };

        serde_json::to_string_pretty(&data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
    }

    /// Enable/disable profiling
    pub fn set_recording(&mut self, enabled: bool) {
        self.recording = enabled;
    }

    pub fn is_recording(&self) -> bool {
        self.recording
    }

    pub fn effect_type_count(&self) -> usize {
        self.effect_metrics.len()
    }

    pub fn gc_pause_count(&self) -> usize {
        self.gc_pauses.len()
    }

    pub fn cache_count(&self) -> usize {
        self.cache_stats.len()
    }

    pub fn clear(&mut self) {
        self.effect_metrics.clear();
        self.gc_pauses.clear();
        self.cache_stats.clear();
        self.scheduling_stats.total_tasks_created = 0;
        self.scheduling_stats.total_context_switches = 0;
        self.scheduling_stats.total_wait_time_us = 0;
        self.scheduling_stats.max_queue_length = 0;
    }
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_metrics() {
        let mut metrics = EffectMetrics::new("io");
        metrics.record_latency(50);
        metrics.record_latency(100);
        metrics.record_latency(150);

        assert_eq!(metrics.total_calls, 3);
        assert_eq!(metrics.min_latency_us, 50);
        assert_eq!(metrics.max_latency_us, 150);
        assert!(metrics.average_latency_us() > 90.0);
    }

    #[test]
    fn test_p99_latency() {
        let mut metrics = EffectMetrics::new("io");

        for _ in 0..100 {
            metrics.record_latency(50);
        }
        metrics.record_latency(5000);

        let p99 = metrics.p99_latency_us();
        assert!(p99 >= 50);
    }

    #[test]
    fn test_cache_hit_stats() {
        let mut stats = CacheHitStats::new("proof_cache");
        stats.total_accesses = 100;
        stats.cache_hits = 75;
        stats.cache_misses = 25;

        assert_eq!(stats.hit_rate(), 0.75);
    }

    #[test]
    fn test_record_effect_latency() {
        let mut profiler = Profiler::new();
        profiler.record_effect_latency("io", 100);
        profiler.record_effect_latency("io", 200);

        let metrics = profiler.get_effect_metrics("io").unwrap();
        assert_eq!(metrics.total_calls, 2);
    }

    #[test]
    fn test_record_context_switch() {
        let mut profiler = Profiler::new();
        profiler.record_context_switch(50);
        profiler.record_context_switch(75);

        assert_eq!(profiler.scheduling_stats.total_context_switches, 2);
        assert_eq!(profiler.scheduling_stats.total_wait_time_us, 125);
    }

    #[test]
    fn test_record_gc_pause() {
        let mut profiler = Profiler::new();
        profiler.record_gc_pause(100, 1024, 10000, 8976);

        assert_eq!(profiler.gc_pause_count(), 1);

        let (pauses, avg, freed) = profiler.gc_statistics();
        assert_eq!(pauses, 1);
        assert_eq!(freed, 1024);
    }

    #[test]
    fn test_record_cache_access() {
        let mut profiler = Profiler::new();
        profiler.record_cache_access("proof", true);
        profiler.record_cache_access("proof", true);
        profiler.record_cache_access("proof", false);

        assert_eq!(profiler.cache_hit_rate("proof"), Some(2.0 / 3.0));
    }

    #[test]
    fn test_profiling_disabled() {
        let mut profiler = Profiler::new();
        profiler.set_recording(false);

        profiler.record_effect_latency("io", 100);
        assert_eq!(profiler.effect_type_count(), 0);
    }

    #[test]
    fn test_export_json() {
        let mut profiler = Profiler::new();
        profiler.record_effect_latency("io", 100);
        profiler.record_task_creation();

        let json = profiler.export_json().unwrap();
        assert!(!json.is_empty());
    }

    #[test]
    fn test_clear() {
        let mut profiler = Profiler::new();
        profiler.record_effect_latency("io", 100);
        profiler.record_gc_pause(50, 512, 5000, 4488);

        assert!(profiler.effect_type_count() > 0);
        profiler.clear();
        assert_eq!(profiler.effect_type_count(), 0);
        assert_eq!(profiler.gc_pause_count(), 0);
    }
}
