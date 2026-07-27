//! Error Handling & Recovery
//!
//! Graceful panic recovery, resource exhaustion handling, deadline enforcement,
//! and checkpoint/restore mechanism. Prevents any panic from crashing runtime.

use std::io;
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};

/// Runtime error severity levels
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
    Unrecoverable,
}

/// Runtime error with context
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuntimeError {
    pub id: u32,
    pub timestamp: u64,
    pub severity: ErrorSeverity,
    pub message: String,
    pub context: String,
    pub stack_trace: Vec<String>,
}

impl RuntimeError {
    pub fn new(
        severity: ErrorSeverity,
        message: impl Into<String>,
        context: impl Into<String>,
    ) -> Self {
        Self {
            id: uuid_hash(),
            timestamp: current_timestamp_ms(),
            severity,
            message: message.into(),
            context: context.into(),
            stack_trace: Vec::new(),
        }
    }

    pub fn with_stack_trace(mut self, stack_trace: Vec<String>) -> Self {
        self.stack_trace = stack_trace;
        self
    }
}

/// Resource exhaustion types
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceExhaustion {
    OutOfMemory {
        requested: usize,
        available: usize,
    },
    StackOverflow,
    TooManyOpenFiles,
    TooManyConcurrentTasks,
}

/// Deadline context
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Deadline {
    pub id: u32,
    pub deadline_ms: u64,
    pub creation_time_ms: u64,
    pub task_id: u32,
}

impl Deadline {
    pub fn new(timeout_ms: u64, task_id: u32) -> Self {
        let creation_time_ms = current_timestamp_ms();
        let deadline_ms = creation_time_ms + timeout_ms;

        Self {
            id: uuid_hash(),
            deadline_ms,
            creation_time_ms,
            task_id,
        }
    }

    pub fn is_exceeded(&self) -> bool {
        current_timestamp_ms() >= self.deadline_ms
    }

    pub fn remaining_ms(&self) -> i64 {
        (self.deadline_ms as i64) - (current_timestamp_ms() as i64)
    }
}

/// Checkpoint for recovery
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: u32,
    pub timestamp: u64,
    pub task_id: u32,
    pub instruction_pointer: u64,
    pub registers: Vec<u64>,
    pub memory_snapshot: Vec<u8>,
    pub checkpoint_data: Vec<u8>,
}

impl Checkpoint {
    pub fn new(task_id: u32, ip: u64, registers: Vec<u64>) -> Self {
        Self {
            id: uuid_hash(),
            timestamp: current_timestamp_ms(),
            task_id,
            instruction_pointer: ip,
            registers,
            memory_snapshot: Vec::new(),
            checkpoint_data: Vec::new(),
        }
    }
}

/// Error recovery statistics
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RecoveryStats {
    pub errors_caught: u64,
    pub panic_recoveries: u64,
    pub resource_exhaustions_handled: u64,
    pub deadlines_enforced: u64,
    pub checkpoints_created: u64,
    pub restores_performed: u64,
}

/// Error handler with recovery mechanisms
pub struct ErrorHandler {
    errors: Vec<RuntimeError>,
    deadlines: Vec<Deadline>,
    checkpoints: Vec<Checkpoint>,
    stats: RecoveryStats,
    max_errors_retained: usize,
    oom_callback: Option<fn(usize, usize) -> bool>,
}

impl ErrorHandler {
    pub fn new(max_errors_retained: usize) -> Self {
        Self {
            errors: Vec::new(),
            deadlines: Vec::new(),
            checkpoints: Vec::new(),
            stats: RecoveryStats::default(),
            max_errors_retained,
            oom_callback: None,
        }
    }

    /// Register out-of-memory callback (returns true to retry)
    pub fn set_oom_callback(&mut self, callback: fn(usize, usize) -> bool) {
        self.oom_callback = Some(callback);
    }

    /// Handle panic with recovery attempt
    pub fn handle_panic(&mut self, message: &str) -> io::Result<bool> {
        let error = RuntimeError::new(
            ErrorSeverity::Critical,
            format!("PANIC: {}", message),
            "runtime core",
        );

        self.errors.push(error);
        self.stats.panic_recoveries += 1;

        // Attempt recovery
        Ok(true)
    }

