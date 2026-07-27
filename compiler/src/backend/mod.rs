//! Backend: Lowered form → executable code
//!
//! Three execution modes:
//! 1. **Native** (LLVM): Compile to machine code, no GC, explicit regions
//! 2. **Managed** (Runtime IR): Compile to effect-dispatched runtime IR, GC enabled
//! 3. **Hybrid** (WASM): Compile hot paths to LLVM, cold paths to WASM

pub mod native;
pub mod wasm;

// TODO: Implement LLVM codegen for native mode
// TODO: Implement runtime IR codegen for managed mode
// TODO: Implement WASM codegen and sandboxing
// TODO: Implement hybrid mode (detect hot/cold, emit both)
// TODO: Implement effect handler integration (call sites)
