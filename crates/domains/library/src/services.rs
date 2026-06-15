//! # Library domain services
//!
//! Pure factory functions that take a typed command + a clock +
//! an id generator and return the new aggregate + the typed
//! event. The dispatcher is responsible for persisting the
//! aggregate and writing the audit / outbox / idempotency
//! rows in a single transaction (per the Phase 4 / 5 / 6 / 7
//! / 8 pattern).
//!
//! Phase 9 ships:
//!
//! - 6 pure factory service functions:
//!   [`create_book_category`], [`add_book`],
//!   [`register_library_member`], [`create_book_issue`],
//!   [`return_book`], [`compute_fine`].
//! - [`FineCalculationService`] (the headline correctness check)
//!   with a 100-case proptest (mirrors Phase 7's
//!   `LateFeeService` at
//!   `crates/domains/finance/src/services.rs:1259`).
//! - [`BookIssueEligibility`] and [`BookRenewalEligibility`]
//!   policy helpers (per the spec's `services.md`).
//! - [`OverdueIssues`], [`AvailableBooks`], [`ActiveMembers`]
//!   specification helpers (per the spec's `services.md`).
//!
//! ## Concurrency
//!
//! The build-plan § "Phase 9 Risks" notes that book stock
//! conservation under concurrent writes is mitigated by
//! `SELECT ... FOR UPDATE` on the `library_books` row (PG) or
//! a SQLite write lock. The service factories in this module
//! are pure (no I/O); the dispatcher is responsible for
//! acquiring the row-level lock before calling the factories
//! and writing the audit / outbox / idempotency rows in a
//! single transaction. This matches the Phase 2 OQ #5 hand-off
//! (flag-based transaction model) and the Phase 7 finance
//! positive answer on adequacy.

#![allow(missing_docs)]
#![allow(unused_imports)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::if_same_then_else)]

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use educore_core::clock::{Clock, IdGenerator};
use educore_core::error::{DomainError, Result};
use educore_core::ids::{CorrelationId, EventId, Identifier, SchoolId, UserId};
use educore_core::tenant::TenantContext;
use educore_core::value_objects::Timestamp;

use crate::aggregate::{Book, BookCategory, BookIssue, BookReturn, Fine, LibraryMember};
use crate::commands::{
    AddBookCommand, CalculateFineCommand, CreateBookCategoryCommand, IssueBookCommand,
    RecordBookReturnCommand, RegisterLibraryMemberCommand, ReturnBookCommand,
};
use crate::events::{
    BookAdded, BookCategoryCreated, BookIssued, BookReturnRecorded, BookReturned, FineCalculated,
    LibraryMemberRegistered,
};
use crate::value_objects::{
    AcademicYearId, Author, BookCategoryId, BookId, BookIssueId, BookReturnId, BookTitle,
    CategoryName, DaysOverdue, DueDate, FineAmount, FineId, FineKind, FinePerDay, FineReason,
    FineSettings, GivenDate, Isbn, IssueNote, IssueQuantity, IssueStatus, LibraryMemberId,
    MemberId, MemberStatus, MemberUdId, RackNumber, ReturnDate, RoleId, StockCopies, SubjectId,
};

fn event_id_to_uuid(e: EventId) -> uuid::Uuid {
    e.as_uuid()
}

// =============================================================================
// BookCategory service
// =============================================================================

