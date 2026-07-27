//! Native Backend: RMIR → LLVM → Machine Code
//!
//! Generates native machine code with:
//! - Zero garbage collection overhead
//! - Explicit stack frame-based regions
//! - Direct system calls (no handler dispatch)
//! - Proof certificate embedded in binary metadata

// TODO: Implement RMIR to LLVM IR translation
// TODO: Implement region lifecycle as stack frame management
// TODO: Implement direct syscall lowering for IO effects
// TODO: Implement binary metadata section for proof certificate
// TODO: Integrate with LLVM API (via llvm-sys crate)
