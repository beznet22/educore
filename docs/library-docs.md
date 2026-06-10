# Educore Library Documentation

This document is the consumer-facing entry point. It demonstrates how a
consumer application constructs an engine, plugs in adapters, and drives the
school domain.

## Construction

```rust
use educore::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = Engine::builder()
        .storage(PostgresAdapter::connect(env::var("DATABASE_URL")?).await?)
        .auth(JwtAuthProvider::from_env()?)
        .notify(EmailNotifier::from_env()?)
        .event_bus(InProcessBus::new())
        .clock(SystemClock::new())
        .id_gen(UuidV7Generator::new())
        .build()
        .await?;

    Ok(())
}
```

## Tenant Context

Every command and query runs in a tenant context. The tenant is taken from
the authenticated session.

```rust
let session = engine
    .auth()
    .authenticate("Bearer eyJhbGciOi...").await?;

let tenant = TenantContext::new(session.school_id(), session.user_id());
```

## Calling a Command

```rust
use educore::academic::commands::*;

let student = engine
    .students()
    .admit(AdmitStudentCommand {
        tenant: tenant.clone(),
        admission_no: AdmissionNumber::new("ADM-2026-0001")?,
        first_name: "Ada".into(),
        last_name: "Lovelace".into(),
        date_of_birth: NaiveDate::from_ymd_opt(2010, 12, 10).unwrap(),
        gender: Gender::Female,
        guardian: GuardianSpec {
            full_name: "Anne Isabella Milbanke".into(),
            relation: GuardianRelation::Mother,
            phone: PhoneNumber::parse("+44 20 7946 0958")?,
            email: Some(EmailAddress::parse("guardian@example.com")?),
        },
        class_id: ClassId::new(tenant.school_id()),
        section_id: SectionId::new(tenant.school_id()),
        academic_year: AcademicYear::current(tenant.school_id(), &clock).await?,
    })
    .await?;

println!("Admitted {}", student.full_name());
```

## Querying

Domain records are exposed to the query layer through the
`#[derive(DomainQuery)]` procedural macro, which lives in the
`educore-query-derive` crate. The macro emits a typed `*Field` enum
and a `*QueryBuilder` state struct per aggregate — there is no
hand-written `StudentField` in the consumer codebase, the type is
generated from the struct definition on every compile.

Domain-specific scopes (e.g. `.active()`, `.in_class()`) are
implemented as extension traits in your code, layered on top of the
macro-generated builder. The macro produces a structurally complete
but semantically neutral builder; humans author the vocabulary.

### A typed, scoped query

```rust
use educore::academic::query::*;

let page = engine
    .students()
    .query()
    .active()                          // extension trait: StudentQueryScopes
    .in_class(class_id)                // extension trait
    .order_by(StudentField::LastName)  // macro-generated field enum
    .page(0, 50)
    .await?;

for s in page.items {
    println!("{} {}", s.first_name, s.last_name);
}
```

### Nested relational filters (`where_has`)

Cross-aggregate filters are written as closures over the related
entity's macro-generated builder. The closure compiles into a typed
`HasRelation(relation, Box<QueryNode<RelatedField>>)` AST node; the
storage adapter is responsible for translating that node into the
storage dialect.

```rust
let students = engine
    .students()
    .query()
    .active()
    .where_has(StudentRelation::Parent, |parent_q| {
        parent_q.where_eq(ParentField::BillingStatus, BillingStatus::Active)
    })
    .order_by(StudentField::LastName)
    .page(0, 50)
    .await?;
```

### Strict eager loading (`.with(...)`)

Related fields are never loaded implicitly. `.with(...)` is the only
way to populate them, and the repository must complete all hydration
before returning. Omitting `.with(...)` leaves the field `None` (or
empty for `Vec<T>`). Lazy accessors and async getters on domain
models do not exist.

```rust
let students = engine
    .students()
    .query()
    .active()
    .where_has(StudentRelation::Parent, |parent_q| {
        parent_q.where_eq(ParentField::BillingStatus, BillingStatus::Active)
    })
    .with(StudentRelation::Parent)
    .order_by(StudentField::LastName)
    .page(0, 50)
    .await?;

for s in students {
    if let Some(parent) = &s.parent {
        println!("{} -> {}", s.last_name, parent.last_name);
    }
}
```

## Subscribing to Events

```rust
use educore::events::*;

let mut sub = engine
    .events()
    .subscribe::<StudentAdmitted>()
    .await?;

while let Some(event) = sub.next().await {
    println!("admitted: {:?}", event);
}
```

## Capability Check

```rust
if !engine
    .rbac()
    .has_capability(tenant.user_id(), Capability::StudentAdmit)
    .await?
{
    return Err(DomainError::forbidden("missing capability"));
}
```

## Common Workflows

- `engine.students().admit(cmd).await?` — admit a student.
- `engine.students().promote(cmd).await?` — promote a class-section.
- `engine.attendance().mark(cmd).await?` — mark attendance for a class.
- `engine.assessment().enter_marks(cmd).await?` — enter marks.
- `engine.assessment().publish_result(cmd).await?` — publish results.
- `engine.fees().generate_invoice(cmd).await?` — generate a fees invoice.
- `engine.fees().record_payment(cmd).await?` — record a payment.
- `engine.hr().generate_payroll(cmd).await?` — generate monthly payroll.

## Error Handling

```rust
match engine.students().admit(cmd).await {
    Ok(student) => { /* ... */ }
    Err(DomainError::Validation { field, reason }) => { /* ... */ }
    Err(DomainError::Conflict { entity, reason }) => { /* ... */ }
    Err(DomainError::NotFound { entity, id }) => { /* ... */ }
    Err(DomainError::Forbidden { reason }) => { /* ... */ }
    Err(DomainError::Infrastructure(source)) => { /* ... */ }
}
```

## Lifetimes

Engines are cheap to clone (they are `Arc`-backed). They are intended to live
for the life of the consumer process. There is no global state.

## Sample Programs

A complete `examples/admit_and_enroll.rs` is provided in the workspace that:

- Constructs an engine with an in-memory storage adapter.
- Admits a student.
- Enrolls the student in a class-section.
- Marks attendance for a week.
- Enters mid-term marks.
- Generates and pays a fees invoice.
- Prints the resulting student record.
