/// Native (LLVM) and Managed (Interpreter) execution engines

use std::io;

pub struct NativeExecutor;

impl NativeExecutor {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self, _bytecode: &[u8]) -> io::Result<i32> {
        // LLVM JIT compilation would happen here
        // For now, return success
        Ok(0)
    }
}

pub struct ManagedExecutor {
    ip: u32,
    registers: Vec<u64>,
    exit_code: i32,
}

impl ManagedExecutor {
    pub fn new() -> Self {
        Self {
            ip: 0,
            registers: vec![0; 256],
            exit_code: 0,
        }
    }

    pub fn execute(&mut self, _bytecode: &[u8]) -> io::Result<i32> {
        // Bytecode interpretation happens here
        // For now, return the exit code
        Ok(self.exit_code)
    }

    pub fn set_exit_code(&mut self, code: i32) {
        self.exit_code = code;
    }
}

impl Default for NativeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ManagedExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_native_executor() {
        let executor = NativeExecutor::new();
        let result = executor.execute(&vec![0, 1, 2]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_managed_executor() {
        let mut executor = ManagedExecutor::new();
        executor.set_exit_code(42);
        let result = executor.execute(&vec![0, 1, 2]);
        assert_eq!(result.unwrap(), 42);
    }
}
