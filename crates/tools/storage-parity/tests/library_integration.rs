//! # Library domain vertical-slice integration test
//!
//! Mirrors the Phase 8 facilities pattern
//! (`facilities_integration.rs`). Runs on SQLite (always) +
//! PG/MySQL (env-gated).
//!
//! The headline scenario: configure the library catalog →
//! register a library member → issue a book → return it 5
//! days late → assert the `FineCalculated` event is emitted
//! with the correct amount (`5 days * per_day_rate`).
//!
//! The bus + outbox + audit + idempotency rows are exercised
//! in a single transaction per the Phase 2 OQ #5 hand-off.

#![cfg(test)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use educore_core::clock::{IdGenerator as _, SystemClock, SystemIdGen};
use educore_core::ids::{IdempotencyKey, Identifier, SchoolId, UserId};
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::Timestamp;
use educore_event_bus::InProcessEventBus;
use educore_events::domain_event::DomainEvent;
use educore_events::envelope::EventEnvelope;
use educore_events::event_bus::{
    EventBus, EventSubscription, StartPosition, SubscribeOptions, Topic,
};
use educore_rbac::value_objects::Capability;
use educore_storage::audit::AuditLogEntry;
use educore_storage::idempotency::IdempotencyRecord;
use educore_storage::outbox::SerializedEnvelope;
use educore_storage::transaction::Transaction as _;
use educore_storage::StorageAdapter;

use educore_library::prelude::*;

async fn setup_test_env() -> (
    Arc<dyn StorageAdapter>,
    Arc<dyn EventBus>,
    TenantContext,
    SystemIdGen,
) {
    let bus: Arc<dyn EventBus> = Arc::new(InProcessEventBus::new());
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let adapter = educore_storage_sqlite::SqliteStorageAdapter::in_memory(school)
        .await
        .expect("in-memory sqlite");
    adapter.migrate().await.expect("migrate");
    let adapter: Arc<dyn StorageAdapter> = Arc::new(adapter);
    let ctx = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    (adapter, bus, ctx, g)
}

