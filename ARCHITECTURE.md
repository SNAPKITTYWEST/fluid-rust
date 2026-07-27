# FLUID RUST: Complete Architecture

A unified language runtime that proves memory safety and effect correctness by integrating **Liquid Rust ownership verification** with **Haskell-style algebraic effect handlers**.

---

## Layer 1: Liquid Rust (Systems Substrate)

**Core Principle:** Rust's ownership model is a **physical law** — you cannot violate it, even in principle.

### Type System Extensions

```
τ ::= T<ρ, φ>  (type parameterized by region ρ and refinement φ)

φ ::= true
    | v : int{x | x > 0}          (dependent type)
    | unique(p)                     (linear ownership)
    | borrowed<'a>(p, mode)         (borrow invariant)
    | effect_captured(eff)          (effect witness)
```

### Ownership as Physical Law

- **Linear ownership (`unique`):** Exactly one path to the value. Consumed by move, assignment, or explicit drop.
- **Borrowed access (`borrowed<'a>`):** Shared (`&T`) or mutable (`&mut T`). Lifetime `'a` enforces no use-after-free.
- **Region state machine:** Every region has a proof of lifecycle: `Unentered → Active(S) → Closed`.

### Proof-Carrying MIR (RMIR)

**RMIR is Rust's MIR with:**
- Ownership facts (move, borrow, consume)
- Region transitions (enter, allocate, deallocate, exit)
- Effect signatures (which effects this function may trigger)
- Proof obligations (lemmas the prover must verify)

**Example RMIR fragment:**
```
fn process_buffer(buf: &mut [u8]) -> io::Result<()> {
    // RMIR: borrow_mut(buf, 'a)
    region_enter(stack)
    // RMIR: region_status(stack) = Active({})
    
    ptr = allocate(stack, 1024)
    // RMIR: region_status(stack) = Active({ptr})
    //       capability(stack, ptr, write) held
    
    effect io::write(ptr, buf)
    // RMIR: proof_obligation(effect_io_write_precondition)
    //       effect_witnessed(io, state_change(input=buf → output=written))
    
    deallocate(stack, ptr)
    // RMIR: region_status(stack) = Active({})
    
    region_exit(stack)
    // RMIR: region_status(stack) = Closed
    // RMIR: all deallocations witnessed
}
```

### State Machine Semantics

Every value has an **execution state:**

```
┌─────────┐
│ Created │ (ownership established)
└────┬────┘
     │ borrow()
     ▼
┌──────────────┐
│   Borrowed   │ (shared or mutable access)
└────┬─────────┘
     │ release()
     ▼
┌─────────┐
│ Owned   │ (exclusive again)
└────┬────┘
     │ move() or drop()
     ▼
┌─────────────┐
│ Consumed    │ (invalid, cannot use)
└─────────────┘
```

Every state transition produces **proof obligations** the discrete prover must discharge.

---

## Layer 2: Haskell-Style Managed Runtime (Computational Model)

**Core Principle:** Effects are **composable handlers**, not baked-in primitives.

### Algebraic Effects

Eight core effects encapsulate all runtime behavior:

```
Effect ::= IO(request)          (file I/O, network, system calls)
         | State(get/put)       (mutable reference cells)
         | Async(spawn/yield)   (task scheduling, continuations)
         | Region(enter/exit)   (region lifecycle management)
         | GC(trace/collect)    (garbage collection operations)
         | Exception(throw/try) (error handling)
         | FFI(call/import)     (foreign function boundary)
         | Concurrency(lock)    (atomicity, mutex, atomic ops)
```

### Effect Handler Pattern

```rust
pub trait EffectHandler {
    fn handle(&mut self, effect: Effect) -> Continuation;
}

pub struct Continuation {
    // The computation resumes here after the handler services the request.
    // May contain new state, side effects, or an error.
    next_state: RuntimeState,
    control_flow: ControlFlow,
}
```

**Handlers live in the runtime**, not the user code. The compiler inserts `effect_emit` calls; the runtime's dispatcher routes them to handlers.

### Lazy Evaluation + Call-by-Need

- Values are not eagerly computed.
- Functions return **thunks** (suspended computations).
- Thunks are forced when demanded; results are cached.
- This enables effect composition without forcing order.

