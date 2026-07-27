# Contributing to FLUID RUST

Welcome! This guide explains how to contribute to the FLUID RUST verified systems language.

---

## Development Setup

### Prerequisites

- **Rust 1.70+** (for compiler crate)
- **Clingo 5.x+** (ASP solver)
- **Z3 4.12+** (SMT solver)
- **Lean 4.0+** (for formal verification)
- **Python 3.9+** (for tooling scripts)

### Build from Source

```bash
# Clone repository
git clone https://github.com/SNAPKITTYWEST/fluid-rust.git
cd fluid-rust

# Install dependencies
cargo fetch

# Build all crates
cargo build --workspace --release

# Run tests
cargo test --workspace --lib

# Run integration tests
cargo test --workspace --test '*'
```

### Development Environment

```bash
# Set up development mode
cargo build --workspace

# Run formatter
cargo fmt --all

# Run clippy linter
cargo clippy --workspace --all-targets -- -D warnings

# Check documentation
cargo doc --workspace --no-deps

# Watch for changes (requires cargo-watch)
cargo watch -x test
```

---

## Project Structure & Module Responsibilities

### Layer 1: Compiler (`compiler/`)

**Responsibility:** Rust source code → RMIR bytecode + proof obligations

**Key modules:**
- `frontend/` — Rust HIR parsing, elaboration, refinement inference
- `rmir/` — RMIR IR definition, state machine, SSA construction
- `backend/` — LLVM codegen, WASM codegen, native codegen

**New contributor task:** Implement elaboration for one Rust construct (e.g., match expressions, loops, closures).

### Layer 2: Prover (`prover/`)

**Responsibility:** RMIR bytecode → proof certificate (ASP + SMT)

**Key modules:**
- `asp/` — RMIR → ASP facts, rule generation, clingo integration
- `smt/` — Constraint extraction, Z3 integration, satisfiability checking
- `certificate.rs` — Proof certificate serialization, cryptographic sealing
- `verifier.rs` — Tiny trusted certificate checker (~150 lines)

**New contributor task:** Add ASP rules for a new proof obligation type (e.g., deadlock prevention for concurrency).

### Layer 3: Runtime (`runtime/`)

**Responsibility:** Execute RMIR with algebraic effect handlers

**Key modules:**
- `effect_handler.rs` — Effect dispatcher, handler trait, all 8 effects
- `scheduler.rs` — Task scheduling, continuation management
- `gc.rs` — Garbage collection (managed mode)
- `native.rs` — Native execution engine (LLVM mode)
- `managed.rs` — Managed execution engine (runtime mode)

**New contributor task:** Implement one effect handler (e.g., FFI, Concurrency, State).

### Specifications (`spec/`)

**Responsibility:** Formal definitions (not code)

**Key files:**
- `RMIR_SPEC.md` — RMIR instruction semantics
- `EFFECT_HANDLER_SPEC.md` — Effect request/response ABI
- `ASP_RULES.pl` — Logic programming rules

**New contributor task:** Write specification for a new effect or RMIR instruction.

---

## Code Style Guide

### Rust Code

```rust
// Module structure
mod ownership;      // Separate concerns
mod state;
mod effect;

use crate::rmir::{Instruction, ExecutionState};

/// Elaborate a Rust pattern into RMIR instructions.
/// 
/// Returns a vector of RMIR instructions with attached proof obligations.
/// 
/// # Errors
/// 
/// Returns `ElaborationError` if the pattern cannot be elaborated.
pub fn elaborate_pattern(pattern: &Pattern) -> Result<Vec<Instruction>> {
    // Implementation
}

// Constants use SCREAMING_SNAKE_CASE
const MAX_REGION_DEPTH: usize = 1024;

// Functions use snake_case
fn extract_proof_obligations(rmir: &[Instruction]) -> Vec<ProofGoal> {
    // Implementation
}

// Struct fields use snake_case, visibility explicit
pub struct ExecutionState {
    pub value_ssa: HashMap<Id, Value>,
    capability_ssa: HashMap<Id, Capability>,  // private by default
}

// Error types end in Error
#[derive(Debug)]
pub enum ElaborationError {
    UnknownPattern(String),
    TypeMismatch { expected: Type, actual: Type },
}

// Comments are sparse; use clear naming instead
fn is_linear_type(ty: &Type) -> bool {
    matches!(ty, Type::Linear { .. })
}
// ✓ Clear name; no need for "// check if type is linear"
```

