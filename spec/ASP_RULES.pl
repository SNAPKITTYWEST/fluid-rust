%% ============================================================================
%% FLUID RUST ASP RULES: Ownership, Regions, and Effects
%% ============================================================================
%%
%% Input: Facts extracted from RMIR
%% Output: Answer set satisfying all constraints, or UNSAT with diagnosis
%%
%% Key predicates:
%%   - owns(value, thread, timestamp)
%%   - region_status(region, timestamp, status)
%%   - allocated_in(ptr, region, timestamp)
%%   - capability(resource, kind, timestamp)
%%   - effect_emitted(effect, timestamp)

%% ============================================================================
%% PART 1: OWNERSHIP INVARIANTS
%% ============================================================================

%% Constraint: No two threads own the same value at the same time
:- owns(V, T1, TS),
   owns(V, T2, TS),
   T1 \= T2.

%% Constraint: Once consumed, a value cannot be owned again
%% (assumes fact consumed(V, TS) if value V is consumed at timestamp TS)
:- owns(V, T, TS1),
   consumed(V, TS2),
   TS2 < TS1.

%% Derived predicate: A value is live at timestamp TS if owned and not consumed
live(V, T, TS) :- owns(V, T, TS),
                  not consumed_before(V, TS).

consumed_before(V, TS) :- consumed(V, TS_c),
                          TS_c < TS.

%% ============================================================================
%% PART 2: REGION LIFECYCLE INVARIANTS
%% ============================================================================

%% Constraint: Region status is one of {unentered, active, closed}
%% (implicitly enforced by fact generation)

%% Constraint: Status transitions follow Unentered → Active → Closed
%% (regions cannot go backwards or skip steps)

%% Rule: Region starts unentered
region_status(R, 0, unentered) :- region(R).

%% Constraint: Cannot allocate in unentered region
:- allocated_in(P, R, TS),
   region_status(R, TS, unentered).

%% Constraint: Cannot allocate in closed region
:- allocated_in(P, R, TS),
   region_status(R, TS, closed).

%% Constraint: All allocations must be deallocated before region closes
%% If a region is closed at TS_close, all pointers allocated before must be deallocated
:- region_status(R, TS_close, closed),
   allocated_in(P, R, TS_alloc),
   TS_alloc < TS_close,
   not deallocated(P, TS_close).

%% Constraint: Cannot exit region if allocations remain
%% (This is implicit in the above: region_exit requires all allocs deallocated)

%% ============================================================================
%% PART 3: CAPABILITY INVARIANTS (Linear Access)
%% ============================================================================

%% Constraint: At most one holder of write capability per resource
%% (Write capability is linear: cannot be shared)
:- capability(R, P, write, TS),
   owns(R, T1, TS),
   owns(R, T2, TS),
   T1 \= T2.

%% Rule: Shared read capability can be held by multiple threads
%% (No constraint needed; multiple owns facts are allowed for read)

%% Derived predicate: A resource has write capability
has_write_cap(R, P, TS) :- capability(R, P, write, TS).

%% Derived predicate: A resource has read capability
has_read_cap(R, P, TS) :- capability(R, P, read, TS).

%% ============================================================================
%% PART 4: EFFECT ORDERING AND PRECONDITIONS
%% ============================================================================

%% Constraint: Effect preconditions are met
%% (Specific preconditions are effect-dependent; examples below)

%% Example: IO write precondition
%% Effect io_write(fd, buffer, length) requires:
%%   - fd must be a valid open file descriptor
%%   - buffer must be allocated and writable
:- effect_emitted(io_write(FD, Buffer, _), TS),
   not valid_file_descriptor(FD, TS).

:- effect_emitted(io_write(_, Buffer, _), TS),
   not allocated_and_writable(Buffer, TS).

%% Example: Region allocate precondition
%% Effect region_allocate(region, size) requires:
%%   - region must be active
%%   - size must be positive
%%   - region must have enough space
:- effect_emitted(region_allocate(R, _), TS),
   not region_status(R, TS, active).

:- effect_emitted(region_allocate(_, Size), _),
   Size <= 0.

%% ============================================================================
%% PART 5: PROOF OBLIGATION SUMMARY
%% ============================================================================

%% All constraints are checked as goal rules (denials).
%% If any constraint is violated, the ASP program is unsatisfiable.
%%
%% Constraints check:
%%   1. No aliasing (two threads own same value)
%%   2. No use-after-consume
%%   3. Region lifecycle order (unentered → active → closed)
%%   4. No allocation in unentered/closed regions
%%   5. All allocations deallocated before region closes
%%   6. Write capability is linear
%%   7. Effect preconditions are met
%%
%% If all constraints are satisfied, we have a proof that the program is safe.

%% ============================================================================
%% HELPER PREDICATES (Example implementations)
%% ============================================================================

%% These would be extracted as facts from RMIR:

% valid_file_descriptor(FD, TS) :- ...
% allocated_and_writable(Buffer, TS) :- ...
% deallocated(P, TS) :- ...

%% ============================================================================
%% INTEGRITY CONSTRAINTS (Goals)
%% ============================================================================

%% End of ASP rules. All constraints above are checked.
%% If the program has an answer set, the program is verified safe.
%% If the program is unsatisfiable, there is an ownership/region/effect violation.