### Continuation-Based Control Flow

```
fn async_task() {
    effect Async(spawn(|| {
        effect IO(read(file))  // Suspend here; handler decides when/how
        .then(|data| {
            process(data)      // Resume with result
        })
    }))
}
```

The runtime's **continuation stack** manages all control flow:
- `yield` → save frame, run another task
- `resume` → pop frame, restore state
- `throw` → unwind stack until handler found

### Managed vs. Native Execution

- **Managed mode:** All effects routed through handlers; GC collects orphaned regions.
- **Native mode:** Direct LLVM → machine; regions must be explicitly managed; no GC overhead.
- **Hybrid mode:** Hot path in native, cold path in managed; handlers bridge the gap.

---

## Layer 3: Discrete Proof Engine (Verification Model)

**Core Principle:** Ownership, regions, and effects are **logical facts**; we prove them with declarative solvers.

### ASP (Answer Set Programming) for Ownership & Regions

**ASP Rule Engine** (clingo):

1. **Extract RMIR facts** into ASP:
   ```prolog
   % Ownership facts
   owns(value_42, thread_0, timestamp_100).
   capability(value_42, write, timestamp_100).
   
   % Region facts
   region_status(stack_1, timestamp_100, active).
   allocated_in(ptr_5, stack_1, timestamp_100).
   
   % Effect facts
   effect_emitted(io_write, timestamp_100).
   effect_precondition_witnessed(io_write, timestamp_100).
   ```

2. **ASP Rules** verify invariants:
   ```prolog
   % Invariant: No two threads own the same value
   :- owns(V, T1, TS), owns(V, T2, TS), T1 != T2.
   
   % Invariant: Active region must have all allocations deallocated before close
   :- region_status(R, TS_close, closed),
      allocated_in(P, R, TS_alloc),
      TS_alloc < TS_close,
      not deallocated(P, TS_close).
   
   % Invariant: No use-after-consume
   :- owns(V, T, TS_use), consumed(V, TS_consume), TS_consume < TS_use.
   ```

3. **Solver answers:** If clingo finds an answer set satisfying all rules, ownership is verified.

### SMT (Satisfiability Modulo Theories) for Bounds & Numeric Constraints

**Z3 SMT Solver** for numeric invariants:

```smt2
; Region constraints
(declare-const size_stack_1 Int)
(declare-const used_stack_1 Int)

(assert (> size_stack_1 0))
(assert (>= used_stack_1 0))
(assert (<= used_stack_1 size_stack_1))

; Effect preconditions (e.g., file offset bounds)
(declare-const file_offset Int)
(assert (>= file_offset 0))
(assert (< file_offset (read_file_size file_handle)))

; Check satisfiability
(check-sat)
```

**Proof obligations from RMIR** become SMT assertions:
- Region size bounds
- Effect preconditions (e.g., valid file handles)
- Numeric invariants (e.g., pointer alignment)

### Unified Proof Certificate

The prover outputs a **proof certificate** (JSON):

```json
{
  "program_hash": "sha256:abc...",
  "rmir_instructions": [
    {"id": 0, "op": "region_enter", "region": "stack_0"},
    {"id": 1, "op": "allocate", "region": "stack_0", "ptr": "ptr_5"},
    ...
  ],
  "asp_facts": [
    "region_status(stack_0, 0, unentered).",
    "region_status(stack_0, 1, active).",
    ...
  ],
  "asp_rules": "[...ASP rule set...]",
  "asp_answer_set": "[...minimal answer set...]",
  "smt_assertions": "(assert (> size_stack_0 0)) ...",
  "smt_model": "{size_stack_0: 4096, used_stack_0: 1024, ...}",
  "verifier_signature": "ed25519:xyz...",
  "timestamp": "2026-07-26T12:34:56Z"
}
```

### Verifier (Tiny Trusted Component)

The verifier is ~200 lines of code that:
1. Computes RMIR hash
2. Re-runs ASP solver with extracted facts + rules
3. Re-runs SMT solver with extracted assertions
4. Verifies signatures
5. Outputs **proof_valid** or **proof_invalid**

No need to trust the compiler; we trust only the solver output.

---

## Layer 4: Execution Modes

### Native Execution (LLVM)

**Lowering:** RMIR → Rust MIR → LLVM IR → machine code

