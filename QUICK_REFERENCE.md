# FLUID RUST: Quick Reference Guide

One-page summary of key concepts, ABIs, and debugging strategies.

---

## Concepts at a Glance

### Liquid Rust Types

```rust
// Value types (traditional Rust)
x: i32                          // Immutable value
mut y: i32                      // Mutable value (affine)

// Ownership types (linear)
buf: linear {μ, τ}            // Linear: used exactly once
ptr: affine {μ, τ}            // Affine: used at most once

// Refined types (with constraints)
n: usize {n > 0 && n <= 1024}  // n is in range [1, 1024]
idx: usize {idx < vec.len()}   // idx is valid array index

// Region types (with lifetime)
stack: Region<'a>              // Scoped memory region
buf: &'a mut Region<'a>        // Mutable region reference
```

### RMIR Instructions

```
Assign(x, val)              // x := val (Value SSA)
Move(x, y)                  // x := y; y invalidated (linear)
Borrow(x, y, µ, τ)         // x := &_µ^τ y (borrow with mode, lifetime)
Consume(x)                  // x consumed; no further use
Transition(eff, goal)       // Execute effect; attach proof goal
Allocate(r, T, sz, n)       // Allocate sz * sizeof(T) in region r
Deallocate(r, x)            // Deallocate x from region r
Assert(pred, proof)         // Assert predicate; include proof
```

### Proof Obligations (Goals)

```
OwnershipInvariant(x)       // x used exactly once (linear) or at most once (affine)
RegionSafety(r)             // Region r is active (not closed)
LinearityConstraint(v)      // Value v has no aliases
EffectOrdering(e1, e2)      // Effect e1 must precede e2
BoundsCheck(idx, len)       // idx < len (compile-time verified)
EffectPrecondition(e)       // Effect e's preconditions are satisfied
```

---

## Compilation Pipeline

```
Rust Source Code
    ↓
Frontend (HIR → Elaboration)
    • Parse Rust syntax
    • Infer types + refinements
    • Extract ownership facts
    ↓
RMIR Generation (Elaborate → Proof Obligations)
    • Four-part SSA (Value SSA, Capability SSA, Region FSM, Effects)
    • Attach proof goals to each instruction
    • Output bytecode + goals (JSON)
    ↓
ASP Extraction (RMIR → Logic Program)
    • Convert RMIR instructions to ASP facts
    • Generate ASP rules (ownership, region lifecycle, etc.)
    • Feed to clingo solver
    ↓
SMT Bridge (Numeric Constraints → Z3)
    • Extract bounds constraints from refined types
    • Query SMT solver for satisfiability
    • Collect satisfying assignment
    ↓
Proof Certificate Generation
    • Merge ASP model + SMT certificate
    • Cryptographic seal (Blake3 hash)
    • Output JSON certificate
    ↓
Execution Engine Selection
    ├─→ Native (LLVM) for systems code
    ├─→ Managed (Runtime) for applications
    └─→ Hybrid (WASM) for verification
    ↓
Compiled Binary (with embedded or separable certificate)
```

---

## Effect Handler ABI

### Request Format (from program to runtime)

```json
{
  "effect": "IO",
  "operation": "read",
  "fd": 3,
  "buffer": { "ptr": "0x7fff0000", "capacity": 4096 },
  "count": 1024,
  "proof_obligation": "effect_precondition(read(fd=3))"
}
```

### Response Format (from runtime to program)

```json
{
  "status": "success",
  "bytes_read": 512,
  "continuation": { "id": "cont_42", "stack_depth": 3 },
  "proof_certificate": { "ast_model": [...] }
}
```

### All Eight Effects