### Python Tooling

```python
# Similar structure, but idiomatic Python
from dataclasses import dataclass
from typing import List, Optional

@dataclass
class ProofObligation:
    """A proof obligation extracted from RMIR bytecode."""
    kind: str
    goal: str
    context: Optional[dict] = None

def extract_obligations(rmir_bytecode: bytes) -> List[ProofObligation]:
    """Extract proof obligations from RMIR bytecode."""
    # Implementation
    pass
```

### Markdown Documentation

```markdown
## Section Title

Introductory paragraph explaining the concept.

### Subsection

Code examples should be fenced with language tag:

    ```rust
    fn example() {}
    ```

Tables for structured data:

    | Column 1 | Column 2 |
    |----------|----------|
    | Value    | Value    |

ASCII diagrams for architecture:

    ```
    ┌──────┐
    │ Node │
    └──────┘
    ```
```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_type_elaboration() {
        let ty = Type::Linear { ... };
        let instr = elaborate_type(&ty).unwrap();
        assert!(matches!(instr, Instruction::Assign { .. }));
    }

    #[test]
    fn test_region_state_machine() {
        let mut state = ExecutionState::new();
        let r1 = state.enter_region().unwrap();
        assert_eq!(state.region_status(r1), RegionStatus::Active);
        state.exit_region(r1).unwrap();
        assert_eq!(state.region_status(r1), RegionStatus::Closed);
    }
}
```

**Coverage:** Aim for >80% line coverage. Critical paths should be >95%.

### Integration Tests

```rust
// tests/integration_test.rs
#[test]
fn test_full_pipeline() {
    let source = r#"
        fn safe_read(buf: &mut Region, n: usize {n <= 1024}) {
            region_enter(buf);
            let ptr = allocate(buf, n);
            region_exit(buf);
        }
    "#;

    let program = compile(source).expect("compilation failed");
    let cert = program.proof_certificate();
    assert!(cert.verify().is_ok());
}
```

### Specification Tests

ASP and SMT tests in `spec/`:

```prolog
% spec/test_ownership.pl
% Test: linear value used exactly once

:- linear(x), not uses(x).           % ERROR: x not used
:- linear(x), uses(x), uses(x).      % ERROR: x used twice
```

Run with clingo:

```bash
clingo spec/ownership.pl spec/test_ownership.pl 0
```

---

## Contribution Workflow

### 1. Create an Issue

Describe what you want to work on:

```
Title: Implement FFI effect handler

Description:
- Add new FFI effect to effect_handler.rs
- Implement handler that calls C functions
- Add 5+ tests
- Update EFFECT_HANDLER_SPEC.md

Difficulty: Medium
Related: Layer 3 (Runtime)
```

### 2. Fork & Branch

```bash
git checkout -b layer3/ffi-effect-handler

# Commit frequently with descriptive messages
git commit -m "Add FFI effect request/response types"
git commit -m "Implement FFI effect handler dispatch"
git commit -m "Add tests for C interop"
```

### 3. Code Review Checklist

Before submitting a PR, verify:

- [ ] **Functionality**: Code works as intended
- [ ] **Tests**: All new code has tests (>80% coverage)
- [ ] **Documentation**: Public APIs have doc comments
- [ ] **Formatting**: `cargo fmt` passes
- [ ] **Linting**: `cargo clippy` passes with no warnings
- [ ] **Compilation**: `cargo build --workspace --release` succeeds
- [ ] **Tests pass**: `cargo test --workspace --all-targets`

### 4. Submit Pull Request

```
Title: Layer 3: Implement FFI effect handler

Description:

## What does this PR do?

Adds FFI (Foreign Function Interface) effect handler to runtime:
- New `Ffi` effect type with request/response ABI
- Handler dispatch for C function calls
- Signature validation and error handling

## Implementation notes

- FFI requests validated against function signature
- C return values converted to RMIR values
- Error handling via Exception effect

## Tests

- 8 tests: basic calls, signatures, errors, variance
- Integration test: calls C library function
- Coverage: 94%

## Breaks anything?

No breaking changes. FFI is additive (8th effect handler).

## Size & complexity