- No runtime overhead; regions are stack frames.
- Proof certificate baked into binary metadata.
- Direct syscalls; no effect handler dispatch.

```c
// Generated native code (pseudocode)
int process_buffer(uint8_t* buf, size_t len) {
    // Region enter: allocate stack frame
    uint8_t stack[4096];
    struct Region stack_region = {
        .base = stack,
        .size = 4096,
        .used = 0,
        .status = ACTIVE,
    };
    
    // Allocate
    uint8_t* ptr = stack + stack_region.used;
    stack_region.used += 1024;
    assert(stack_region.used <= 4096);
    
    // I/O (direct syscall)
    ssize_t written = write(STDOUT_FILENO, ptr, 1024);
    if (written < 0) goto error;
    
    // Deallocate (implicit: used -= 1024)
    stack_region.used -= 1024;
    
    // Region exit
    assert(stack_region.used == 0);
    return 0;
    
error:
    return -1;
}
```

### Managed Execution (Runtime IR)

**Lowering:** RMIR → Runtime IR → Effect dispatch → Handler

- All effects routed through handler system.
- GC manages memory (regions become GC heap objects).
- Lazy evaluation, continuations for async.

```rust
// Generated managed code (pseudocode)
fn process_buffer(buf: &[u8]) -> impl Future<Output = io::Result<()>> {
    async move {
        let stack = effect_emit(Region(enter)).await;
        let ptr = effect_emit(Region(allocate(stack, 1024))).await;
        
        let written = effect_emit(IO(write(ptr, buf))).await?;
        
        effect_emit(Region(deallocate(stack, ptr))).await;
        effect_emit(Region(exit(stack))).await;
        
        Ok(())
    }
}
```

The **effect_emit** call suspends the task; the runtime's handler services it and resumes.

### Hybrid Execution (WASM)

**Lowering:** RMIR → WASM bytecode → sandbox

- Hot paths: compile to native LLVM
- Cold paths or untrusted code: WASM interpreter
- Bridge: WASM imports/exports for effect dispatch

```wat
(module
  (import "env" "effect_emit" (func $emit (param i32) (result i32)))
  
  (func $process_buffer (param $buf i32) (param $len i32) (result i32)
    (local $ptr i32)
    (local $stack i32)
    
    ;; region_enter
    (local.set $stack (call $emit (i32.const 1)))
    
    ;; allocate
    (local.set $ptr (call $emit (i32.const 2)))
    
    ;; ... I/O via $emit ...
    
    (i32.const 0)  ;; success
  )
)
```

---

## Integration Points (Explicit ABI Boundaries)

### 1. Compiler → Prover: RMIR Bytecode Format

**File:** `rmir.pb` (Protocol Buffers or custom bytecode)

```
RMIR_BYTECODE ::= [VERSION (u32)] [CHECKSUM (u256)] [INSTRUCTIONS*]

INSTRUCTION ::= [OPCODE (u8)] [ARG_COUNT (u8)] [ARGS*]

OPCODE:
  0x00 = region_enter(region_id: u32)
  0x01 = region_exit(region_id: u32)
  0x02 = allocate(region_id: u32, size: u32, ptr_id: u32)
  0x03 = deallocate(region_id: u32, ptr_id: u32)
  0x04 = borrow(value_id: u32, borrow_id: u32, mode: u8, lifetime: u32)
  0x05 = consume(value_id: u32)
  0x06 = effect_emit(effect_kind: u8, payload: [u8])
  0x07 = assert(predicate_id: u32)
```

### 2. Prover → Runtime: Proof Certificate Format

**File:** `proof_cert.json`

```json
{
  "metadata": {
    "program_hash": "sha256:...",
    "timestamp": "2026-07-26T12:34:56Z",
    "verifier_version": "0.1.0"
  },
  "facts": {
    "ownership": [...],
    "regions": [...],
    "effects": [...]
  },
  "proof_summary": {
    "asp_solvable": true,
    "smt_satisfiable": true,
    "all_obligations_discharged": true
  },
  "signature": "ed25519:..."
}
```

### 3. Runtime → Native/Managed: Effect Handler ABI

