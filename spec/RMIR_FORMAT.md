# RMIR Bytecode Format Specification

**Version:** 1.0  
**Status:** Phase P1 Complete  
**Author:** FLUID RUST Design Team

---

## Overview

RMIR (Refinement MIR) is a proof-carrying intermediate representation that encodes:
- **Value SSA** — Traditional static single assignment (values, assignments, phi nodes)
- **Capability SSA** — Ownership tracking (move, borrow, consume)
- **Region FSM** — Region lifecycle state machine (unentered → active → closed)
- **Effect Transitions** — Verified state changes (IO, State, Async, Region, GC, Exception, FFI, Concurrency)

This specification defines:
1. Binary bytecode format (machine-efficient)
2. JSON schema (human-readable, proof-exchange)
3. All 32 opcodes with arguments
4. Type encoding (12 kinds)
5. Proof obligation encoding

---

## Binary Format

### File Structure

```
┌─────────────────────────────────────────────┐
│ Magic (4 bytes): "RMIR"                     │
│ Version (u32 LE): 0x00000001               │
│ Flags (u8): [reserved]                      │
│ Padding (3 bytes): 0x00                     │
│ Metadata Section                            │
│ │ Length (u32 LE)                           │
│ │ Timestamp (u64 LE, Unix seconds)          │
│ │ Source file length (u32 LE)               │
│ │ [Source file path (UTF-8)]                │
│ │ Compiler version length (u8)              │
│ │ [Compiler version (UTF-8, max 255 bytes)] │
│ │ [Padding to 16-byte boundary]             │
│ Instruction Section                         │
│ │ Count (u32 LE)                            │
│ │ [Instructions (variable length)]          │
│ Proof Goal Section                          │
│ │ Count (u32 LE)                            │
│ │ [Proof goals (variable length)]           │
│ Checksum (Blake3, 32 bytes)                 │
└─────────────────────────────────────────────┘

Total header: 16 bytes (magic + version + flags + padding)
Metadata: 12 bytes + variable
Instructions: 4 bytes + variable
Goals: 4 bytes + variable
Checksum: 32 bytes
```

### Variable-Length Integer Encoding

```
Values 0-127:        Single byte (0x00-0x7F)
Values 128-16383:    Two bytes (0x80-0xBF, then 0x00-0xFF)
Values 16384+:       0xC0 + 4 bytes (u32 LE)

Example:
  127 → 0x7F
  128 → 0x80 0x00
  255 → 0x80 0x7F
  256 → 0x81 0x00
  16383 → 0xBF 0x7F
  16384 → 0xC0 0x00 0x40 0x00 0x00
```

---

## Type Encoding (12 Kinds)

```
Type ID (u32):
  0x0000 = Unit
  0x0001 = Bool
  0x0002 = Int(signedness, width)    // signedness: 0=signed, 1=unsigned
  0x0003 = Float(width)              // width: 32 or 64
  0x0004 = Pointer(pointee_type_id)
  0x0005 = Reference(region_id, lifetime_id, mode, pointee_type_id)
  0x0006 = Array(element_type_id, length)
  0x0007 = Struct(field_count, [field_type_ids...])
  0x0008 = Enum(variant_count, [variant_type_ids...])
  0x0009 = Function(param_count, [param_type_ids...], return_type_id)
  0x000A = Refined(base_type_id, predicate_id)
  0x000B = Region(region_id, lifetime_id)

Example encodings:
  Bool          → 0x00000001
  u32           → 0x00000002 0x00000001 0x00000020
  &str          → 0x00000005 0x00000001 (Array(u8, unknown_len))
  Region<'a>    → 0x0000000B 0x00000001 0x00000001
```

---

## All 32 Opcodes

### Value SSA (5 opcodes)

