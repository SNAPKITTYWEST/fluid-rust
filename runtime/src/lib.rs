//! FLUID RUST Runtime - Executes verified RMIR bytecode
//!
//! Phase P5: Production Hardening
//! - Proof caching (WORM ledger)
//! - Effect batching & optimization
//! - JIT specialization
//! - Error handling & recovery
//! - Performance profiling
//! - Production configuration

pub mod effect_handler_impl;
pub mod scheduler_impl;
pub mod gc_impl;
pub mod executor_impl;

// Phase P5 modules
pub mod proof_cache;
pub mod effect_optimizer;
pub mod jit_specializer;
pub mod error_handler;
pub mod profiler;
pub mod config;

pub use effect_handler_impl::{EffectRequest, EffectResponse, EffectHandler};
pub use scheduler_impl::{Task, TaskStatus, Scheduler};
pub use gc_impl::GarbageCollector;
pub use executor_impl::{NativeExecutor, ManagedExecutor};

// Phase P5 exports
pub use proof_cache::ProofCache;
pub use effect_optimizer::EffectOptimizer;
pub use jit_specializer::JitSpecializer;
pub use error_handler::ErrorHandler;
pub use profiler::Profiler;
pub use config::ProductionConfig;

/// Execute verified RMIR bytecode
pub fn execute(bytecode: &[u8]) -> std::io::Result<i32> {
    let mut executor = ManagedExecutor::new();
    executor.execute(bytecode)
}