**Effect Request:**
```c
struct EffectRequest {
    uint8_t effect_kind;      // IO, State, Async, Region, GC, Exception, FFI, Concurrency
    uint32_t request_id;      // Unique ID for this request
    void* payload;            // Effect-specific data
    size_t payload_size;
};
```

**Handler Response:**
```c
struct EffectResponse {
    uint32_t request_id;      // Matches request
    int32_t status;           // 0 = ok, <0 = error
    void* result;             // Effect-specific result
    size_t result_size;
};
```

### 4. Lowering Rules: RMIR → Native & Managed

| RMIR Instruction | Native Lowering | Managed Lowering |
|------------------|-----------------|------------------|
| `region_enter(R)` | Stack frame allocation | `effect_emit(Region(enter))` |
| `allocate(R, sz, P)` | Bump pointer `R.used += sz` | `effect_emit(Region(allocate))` + await |
| `effect_emit(E)` | Direct syscall (if I/O) | Handler dispatch + continuation |
| `consume(V)` | Drop value (no-op) | Effect to GC if needed |
| `region_exit(R)` | Assert `R.used == 0` | `effect_emit(Region(exit))` + collect |

---

## Data Flow Diagram

```
Source Code (Rust + Liquid Types)
    │
    ▼
┌─────────────────────────────────────┐
│  Compiler: Frontend (Elaboration)   │  Layer 1: Liquid Rust
│  - Parse Rust syntax + refinements  │
│  - Extract ownership facts          │
│  - Track linear capabilities        │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│  Compiler: RMIR Generation          │  Layer 1: Proof-Carrying MIR
│  - Build execution state machine    │
│  - Embed proof obligations          │
│  - Generate RMIR bytecode           │
└─────────────────────────────────────┘
    │
    ├──────────────────────────────────────┐
    │                                      │
    ▼                                      ▼
┌──────────────────────┐      ┌──────────────────────────┐
│  Prover: ASP Facts   │      │  Prover: SMT Constraints │ Layer 3: Discrete Proof
│  (clingo solver)     │      │  (Z3 SMT solver)         │
└──────────────────────┘      └──────────────────────────┘
    │                                      │
    └──────────────────┬───────────────────┘
                       │
                       ▼
                ┌──────────────────────┐
                │  Proof Certificate   │   Layer 3: Unified Proof
                │  (JSON + signature)  │
                └──────────────────────┘
                       │
        ┌──────────────┴──────────────┐
        │                             │
        ▼                             ▼
    ┌──────────────┐         ┌─────────────────┐
    │  Native      │         │  Managed        │  Layer 4: Execution
    │  Lowering    │         │  Lowering       │
    │  (LLVM)      │         │  (Effect IR)    │
    └──────────────┘         └─────────────────┘
        │                             │
        ▼                             ▼
    ┌──────────────┐         ┌─────────────────┐
    │  Machine     │         │  Runtime        │  Layer 4: Execution Engines
    │  Code        │         │  Handlers       │
    └──────────────┘         └─────────────────┘
```

---

## Key Invariants Enforced

1. **No use-after-free:** ASP rules prevent access to consumed or deallocated values.
2. **No aliasing violations:** Linear ownership rules prevent mutable aliasing.
3. **Region lifecycle:** Unentered → Active → Closed, with no leaks.
4. **Effect ordering:** Effects emitted in a consistent order; handlers execute correctly.
5. **Proof validity:** Every compiled binary carries a proof certificate; verifier rejects invalid programs.

---

## Next Steps

1. **Implement compiler frontend** (elaboration.rs, ownership.rs)
2. **Implement RMIR IR** (ir.rs, state.rs, effect.rs, capability.rs)
3. **Implement ASP extractor** (extractor.rs, rules.rs, solver.rs)
4. **Implement SMT bridge** (z3_bridge.rs, constraints.rs)
5. **Implement lowering** (normal_mir.rs, llvm.rs, wasm.rs, native.rs)
6. **Implement runtime** (effect_handler.rs, scheduler.rs, gc.rs)
7. **Write verifier** (~200 lines, trusted component)
8. **Add documentation** (spec/*.md files)
9. **Run examples** (examples/*.rs)

---

**Design by:** Ahmad's Integral Architecture  
**Verified by:** SNAPKITTYWEST Autonomous Systems  
**Status:** Foundation Layer Complete (2026-07-26)