```
0x00: Assign(dest_id: u32, value_id: u32)
      Write a value to a variable

0x01: Move(dest_id: u32, src_id: u32)
      Move ownership (src_id becomes invalid after)

0x02: Phi(dest_id: u32, pred_count: u8, [src_ids: u32...])
      Merge values from multiple control flow paths

0x03: Call(dest_id: u32, func_id: u32, arg_count: u8, [arg_ids: u32...])
      Call a function with arguments, store result

0x04: Return(value_id: u32)
      Return from function
```

### Ownership (4 opcodes)

```
0x05: Borrow(dest_id: u32, src_id: u32, mode: u8, lifetime_id: u32)
      Create a borrowed reference (mode: 0=immutable, 1=mutable, 2=unique)

0x06: Consume(value_id: u32)
      Mark a linear value as consumed (no further use allowed)

0x07: CapabilityGrant(value_id: u32, capability_id: u32)
      Grant a capability to a value

0x08: CapabilityRevoke(value_id: u32, capability_id: u32)
      Revoke a capability from a value
```

### Regions (3 opcodes)

```
0x09: RegionEnter(region_id: u32)
      Enter a region scope (state: unentered → active)

0x0A: RegionExit(region_id: u32)
      Exit a region scope (state: active → closed, deallocate all)

0x0B: RegionMerge(dest_region_id: u32, src_region_ids: u8, [src_ids: u32...])
      Merge multiple regions into one
```

### Effects (3 opcodes)

```
0x0C: EffectEnter(effect_kind: u8)
      Enter an effect scope (enable effect handling)

0x0D: EffectExit(effect_kind: u8)
      Exit an effect scope (disable effect handling)

0x0E: Transition(effect_kind: u8, proof_obligation_id: u32)
      Perform an effect transition (requires proof)
```

### Memory (4 opcodes)

```
0x0F: ReadMemory(dest_id: u32, ptr_id: u32, offset: u32, type_id: u32)
      Read from memory at pointer + offset

0x10: WriteMemory(ptr_id: u32, offset: u32, value_id: u32)
      Write to memory at pointer + offset

0x11: Allocate(region_id: u32, type_id: u32, size_id: u32, dest_ptr_id: u32)
      Allocate memory in region, store pointer

0x12: Deallocate(region_id: u32, ptr_id: u32)
      Deallocate memory (freed in region)
```

### Control Flow (5 opcodes)

```
0x13: Split(condition_id: u32, true_label: u32, false_label: u32)
      Conditional branch

0x14: Join(label: u32)
      Merge control flow (label for phi nodes)

0x15: Assert(predicate_id: u32, proof_id: u32)
      Assert a predicate with proof witness

0x16: Assume(predicate_id: u32)
      Assume a predicate (for analysis, no runtime check)

0x17: Refinement(value_id: u32, predicate_id: u32)
      Refine a value's type with a predicate
```

### Concurrency (5 opcodes)

```
0x18: Fork(task_id: u32, arg_ids: u8, [arg_ids: u32...])
      Spawn a concurrent task

0x19: JoinThread(task_id: u32, dest_id: u32)
      Wait for task to complete, get result

0x1A: Lock(mutex_id: u32, dest_id: u32)
      Acquire lock (dest_id = guard)

0x1B: Unlock(guard_id: u32)
      Release lock

0x1C: Synchronize(sync_id: u32, mode: u8)
      Synchronization point (mode: 0=barrier, 1=once, 2=wait)
```

### Exceptions (2 opcodes)

```
0x1D: Catch(exception_type_id: u32, handler_label: u32)
      Catch exception, jump to handler

0x1E: Throw(exception_id: u32)
      Throw exception
```

---

## Proof Obligation Encoding

```
Goal ID (u32): Index into proof goals array

Goal Types:
  0x00: OwnershipInvariant(value_id)
  0x01: RegionSafety(region_id)
  0x02: LinearityConstraint(value_id)
  0x03: EffectOrdering(effect1_id, effect2_id)
  0x04: BoundsCheck(index_id, length_id)
  0x05: EffectPrecondition(effect_kind)
  0x06: TypeRefinement(value_id, predicate_id)
  0x07: DataRaceFreedom(value_id, thread_count)
  0x08: DeadlockFreedom(lock_graph_id)

Encoding:
  [goal_type: u8]
  [goal_id: u32]
  [arg_count: u8]
  [args: variable]
```