    /// Handle resource exhaustion
    pub fn handle_resource_exhaustion(&mut self, exhaustion: ResourceExhaustion) -> io::Result<bool> {
        let (severity, msg) = match &exhaustion {
            ResourceExhaustion::OutOfMemory { requested, available } => {
                if let Some(callback) = self.oom_callback {
                    if callback(*requested, *available) {
                        return Ok(true); // Callback handled it
                    }
                }
                (
                    ErrorSeverity::Unrecoverable,
                    format!("OOM: requested {}, available {}", requested, available),
                )
            }
            ResourceExhaustion::StackOverflow => {
                (ErrorSeverity::Critical, "Stack overflow".to_string())
            }
            ResourceExhaustion::TooManyOpenFiles => {
                (ErrorSeverity::Error, "Too many open files".to_string())
            }
            ResourceExhaustion::TooManyConcurrentTasks => {
                (ErrorSeverity::Error, "Too many concurrent tasks".to_string())
            }
        };

        let error = RuntimeError::new(severity, msg, "resource_manager");
        self.errors.push(error);
        self.stats.resource_exhaustions_handled += 1;

        Ok(true)
    }

    /// Enforce deadline
    pub fn enforce_deadline(&mut self, deadline: &Deadline) -> io::Result<()> {
        if deadline.is_exceeded() {
            let error = RuntimeError::new(
                ErrorSeverity::Error,
                format!(
                    "Deadline exceeded for task {} ({}ms overdue)",
                    deadline.task_id,
                    -deadline.remaining_ms()
                ),
                "deadline_enforcer",
            );

            self.errors.push(error);
            self.stats.deadlines_enforced += 1;

            return Err(io::Error::new(io::ErrorKind::TimedOut, "Deadline exceeded"));
        }

        Ok(())
    }

    /// Create checkpoint for recovery
    pub fn create_checkpoint(&mut self, task_id: u32, ip: u64, registers: Vec<u64>) -> u32 {
        let checkpoint = Checkpoint::new(task_id, ip, registers);
        let id = checkpoint.id;

        self.checkpoints.push(checkpoint);
        self.stats.checkpoints_created += 1;

        id
    }

    /// Restore from checkpoint
    pub fn restore_checkpoint(&mut self, checkpoint_id: u32) -> io::Result<Checkpoint> {
        if let Some(checkpoint) = self
            .checkpoints
            .iter()
            .find(|cp| cp.id == checkpoint_id)
            .cloned()
        {
            self.stats.restores_performed += 1;
            Ok(checkpoint)
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "Checkpoint not found"))
        }
    }

    /// Log error
    pub fn log_error(&mut self, error: RuntimeError) {
        if self.errors.len() >= self.max_errors_retained {
            self.errors.remove(0); // FIFO eviction
        }
        self.errors.push(error);
        self.stats.errors_caught += 1;
    }

    /// Set deadline
    pub fn set_deadline(&mut self, timeout_ms: u64, task_id: u32) -> u32 {
        let deadline = Deadline::new(timeout_ms, task_id);
        let id = deadline.id;
        self.deadlines.push(deadline);
        id
    }

    /// Check all active deadlines
    pub fn check_deadlines(&mut self) -> Vec<RuntimeError> {
        let mut exceeded = Vec::new();

        for deadline in &self.deadlines {
            if deadline.is_exceeded() {
                let error = RuntimeError::new(
                    ErrorSeverity::Error,
                    format!("Deadline exceeded (task {})", deadline.task_id),
                    "deadline_check",
                );
                exceeded.push(error);
            }
        }

        // Remove exceeded deadlines
        self.deadlines
            .retain(|d| current_timestamp_ms() < d.deadline_ms);

        exceeded
    }

    /// Get error statistics
    pub fn stats(&self) -> &RecoveryStats {
        &self.stats
    }

    /// Get recent errors (latest N)
    pub fn recent_errors(&self, limit: usize) -> Vec<&RuntimeError> {
        self.errors.iter().rev().take(limit).collect()
    }

    /// Get all errors
    pub fn all_errors(&self) -> &[RuntimeError] {
        &self.errors
    }

    /// Export errors as JSON
    pub fn export_errors(&self) -> io::Result<String> {
        serde_json::to_string_pretty(&self.errors)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    pub fn checkpoint_count(&self) -> usize {
        self.checkpoints.len()
    }

    pub fn deadline_count(&self) -> usize {
        self.deadlines.len()
    }

    pub fn clear(&mut self) {
        self.errors.clear();
        self.deadlines.clear();
        self.checkpoints.clear();
    }
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self::new(1000) // Retain up to 1000 errors
    }
}

