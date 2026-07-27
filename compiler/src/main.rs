// fluidc: Fluid Rust Compiler
//
// Main entry point for the Fluid Rust compiler.
// Currently a stub; full CLI implementation pending.

fn main() {
    eprintln!("FLUID RUST COMPILER v0.1.0");
    eprintln!("Usage: fluidc <input.rs> [-o <output>]");
    eprintln!();
    eprintln!("Modes:");
    eprintln!("  --native    Compile to LLVM (default)");
    eprintln!("  --managed   Compile to runtime IR");
    eprintln!("  --hybrid    Compile to WASM + native hybrid");
    eprintln!("  --prove     Only generate proof obligations (don't emit code)");
    eprintln!();
    eprintln!("Status: Compiler frontend under construction");
}