---

## JSON Schema

```json
{
  "rmir_version": "1.0",
  "program_hash": "blake3:...",
  "metadata": {
    "timestamp": "2026-07-26T14:33:22Z",
    "source_file": "main.rs",
    "compiler_version": "0.1.0"
  },
  "instructions": [
    {
      "id": 0,
      "opcode": "RegionEnter",
      "args": { "region_id": 1 }
    },
    {
      "id": 1,
      "opcode": "Allocate",
      "args": {
        "region_id": 1,
        "type_id": 2,
        "size_id": 3,
        "dest_ptr_id": 4
      },
      "proof_obligations": [5, 6]
    }
  ],
  "proof_goals": [
    {
      "id": 0,
      "kind": "RegionSafety",
      "args": { "region_id": 1 }
    }
  ],
  "types": [
    { "id": 0, "kind": "Unit" },
    { "id": 1, "kind": "Bool" },
    { "id": 2, "kind": "Int", "signedness": "unsigned", "width": 8 }
  ]
}
```

---

## Example: Simple Region Program

### Rust Code
```rust
fn example() {
    region_enter(stack);
    let ptr = allocate::<u8>(stack, 1024);
    write(ptr, data);
    region_exit(stack);
}
```

### RMIR Binary (hex)
```
52 4D 49 52                          // "RMIR" (magic)
01 00 00 00                          // version 1
00 00 00 00                          // flags + padding
0C 00 00 00                          // metadata length
26 C1 5C 66 00 00 00 00              // timestamp
04                                   // source file length
6D 61 69 6E                          // "main"
...
04 00 00 00                          // 4 instructions
09 01 00 00 00                       // RegionEnter(1)
11 01 02 03 04                       // Allocate(1,2,3,4)
10 04 00 00 64 61 74 61              // WriteMemory(4, 0, data)
0A 01 00 00 00                       // RegionExit(1)
01 00 00 00                          // 1 proof goal
01 01 00 00 00                       // RegionSafety(1)
[32 bytes Blake3 checksum]
```

### RMIR JSON
```json
{
  "program_hash": "blake3:...",
  "instructions": [
    { "id": 0, "opcode": "RegionEnter", "args": { "region_id": 1 } },
    { "id": 1, "opcode": "Allocate", "args": { "region_id": 1, "type_id": 2, "size_id": 3, "dest_ptr_id": 4 }, "proof_obligations": [0] },
    { "id": 2, "opcode": "WriteMemory", "args": { "ptr_id": 4, "offset": 0, "value_id": 5 } },
    { "id": 3, "opcode": "RegionExit", "args": { "region_id": 1 } }
  ],
  "proof_goals": [
    { "id": 0, "kind": "RegionSafety", "region_id": 1 }
  ]
}
```

---

## Design Invariants

1. **Completeness:** All Rust constructs can be encoded
2. **Unambiguity:** Bytecode → RMIR is deterministic
3. **Roundtrip:** encode(decode(X)) = X
4. **Efficiency:** Binary form is compact (<10% source size)
5. **Verifiability:** Checksum catches corruption
6. **Extensibility:** Version field allows future formats
7. **Type Safety:** All type references are valid IDs
8. **Proof Integrity:** All proof goals tagged with IDs

---

## Versioning Strategy

- **Current:** Version 1 (this spec)
- **Future:** Version 2+ can extend opcodes/goals without breaking v1 readers
- **Compatibility:** Readers check version; reject if unsupported
- **Migration:** Converter tools for v1→v2 translation

---

**Status:** Ready for implementation  
**Next:** Implement encoder/decoder in Phase P1
