# Test Strategy

## Goal

Establish a comprehensive test pyramid for SMSengine consumers and
provide a clear, repeatable test workflow.

## Test Pyramid

```text
                /\
               /  \           E2E (few, slow, run in CI)
              /----\
             /      \         Integration (per workflow)
            /--------\
           /          \       Component (per command, per repository)
          /------------\
         /              \     Unit (per value object, per policy)
        /------------------\
```

| Layer          | Count         | Speed       | What it covers                          |
| -------------- | ------------- | ----------- | --------------------------------------- |
| Unit           | Many          | µs          | Validation, policies, computations      |
| Component      | Many          | ms          | Commands, repositories, events          |
| Integration    | Per workflow  | s           | Cross-domain flows                      |
| E2E            | Few           | 10s of s    | Full system scenarios                   |

## Unit Tests

Live alongside code in `#[cfg(test)] mod tests`. Cover:

- Every value object's `validate()` happy path and at least one error
  path.
- Every policy's decision table.
- Every specification's combinator behavior.
- Every domain service's pure function.
- Every event's serialization roundtrip.

Example:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_address_rejects_invalid() {
        assert!(EmailAddress::parse("not-an-email").is_err());
        assert!(EmailAddress::parse("a@b.c").is_ok());
        assert!(EmailAddress::parse("").is_err());
    }
}
```

## Component Tests

Live in `tests/`. One file per command family or per repository. Use
the in-memory or testcontainer-backed storage adapter.

```rust
#[tokio::test]
async fn admit_student_creates_record() {
    let engine = test_engine().await;
    let tenant = test_tenant();

    let student = engine
        .students()
        .admit(AdmitStudentCommand { tenant, ... })
        .await
        .unwrap();

    assert_eq!(student.status, StudentStatus::Active);
    let record = engine.student_records()
        .default_for_student(student.id).await.unwrap();
    assert!(record.is_some());
}
```

Every command has at least:
- One happy path test.
- Two error path tests (validation, conflict, forbidden).

## Integration Tests

Cover cross-domain workflows:

- `tests/test_admission_workflow.rs` — admit → library membership → fees
  assignment → welcome message.
- `tests/test_promotion_workflow.rs` — promote → fees rollover →
  attendance reset.
- `tests/test_fee_collection.rs` — generate invoice → pay → receipt →
  ledger.
- `tests/test_result_publication.rs` — enter marks → compute → publish
  → notify.

Integration tests run against a real database (testcontainers for
PostgreSQL, in-memory for SQLite).

## E2E Tests

A small number of E2E tests cover the full stack:

- A school onboarding scenario.
- A full academic year (admit → attend → examine → promote).
- A multi-tenant SaaS scenario (two schools, isolation).

E2E tests use the consumer's actual adapters.

## Idempotency Tests

Every command that should be idempotent has a test:

```rust
#[tokio::test]
async fn admit_student_is_idempotent_on_admission_no() {
    let engine = test_engine().await;
    let tenant = test_tenant();
    let cmd = admit_cmd(tenant);

    let s1 = engine.students().admit(cmd.clone()).await.unwrap();
    let s2 = engine.students().admit(cmd).await.unwrap();
    assert_eq!(s1.id, s2.id);
}
```

## Conflict Tests

```rust
#[tokio::test]
async fn admit_student_rejects_duplicate_admission_no() {
    let engine = test_engine().await;
    let tenant = test_tenant();
    engine.students().admit(admit_cmd(tenant)).await.unwrap();
    let err = engine.students().admit(admit_cmd(tenant)).await.unwrap_err();
    assert!(matches!(err, DomainError::Conflict { .. }));
}
```

## Permission Tests

```rust
#[tokio::test]
async fn admit_student_requires_capability() {
    let engine = test_engine().await;
    let tenant = test_tenant_without(Capability::StudentAdmit);
    let err = engine.students().admit(admit_cmd(tenant)).await.unwrap_err();
    assert!(matches!(err, DomainError::Forbidden { .. }));
}
```

## Tenant Isolation Tests

```rust
#[tokio::test]
async fn cross_tenant_read_is_blocked() {
    let engine = test_engine().await;
    let school_a = test_school();
    let school_b = test_school();
    let s = engine.students().admit(admit_cmd(school_a)).await.unwrap();
    let err = engine.students()
        .get(s.id)
        .with_tenant(school_b)
        .await
        .unwrap_err();
    assert!(matches!(err, DomainError::Forbidden { .. }));
}
```

## Storage Adapter Tests

The storage adapter ships with:

- A unit test for every repository method.
- An integration test against testcontainers.
- A parity test (PostgreSQL vs. SQLite vs. document store).
- A tenancy test.
- A load test (10k records in <1s).

### Storage Adapter Parity Tests

The same query AST — built through the macro-generated
`*QueryBuilder` — must produce identical results across the
PostgreSQL, SQLite, SurrealDB, and MongoDB adapters. Parity is
enforced by running the same query fixture suite against each
adapter and diffing the materialized result sets. A mismatch fails
the build, because AST translation is the adapter's sole
responsibility and storage-specific behaviour must never leak into
the consumer layer.

## Event Tests

Every event has:

- A serialization roundtrip test.
- An envelope construction test.
- A subscription test (in-process bus).
- A replay test.

## Snapshot Tests

For event payloads, use `insta` snapshots:

```rust
#[test]
fn student_admitted_event_serializes_to_canonical_json() {
    let event = StudentAdmitted { ... };
    let json = serde_json::to_string_pretty(&event).unwrap();
    insta::assert_snapshot!(json);
}
```

For the macro-generated `StudentField` enum, snapshot the expansion
itself so any drift between the struct and its emitted queryable
fields is caught in CI:

```rust
#[test]
fn student_field_enum_expansion_is_stable() {
    let expansion = smsengine_query_derive::__expand_for_tests::<Student>();
    insta::assert_snapshot!(expansion);
}
```

## Macro Snapshot Tests

The `#[derive(DomainQuery)]` expansion is deterministic and is
captured per domain aggregate with `insta`. Both the generated
`*Field` enum and the `*QueryBuilder` state type are snapshot-tested;
a change to the macro output is a deliberate breaking change and
must be approved by reviewers.