- +420 lines (handler implementation)
- +340 lines (tests)
- +60 lines (spec updates)
```

### 5. Address Review Comments

```bash
# Make changes based on feedback
git add src/runtime/effect_handler.rs tests/ffi_tests.rs
git commit -m "Address review: add signature validation"

# Force push (only if you own the branch)
git push --force-with-lease origin layer3/ffi-effect-handler
```

### 6. Merge

Once approved:

```bash
# Merge via GitHub UI (creates merge commit)
# Or locally:
git checkout main
git pull origin main
git merge --no-ff layer3/ffi-effect-handler
git push origin main
```

---

## Common Contribution Areas

### ✅ Easy (Start Here)

- Add doc comments to public APIs (`src/**/*.rs`)
- Write unit tests for existing functions
- Improve error messages
- Add examples (`examples/`)
- Update documentation (`.md` files)

**Effort:** 1-2 hours  
**Difficulty:** Low  
**Skills needed:** Rust basics, testing basics

### 🟨 Medium

- Implement one RMIR instruction elaboration (e.g., match expressions)
- Add one effect handler (e.g., State, Async)
- Write ASP rules for a new proof obligation type
- Implement SMT constraint extraction

**Effort:** 4-8 hours  
**Difficulty:** Medium  
**Skills needed:** Type systems, logic programming (for ASP), solver integration (for SMT)

### 🔴 Hard

- Implement entire compiler phase (elaboration → RMIR)
- Add execution mode (e.g., WASM, Hybrid)
- Rewrite proof verifier for efficiency
- Add optimization passes to backend

**Effort:** 20-40 hours  
**Difficulty:** Hard  
**Skills needed:** Compiler design, formal methods, systems programming

---

## Design Decision Documentation

When adding significant features, document your design:

1. **Create a file:** `docs/decisions/0001-feature-name.md`
2. **Use ADR format:** Status, Context, Decision, Consequences
3. **Example:**

```markdown
# 0042: ASP Solver Integration Strategy

## Status: Accepted

## Context

We need to integrate clingo (ASP solver) for discrete proof verification.
Options:
1. Shell out to clingo (simple, but slow)
2. Use clingo Rust bindings (faster, but adds dependency)
3. Implement custom ASP solver (complex, but full control)

## Decision

Use option 2 (clingo Rust bindings) because:
- Clingo is mature and battle-tested
- Bindings are well-maintained
- Performance is adequate for compile-time use

## Consequences

- Added dependency on clingo (binary, must be installed)
- Build time increases by ~30%
- Proof verification time decreases by ~50%
```

---

## Performance Considerations

### Profiling

```bash
# Profile compiler
cargo build --release -p fluid-rust-compiler
perf record -g target/release/fluidrust compile large_program.rs
perf report

# Profile runtime
cargo build --release -p fluid-rust-runtime
cargo bench --bench effect_dispatch
```

### Optimization

- Proof certificate generation is CPU-bound → parallelize with rayon
- RMIR elaboration is memory-bound → use arena allocators
- Effect dispatch is latency-critical → minimize indirection (use vtable or match)

---

## Security Considerations

### Code Safety

- Always use `unsafe { }` blocks sparingly and document why
- Audit all pointer operations (FFI in particular)
- Validate untrusted inputs at system boundaries
- Test with miri (undefined behavior detector)

### Proof Soundness

- Proof certificates must be cryptographically sealed
- Trusted verifier must be <200 lines (audit-friendly)
- All external solver calls must be logged + validated
- Reproducible builds (deterministic proofs)

---

## Becoming a Maintainer

**Requirements:**
1. 5+ merged PRs across multiple layers
2. 2+ code reviews approved by existing maintainers
3. Deep understanding of 2+ layers (compiler, prover, runtime)
4. Commitment to respond to issues within 48 hours

**Responsibilities:**
- Review PRs (timely, constructive)
- Triage issues (label, assign)
- Maintain code quality (run checks, enforce style)
- Mentor new contributors

---

## Questions?

- **Architecture:** See `ARCHITECTURE.md`
- **Design decisions:** Check `docs/decisions/`
- **Quick reference:** See `QUICK_REFERENCE.md`
- **Specifications:** See `spec/`

**Open an issue or ask in discussions!**

---

**Last updated:** 2026-07-26  
**Version:** 1.0 (Foundation Layer)
