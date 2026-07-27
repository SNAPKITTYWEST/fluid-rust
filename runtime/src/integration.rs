//! Runtime integration: complete execution pipeline

use crate::{
    effect_handler_impl::{
        AsyncHandler, ConcurrencyHandler, ExceptionHandler, FFIHandler, GCHandler, IOHandler,
        RegionHandler, StateHandler,
    },
    EffectHandler, EffectRequest, EffectResponse, GarbageCollector, ManagedExecutor,
    NativeExecutor, Scheduler, Task,
};
use std::io;

/// Complete FLUID RUST runtime environment
pub struct Runtime {
    scheduler: Scheduler,
    gc: GarbageCollector,
    effect_handlers: EffectHandlers,
    managed_executor: ManagedExecutor,
    native_executor: NativeExecutor,
}

struct EffectHandlers {
    io: IOHandler,
    state: StateHandler,
    async_: AsyncHandler,
    region: RegionHandler,
    gc: GCHandler,
    exception: ExceptionHandler,
    ffi: FFIHandler,
    concurrency: ConcurrencyHandler,
}

impl Runtime {
    /// Create a new runtime instance
    pub fn new() -> Self {
        Self {
            scheduler: Scheduler::new(),
            gc: GarbageCollector::new(),
            effect_handlers: EffectHandlers {
                io: IOHandler,
                state: StateHandler::new(),
                async_: AsyncHandler,
                region: RegionHandler,
                gc: GCHandler,
                exception: ExceptionHandler,
                ffi: FFIHandler,
                concurrency: ConcurrencyHandler::new(),
            },
            managed_executor: ManagedExecutor::new(),
            native_executor: NativeExecutor::new(),
        }
    }

    /// Execute RMIR bytecode with effect handling
    pub fn execute(&mut self, bytecode: &[u8]) -> io::Result<i32> {
        // Spawn task
        let _task_id = self.scheduler.spawn_task(bytecode.to_vec());

        // Schedule task
        if let Some(mut task) = self.scheduler.schedule() {
            // Execute task
            let result = self.managed_executor.execute(&task.bytecode)?;

            // Complete task
            self.scheduler.complete_task(task, result);

            Ok(result)
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "No task to schedule"))
        }
    }

    /// Handle an effect request
    pub fn handle_effect(&mut self, request: EffectRequest) -> io::Result<EffectResponse> {
        match request {
            EffectRequest::IO { .. } => self.effect_handlers.io.handle(request.clone()),
            EffectRequest::State { .. } => self.effect_handlers.state.handle(request.clone()),
            EffectRequest::Async { .. } => self.effect_handlers.async_.handle(request.clone()),
            EffectRequest::Region { .. } => self.effect_handlers.region.handle(request.clone()),
            EffectRequest::GC { .. } => self.effect_handlers.gc.handle(request.clone()),
            EffectRequest::Exception { .. } => {
                self.effect_handlers.exception.handle(request.clone())
            }
            EffectRequest::FFI { .. } => self.effect_handlers.ffi.handle(request.clone()),
            EffectRequest::Concurrency { .. } => {
                self.effect_handlers.concurrency.handle(request.clone())
            }
        }
    }

    /// Run garbage collection
    pub fn collect_garbage(&mut self) -> usize {
        self.gc.collect()
    }

    /// Get heap size
    pub fn heap_size(&self) -> usize {
        self.gc.heap_size()
    }

    /// Get scheduler queue length
    pub fn queue_length(&self) -> usize {
        self.scheduler.queue_length()
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let runtime = Runtime::new();
        assert_eq!(runtime.queue_length(), 0);
        assert_eq!(runtime.heap_size(), 0);
    }

    #[test]
    fn test_effect_handling() {
        let mut runtime = Runtime::new();

        // Handle state effect
        let request = EffectRequest::State {
            op: "set".to_string(),
            cell_id: 0,
            value: 42,
        };

        let response = runtime.handle_effect(request);
        assert!(response.is_ok());

        if let Ok(EffectResponse::State { new_value, .. }) = response {
            assert_eq!(new_value, 42);
        }
    }

    #[test]
    fn test_garbage_collection() {
        let mut runtime = Runtime::new();

        // Allocate memory
        let _ptr1 = runtime.gc.allocate(1024);
        let _ptr2 = runtime.gc.allocate(512);

        assert_eq!(runtime.heap_size(), 1536);

        // Collect
        let freed = runtime.collect_garbage();
        assert_eq!(freed, 1536);
        assert_eq!(runtime.heap_size(), 0);
    }

    #[test]
    fn test_execute_bytecode() {
        let mut runtime = Runtime::new();
        let bytecode = vec![0, 1, 2, 3];
        let result = runtime.execute(&bytecode);
        assert!(result.is_ok());
    }
}
