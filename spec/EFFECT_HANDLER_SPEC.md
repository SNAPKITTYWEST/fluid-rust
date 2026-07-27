# Effect Handler Specification: ABI and Runtime Integration

## Overview

The effect handler ABI defines the binary interface between:
- **Native code** ↔ **Runtime effect handlers**
- **Managed code** ↔ **Runtime effect handlers**
- **WASM sandbox** ↔ **Native bridge**

## Effect Types

Eight core effects encapsulate all runtime behavior:

| Effect | Purpose | Handler |
|--------|---------|---------|
| `IO` | File I/O, network, system calls | `IOHandler` |
| `State` | Mutable reference cells, memory operations | `StateHandler` |
| `Async` | Task spawning, yield, resume | `AsyncHandler` |
| `Region` | Region lifecycle management | `RegionHandler` |
| `GC` | Garbage collection operations | `GCHandler` |
| `Exception` | Error handling, try/throw | `ExceptionHandler` |
| `FFI` | Foreign function calls | `FFIHandler` |
| `Concurrency` | Atomicity, locks, atomic operations | `ConcurrencyHandler` |

## Effect Request Format

**C ABI (binary compatible across languages):**

```c
struct EffectRequest {
    uint32_t request_id;      // Unique ID for this request (for correlation)
    uint8_t effect_kind;      // Effect type (0=IO, 1=State, etc.)
    uint32_t payload_offset;  // Offset to payload buffer
    uint32_t payload_size;    // Size of payload in bytes
};
```

**Size:** 12 bytes

## Effect Response Format

**C ABI (binary compatible):**

```c
struct EffectResponse {
    uint32_t request_id;      // Echoes request_id
    int32_t status;           // 0 = success, <0 = error
    uint32_t result_offset;   // Offset to result buffer
    uint32_t result_size;     // Size of result in bytes
};
```

**Size:** 12 bytes

## Effect-Specific Payloads

### IO Effect

**Request payload (opcode, args):**

```
[OPCODE (u8)] [ARGS*]

OPCODE:
  0 = read(fd, buffer_offset, length)
  1 = write(fd, buffer_offset, length)
  2 = open(path_offset, flags)
  3 = close(fd)

Example: write(1, 0x1000, 1024)
  [0x01] [fd=1] [buf_offset=0x1000] [length=1024]
```

**Response payload:**
```
[RESULT (i64)] = bytes written or error code
```

### State Effect

**Request payload:**

```
[OPCODE (u8)] [ARGS*]

OPCODE:
  0 = get(cell_id)
  1 = put(cell_id, value_offset, value_size)
```

**Response payload:**
```
[VALUE*] = current value from cell
```

### Async Effect

**Request payload:**

```
[OPCODE (u8)] [ARGS*]

OPCODE:
  0 = spawn(thunk_offset, arg_offset, arg_size)
  1 = yield()
  2 = resume(task_id)
```

**Response payload:**
```
[TASK_ID (u32)] = for spawn
[STATUS (u32)] = for yield/resume
```

### Region Effect

**Request payload:**

```
[OPCODE (u8)] [ARGS*]

OPCODE:
  0 = enter(region_id)
  1 = exit(region_id)
  2 = allocate(region_id, size)
  3 = deallocate(region_id, ptr_id)
```

**Response payload:**
```
[RESULT (u32)] = ptr_id for allocate, status for others
```

### GC Effect

**Request payload:**

```
[OPCODE (u8)] [ARGS*]

OPCODE:
  0 = trace(root_offset, root_count)
  1 = collect()
```

**Response payload:**
```
[COLLECTED (u32)] = number of objects collected
[HEAP_SIZE (u32)] = heap size after collection
```

### Exception Effect

**Request payload:**

```
[OPCODE (u8)] [ARGS*]

OPCODE:
  0 = throw(exception_offset, exception_size)
  1 = try(handler_offset)
```

**Response payload:**
```
[STATUS (u32)] = 0 if handled, <0 if unhandled
```

### FFI Effect

**Request payload:**

```
[OPCODE (u8)] [ARGS*]

OPCODE:
  0 = call(func_name_offset, func_name_len, arg_offset, arg_count)
```

**Response payload:**
```
[RESULT*] = return value(s) from function
```

### Concurrency Effect

**Request payload:**

```
[OPCODE (u8)] [ARGS*]

OPCODE:
  0 = lock(lock_id)
  1 = unlock(lock_id)
  2 = atomic_load(address)
  3 = atomic_store(address, value)
```

**Response payload:**
```
[STATUS (u32)] = 0 on success, <0 on failure
[VALUE (u64)] = for atomic_load
```

## Continuation Contract

**After an effect is handled:**

1. Handler updates task's runtime state
2. Handler returns EffectResponse with status
3. If status = 0: Task resumes at next instruction
4. If status < 0: Exception is raised (propagates if unhandled)
5. If status = suspended: Task yields, handler resumes later

## ABI Guarantee

All handlers maintain these invariants:

- **Atomicity:** Effect is atomic; no partial results
- **Consistency:** Heap/state remains valid after effect
- **Ordering:** Effects from different regions don't race
- **Determinism:** Same inputs → same outputs (for provability)

## Integration Points

### Native → Handler

```rust
// Generated native code calls this function
extern "C" fn effect_emit(request: *const EffectRequest) -> EffectResponse {
    let req = unsafe { *request };
    // Dispatch to handler based on req.effect_kind
    // Return response
}
```

### Managed → Handler

```rust
// Managed executor dispatches effect through handler trait
let response = handler.handle(EffectRequest { .. })?;
task.state = response.next_state;
```

### WASM → Native Bridge

```wasm
(import "env" "effect_emit" (func $emit (param i32) (result i32)))

(call $emit (i32.load (global.get $request_ptr)))
```

---

## Related Documents

- [`RMIR_SPEC.md`](./RMIR_SPEC.md) — RMIR instruction semantics
- [`ARCHITECTURE.md`](../ARCHITECTURE.md) — Full system design
- [`runtime/src/abi.rs`](../runtime/src/abi.rs) — Runtime ABI implementation