```rust
#[test]
fn student_query_builder_expansion_is_stable() {
    let expansion = smsengine_query_derive::__expand_for_tests::<Student>();
    insta::assert_snapshot!(expansion);
}
```

## `where_has` AST Tests

Verify that the closure body compiles into a
`HasRelation(relation, Box<QueryNode<RelatedField>>)` node on the
parent AST. The closure must preserve the related entity's
macro-generated builder type so the inner `QueryNode` carries the
correct `FieldKind`.

```rust
#[test]
fn where_has_emits_has_relation_node() {
    let node = StudentQueryBuilder::new(school_id)
        .where_has(StudentRelation::Parent, |parent_q| {
            parent_q.where_eq(ParentField::BillingStatus, BillingStatus::Active)
        })
        .into_node();

    match node {
        QueryNode::HasRelation(StudentRelation::Parent, inner) => {
            assert!(matches!(
                *inner,
                QueryNode::Eq(ParentField::BillingStatus, _)
            ));
        }
        other => panic!("expected HasRelation, got {:?}", other),
    }
}
```

## Eager Loading Tests

For every relation declared on an aggregate, three assertions are
required:

(a) the relation field is hydrated when `.with(...)` is requested;
(b) the relation field remains `None` (or empty for `Vec<T>`) when
    `.with(...)` is omitted;
(c) hydration failures surface as `StorageError::HydrationFailure`
    and are mapped to `DomainError::Infrastructure`.

```rust
#[tokio::test]
async fn with_hydrates_parent_and_omission_keeps_none() {
    let engine = test_engine().await;

    // (a) .with(...) hydrates the relation field.
    let hydrated = engine.students().query()
        .with(StudentRelation::Parent)
        .first()
        .await
        .unwrap();
    assert!(hydrated.parent.is_some());

    // (b) omitting .with(...) leaves the field None.
    let plain = engine.students().query()
        .first()
        .await
        .unwrap();
    assert!(plain.parent.is_none());
}

#[tokio::test]
async fn hydration_failure_maps_to_hydration_failure() {
    let failing = test_engine_with_failing_hydration().await;

    // (c) hydration failures surface as StorageError::HydrationFailure.
    let err = failing.students().query()
        .with(StudentRelation::Parent)
        .first()
        .await
        .unwrap_err();
    assert!(matches!(err, StorageError::HydrationFailure { .. }));
}
```

## Macro Output Drift Tests

The procedural macro is tested in isolation: for every aggregate,
the expansion of `#[derive(DomainQuery)]` is captured as an `insta`
snapshot. Any unintended change in the generated `*Field` enum, the
`*QueryBuilder` state struct, or the `*Relation` enum produces a
snapshot diff in CI. Snapshots are committed; reviewers must approve
any change to the expansion output, since drift in generated types
ripples through every consumer of the aggregate.

## Coverage

Target: **>90% line coverage** for domain code. Adapters may have
lower coverage (infrastructure code is hard to test). The CI gate
fails on coverage regression.

## Mutation Testing

Use `cargo-mutants` to verify test quality. Target: **>80% mutants
caught**.

## Performance Tests

Use `criterion` for micro-benchmarks. Use a load-test harness for
end-to-end performance (10k attendance marks, 100k invoice
generation, etc.). Track regressions in CI.

## CI Workflow

```yaml
- cargo fmt --all -- --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- cargo mutants
- cargo bench --workspace
- docker compose -f ci/postgres.yml up -d
- cargo test --workspace --features integration
- docker compose -f ci/postgres.yml down
```

## Test Utilities

The engine ships a `smsengine-test` crate that provides:

- `test_engine()` — a pre-configured engine with in-memory storage.
- `test_tenant(school_id)` — a tenant context.
- `test_clock()` — a frozen clock for deterministic timestamps.
- `assert_events_published!` — assert that specific events were
  emitted.
- `assert_audit_record!` — assert that a specific audit record was
  written.