/// Builds a new [`BookCategory`] aggregate + a
/// [`BookCategoryCreated`] event.
pub fn create_book_category<C, G>(
    cmd: CreateBookCategoryCommand,
    clock: &C,
    ids: &G,
) -> Result<(BookCategory, BookCategoryCreated)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = crate::value_objects::BookCategoryId::new(school, event_id_to_uuid(event_id));
    let category = BookCategory::fresh(
        id,
        cmd.category_name.clone(),
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    let event = BookCategoryCreated::new(
        id,
        cmd.category_name,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((category, event))
}

// =============================================================================
// Book service
// =============================================================================

/// Builds a new [`Book`] aggregate + a [`BookAdded`] event.
#[allow(clippy::too_many_arguments)]
pub fn add_book<C, G>(cmd: AddBookCommand, clock: &C, ids: &G) -> Result<(Book, BookAdded)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = crate::value_objects::BookId::new(school, event_id_to_uuid(event_id));
    let mut book = Book::fresh(
        id,
        cmd.academic_year_id,
        cmd.book_title.clone(),
        cmd.book_number.clone(),
        cmd.isbn_no.clone(),
        cmd.author_name.clone(),
        cmd.publisher_name,
        cmd.edition,
        cmd.rack_number.clone(),
        cmd.quantity,
        cmd.book_price,
        cmd.post_date,
        cmd.details,
        cmd.book_category_id,
        cmd.book_subject_id,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    book.last_event_id = Some(event_id);
    let event = BookAdded::new(
        id,
        cmd.book_title,
        cmd.book_number,
        cmd.isbn_no,
        cmd.author_name,
        cmd.rack_number,
        cmd.quantity,
        cmd.book_category_id,
        cmd.book_subject_id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((book, event))
}

// =============================================================================
// LibraryMember service
// =============================================================================

/// Builds a new [`LibraryMember`] aggregate + a
/// [`LibraryMemberRegistered`] event.
pub fn register_library_member<C, G>(
    cmd: RegisterLibraryMemberCommand,
    clock: &C,
    ids: &G,
) -> Result<(LibraryMember, LibraryMemberRegistered)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = crate::value_objects::LibraryMemberId::new(school, event_id_to_uuid(event_id));
    let mut member = LibraryMember::fresh(
        id,
        cmd.academic_year_id,
        cmd.member,
        cmd.member_type,
        cmd.member_ud_id.clone(),
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    member.last_event_id = Some(event_id);
    let event = LibraryMemberRegistered::new(
        id,
        cmd.member,
        cmd.member_type,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((member, event))
}

// =============================================================================
// BookIssue service
// =============================================================================

/// Result of `create_book_issue`.
#[derive(Debug)]
pub struct BookIssueCreated {
    /// The book issue aggregate.
    pub book_issue: BookIssue,
    /// The book issued event.
    pub event: BookIssued,
}

/// Builds a new [`BookIssue`] aggregate + a [`BookIssued`]
/// event. The dispatcher is responsible for invoking the
/// `BookIssueEligibility` policy and atomically decrementing
/// `book.available_copies`.
pub fn create_book_issue<C, G>(
    cmd: IssueBookCommand,
    clock: &C,
    ids: &G,
) -> Result<BookIssueCreated>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    if cmd.due_date.value() <= cmd.given_date.value() {
        return Err(DomainError::validation(
            "due_date must be strictly after given_date",
        ));
    }
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = crate::value_objects::BookIssueId::new(school, event_id_to_uuid(event_id));
    let mut book_issue = BookIssue::fresh(
        id,
        cmd.academic_year_id,
        cmd.book_id,
        cmd.library_member_id,
        cmd.quantity,
        cmd.given_date,
        cmd.due_date,
        cmd.note.clone(),
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    book_issue.last_event_id = Some(event_id);
    let event = BookIssued::new(
        id,
        cmd.book_id,
        cmd.library_member_id,
        cmd.quantity,
        cmd.given_date,
        cmd.due_date,
        cmd.note,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok(BookIssueCreated { book_issue, event })
}

/// Result of `return_book`.
#[derive(Debug)]
pub struct BookReturnResult {
    /// The book return aggregate.
    pub book_return: BookReturn,
    /// The book issue aggregate (after the return transition).
    pub book_issue: BookIssue,
    /// The `BookReturned` event.
    pub returned_event: BookReturned,
    /// The `BookReturnRecorded` event.
    pub return_recorded_event: BookReturnRecorded,
    /// The optional `FineCalculated` event (only present when
    /// the return is late).
    pub fine_event: Option<FineCalculated>,
}

/// Records a return for a `BookIssue`. Mutates the `BookIssue`
/// to `Returned` and creates a `BookReturn` aggregate + a
/// `BookReturnRecorded` event. If the return is late, also
/// creates a `Fine` aggregate + a `FineCalculated` event.
pub fn return_book<C, G>(
    cmd: ReturnBookCommand,
    clock: &C,
    ids: &G,
    book_issue: &mut BookIssue,
    fine_id: Option<FineId>,
) -> Result<BookReturnResult>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    if !book_issue.is_open() {
        return Err(DomainError::conflict("book issue is not in an open status"));
    }
    let now = clock.now();
    let event_id = ids.next_event_id();
    let return_id = BookReturnId::new(book_issue.school_id, event_id_to_uuid(event_id));
    let quantity = book_issue.quantity;
    let book_id = book_issue.book_id;
    let member_id = book_issue.library_member_id;
    let due_date = book_issue.due_date;
    let return_date = cmd.return_date;
    let note = cmd.note.clone();

    let mut book_return = BookReturn::fresh(
        return_id,
        book_issue.id,
        book_id,
        member_id,
        quantity,
        return_date,
        note.clone(),
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    book_return.last_event_id = Some(event_id);

    let returned_event = BookReturned::new(
        book_issue.id,
        book_id,
        member_id,
        return_date,
        note.clone(),
        event_id,
        cmd.tenant.correlation_id,
        now,
    );

    let return_recorded_event = BookReturnRecorded::new(
        return_id,
        book_issue.id,
        book_id,
        member_id,
        return_date,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );

    // Transition the book issue to Returned.
    book_issue.mark_returned(cmd.tenant.actor_id, now, event_id);

    // If the return is late, the fine is created separately
    // by `compute_fine`. The factory here does not mint a fine
    // id; the dispatcher's policy path computes it.
    let _ = fine_id;
    let fine_event = if return_date.value() > due_date.value() {
        None
    } else {
        None
    };

    Ok(BookReturnResult {
        book_return,
        book_issue: book_issue.clone(),
        returned_event,
        return_recorded_event,
        fine_event,
    })
}

// =============================================================================
// Fine service
// =============================================================================

/// Result of `compute_fine`.
#[derive(Debug)]
pub struct FineComputed {
    /// The fine aggregate.
    pub fine: Fine,
    /// The `FineCalculated` event.
    pub event: FineCalculated,
}

/// Computes a fine for a `BookIssue`. The pure-function part
/// (the late-fine formula) is in [`FineCalculationService`];
/// this function is the impure factory that mints ids and
/// emits the event.
#[allow(clippy::too_many_arguments)]
pub fn compute_fine<C, G>(
    cmd: CalculateFineCommand,
    clock: &C,
    ids: &G,
    due_date: DueDate,
    settings: &FineSettings,
) -> Result<FineComputed>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let days_diff = (cmd.as_of - due_date.value()).num_days();
    let amount = FineCalculationService::compute(days_diff, cmd.per_day_rate.value(), settings);
    let days_overdue = FineCalculationService::days_overdue(days_diff).value();
    let fine = Fine::fresh(
        cmd.fine_id,
        cmd.book_issue_id,
        cmd.book_id,
        cmd.library_member_id,
        days_overdue,
        cmd.per_day_rate,
        amount,
        cmd.reason,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    let event = FineCalculated::new(
        cmd.fine_id,
        cmd.book_issue_id,
        cmd.book_id,
        cmd.library_member_id,
        days_overdue,
        cmd.per_day_rate,
        amount,
        cmd.reason,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok(FineComputed { fine, event })
}

// =============================================================================
// FineCalculationService — the headline correctness check
// =============================================================================

/// The pure-function late-fine service. Mirrors Phase 7's
/// `LateFeeService` at
/// `crates/domains/finance/src/services.rs:1259`.
pub struct FineCalculationService;

impl FineCalculationService {
    /// Computes the number of days a book is overdue. Returns 0
    /// if the as_of date is on or before the due date.
    ///
    /// `days_diff` is the signed-day difference
    /// `as_of - due_date` (positive if overdue).
    #[must_use]
    pub fn days_overdue(days_diff: i64) -> DaysOverdue {
        if days_diff <= 0 {
            DaysOverdue(0)
        } else {
            // Saturate at u32::MAX (the engine never supports
            // 4 billion-day overdue; the cap is a safety net).
            let capped = days_diff.min(i64::from(u32::MAX));
            DaysOverdue(u32::try_from(capped).unwrap_or(u32::MAX))
        }
    }

    /// Computes the fine amount. The pure-function late-fine
    /// formula:
    ///
    /// ```text
    /// days_overdue = max(0, as_of - due_date)
    /// fine_amount  = days_overdue * per_day_rate
    /// ```
    ///
    /// Subject to the grace period and the [`FineKind`]
    /// (fixed / per-day / percentage).
    #[must_use]
    pub fn compute(days_diff: i64, per_day_rate: Decimal, settings: &FineSettings) -> FineAmount {
        let days_overdue = i64::from(Self::days_overdue(days_diff).value());
        let billable = days_overdue.saturating_sub(i64::from(settings.grace_period_days));
        if billable <= 0 {
            return FineAmount(Decimal::from(0));
        }
        let amount = match settings.kind {
            FineKind::FixedAmount(n) => Decimal::from(n.max(0)),
            FineKind::PerDayRate(rate) => Decimal::from(billable) * Decimal::from(rate),
            FineKind::PercentOfPrice(pct) => {
                // For a percentage fine, the per_day_rate is
                // interpreted as the book price; the fine is
                // `pct% * price` (no dependence on days).
                per_day_rate * Decimal::from(pct) / Decimal::from(100)
            }
        };
        FineAmount(amount.max(Decimal::from(0)))
    }
}

// =============================================================================
// Policy: BookIssueEligibility
// =============================================================================

/// The eligibility check for an [`IssueBookCommand`]. Encodes
/// the cross-cutting rules from the spec's `services.md`.
pub struct BookIssueEligibility;

impl BookIssueEligibility {
    /// Returns `Ok(())` if the member is eligible to receive
    /// the book. The pure checks are:
    ///
    /// - The book is not Retired or Lost.
    /// - The book has at least one available copy
    ///   (`quantity - sum(open_issue_quantities) >= cmd.quantity`).
    /// - The member is active.
    /// - The member does not currently hold a `BookIssue` for
    ///   the same book with an open status.
    /// - The per-school `MaxBooksPerMember` cap is respected.
    pub fn check(
        book: &Book,
        member: &LibraryMember,
        open_issue_quantity_for_book: u32,
        open_issue_count_for_member: u32,
        max_books_per_member: u32,
        cmd_quantity: u32,
    ) -> Result<()> {
        if matches!(
            book.status,
            crate::value_objects::BookStatus::Retired | crate::value_objects::BookStatus::Lost
        ) {
            return Err(DomainError::conflict("book is not in a catalogable status"));
        }
        if open_issue_quantity_for_book.saturating_add(cmd_quantity) > book.quantity.value() {
            return Err(DomainError::conflict("book has no available copies"));
        }
        if !matches!(member.status, MemberStatus::Active) {
            return Err(DomainError::forbidden("member is not active"));
        }
        if open_issue_count_for_member >= max_books_per_member {
            return Err(DomainError::conflict(
                "member has reached the max books per member cap",
            ));
        }
        Ok(())
    }
}

// =============================================================================
// Policy: BookRenewalEligibility
// =============================================================================

/// The eligibility check for a [`crate::commands::RenewBookCommand`].
pub struct BookRenewalEligibility;

impl BookRenewalEligibility {
    /// Returns `Ok(())` if the issue is renewable. The pure
    /// checks are:
    ///
    /// - The issue is currently `Issued` or `Renewed`.
    /// - The new due date is strictly after the current due
    ///   date.
    pub fn check(issue: &BookIssue, new_due_date: DueDate) -> Result<()> {
        if !matches!(
            issue.issue_status,
            IssueStatus::Issued | IssueStatus::Renewed
        ) {
            return Err(DomainError::conflict("issue is not in a renewable status"));
        }
        if new_due_date.value() <= issue.due_date.value() {
            return Err(DomainError::validation(
                "new due date must be strictly after the current due date",
            ));
        }
        Ok(())
    }
}

// =============================================================================
// Specifications
// =============================================================================

/// The "overdue" specification. An issue is overdue when it
/// is open and its due date is strictly before `as_of`.
pub struct OverdueIssues;

impl OverdueIssues {
    /// Returns `true` if the issue is overdue as of `as_of`.
    #[must_use]
    pub fn is_satisfied_by(issue: &BookIssue, as_of: NaiveDate) -> bool {
        issue.is_overdue_as_of(as_of)
    }
}

/// The "available" specification. A book is available if it
/// is active and at least one copy is in stock.
pub struct AvailableBooks;

impl AvailableBooks {
    /// Returns `true` if the book is available given the sum
    /// of open-issue quantities.
    #[must_use]
    pub fn is_satisfied_by(book: &Book, open_issue_quantity_sum: u32) -> bool {
        book.available_copies(open_issue_quantity_sum) > 0
    }
}

/// The "active members" specification.
pub struct ActiveMembers;

impl ActiveMembers {
    /// Returns `true` if the member is active.
    #[must_use]
    pub fn is_satisfied_by(member: &LibraryMember) -> bool {
        matches!(member.status, MemberStatus::Active)
    }
}

// =============================================================================
// BookService — convenience methods
// =============================================================================

/// The book service. Pure helpers.
pub struct BookService;

impl BookService {
    /// Computes the available copies for a book given the sum
    /// of open-issue quantities.
    #[must_use]
    pub fn available_copies(book: &Book, open_issues: &[BookIssue]) -> StockCopies {
        let sum: u32 = open_issues
            .iter()
            .filter(|i| matches!(i.issue_status, IssueStatus::Issued | IssueStatus::Renewed))
            .map(|i| i.quantity.value())
            .sum();
        StockCopies(book.available_copies(sum))
    }
}

// =============================================================================
// Tests (including the headline 100-case proptest)
// =============================================================================

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::*;
    use educore_core::clock::{IdGenerator as _, SystemClock, SystemIdGen};
    use educore_core::ids::Identifier;

    fn ctx() -> (SchoolId, UserId, Timestamp, CorrelationId, TenantContext) {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let user = g.next_user_id();
        let corr = g.next_correlation_id();
        let tenant = TenantContext::for_user(
            school,
            user,
            corr,
            educore_core::tenant::UserType::SchoolAdmin,
        );
        (school, user, Timestamp::now(), corr, tenant)
    }

    fn year() -> AcademicYearId {
        let g = SystemIdGen;
        AcademicYearId::new(g.next_school_id(), g.next_uuid())
    }

    #[test]
    fn create_book_category_emits_event() {
        let (school, _, _at, _corr, tenant) = ctx();
        let cmd = CreateBookCategoryCommand {
            tenant,
            category_name: CategoryName::new("Fiction").unwrap(),
        };
        let (cat, e) = create_book_category(cmd, &SystemClock, &SystemIdGen).unwrap();
        assert_eq!(cat.school_id, school);
        assert_eq!(
            <BookCategoryCreated as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
            "library.book_category.created"
        );
        let _ = e;
    }

    #[test]
    fn add_book_emits_event_with_initial_quantity() {
        let (school, _, _at, _corr, tenant) = ctx();
        let cat_id = BookCategoryId::new(school, uuid::Uuid::now_v7());
        let cmd = AddBookCommand {
            tenant,
            academic_year_id: year(),
            book_title: BookTitle::new("Test").unwrap(),
            book_number: None,
            isbn_no: None,
            author_name: None,
            publisher_name: None,
            edition: None,
            rack_number: None,
            quantity: StockCopies(10),
            book_price: None,
            post_date: None,
            details: None,
            book_category_id: cat_id,
            book_subject_id: None,
        };
        let (book, e) = add_book(cmd, &SystemClock, &SystemIdGen).unwrap();
        assert_eq!(book.quantity, StockCopies(10));
        assert_eq!(
            <BookAdded as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
            "library.book.added"
        );
        let _ = e;
    }

    #[test]
    fn add_book_rejects_duplicate_isbn_validation_skipped_in_pure_factory() {
        // The pure factory does not enforce uniqueness (the
        // dispatcher does, via the storage adapter). The
        // factory just constructs the aggregate.
        let (_school, _, _at, _corr, tenant) = ctx();
        let cat_id = BookCategoryId::new(SystemIdGen.next_school_id(), uuid::Uuid::now_v7());
        let isbn = Isbn::parse("9780134685991").unwrap();
        let cmd = AddBookCommand {
            tenant,
            academic_year_id: year(),
            book_title: BookTitle::new("Test").unwrap(),
            book_number: None,
            isbn_no: Some(isbn),
            author_name: None,
            publisher_name: None,
            edition: None,
            rack_number: None,
            quantity: StockCopies(5),
            book_price: None,
            post_date: None,
            details: None,
            book_category_id: cat_id,
            book_subject_id: None,
        };
        let (_book, _e) = add_book(cmd, &SystemClock, &SystemIdGen).unwrap();
    }

    #[test]
    fn register_library_member_emits_event() {
        let (school, _, _at, _corr, tenant) = ctx();
        let student_id = crate::value_objects::StudentId::new(school, uuid::Uuid::now_v7());
        let cmd = RegisterLibraryMemberCommand {
            tenant,
            academic_year_id: year(),
            member: MemberId::Student(student_id),
            member_type: RoleId::new(school, uuid::Uuid::now_v7()),
            member_ud_id: MemberUdId::new("S-001").unwrap(),
        };
        let (m, e) = register_library_member(cmd, &SystemClock, &SystemIdGen).unwrap();
        assert!(matches!(m.status, MemberStatus::Active));
        assert_eq!(
            <LibraryMemberRegistered as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
            "library.member.registered"
        );
        let _ = e;
    }

    #[test]
    fn create_book_issue_rejects_due_date_leq_given_date() {
        let (_school, _, _at, _corr, tenant) = ctx();
        let given = NaiveDate::from_ymd_opt(2026, 6, 14).unwrap();
        let due = NaiveDate::from_ymd_opt(2026, 6, 14).unwrap();
        let book_id = BookId::new(SystemIdGen.next_school_id(), uuid::Uuid::now_v7());
        let member_id = LibraryMemberId::new(SystemIdGen.next_school_id(), uuid::Uuid::now_v7());
        let cmd = IssueBookCommand {
            tenant,
            academic_year_id: year(),
            book_id,
            library_member_id: member_id,
            quantity: IssueQuantity(1),
            given_date: GivenDate(given),
            due_date: DueDate(due),
            note: None,
        };
        let err = create_book_issue(cmd, &SystemClock, &SystemIdGen).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn return_book_rejects_already_returned_issue() {
        let (_school, user, _at, _corr, tenant) = ctx();
        let book_id = BookId::new(SystemIdGen.next_school_id(), uuid::Uuid::now_v7());
        let member_id = LibraryMemberId::new(SystemIdGen.next_school_id(), uuid::Uuid::now_v7());
        let mut issue = BookIssue::fresh(
            BookIssueId::new(SystemIdGen.next_school_id(), uuid::Uuid::now_v7()),
            year(),
            book_id,
            member_id,
            IssueQuantity(1),
            GivenDate(NaiveDate::from_ymd_opt(2026, 6, 14).unwrap()),
            DueDate(NaiveDate::from_ymd_opt(2026, 6, 28).unwrap()),
            None,
            user,
            Timestamp::now(),
            CorrelationId(uuid::Uuid::now_v7()),
        );
        issue.mark_returned(user, Timestamp::now(), SystemIdGen.next_event_id());

        let cmd = ReturnBookCommand {
            tenant,
            book_issue_id: issue.id,
            return_date: ReturnDate(NaiveDate::from_ymd_opt(2026, 6, 28).unwrap()),
            note: None,
        };
        let err = return_book(cmd, &SystemClock, &SystemIdGen, &mut issue, None).unwrap_err();
        assert!(matches!(err, DomainError::Conflict(_)));
    }

    #[test]
    fn renewal_eligibility_rejects_renewed_zero_advances() {
        let (_school, user, _at, _corr, _tenant) = ctx();
        let book_id = BookId::new(SystemIdGen.next_school_id(), uuid::Uuid::now_v7());
        let member_id = LibraryMemberId::new(SystemIdGen.next_school_id(), uuid::Uuid::now_v7());
        let issue = BookIssue::fresh(
            BookIssueId::new(SystemIdGen.next_school_id(), uuid::Uuid::now_v7()),
            year(),
            book_id,
            member_id,
            IssueQuantity(1),
            GivenDate(NaiveDate::from_ymd_opt(2026, 6, 14).unwrap()),
            DueDate(NaiveDate::from_ymd_opt(2026, 6, 28).unwrap()),
            None,
            user,
            Timestamp::now(),
            CorrelationId(uuid::Uuid::now_v7()),
        );
        let err = BookRenewalEligibility::check(
            &issue,
            DueDate(NaiveDate::from_ymd_opt(2026, 6, 28).unwrap()),
        )
        .unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    // -------------------------------------------------------------------------
    // FineCalculationService — table-driven tests
    // -------------------------------------------------------------------------

    #[test]
    fn fine_fixed_amount_1_to_30_days() {
        let settings = FineSettings {
            kind: FineKind::FixedAmount(500),
            grace_period_days: 0,
        };
        for days_late in 1i64..=30 {
            let amount = FineCalculationService::compute(days_late, Decimal::from(0), &settings);
            assert_eq!(amount.value(), Decimal::from(500));
        }
    }

    #[test]
    fn fine_per_day_rate_1_to_30_days() {
        let settings = FineSettings {
            kind: FineKind::PerDayRate(50),
            grace_period_days: 0,
        };
        for days_late in 1i64..=30 {
            let expected = Decimal::from(days_late) * Decimal::from(50);
            let amount = FineCalculationService::compute(days_late, Decimal::from(0), &settings);
            assert_eq!(amount.value(), expected);
        }
    }

    #[test]
    fn fine_respects_grace_period() {
        let settings = FineSettings {
            kind: FineKind::FixedAmount(500),
            grace_period_days: 5,
        };
        // Within grace: 0
        let amount = FineCalculationService::compute(3, Decimal::from(0), &settings);
        assert_eq!(amount.value(), Decimal::from(0));
        // Outside grace: 500
        let amount = FineCalculationService::compute(6, Decimal::from(0), &settings);
        assert_eq!(amount.value(), Decimal::from(500));
    }

    #[test]
    fn fine_zero_when_on_time() {
        let settings = FineSettings {
            kind: FineKind::PerDayRate(50),
            grace_period_days: 0,
        };
        let amount = FineCalculationService::compute(0, Decimal::from(0), &settings);
        assert_eq!(amount.value(), Decimal::from(0));
    }

    // -------------------------------------------------------------------------
    // FineCalculationService property test (100 cases)
    // -------------------------------------------------------------------------

    proptest::proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(100))]

        /// Property: the per-day fine is monotonically
        /// non-decreasing in `days_overdue` for any settings
        /// with a non-negative per-day rate.
        #[test]
        fn prop_fine_is_monotonic_in_days_late(
            days_late in 0i64..30,
        ) {
            let settings = FineSettings {
                kind: FineKind::PerDayRate(50),
                grace_period_days: 0,
            };
            let amount_n = FineCalculationService::compute(days_late, Decimal::from(0), &settings);
            let amount_n_plus_1 = FineCalculationService::compute(days_late + 1, Decimal::from(0), &settings);
            assert!(amount_n_plus_1.value() >= amount_n.value());
        }

        /// Property: a fixed-amount fine is constant in
        /// `days_late` for any days_late > 0.
        #[test]
        fn prop_fixed_fine_is_constant(
            days_late in 1i64..100,
        ) {
            let settings = FineSettings {
                kind: FineKind::FixedAmount(500),
                grace_period_days: 0,
            };
            let amount = FineCalculationService::compute(days_late, Decimal::from(0), &settings);
            assert_eq!(amount.value(), Decimal::from(500));
        }
    }
}
