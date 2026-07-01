# Dispatcher Wrapper Pattern

**Audience:** Engine authors implementing domain operations through the
`CommandDispatcher`.

**Source of truth:** `crates/educore/src/dispatch.rs` (skeleton), per-domain
service modules in `crates/domains/*/src/services.rs`, the
`CommandDispatcher` in `crates/cross-cutting/dispatcher/`.

## Why a wrapper layer?

The engine's architecture per `docs/architecture.md` § "Command Bus +
Dispatcher" defines two layers:

1. **Domain services** — pure Rust functions that take `(aggregate,
   clock, ids, …)` and return `(Aggregate, Event)`. No auth, no
   outbox, no audit. Easy to unit-test.
2. **Dispatcher** — the cross-cutting layer that wraps every service
   call with: RBAC check → idempotency lookup → aggregate load →
   domain logic → event emission → persistence → bus publish → audit
   write — all in a single transaction.

The wrapper pattern keeps domain services pure (so they remain
testable and the `Aggregate + Clock + Ids` signature is unchanged) by
moving the cross-cutting pipeline into a thin wrapper at the umbrella
crate level. **No breaking change to service function signatures.**

## Pattern

For every public domain service function `domain::verb_thing(args)`,
produce a `crates/educore/src/dispatch.rs::dispatch_verb_thing(cmd, deps)`
that:

1. Validates the `TenantContext` (school, actor, correlation).
2. Calls `CommandDispatcher::dispatch(&cmd, |cmd| async move {
   domain::verb_thing(cmd, &clock, &ids, …))`.
3. The dispatcher internally:
   - Resolves the `Command`'s `required_capabilities()`.
   - Calls `RbacPort::require(actor, &caps)`. On failure → `DomainError::Forbidden`.
   - Checks idempotency table for `IdempotencyKey`. If present and
     matches → return cached outcome. If conflict → `DomainError::IdempotencyConflict`.
   - Loads the aggregate by id.
   - Calls the closure (the pure service function).
   - In a single transaction: persist aggregate, append outbox row,
     write audit row, publish to bus.
   - Records outcome in idempotency table.
4. Returns the typed result to the caller.

## Concrete shape

```rust
// crates/educore/src/dispatch.rs (skeleton — full impl in step 2+)

pub async fn dispatch_admit_student(
    dispatcher: &CommandDispatcher,
    cmd: AdmitStudentCommand,
) -> Result<StudentAdmitted, DomainError> {
    dispatcher.dispatch(cmd, |cmd| async move {
        educore_academic::services::admit_student(cmd, &clock, &ids, &uniqueness)
    }).await
}
```

## Why this is spec-conformant

`docs/architecture.md` shows:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          Command Bus + Dispatcher                        │
│   Authn → Authz → Validation → Aggregate Load → Domain Logic            │
│   → Event Emission → Persistence → Bus Publish → Audit Write            │
└─────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                          Domain Core (pure Rust)                         │
│   Aggregates • Entities • Value Objects • Domain Services               │
└─────────────────────────────────────────────────────────────────────────┘
```

Domain Core includes "Domain Services" alongside Aggregates. The
dispatcher wraps the domain service — it doesn't replace the service
signature. This is the spec-conformant pattern (per the user's explicit
confirmation during scoping).

## Migration path for test harnesses

`crates/tools/storage-parity/tests/` and `crates/tools/testkit/` currently
call domain services directly. Migration:

**Before:**
```rust
let (student, event) = academic::services::admit_student(cmd, &clock, &ids, &uniqueness).await?;
```

**After:**
```rust
let event = educore::dispatch::dispatch_admit_student(&dispatcher, cmd).await?;
```

This adds:
- RBAC enforcement (test must declare required caps)
- Idempotency (test must use unique keys)
- Audit row (asserted via dispatcher fixture)
- Bus publish (asserted via dispatcher fixture)

The aggregate-level tests (which exercise `aggregate.rs` directly
without going through services) remain valid — they test invariants
in isolation.

## Why this scope is large

Per `docs/audit_reports/stub_vs_implementation.md`, the engine has
**~358 service functions** across 10 domains. Building a wrapper per
function is ~358 small wrapper bodies + ~358 dispatcher-integration
tests. The prior ferment's deferral pattern showed this is too much
for a single sub-agent dispatch.

**Per-domain wrapper sub-batches:** Each domain needs its own
focused sub-step:
- Step 2: academic (38) + assessment (74) + attendance (16) = 128 wrappers
- Step 3: remaining 7 domains = 230 wrappers

These are deferred in this ferment; the spec doc + dispatch.rs
skeleton (Step 1) provides the foundation for focused future work.

## Acceptance criteria (per service function)

A wrapper is complete when:
- [ ] Service function has a matching `dispatch_<verb>(...)` in
      `crates/educore/src/dispatch.rs`.
- [ ] The wrapper signature uses the corresponding `*Command` struct
      (not raw service args).
- [ ] The wrapper calls `dispatcher.dispatch(cmd, |cmd| ...)` with the
      correct domain closure.
- [ ] An integration test exercises the wrapper end-to-end with a
      valid `TenantContext`.
- [ ] An integration test exercises the rejection path (user lacks
      required capability).
- [ ] The wrapper is re-exported from `crates/educore/src/lib.rs`.

## See also

- `crates/cross-cutting/dispatcher/` — the `CommandDispatcher` impl
- `crates/cross-cutting/dispatcher/tests/forbidden_rejection.rs` —
  the 10 spec-justified rejection tests (Phase 5 Wave 37)
- `docs/architecture.md` § "Command Bus + Dispatcher" — authoritative
  source for this pattern
- `docs/guides/capability-rbac.md` — RBAC integration overview