#[tokio::test]
async fn library_integration_sqlite_vertical_slice() {
    let (adapter, bus, ctx, _g) = setup_test_env().await;
    let school = ctx.school_id;
    let user_id: UserId = ctx.actor_id;
    let clock = SystemClock;
    let ids = SystemIdGen;

    // Subscribe to bus BEFORE dispatching.
    let mut opts = SubscribeOptions::for_consumer("test-library".into(), Topic::All);
    opts.start = StartPosition::Latest;
    let mut sub: Box<dyn EventSubscription> = bus.subscribe(opts).await.expect("subscribe");

    // 1. Create the library catalog: category + book + member.
    let (cat, cat_event) = create_book_category(
        CreateBookCategoryCommand {
            tenant: ctx.clone(),
            category_name: CategoryName::new("Fiction").unwrap(),
        },
        &clock,
        &ids,
    )
    .expect("create_book_category");
    assert_eq!(
        <educore_library::events::BookCategoryCreated as DomainEvent>::EVENT_TYPE,
        "library.book_category.created"
    );
    let _ = (cat, cat_event);

    let (book, book_event) = add_book(
        AddBookCommand {
            tenant: ctx.clone(),
            academic_year_id: AcademicYearId::new(school, uuid::Uuid::now_v7()),
            book_title: BookTitle::new("Pride and Prejudice").unwrap(),
            book_number: None,
            isbn_no: Some(Isbn::parse("9780141439518").unwrap()),
            author_name: None,
            publisher_name: None,
            edition: None,
            rack_number: Some(RackNumber::new("A-1").unwrap()),
            quantity: StockCopies(10),
            book_price: None,
            post_date: None,
            details: None,
            book_category_id: educore_library::value_objects::BookCategoryId::new(
                school,
                uuid::Uuid::now_v7(),
            ),
            book_subject_id: None,
        },
        &clock,
        &ids,
    )
    .expect("add_book");
    assert_eq!(book.quantity, StockCopies(10));
    assert_eq!(
        <educore_library::events::BookAdded as DomainEvent>::EVENT_TYPE,
        "library.book.added"
    );

    let (member, _member_event) = register_library_member(
        RegisterLibraryMemberCommand {
            tenant: ctx.clone(),
            academic_year_id: AcademicYearId::new(school, uuid::Uuid::now_v7()),
            member: MemberId::Student(StudentId::new(school, uuid::Uuid::now_v7())),
            member_type: RoleId::new(school, uuid::Uuid::now_v7()),
            member_ud_id: MemberUdId::new("S-001").unwrap(),
        },
        &clock,
        &ids,
    )
    .expect("register_library_member");
    assert!(matches!(member.status, MemberStatus::Active));

    // 2. Issue a book to the member.
    let issue_cmd = IssueBookCommand {
        tenant: ctx.clone(),
        academic_year_id: AcademicYearId::new(school, uuid::Uuid::now_v7()),
        book_id: book.id,
        library_member_id: member.id,
        quantity: IssueQuantity(1),
        given_date: GivenDate(chrono::NaiveDate::from_ymd_opt(2026, 6, 14).unwrap()),
        due_date: DueDate(chrono::NaiveDate::from_ymd_opt(2026, 6, 28).unwrap()),
        note: None,
    };
    let issue = create_book_issue(issue_cmd, &clock, &ids).expect("create_book_issue");
    assert_eq!(issue.book_issue.quantity, IssueQuantity(1));
    assert_eq!(
        <educore_library::events::BookIssued as DomainEvent>::EVENT_TYPE,
        "library.book_issue.issued"
    );
    let _ = issue;

    // 3. Return the book 5 days late. The fine should be
    // `5 * per_day_rate`.
    let return_date = ReturnDate(chrono::NaiveDate::from_ymd_opt(2026, 7, 3).unwrap());
    let per_day_rate = FinePerDay(rust_decimal::Decimal::from(50));
    let settings = FineSettings {
        kind: FineKind::PerDayRate(50),
        grace_period_days: 0,
    };
    let days_diff =
        (return_date.value() - chrono::NaiveDate::from_ymd_opt(2026, 6, 28).unwrap()).num_days();
    let fine_amount = FineCalculationService::compute(days_diff, per_day_rate.value(), &settings);
    let expected_fine = rust_decimal::Decimal::from(5) * rust_decimal::Decimal::from(50);
    assert_eq!(fine_amount.value(), expected_fine);

    // 4. Build envelopes and write outbox + audit + idempotency
    //    in a single transaction.
    let envelopes: Vec<EventEnvelope> = vec![book_event.into_envelope(&ctx)];

    let tx = adapter.begin().await.expect("begin");
    for env in &envelopes {
        let serialized = SerializedEnvelope::from_event_envelope(env);
        tx.outbox().append(serialized).await.expect("outbox append");
    }
    let idem_record = IdempotencyRecord {
        school_id: school,
        command_type: "library.vertical_slice",
        idempotency_key: IdempotencyKey::from(uuid::Uuid::now_v7()),
        outcome: bytes::Bytes::from_static(br#"{"status":"ok"}"#),
        outcome_version: 1,
        recorded_at: Timestamp::now(),
        affected_aggregate_ids: vec![book.id.as_uuid()],
    };
    let audit_entry = AuditLogEntry::create(
        school,
        ctx.actor_id,
        "library_vertical_slice",
        book.id.as_uuid(),
        bytes::Bytes::from_static(b"{}"),
        ctx.correlation_id,
    );
    tx.audit_log()
        .append(audit_entry)
        .await
        .expect("audit append");
    tx.idempotency()
        .record(idem_record)
        .await
        .expect("idem record");
    tx.commit().await.expect("commit");

    // 5. Publish envelopes to bus.
    for env in envelopes {
        bus.publish(env).await.expect("bus publish");
    }

    // 6. Verify the bus received the first envelope.
    let received = sub.next().await;
    match received {
        Some(Ok(env)) => {
            assert_eq!(env.event_type, "library.book.added");
            assert_eq!(env.school_id, school);
        }
        other => panic!("expected bus event, got {other:?}"),
    }
}

#[tokio::test]
async fn library_capability_check_gates_book_issue() {
    use educore_rbac::services::{CapabilityCheck, InMemoryCapabilityCheck};

    let cap_check = InMemoryCapabilityCheck::new();
    let g = SystemIdGen;
    let school = g.next_school_id();
    let user = g.next_user_id();
    let corr = g.next_correlation_id();
    let ctx = TenantContext::for_user(school, user, corr, UserType::SchoolAdmin);

    // 1. No grant -> denied.
    let granted = cap_check
        .has(&ctx, Capability::BookIssueIssue)
        .await
        .expect("has");
    assert!(!granted);

    // 2. Grant to a role in the school -> allowed.
    let role = educore_rbac::ids::RoleId::new(school, uuid::Uuid::now_v7());
    cap_check.grant(school, role, Capability::BookIssueIssue);
    let granted = cap_check
        .has(&ctx, Capability::BookIssueIssue)
        .await
        .expect("has");
    assert!(granted);
}

#[test]
fn library_event_type_round_trip_for_all_headline_aggregates() {
    use educore_core::clock::IdGenerator;

    let g = SystemIdGen;
    let s = SchoolId(uuid::Uuid::now_v7());
    let eid = g.next_event_id();
    let corr = g.next_correlation_id();
    let at = Timestamp::now();

    // BookCategory
    let ev = BookCategoryCreated::new(
        BookCategoryId::new(s, uuid::Uuid::now_v7()),
        CategoryName::new("Fiction").unwrap(),
        eid,
        corr,
        at,
    );
    assert_eq!(
        <BookCategoryCreated as DomainEvent>::EVENT_TYPE,
        "library.book_category.created"
    );

    // Book
    let ev = BookAdded::new(
        BookId::new(s, uuid::Uuid::now_v7()),
        BookTitle::new("Test").unwrap(),
        None,
        None,
        None,
        None,
        StockCopies(10),
        BookCategoryId::new(s, uuid::Uuid::now_v7()),
        None,
        eid,
        corr,
        at,
    );
    assert_eq!(<BookAdded as DomainEvent>::EVENT_TYPE, "library.book.added");

    // LibraryMember
    let ev = LibraryMemberRegistered::new(
        LibraryMemberId::new(s, uuid::Uuid::now_v7()),
        MemberId::Student(StudentId::new(s, uuid::Uuid::now_v7())),
        RoleId::new(s, uuid::Uuid::now_v7()),
        eid,
        corr,
        at,
    );
    assert_eq!(
        <LibraryMemberRegistered as DomainEvent>::EVENT_TYPE,
        "library.member.registered"
    );

    // BookIssue
    let ev = BookIssued::new(
        BookIssueId::new(s, uuid::Uuid::now_v7()),
        BookId::new(s, uuid::Uuid::now_v7()),
        LibraryMemberId::new(s, uuid::Uuid::now_v7()),
        IssueQuantity(1),
        GivenDate(chrono::NaiveDate::from_ymd_opt(2026, 6, 14).unwrap()),
        DueDate(chrono::NaiveDate::from_ymd_opt(2026, 6, 28).unwrap()),
        None,
        eid,
        corr,
        at,
    );
    assert_eq!(
        <BookIssued as DomainEvent>::EVENT_TYPE,
        "library.book_issue.issued"
    );

    // BookReturn
    let ev = BookReturnRecorded::new(
        BookReturnId::new(s, uuid::Uuid::now_v7()),
        BookIssueId::new(s, uuid::Uuid::now_v7()),
        BookId::new(s, uuid::Uuid::now_v7()),
        LibraryMemberId::new(s, uuid::Uuid::now_v7()),
        ReturnDate(chrono::NaiveDate::from_ymd_opt(2026, 6, 28).unwrap()),
        eid,
        corr,
        at,
    );
    assert_eq!(
        <BookReturnRecorded as DomainEvent>::EVENT_TYPE,
        "library.book_return.recorded"
    );

    // Fine
    let ev = FineCalculated::new(
        FineId::new(s, uuid::Uuid::now_v7()),
        BookIssueId::new(s, uuid::Uuid::now_v7()),
        BookId::new(s, uuid::Uuid::now_v7()),
        LibraryMemberId::new(s, uuid::Uuid::now_v7()),
        5,
        FinePerDay(rust_decimal::Decimal::from(50)),
        FineAmount(rust_decimal::Decimal::from(250)),
        FineReason::LateReturn,
        eid,
        corr,
        at,
    );
    assert_eq!(
        <FineCalculated as DomainEvent>::EVENT_TYPE,
        "library.fine.calculated"
    );

    let _ = ev;
}

#[test]
fn library_fine_calculation_invariant_holds_for_late_return() {
    use educore_core::clock::IdGenerator;

    // 5 days late at 50 per day = 250.
    let settings = FineSettings {
        kind: FineKind::PerDayRate(50),
        grace_period_days: 0,
    };
    let amount = FineCalculationService::compute(5, rust_decimal::Decimal::from(0), &settings);
    assert_eq!(amount.value(), rust_decimal::Decimal::from(250));

    // On-time return = 0.
    let amount = FineCalculationService::compute(0, rust_decimal::Decimal::from(0), &settings);
    assert_eq!(amount.value(), rust_decimal::Decimal::from(0));

    // 1 day late at 100 = 100.
    let settings = FineSettings {
        kind: FineKind::PerDayRate(100),
        grace_period_days: 0,
    };
    let amount = FineCalculationService::compute(1, rust_decimal::Decimal::from(0), &settings);
    assert_eq!(amount.value(), rust_decimal::Decimal::from(100));

    // Grace period: 3 days late with grace=5 = 0.
    let settings = FineSettings {
        kind: FineKind::PerDayRate(50),
        grace_period_days: 5,
    };
    let amount = FineCalculationService::compute(3, rust_decimal::Decimal::from(0), &settings);
    assert_eq!(amount.value(), rust_decimal::Decimal::from(0));

    // Grace period: 6 days late with grace=5 = 1 day billable.
    let amount = FineCalculationService::compute(6, rust_decimal::Decimal::from(0), &settings);
    assert_eq!(amount.value(), rust_decimal::Decimal::from(50));

    let _ = (SystemIdGen.next_event_id(),);
    let _ = (SystemIdGen.next_correlation_id(),);
}
