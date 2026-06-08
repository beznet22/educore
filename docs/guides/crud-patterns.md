# CRUD Patterns Guide

## Goal

Standardize the way the engine handles create, read, update, and
delete commands. Every aggregate follows the same patterns, making
the engine predictable and easy to learn.

## Naming

Commands follow `Verb<Noun>`:

- `Create<Class>` — creates a new aggregate.
- `Update<Class>` — updates a mutable aggregate.
- `Delete<Class>` — soft- or hard-deletes.
- `Get<Class>` (query) — reads a single aggregate.
- `List<Class>` (query) — reads many aggregates.
- `Find<Class>` (query) — finds an aggregate by a natural key.

The `Command` suffix is optional in code but recommended in docs:

```rust
pub struct CreateClassCommand { ... }
pub struct UpdateClassCommand { ... }
```

## Create

```rust
pub struct CreateClassCommand {
    pub tenant: TenantContext,
    pub class_name: ClassName,
    pub pass_mark: PassMark,
}
```

- The aggregate is constructed from the command in a `TryFrom`.
- The aggregate's invariants are checked during construction.
- A `Created` event is emitted.
- The aggregate is persisted.

```rust
impl CreateClassCommand {
    pub async fn execute(self, repo: &dyn ClassRepository, events: &mut Outbox) -> Result<Class> {
        let class = Class::try_new(self.class_name, self.pass_mark, self.tenant.school_id)?;
        repo.insert(&class).await?;
        events.append(ClassCreated { class_id: class.id, class_name: class.name, ... }.into()).await?;
        Ok(class)
    }
}
```

## Read

```rust
let class = engine.classes().get(class_id).await?;
```

Reads return `Option<Aggregate>` for `get` and `Vec<Aggregate>` for
`list`. Errors are infrastructure only; missing aggregates are
`Ok(None)`.

## Update

```rust
pub struct UpdateClassCommand {
    pub tenant: TenantContext,
    pub class_id: ClassId,
    pub patch: ClassPatch,
}

pub struct ClassPatch {
    pub class_name: Option<ClassName>,
    pub pass_mark: Option<PassMark>,
}
```

- Only mutable fields are patchable.
- The aggregate is loaded, the patch applied, invariants re-checked.
- An `Updated` event is emitted.
- The aggregate is persisted.

The patch is `Option<Field>`-based: `None` means "don't change", `Some(v)` means "set to v". This avoids the need for `Default` on every field.

## Delete

```rust
pub struct DeleteClassCommand {
    pub tenant: TenantContext,
    pub class_id: ClassId,
    pub reason: Option<String>,
}
```

Most aggregates support **soft delete** (`active_status = 0`).
Hard delete is reserved for aggregates with no historical references
(e.g. registration fields, dashboards).

A delete that would orphan referenced aggregates is rejected:

```rust
let referenced = engine.student_records().count_for_class(class_id).await?;
if referenced > 0 {
    return Err(DomainError::Conflict {
        entity: "Class",
        reason: format!("{} student records reference this class", referenced),
    });
}
```

## Commands That Aren't Pure CRUD

Some commands are domain actions, not CRUD:

- `PromoteStudent` — moves a student forward.
- `PublishResult` — finalizes a workflow.
- `RecordPayment` — an event in a financial flow.

These follow the same pattern (load → mutate → validate → emit →
persist) but are not labeled `Create`/`Update`/`Delete`.

## Authorization

Every command requires a capability. CRUD commands have natural
mappings:

| Command                | Capability                |
| ---------------------- | ------------------------- |
| `Create<Class>`        | `Class.Create`            |
| `Update<Class>`        | `Class.Update`            |
| `Delete<Class>`        | `Class.Delete`            |
| `Get<Class>`           | `Class.Read`              |
| `List<Class>`          | `Class.Read`              |

Read capabilities are not implied by write capabilities. A consumer
may grant `Class.Read` to a parent without granting `Class.Create`.

## Idempotency

All commands are idempotent via `IdempotencyKey`. A retry returns
the same outcome.

## Concurrency

The engine uses optimistic concurrency: every aggregate has a
`version` field. A command that loads an aggregate at version `v`
and tries to persist at version `v` succeeds; a version mismatch
returns `Conflict::StaleVersion`. The caller may re-load and retry.

## Tenant Isolation

Every command carries `TenantContext` and the engine rejects commands
whose target aggregate is in a different school.

## Testing

Every CRUD command has:

- A happy path test.
- A test of validation failure.
- A test of conflict (e.g. duplicate name).
- A test of authorization (missing capability).
- A test of tenant isolation.
- A test of concurrency (stale version).
- A test of idempotency (replay returns same outcome).

## Anti-Patterns

- ❌ Bypassing the engine to write directly to the database.
- ❌ Constructing aggregates without `TryFrom` validation.
- ❌ Patching fields that should be immutable (admission number,
  school id).
- ❌ Emitting no events (the engine should always know).
- ❌ Forgetting the tenant context.
- ❌ Returning domain errors as `anyhow::Error`.
