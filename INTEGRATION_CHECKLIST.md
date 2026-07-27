# FLUID RUST Integration Points Checklist

This document tracks the four critical integration points where components connect.

---

## 1. Compiler → Prover: RMIR Bytecode Format

### What is exchanged?
- **Input:** RMIR AST (in-memory Rust struct from `compiler/src/rmir/ir.rs`)
- **Output:** RMIR bytecode file (on disk) + proof obligations

### Current Design

**Bytecode Format:** Binary (custom serialization)

```
RMIR_BYTECODE ::= [VERSION (u32)] [CHECKSUM (u256)] [INSTRUCTIONS*]

INSTRUCTION ::= [OPCODE (u8)] [ARG_COUNT (u8)] [ARGS*]

OPCODE mapping (compiler/src/rmir/ir.rs):
  0x00 = RegionEnter
  0x01 = RegionExit
  0x02 = Allocate
  0x03 = Deallocate
  0x04 = Borrow
  0x05 = Consume
  0x06 = Move
  0x07 = Assign
  0x08 = EffectEmit
  0x09 = Assert
```

### Integration Checklist

- [ ] **Serialization (compiler):** Implement `RmirFunction::to_bytecode()` in `compiler/src/rmir/ir.rs`
  - [ ] Version encoding
  - [ ] Instruction opcode mapping
  - [ ] Argument packing
  - [ ] Checksum calculation (SHA256)

- [ ] **Deserialization (prover):** Implement `RmirBytecode::from_file()` in `prover/src/asp/extractor.rs`
  - [ ] Version checking
  - [ ] Opcode decoding
  - [ ] Argument unpacking
  - [ ] Checksum verification

- [ ] **File format:**
  - [ ] File extension: `.rmir` (or `.rmirb` for bytecode)
  - [ ] Location: compiler outputs to `target/rmir/`
  - [ ] Naming: `{function_name}.rmir`

- [ ] **Testing:**
  - [ ] Unit tests for serialization round-trip
  - [ ] Integration test: compile simple_region.rs → .rmir → verify bytecode

---

## 2. Prover → Runtime: Proof Certificate Format

### What is exchanged?
- **Input:** Proof obligations (from RMIR), ASP/SMT results
- **Output:** JSON proof certificate + Ed25519 signature

### Current Design

**Certificate Format:** JSON (see `prover/src/certificate.rs`)

```json
{
  "metadata": {
    "program_hash": "sha256:...",
    "program_name": "process_buffer",
    "timestamp": "2026-07-26T12:34:56Z",
    "verifier_version": "0.1.0",
    "compiler_version": "0.1.0"
  },
  "facts": {
    "rmir_instructions": [...],
    "ownership_facts": [...],
    "region_facts": [...]
  },
  "asp_result": {
    "satisfiable": true,
    "answer_set": "owns(42, 0, 100). region_status(0, 0, active). ..."
  },
  "smt_result": {
    "satisfiable": true,
    "model": { "size_stack_0": "4096", "used_stack_0": "1024" }
  },
  "signature": "ed25519:..."
}
```

### Integration Checklist

- [ ] **Certificate generation (prover):**
  - [ ] Implement `Prover::generate_certificate()` in `prover/src/certificate.rs`
  - [ ] Bundle RMIR facts
  - [ ] Include ASP answer set
  - [ ] Include SMT model
  - [ ] Sign with Ed25519 (see `prover/src/verifier.rs`)

- [ ] **Certificate validation (verifier):**
  - [ ] Implement tiny verifier (~200 lines) in `prover/src/verifier.rs`
  - [ ] Parse JSON certificate
  - [ ] Verify Ed25519 signature
  - [ ] Re-run ASP solver on extracted facts
  - [ ] Re-run SMT solver on extracted constraints
  - [ ] Check both solvers report SATISFIABLE

- [ ] **File format:**
  - [ ] File extension: `.proof` or `.proof_cert`
  - [ ] Location: prover outputs to `target/proofs/`
  - [ ] Naming: `{program_hash}.proof`

- [ ] **Testing:**
  - [ ] Unit tests for certificate serialization
  - [ ] Unit tests for verifier on valid certificates
  - [ ] Unit tests for verifier rejection of invalid certificates
  - [ ] Integration test: generate cert → verify cert

---

## 3. Runtime → Native/Managed: Effect Handler ABI

### What is exchanged?
- **Input:** Effect request (from executing code)
- **Output:** Effect response (from handler)

### Current Design

**ABI Format:** C struct (binary compatible)

```c
struct EffectRequest {
    uint32_t request_id;
    uint8_t effect_kind;      // 0=IO, 1=State, 2=Async, 3=Region, ...
    uint32_t payload_offset;
    uint32_t payload_size;
};

struct EffectResponse {
    uint32_t request_id;
    int32_t status;           // 0=success, <0=error
    uint32_t result_offset;
    uint32_t result_size;
};
```

### Integration Checklist

- [ ] **Effect dispatch (runtime):**
  - [ ] Implement `EffectDispatcher::dispatch()` in `runtime/src/effect_handler.rs`
  - [ ] Route by effect_kind to appropriate handler
  - [ ] Parse payload based on opcode
  - [ ] Call handler
  - [ ] Return response

- [ ] **Native code generation (compiler backend):**
  - [ ] Emit `call effect_emit(request_ptr)` instruction for each EffectEmit RMIR instr
  - [ ] Pack EffectRequest on stack
  - [ ] Unpack EffectResponse
  - [ ] Check status and branch on error
  - [ ] Continue with result

