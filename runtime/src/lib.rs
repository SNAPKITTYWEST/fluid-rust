//! Fluid Rust Runtime: Managed Execution Engine
//!
//! The runtime provides:
//! - Algebraic effect handler dispatch
//! - Task scheduler and continuation management
//! - Garbage collector (for managed mode)
//! - Effect ABI bridge (for native/managed interop)
//!
//! The runtime is NOT the execution engine itself; it's the service layer
//! that handles effects (IO, State, Async, Region, GC, Exception, FFI, Concurrency).

pub mod effect_handler;
pub mod effect;
pub mod scheduler;
pub mod gc;
pub mod native;
pub mod managed;
pub mod abi;

// TODO: Implement effect dispatcher
// TODO: Implement task scheduler
// TODO: Implement GC integration
// TODO: Implement ABI bridge for native/managed interop