| Effect | Request | Response | Proof Obligation |
|--------|---------|----------|------------------|
| **IO** | `{op, fd, buffer, count}` | `{status, bytes}` | `fd >= 0 && buffer.capacity >= count` |
| **State** | `{op, cell_id, value}` | `{status, old_value}` | `cell_id exists && typed(value)` |
| **Async** | `{spawn, task, args}` | `{task_id}` | `task callable with args` |
| **Region** | `{enter\|exit, region_id}` | `{status}` | `region FSM valid transition` |
| **GC** | `{collect}` | `{freed_bytes}` | `GC not currently running` |
| **Exception** | `{throw, error}` | N/A (unwind) | `error type matches expected` |
| **FFI** | `{call, func_ptr, args}` | `{retval}` | `func_ptr valid && args match signature` |
| **Concurrency** | `{lock, mutex_id}` | `{acquired\|timeout}` | `mutex_id exists && not deadlock` |

---

## Region Finite State Machine

```
       ┌─────────────────┐
       │   Unentered     │
       │ (before scope)  │
       └────────┬────────┘
                │
         region_enter()
                │
       ┌────────▼────────┐
       │     Active      │
       │ (in scope)      │
       └────────┬────────┘
                │
         allocate()    deallocate()
         write()       ...
                │
       ┌────────▼────────┐
       │     Closed      │
       │ (after scope)   │
       └─────────────────┘

Invariants:
- Allocate only in Active state
- Write only to pointers allocated in current Active region
- No access after Closed (caught at compile time)
- Transition to Closed deallocates all outstanding pointers
```

---

## ASP Rule Examples

```prolog
% Ownership: each linear value used exactly once
:- linear(X), #count { uses(X) } != 1.

% Region lifecycle: can only deallocate if active
:- deallocate(R), not active(R).

% Effect ordering: IO before Exception
:- effect(io(E1)), effect(exception(E2)), E2 < E1.

% Capability linearity: each borrowed reference has one owner
:- borrow(B, R, _), #count { owner(B) } != 1.

% Safe writes: only write to regions that are active
safe_write(R) :- write(R), active(R).
```

---

## Debugging Guide

### Issue: "Proof obligation failed: LinearityConstraint(x)"

**Meaning:** Variable `x` (linear type) is used more than once or not at all.

**Fix:**
```rust
// ❌ WRONG: x used twice (linear)
let x: linear {...} = ...;
f(x);
g(x);  // ERROR: x already moved

// ✅ CORRECT: use only once
let x: linear {...} = ...;
f(move(x));
```

### Issue: "Region safety failed: Region r is closed"

**Meaning:** You're accessing a region after its scope ended.

**Fix:**
```rust
// ❌ WRONG: ptr used after region_exit
region_enter(stack);
let ptr = allocate(stack, 1024);
region_exit(stack);
write(ptr, data);  // ERROR: stack is closed

// ✅ CORRECT: all access before region_exit
region_enter(stack);
let ptr = allocate(stack, 1024);
write(ptr, data);
region_exit(stack);
```

### Issue: "Effect precondition failed: fd >= 0"

**Meaning:** File descriptor is negative (invalid).

**Fix:**
```rust
// ❌ WRONG: fd not checked
let fd = open(path)?;
read(fd, buf, 1024);  // ERROR: fd might be -1 on error

// ✅ CORRECT: fd checked before use
let fd = open(path)?;
assert(fd >= 0, "open succeeded");
read(fd, buf, 1024);
```

### Issue: "Bounds check failed: idx < vec.len()"

**Meaning:** Array index out of bounds.

**Fix:**
```rust
// ❌ WRONG: no bounds check
let val = vec[idx];  // ERROR: idx might be >= vec.len()

// ✅ CORRECT: bounds verified
assert(idx < vec.len(), "index in bounds");
let val = vec[idx];

// Or use refined type
fn safe_index(vec: &Vec<T>, idx: usize {idx < vec.len()}) -> &T {
    vec.get_unchecked(idx)  // Now provably safe
}
```

---

## Proof Certificate Structure

