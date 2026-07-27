// fluid-prover: Discrete Proof Engine
//
// Main entry point for the proof engine.
// Currently a stub; full CLI implementation pending.

fn main() {
    eprintln!("FLUID RUST PROVER v0.1.0");
    eprintln!("Usage: fluid-prover <program.rmir> [--asp] [--smt] [--verify]");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --asp       Use ASP solver (clingo) for ownership/region facts");
    eprintln!("  --smt       Use SMT solver (Z3) for numeric constraints");
    eprintln!("  --verify    Verify an existing proof certificate");
    eprintln!();
    eprintln!("Status: Prover core under construction");
}
