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
pub mod executor_impl;
pub mod gc_impl;
pub mod integration;
pub mod scheduler_impl;

// Phase P5 modules
pub mod config;
pub mod effect_optimizer;
pub mod error_handler;
pub mod jit_specializer;
pub mod profiler;
pub mod proof_cache;

pub use effect_handler_impl::{EffectHandler, EffectRequest, EffectResponse};
pub use executor_impl::{ManagedExecutor, NativeExecutor};
pub use gc_impl::GarbageCollector;
pub use integration::Runtime;
pub use scheduler_impl::{Scheduler, Task, TaskStatus};

// Phase P5 exports
pub use config::ProductionConfig;
pub use effect_optimizer::EffectOptimizer;
pub use error_handler::ErrorHandler;
pub use jit_specializer::JitSpecializer;
pub use profiler::Profiler;
pub use proof_cache::ProofCache;

/// Execute verified RMIR bytecode with full runtime
pub fn execute(bytecode: &[u8]) -> std::io::Result<i32> {
    let mut runtime = Runtime::new();
    runtime.execute(bytecode)
}