- [ ] **Managed code generation (compiler backend):**
  - [ ] Generate `effect_emit(effect)` call for each EffectEmit RMIR instr
  - [ ] Suspend task and yield to scheduler
  - [ ] Resume with handler result

- [ ] **WASM bridge:**
  - [ ] Expose `effect_emit` as WASM import
  - [ ] Marshal EffectRequest to linear memory
  - [ ] Call host function
  - [ ] Unmarshal EffectResponse

- [ ] **Testing:**
  - [ ] Unit tests for each handler (IO, State, Async, Region, GC, Exception, FFI, Concurrency)
  - [ ] Integration test: native code → effect request → handler → response → execution continues
  - [ ] Integration test: managed code → effect request → handler → response → task resumes

---

## 4. Lowering Rules: RMIR → Native & Managed

### What is exchanged?
- **Input:** RMIR instruction + execution state
- **Output:** Native LLVM IR / Managed runtime IR + proof obligations forwarded

### Current Design

**Lowering Rules:** Transformation rules (documented in `spec/RMIR_SPEC.md`)

| RMIR Instruction | Native Lowering | Managed Lowering |
|------------------|-----------------|------------------|
| `region_enter(R)` | `alloca(stack_frame_size)` | `effect_emit(Region(enter))` |
| `allocate(R, sz, P)` | `bump_pointer += sz; assert(bump_pointer < frame_size)` | `effect_emit(Region(allocate))` + await |
| `effect_emit(E)` | Direct syscall (if I/O) or handler call | Handler dispatch + continuation |
| `consume(V)` | `// no-op` (value already dead) | Effect to GC if heap-allocated |
| `region_exit(R)` | `assert(bump_pointer == frame_end)` | `effect_emit(Region(exit))` + collect |

### Integration Checklist

- [ ] **Native lowering (LLVM):**
  - [ ] Implement `lower_rmir_to_llvm()` in `compiler/src/backend/native.rs`
  - [ ] Lower region_enter → stack frame allocation
  - [ ] Lower region_exit → validation asserts
  - [ ] Lower allocate → bump pointer increment
  - [ ] Lower deallocate → bump pointer decrement
  - [ ] Lower effect_emit → direct syscall or handler call
  - [ ] Preserve proof obligations in binary metadata

- [ ] **Managed lowering (Runtime IR):**
  - [ ] Implement `lower_rmir_to_managed()` in `compiler/src/backend/managed.rs`
  - [ ] Lower all instructions to effect dispatch calls
  - [ ] Generate continuation points for async effects
  - [ ] Preserve proof obligations in runtime metadata

- [ ] **Proof obligation forwarding:**
  - [ ] All proof obligations from RMIR must appear in final executable
  - [ ] Native: embedded in binary metadata section
  - [ ] Managed: stored in runtime metadata
  - [ ] Verifier can re-check proofs at runtime if needed

- [ ] **Testing:**
  - [ ] Unit tests: RMIR instruction → lowered form (native)
  - [ ] Unit tests: RMIR instruction → lowered form (managed)
  - [ ] Integration test: round-trip RMIR → lower → verify proof obligations preserved
  - [ ] End-to-end test: simple_region.rs → RMIR → lower (both modes) → execute

---

## 5. Verification & Testing

### Cross-Integration Testing

- [ ] **Full pipeline (simple_region.rs):**
  1. Compile to RMIR bytecode
  2. Prove (ASP + SMT)
  3. Generate certificate
  4. Verify certificate
  5. Lower to native
  6. Execute native
  7. Verify proof obligations discharged

- [ ] **Full pipeline (simple_region.rs, managed mode):**
  1. Compile to RMIR bytecode
  2. Prove (ASP + SMT)
  3. Generate certificate
  4. Verify certificate
  5. Lower to managed IR
  6. Execute in managed runtime with handlers
  7. Verify proof obligations discharged

- [ ] **Proof certificate validation:**
  - [ ] Generate cert for valid program
  - [ ] Generate cert for program with use-after-free (should fail)
  - [ ] Verify cert is deterministic (same program → same cert)
  - [ ] Verify cert signature
  - [ ] Verify offline verifier passes valid certs and rejects invalid ones

---

## 6. Build & Deployment

- [ ] **Cargo workspace:**
  - [ ] All three crates build: `cargo build --workspace`
  - [ ] All tests pass: `cargo test --workspace`
  - [ ] Binaries available: `./target/debug/fluidc`, `./target/debug/fluid-prover`

- [ ] **CI/CD:**
  - [ ] GitHub Actions workflows for build + test
  - [ ] Proof generation benchmarks (time and memory)
  - [ ] Verifier benchmarks

- [ ] **Documentation:**
  - [ ] Each module documented
  - [ ] ABI specifications finalized
  - [ ] Examples working end-to-end
  - [ ] Integration guide for new backends

---

## Timeline Estimate

| Phase | Deliverable | Effort |
|-------|-------------|--------|
| P1 | RMIR serialization (1) | 40 hours |
| P2 | ASP/SMT integration + certificate (2) | 60 hours |
| P3 | Effect handler ABI (3) | 40 hours |
| P4 | Lowering rules (4) | 80 hours |
| P5 | Testing + documentation (5, 6) | 60 hours |
| **Total** | Full integration | **280 hours** |

---

## Sign-Off

- [ ] All four integration points implemented
- [ ] All tests passing
- [ ] simple_region.rs runs end-to-end (native mode)
- [ ] simple_region.rs runs end-to-end (managed mode)
- [ ] Proof certificates verified offline
- [ ] Ready for production pilot