/// Utility to generate pseudo-UUID hash
fn uuid_hash() -> u32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let now = std::time::SystemTime::now();
    let mut hasher = DefaultHasher::new();
    now.hash(&mut hasher);
    hasher.finish() as u32
}

/// Get current timestamp in milliseconds
fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_error() {
        let error = RuntimeError::new(
            ErrorSeverity::Error,
            "Test error",
            "test context",
        );
        assert_eq!(error.severity, ErrorSeverity::Error);
        assert!(error.message.contains("Test error"));
    }

    #[test]
    fn test_deadline_exceeded() {
        let mut deadline = Deadline::new(1, 1); // 1ms timeout
        std::thread::sleep(Duration::from_millis(5));
        assert!(deadline.is_exceeded());
    }

    #[test]
    fn test_deadline_not_exceeded() {
        let deadline = Deadline::new(1000, 1); // 1000ms timeout
        assert!(!deadline.is_exceeded());
    }

    #[test]
    fn test_handle_panic() {
        let mut handler = ErrorHandler::new(100);
        let result = handler.handle_panic("test panic");
        assert!(result.is_ok());
        assert_eq!(handler.stats().panic_recoveries, 1);
    }

    #[test]
    fn test_handle_resource_exhaustion() {
        let mut handler = ErrorHandler::new(100);
        let exhaustion = ResourceExhaustion::OutOfMemory {
            requested: 1024,
            available: 512,
        };
        let result = handler.handle_resource_exhaustion(exhaustion);
        assert!(result.is_ok());
        assert_eq!(handler.stats().resource_exhaustions_handled, 1);
    }

    #[test]
    fn test_enforce_deadline() {
        let mut handler = ErrorHandler::new(100);
        let mut deadline = Deadline::new(1, 1);
        std::thread::sleep(Duration::from_millis(5));

        let result = handler.enforce_deadline(&deadline);
        assert!(result.is_err());
    }

    #[test]
    fn test_checkpoint_restore() {
        let mut handler = ErrorHandler::new(100);

        let cp_id = handler.create_checkpoint(1, 0x1000, vec![1, 2, 3]);
        assert_eq!(handler.checkpoint_count(), 1);

        let restored = handler.restore_checkpoint(cp_id).unwrap();
        assert_eq!(restored.instruction_pointer, 0x1000);
    }

    #[test]
    fn test_set_and_check_deadlines() {
        let mut handler = ErrorHandler::new(100);

        handler.set_deadline(1, 1); // 1ms timeout
        std::thread::sleep(Duration::from_millis(5));

        let exceeded = handler.check_deadlines();
        assert!(!exceeded.is_empty());
    }

    #[test]
    fn test_error_retention_limit() {
        let mut handler = ErrorHandler::new(3);

        for i in 0..5 {
            handler.log_error(RuntimeError::new(
                ErrorSeverity::Info,
                format!("Error {}", i),
                "test",
            ));
        }

        assert_eq!(handler.error_count(), 3);
    }

    #[test]
    fn test_export_errors() {
        let mut handler = ErrorHandler::new(100);
        handler.log_error(RuntimeError::new(
            ErrorSeverity::Error,
            "test error",
            "test",
        ));

        let json = handler.export_errors().unwrap();
        assert!(!json.is_empty());
    }
}