```json
{
  "program_hash": "blake3(...)",
  "timestamp": "2026-07-26T14:33:22Z",
  
  "asp_proof": {
    "facts": [
      "region(r1, unentered)",
      "allocate(r1, u8)",
      "linear_capability(buf, r1)"
    ],
    "rules": [
      "active(R) :- allocate(R, _), not deallocate(R)"
    ],
    "answer_set": [
      "active(r1)",
      "safe_write(r1)"
    ],
    "status": "SATISFIABLE"
  },
  
  "smt_proof": {
    "constraints": [
      "n > 0",
      "n <= 1024",
      "idx < n"
    ],
    "satisfying_assignment": {
      "n": 512,
      "idx": 100
    },
    "status": "SAT"
  },
  
  "verified_by": "fluidrust-verifier v1.0",
  "signature": "ed25519(...)"
}
```

---

## Performance Characteristics

| Aspect | Native Mode | Managed Mode |
|--------|-------------|--------------|
| **Memory overhead** | Explicit regions (0% extra) | GC metadata (~5-10%) |
| **Latency** | Deterministic | GC pause (ms scale) |
| **Throughput** | CPU-bound (peak FLOPS) | I/O-bound (scheduler) |
| **Proof verification** | Zero-cost (erased at runtime) | Zero-cost (erased at runtime) |
| **Effect dispatch** | Direct syscall | Runtime handler |
| **Best for** | Systems, HPC, real-time | Applications, servers |

---

## Common Patterns

### Safe String Buffer

```rust
fn safe_read_line(stdin: IO, stack: Region<'_>) 
    -> Result<&'_ [u8], Error> {
    region_enter(stack);
    let buf = allocate::<u8>(stack, 1024);
    let n = read(stdin, buf, 1024)?;  // Proof: buf.capacity >= 1024
    region_exit(stack);
    Ok(&buf[0..n])  // Proof: n <= 1024 (from refinement)
}
```

### Safe Index with Bounds

```rust
fn get_nth<T>(vec: &Vec<T>, n: usize {n < vec.len()}) -> &T {
    // Refinement proven at call site
    // No runtime bounds check needed
    unsafe { vec.get_unchecked(n) }
}

// Call site:
let item = get_nth(&my_vec, 42);  // Proof obligation: 42 < my_vec.len()
```

### Composite Effect Usage

```rust
fn process_file_with_state(path: &str, state: &mut i32)
    -> Result<i32, Error> {
    
    let fd = open(path)?;           // IO effect
    
    region_enter(stack);
    let buf = allocate::<u8>(stack, 4096);  // Region effect
    
    on_error! {
        while let Ok(bytes) = read(fd, buf, 4096) {  // IO effect
            *state += bytes;                          // State effect
            if bytes == 0 { break; }
        }
    } catch {
        deallocate(stack, buf);                       // Cleanup
    };
    
    region_exit(stack);
    close(fd)?;                     // IO effect
    Ok(*state)
}

// All three effects proven to compose correctly ✓
```

---

## Tools & Commands

```bash
# Compile program with proof certificate
fluidrust build src/main.rs -o program

# Extract proof certificate (separable)
fluidrust extract-cert program > program.cert.json

# Verify proof offline
fluidrust verify program program.cert.json

# Visualize RMIR bytecode
fluidrust rmir-view program

# Simulate effects (for testing)
fluidrust effect-simulator src/test.rs

# Check ownership invariants only
fluidrust check-ownership src/main.rs

# Check region safety only
fluidrust check-regions src/main.rs

# Performance profiling (native mode)
fluidrust profile --native src/main.rs

# Benchmark proof certificate verification time
fluidrust bench-verify program.cert.json
```

---

## Key Files to Read

1. **ARCHITECTURE.md** — Full design (all 4 layers)
2. **spec/RMIR_SPEC.md** — RMIR instruction reference
3. **spec/EFFECT_HANDLER_SPEC.md** — Effect ABI details
4. **spec/ASP_RULES.pl** — Logic programming rules
5. **examples/simple_region.rs** — End-to-end example

---

**Last updated:** 2026-07-26  
**Version:** 1.0 (Foundation Layer)
