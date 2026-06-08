# Library Domain — Events

Domain events describe facts that have already happened. They
are immutable, append-only records used for cross-domain
integration, audit, and event sourcing.

All events implement:

```rust
pub trait DomainEvent: Serialize + DeserializeOwned + Send + Sync {
    const TYPE: &'static str;
    fn aggregate_id(&self) -> Uuid;
    fn school_id(&self) -> SchoolId;
    fn occurred_at(&self) -> Timestamp;
}
```

The event envelope wraps the event with metadata:

```rust
pub struct EventEnvelope<E> {
    pub event_id: EventId,
    pub event_type: &'static str,
    pub school_id: SchoolId,
    pub aggregate_id: Uuid,
    pub aggregate_type: &'static str,
    pub actor_id: UserId,
    pub correlation_id: CorrelationId,
    pub causation_id: Option<EventId>,
    pub occurred_at: Timestamp,
    pub payload: E,
}
```

## Catalog

### BookCategoryCreated

```rust
pub struct BookCategoryCreated {
    pub book_category_id: BookCategoryId,
    pub category_name: CategoryName,
}
```

### BookCategoryUpdated / BookCategoryDeleted

```rust
pub struct BookCategoryUpdated { pub book_category_id: BookCategoryId, pub changes: Vec<&'static str> }
pub struct BookCategoryDeleted { pub book_category_id: BookCategoryId }
```

### BookAdded

```rust
pub struct BookAdded {
    pub book_id: BookId,
    pub book_title: BookTitle,
    pub book_number: Option<BookNumber>,
    pub isbn_no: Option<Isbn>,
    pub author_name: Option<Author>,
    pub publisher_name: Option<Publisher>,
    pub rack_number: Option<RackNumber>,
    pub quantity: StockCopies,
    pub book_price: Option<BookPrice>,
    pub book_category_id: BookCategoryId,
    pub book_subject_id: Option<SubjectId>,
}
```

### BookUpdated

```rust
pub struct BookUpdated {
    pub book_id: BookId,
    pub changes: Vec<&'static str>,
}
```

### BookDeleted

```rust
pub struct BookDeleted {
    pub book_id: BookId,
}
```

### BookQuantityAdjusted

```rust
pub struct BookQuantityAdjusted {
    pub book_id: BookId,
    pub from_quantity: StockCopies,
    pub to_quantity: StockCopies,
    pub reason: StockAdjustmentReason,
}
```

**Subscribers:**
- `communication` — notify the librarian of acquisitions or
  write-offs.

## Members

### LibraryMemberRegistered

```rust
pub struct LibraryMemberRegistered {
    pub library_member_id: LibraryMemberId,
    pub member: MemberId,
    pub member_type: MemberType,
    pub member_ud_id: MemberUdId,
    pub academic_year_id: AcademicYearId,
}
```

**Subscribers:**
- `communication` — send a welcome note to the member.

### LibraryMemberUpdated

```rust
pub struct LibraryMemberUpdated {
    pub library_member_id: LibraryMemberId,
    pub changes: Vec<&'static str>,
}
```

### LibraryMemberDeactivated

```rust
pub struct LibraryMemberDeactivated {
    pub library_member_id: LibraryMemberId,
    pub reason: String,
}
```

### LibraryMemberReactivated

```rust
pub struct LibraryMemberReactivated {
    pub library_member_id: LibraryMemberId,
}
```

### LibraryMemberDeleted

```rust
pub struct LibraryMemberDeleted {
    pub library_member_id: LibraryMemberId,
}
```

## Issue Lifecycle

### BookIssued

```rust
pub struct BookIssued {
    pub book_issue_id: BookIssueId,
    pub book_id: BookId,
    pub library_member_id: LibraryMemberId,
    pub quantity: IssueQuantity,
    pub given_date: GivenDate,
    pub due_date: DueDate,
    pub note: Option<IssueNote>,
}
```

**Subscribers:**
- `communication` — notify the member of the issue with the
  due date.

### BookReturned

```rust
pub struct BookReturned {
    pub book_issue_id: BookIssueId,
    pub book_id: BookId,
    pub library_member_id: LibraryMemberId,
    pub return_date: ReturnDate,
    pub note: Option<IssueNote>,
}
```

**Subscribers:**
- `finance` — if a `FineCalculated` event is also produced,
  post the receivable.

### BookRenewed

```rust
pub struct BookRenewed {
    pub book_issue_id: BookIssueId,
    pub book_id: BookId,
    pub library_member_id: LibraryMemberId,
    pub from_due_date: DueDate,
    pub to_due_date: DueDate,
    pub renewed_at: Timestamp,
}
```

**Subscribers:**
- `communication` — notify the member of the new due date.

### BookMarkedLost

```rust
pub struct BookMarkedLost {
    pub book_issue_id: BookIssueId,
    pub book_id: BookId,
    pub library_member_id: LibraryMemberId,
    pub quantity: IssueQuantity,
    pub note: Option<IssueNote>,
}
```

**Subscribers:**
- `finance` — post the replacement cost as a receivable if
  configured.
- `communication` — notify the member.

### FineCalculated

```rust
pub struct FineCalculated {
    pub book_issue_id: BookIssueId,
    pub book_id: BookId,
    pub library_member_id: LibraryMemberId,
    pub days_overdue: DaysOverdue,
    pub per_day_rate: FinePerDay,
    pub amount: FineAmount,
    pub waived: bool,
    pub reason: FineReason, // LateReturn, Lost, Manual
    pub calculated_at: Timestamp,
}
```

**Subscribers:**
- `finance` — post the fine as a receivable against the
  member.

### OverdueEvaluated (derived)

The library domain does not emit a separate `OverdueEvaluated`
event. The `Overdue` status is a derived view computed from
the current `as_of` date and the issue's `due_date`. A
scheduled job may publish a notification per overdue issue
through the `communication` domain.

### BookIssueStateChanged (generic)

A generic state-change event is not emitted; concrete events
(`BookIssued`, `BookReturned`, `BookRenewed`,
`BookMarkedLost`, `FineCalculated`) carry the transition
information.
