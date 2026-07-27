# RMIR Specification: Instruction Semantics and State Machine

## Overview

RMIR (Refinement MIR) is the proof-carrying intermediate representation. Each instruction is a primitive operation that transitions the execution state machine and may generate proof obligations.

## Execution State

```rust
pub struct ExecutionState {
    values: HashMap<Id, Value>,           // SSA values
    regions: HashMap<RegionId, RegionStatus>, // Region lifecycle
    capabilities: HashMap<(RegionId, Id), CapabilityKind>, // Linear capabilities
    effects: Vec<Effect>,                 // Emitted effects
    proof_obligations: Vec<ProofObligation>, // Lemmas to prove
    pc: u32,                              // Program counter
}
```

## Region Status State Machine

```
Unentered  --[region_enter]--> Active {allocations}  --[region_exit]--> Closed
           <invalid state>                  |
                                      [allocate/deallocate]
                                            |
                                      modifications allowed
```

## Instructions

### 1. Assign

**Precondition:** None

**Postcondition:**
- `values[id] := value`
- No proof obligations

**Semantics:**
```
Assign { id: 42, value: Int(100) }
→ values[42] = Int(100)
```

---

### 2. Move

**Precondition:**
- `values[src]` exists
- `capability(src, move)` held

**Postcondition:**
- `values[dst] := values[src]`
- `values[src]` invalidated
- **Proof obligation:** `no_use_after_move(src)`

**Semantics:** Transfer ownership from `src` to `dst`.

---

### 3. Borrow

**Precondition:**
- `values[src]` exists
- `capability(src, borrow)` held

**Postcondition:**
- `values[borrow_id] := Reference(src, mode)`
- Borrow valid for program points `[pc, lifetime)`
- **Proof obligation:** `borrow_lifetime_valid(borrow_id, src, lifetime)`

**Semantics:** Create a reference with limited lifetime.

---

### 4. Consume

**Precondition:**
- `values[id]` exists
- `ownership_held(id)`

**Postcondition:**
- `values[id]` invalidated
- **Proof obligation:** `no_use_after_consume(id)`

**Semantics:** Invalidate a value (drop/free).

---

### 5. RegionEnter

**Precondition:**
- `regions[region_id] = Unentered`

**Postcondition:**
- `regions[region_id] := Active { allocations: [] }`
- **Proof obligation:** `region_lifecycle_valid(region_id)`

**Semantics:** Begin a region's lifetime.

---

### 6. RegionExit

**Precondition:**
- `regions[region_id] = Active { allocations: [] }`
- All allocations deallocated

**Postcondition:**
- `regions[region_id] := Closed`
- **Proof obligation:** `region_closed_wellformed(region_id)`

**Semantics:** End a region's lifetime.

---

### 7. Allocate

**Precondition:**
- `regions[region_id] = Active { .. }`
- `size > 0`

**Postcondition:**
- `regions[region_id].allocations += ptr_id`
- `capabilities[(region_id, ptr_id)] := Write`
- **Proof obligation:** `allocation_wellformed(ptr_id, region_id)`

**Semantics:** Allocate memory in a region.

---

### 8. Deallocate

**Precondition:**
- `regions[region_id] = Active { allocations }`
- `ptr_id in allocations`
- `capability(region_id, ptr_id, deallocate)` held

**Postcondition:**
- `regions[region_id].allocations -= ptr_id`
- `capabilities[(region_id, ptr_id)]` removed
- **Proof obligation:** `deallocation_valid(ptr_id)`

**Semantics:** Deallocate memory in a region.

---

### 9. EffectEmit

**Precondition:**
- Effect-specific preconditions (checked by prover)

**Postcondition:**
- `effects += eff`
- **Proof obligation:** `effect_precondition(eff)`

**Semantics:** Request a side effect from the runtime.

---

### 10. Assert

**Precondition:**
- None (prover verifies predicate)

**Postcondition:**
- **Proof obligation:** `assert(predicate)`

**Semantics:** Assert a predicate must hold.

---

## Proof Obligations

Each instruction may generate proof obligations for the discrete prover:

| Obligation | Kind | Examples |
|-----------|------|----------|
| Ownership | Linear usage | `no_use_after_move`, `no_use_after_consume` |
| Regions | Lifecycle | `region_lifecycle_valid`, `region_closed_wellformed` |
| Capabilities | Linear access | `capability_held_at(resource, capability, pc)` |
| Effects | Preconditions | `effect_precondition(io_write)`, `effect_precondition(region_allocate)` |
| Assertions | Predicates | `assert(x > 0)`, `assert(ptr_aligned)` |

---

## Example: Simple Region

```
Function: process_buffer(buf: &mut [u8])

RMIR Instructions:
  0: RegionEnter { region_id: 0 }
     → regions[0] = Active {}
     → obligation: region_lifecycle_valid(0)

  1: Allocate { region_id: 0, size: 1024, ptr_id: 1 }
     → regions[0].allocations = [1]
     → capabilities[(0, 1)] = Write
     → obligation: allocation_wellformed(1, 0)

  2: EffectEmit { effect: IO { request: "write(1, buf, 1024)" } }
     → effects = [IO { ... }]
     → obligation: effect_precondition(io_write, 1, buf)

  3: Deallocate { region_id: 0, ptr_id: 1 }
     → regions[0].allocations = []
     → obligation: deallocation_valid(1)

  4: RegionExit { region_id: 0 }
     → regions[0] = Closed
     → obligation: region_closed_wellformed(0)

Final state: All obligations discharged ✓
```

---

## Serialization Format

**RMIR Bytecode (custom binary):**

```
[VERSION (u32)] [CHECKSUM (u256)] [INSTRUCTIONS*]

INSTRUCTION ::= [OPCODE (u8)] [ARG_COUNT (u8)] [ARGS*]

OPCODE:
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

---

## Invariants Checked

1. **Region Lifecycle:** Unentered → Active → Closed, no other transitions
2. **No Use-After-Free:** Consumed values cannot be accessed
3. **Linear Ownership:** Exactly one path owns a value
4. **Allocation Tracking:** All allocations deallocated before region exit
5. **Effect Ordering:** Effects are deterministically ordered
6. **Borrow Validity:** Borrows don't outlive referents

---

## Related Documents

- [`ARCHITECTURE.md`](../ARCHITECTURE.md) — Full system design
- [`EFFECT_HANDLER_SPEC.md`](./EFFECT_HANDLER_SPEC.md) — Effect ABI
- [`ASP_RULES.pl`](./ASP_RULES.pl) — ASP constraint rules
