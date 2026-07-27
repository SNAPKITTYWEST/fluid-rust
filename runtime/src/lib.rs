//! FLUID RUST Runtime - Executes verified RMIR bytecode

pub mod effect_handler_impl;
pub mod scheduler_impl;
pub mod gc_impl;
pub mod executor_impl;

pub use effect_handler_impl::{EffectRequest, EffectResponse, EffectHandler};
pub use scheduler_impl::{Task, TaskStatus, Scheduler};
pub use gc_impl::GarbageCollector;
pub use executor_impl::{NativeExecutor, ManagedExecutor};

/// Execute verified RMIR bytecode
pub fn execute(bytecode: &[u8]) -> std::io::Result<i32> {
    let mut executor = ManagedExecutor::new();
    executor.execute(bytecode)
}
