# Engineering Execution Standards

These standards apply to every architecture decision, specification, implementation recommendation, generated code sample, repository contract, query abstraction, storage adapter, command, event, and domain service described by the documentation.

SMSengine is production software.

All documentation should assume that the resulting implementation will be deployed in real schools managing:

* students
* guardians
* staff
* attendance
* examinations
* finances
* documents
* compliance records

The generated specifications must be implementable as production-grade Rust software.

---

# Cargo & Workspace Management

Use Cargo as the authoritative build system.

## Build Commands

```bash
cargo build
cargo test
cargo check
cargo clippy
cargo fmt --all
```

---

## Package Management

Always use Cargo for dependency management.

Add dependencies using:

```bash
cargo add <crate> --package <package-name>
```

Never manually edit dependency versions when Cargo can manage them.

Prefer the latest stable ecosystem releases unless a documented compatibility reason exists.

---

## Crate Creation

New crates must be created using:

```bash
cargo new --lib --vcs none crates/<crate-name>
```

Always use:

```text
--vcs none
```

to prevent nested Git repositories.

---

# Workspace Layout

SMSengine is a Cargo workspace.

Root Cargo.toml must remain a virtual workspace.

All crates belong under:

```text
crates/*
```

Examples:

```text
crates/core            (package: smsengine-core)
crates/academic        (package: smsengine-academic)
crates/finance         (package: smsengine-finance)
crates/storage         (package: smsengine-storage)
crates/events          (package: smsengine-events)
```

Avoid dependencies outside the workspace.

Avoid relative path references that escape the workspace.

---

# Commit Attribution

All AI-generated commits must include:

```text
Co-Authored-By: Antigravity <antigravity@google.com>
```

---

# File-Scoped Commands

Use package-scoped commands whenever possible.

| Task   | Command                          |
| ------ | -------------------------------- |
| Build  | cargo build --package <package>  |
| Test   | cargo test --package <package>   |
| Check  | cargo check --package <package>  |
| Clippy | cargo clippy --package <package> |
| Format | cargo fmt --all                  |

---

# Rust Type Safety Requirements

Type safety is a first-class architectural requirement.

Documentation should favor designs that maximize compile-time guarantees.

---

## Panic Safety

Do not use:

```rust
unwrap()
expect()
```

in production execution paths.

Instead:

* propagate errors using `?`
* return Result
* explicitly document invariants when panic is impossible

---

## Dead Code

Do not silence the compiler using:

```rust
#[allow(dead_code)]
```

or:

```rust
_unused
```

Unused code must be:

* removed
* connected
* tracked as follow-up work

---

## Numeric Conversions

Avoid lossy casts.

Forbidden:

```rust
value as u32
```

Preferred:

```rust
u32::try_from(value)?
```

Use:

* TryFrom
* TryInto

with explicit error handling.

---

## Public API Error Handling

Every fallible public API must return Result.

Default:

```rust
anyhow::Result<T>
```

Structured errors:

```rust
thiserror
```

should be used when callers need to match error variants.

---

## Object Safety

All trait objects must be object-safe.

Document and verify object safety through compile tests.

Example:

```rust
let _: Box<dyn NotificationProvider>;
```

should compile successfully.

---

# Testing Philosophy

SMSengine follows Test-Driven Development.

Tests are not optional.

Documentation should define expected behavior clearly enough that tests can be derived directly from specifications.

---

## Test Placement

Integration tests:

```text
tests/
```

Examples:

```text
tests/test_student_admission.rs
tests/test_finance_payment.rs
tests/test_result_publication.rs
```

Unit tests remain alongside implementation.

```rust
#[cfg(test)]
mod tests
```

---

## Real-World Tests Only

Reject trivial tests.

Forbidden:

```rust
assert!(true);
```

Forbidden:

```rust
fn it_works()
```

Tests must validate real-world behavior.

Examples:

* student admission lifecycle
* attendance synchronization
* report card generation
* fee payment processing
* permission enforcement
* event publication
* query filtering
* repository behavior

---

## Error Path Coverage

All specifications should identify failure cases.

Tests must validate:

* malformed input
* missing permissions
* duplicate admissions
* invalid state transitions
* repository failures
* event bus failures
* storage failures
* synchronization conflicts

Error paths are as important as successful execution paths.

---

# Query Layer Testing

The query layer described in:

```text
docs/query_layer.md
```

must have comprehensive test coverage.

Validate:

* filtering
* sorting
* pagination
* aggregation
* domain-specific queries
* repository translation
* query correctness
* compile-time safety

Documentation should include expected query behavior and edge cases.

---

# Rust Ecosystem Conventions

Use explicit imports.

Prefer:

```rust
use crate::domain::student::Student;
```

over ambiguous imports.

Public APIs should expose clear crate-level entry points.

Favor re-exports where they improve consumer ergonomics.

---

# TLS & Cross-Compilation Requirements

Cross-platform compatibility is mandatory.

Prefer:

```text
rustls
```

Avoid:

```text
native-tls
```

For crates such as reqwest:

```toml
default-features = false
features = ["rustls-tls"]
```

This requirement exists to support:

* Linux
* Windows
* macOS
* Android
* ARM64
* Embedded deployments

without platform-specific TLS dependencies.

---

# Architecture Validation Criteria

Every generated specification should be evaluated against the following questions:

1. Is this idiomatic Rust?
2. Is this production-ready?
3. Is this type-safe?
4. Is this testable?
5. Is this storage-agnostic?
6. Is this multi-tenant aware?
7. Is this offline-capable?
8. Is this AI-agent friendly?
9. Is this simple enough to maintain?
10. Does it avoid framework-style complexity?

If any answer is "No", revise the specification.

The final SMSengine documentation should guide implementation toward a production-grade Rust domain engine rather than a framework, application, or enterprise architecture experiment.
