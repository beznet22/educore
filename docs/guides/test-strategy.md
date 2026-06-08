# Test Strategy

## Goal

Establish a comprehensive test pyramid for SMScore consumers and
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

The engine ships a `smscore-test` crate that provides:

- `test_engine()` — a pre-configured engine with in-memory storage.
- `test_tenant(school_id)` — a tenant context.
- `test_clock()` — a frozen clock for deterministic timestamps.
- `assert_events_published!` — assert that specific events were
  emitted.
- `assert_audit_record!` — assert that a specific audit record was
  written.
