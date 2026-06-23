# 07 - Audit Appendix - Workflows (deep audit)

**Scope:** wave7-workflows.md (workflow state machines, idempotency, saga compensation, ordering)

**Total findings:** 31

**Severity distribution:** 23 critical, 6 high, 1 medium, 1 low


## Summary Table

| Target | Critical | High | Medium | Low | Total |
| --- | --- | --- | --- | --- | --- |
| Workflows (deep audit) (`WF`) | 23 | 6 | 1 | 1 | 31 |

## Workflows (deep audit) (target id prefix: `WF`)

**Path:** `all workflow specs + state-machine files`  
**Total findings:** 31 (23 critical, 6 high, 1 medium, 1 low)


### FINDING 1 (id: `WF-001`)

- **Source:** `docs/audit_reports/findings/wave7-workflows.md`
- **Severity:** Critical
- **Area:** workflows
- **Location:** `crates/domains/academic/tests/workflows.rs` (missing); all 10 other `crates/domains/<domain>/tests/workflows.rs` (missing).

**Description:**

The build plan mandates `crates/domains/<domain>/tests/workflows.rs` as the **per-domain gate** for `workflows.md` coverage. `find /home/beznet/Workspace/smscore/crates/domains -type d -name tests` returns zero directories; `find … -name workflows.rs` returns zero files. No domain crate ships any `tests/` directory. Every workflow-spec invocation is therefore uncovered by any executable test, regardless of whether the underlying subscriber / handler is wired.

**Expected:**

`docs/build-plan.md:1860` row in the test-file table: "`crates/domains/<d>/tests/workflows.rs` — Multi-aggregate workflows from `workflows.md`".

**Evidence:**

- `docs/build-plan.md:1857-1862`:
    ```text
    | `crates/domains/<d>/tests/aggregate_fields.rs` | Field-level invariants from `aggregates.md`     |
    | `crates/domains/<d>/tests/commands.rs`         | Command handlers from `commands.md`             |
    | `crates/domains/<d>/tests/events.rs`           | Event envelopes from `events.md`                |
    | `crates/domains/<d>/tests/services.rs`         | Domain services from `services.md`              |
    | `crates/domains/<d>/tests/repository.rs`       | Repository port methods from `repositories.md`  |
    | `crates/domains/<d>/tests/value_objects.rs`    | Value-object validation from `value-objects.md` |
    | `crates/domains/<d>/tests/workflows.rs`        | Multi-aggregate workflows from `workflows.md`   |
    ```
  - `find /home/beznet/Workspace/smscore/crates/domains -type d -name tests` returns no output.
  - `find /home/beznet/Workspace/smscore/crates/domains -name workflows.rs -path '*/tests/*'` returns no output.

---

### FINDING 10 (id: `WF-010`)

- **Source:** `docs/audit_reports/findings/wave7-workflows.md`
- **Severity:** Critical
- **Area:** workflows
- **Location:** `docs/specs/communication/workflows.md:85-95` and `crates/domains/communication/src/` (entire crate).

**Description:**

"Absence Notification Workflow" mandates an `AbsentNotificationService` that subscribes to attendance `StudentMarkedAbsent` events and dispatches via configured gateways. No `AbsentNotificationService` exists in `crates/domains/communication/src/services.rs`; no code path consumes the `StudentMarkedAbsent` / `StudentAbsentForDay` event.

**Expected:**

An `AbsentNotificationService` subscriber function (matching the spec's name) wired into the bus.

**Evidence:**

- `docs/specs/communication/workflows.md:85-95`:
    ```text
    3. AbsentNotificationService subscribes:
       a. If the current local time is within the configured window, the
          service picks the matching NotificationSetting and template.
       b. Renders the body with the student's name, class, and date.
       c. Selects a channel (Email / SMS) and a recipient (the primary
          guardian).
       d. Emits AbsentNotificationSent.
    ```
  - `grep -rn 'AbsentNotificationService\|StudentMarkedAbsent\|StudentAbsentForDay' crates/domains/communication/src/` returns zero output.

---

### FINDING 11 (id: `WF-011`)

- **Source:** `docs/audit_reports/findings/wave7-workflows.md`
- **Severity:** Critical
- **Area:** workflows
- **Location:** `docs/specs/assessment/workflows.md:30-41` and `crates/domains/cms/src/`, `crates/domains/communication/src/`.

**Description:**

"Exam Scheduling Workflow" step 4: "On success, `ExamScheduled` is emitted and the routine is published to the public routine page (subscribed by cms)." "Exam-Day Attendance" step 7: "Communication sends a routine notification to guardians." Neither the CMS nor the communication crate contains a subscriber on `ExamScheduled`. The public routine page is never updated; guardian notifications are not sent.

**Expected:**

CMS subscriber on `ExamScheduled` that updates the public routine page; communication subscriber on `ExamScheduled` that dispatches `Notice` to guardians.

**Evidence:**

- `docs/specs/assessment/workflows.md:30-41`:
    ```text
    4. On success, ExamScheduled is emitted and the routine is published
       to the public routine page (subscribed by cms).
    ```
  - `grep -rn 'ExamScheduled' crates/domains/cms/src/ crates/domains/communication/src/` returns zero matches outside `events.rs` definitions.

---

### FINDING 12 (id: `WF-012`)

- **Source:** `docs/audit_reports/findings/wave7-workflows.md`
- **Severity:** Critical
- **Area:** workflows
- **Location:** `docs/specs/assessment/workflows.md:99-105` and `crates/domains/{communication,finance,cms,academic}/src/`.

**Description:**

"Result Publication Workflow" step 7: "Subscribers (communication, finance, cms, academic) react." None of the four domains define a subscriber on `ResultPublished`. Report-card notification, finance roll-over, CMS publish, and academic update never happen after a result is published.

**Expected:**

Four subscribers: communication → notify guardians; finance → no-op or note rollover; cms → publish merit lists; academic → record year-end archive.

**Evidence:**

- `docs/specs/assessment/workflows.md:99-105`:
    ```text
    6. ResultPublished is emitted.
    7. Subscribers (communication, finance, cms, academic) react.
    ```
  - `grep -rn 'ResultPublished' crates/domains/communication/src/ crates/domains/finance/src/ crates/domains/cms/src/ crates/domains/academic/src/` returns zero matches (only assessment-internal references).

---

### FINDING 13 (id: `WF-013`)

- **Source:** `docs/audit_reports/findings/wave7-workflows.md`
- **Severity:** Critical
- **Area:** workflows
- **Location:** `docs/specs/attendance/workflows.md:117-130` and `crates/domains/attendance/src/`.

**Description:**

"Exam-Day Attendance" workflow mandates "The attendance domain subscribes to `ExamAttendanceMarked` and updates the `ClassAttendance` summary used in report cards." The attendance crate contains no subscriber on `ExamAttendanceMarked`. The `ClassAttendance.days_present` / `days_absent` summary used for report cards therefore never reflects exam-day attendance.

**Expected:**

A subscriber on `ExamAttendanceMarked` that updates the `ClassAttendance` aggregate.

**Evidence:**

- `docs/specs/attendance/workflows.md:117-130`:
    ```text
    3. The attendance domain subscribes and recomputes
       ClassAttendance.days_present / days_absent for the relevant
       exam type.
    ```
  - `grep -rn 'ExamAttendanceMarked\|bus.subscribe' crates/domains/attendance/src/` returns zero matches outside the event-type definition in `events.rs`.

---

### FINDING 14 (id: `WF-014`)

- **Source:** `docs/audit_reports/findings/wave7-workflows.md`
- **Severity:** Critical
- **Area:** workflows
- **Location:** `docs/specs/events/workflows.md:24-32` and `crates/domains/attendance/src/`.

**Description:**

"Holiday Configuration Workflow" step 2: "The attendance domain subscribes to `HolidayCreated` and: a. Marks the days as non-instructional. b. Skips attendance expectations for the date range." No subscriber on `HolidayCreated` exists in the attendance crate. `MarkStudentAttendance` will not treat holidays specially because the holiday list is never propagated to the attendance side.

**Expected:**

A subscriber on `EventEnvelope<HolidayCreated>` that marks the date range in the attendance domain.

**Evidence:**

- `docs/specs/events/workflows.md:24-32`:
    ```text
    2. The attendance domain subscribes to HolidayCreated and:
       a. Marks the days as non-instructional.
       b. Skips attendance expectations for the date range.
    ```
  - `grep -rn 'HolidayCreated\|bus.subscribe' crates/domains/attendance/src/` returns zero matches outside `events.rs`.

---

### FINDING 15 (id: `WF-015`)

- **Source:** `docs/audit_reports/findings/wave7-workflows.md`
- **Severity:** Critical
- **Area:** workflows
- **Location:** `docs/specs/events/workflows.md:99-105` and `crates/domains/hr/src/`.

**Description:**

"Incident Resolution Workflow" step 4: "The HR domain subscribes and archives the incident for behavior records." No subscriber on `IncidentResolved` exists in the HR crate. Behavior notes are never written.

**Expected:**

A subscriber on `IncidentResolved` that writes a behavior record on the staff profile.

**Evidence:**

- `docs/specs/events/workflows.md:99-105`:
    ```text
    4. The HR domain subscribes and archives the incident for behavior
       records.
    ```
  - `grep -rn 'IncidentResolved\|bus.subscribe' crates/domains/hr/src/` returns zero matches outside the event-type definition.

---

### FINDING 17 (id: `WF-017`)

- **Source:** `docs/audit_reports/findings/wave7-workflows.md`
- **Severity:** Critical
- **Area:** workflows
- **Location:** `docs/specs/finance/workflows.md:159-168` and `crates/cross-cutting/rbac/src/`.

**Description:**

"Due Fees Login Prevention" step 3: "RBAC subscribes and blocks the user at the authentication port for the role." The `educore-rbac` crate has no subscriber on `BlockLoginForDueFees` / `UnblockLoginForDueFees`. The auth-port block is never applied.

**Expected:**

A subscriber on `BlockLoginForDueFees` in the `educore-rbac` crate that calls the authentication port's `block` method.

**Evidence:**

- `docs/specs/finance/workflows.md:159-168`:
    ```text
    2. For each such user, the system emits BlockLoginForDueFees.
    3. RBAC subscribes and blocks the user at the authentication port
       for the role.
    4. When the user pays in full, the system emits
       UnblockLoginForDueFees and the user is restored.
    ```
  - `grep -rn 'BlockLoginForDueFees\|UnblockLoginForDueFees\|bus.subscribe' crates/cross-cutting/rbac/src/` returns zero matches outside doc comments.

---

### FINDING 18 (id: `WF-018`)

- **Source:** `docs/audit_reports/findings/wave7-workflows.md`
- **Severity:** Critical
- **Area:** workflows
- **Location:** `docs/specs/finance/workflows.md:171-182` and `crates/domains/finance/src/services.rs`.

**Description:**

"Expense Recording" step 3: "If the expense is for a payroll, `PayrollPaymentRecorded` subscribes and an Expense is automatically created." No subscriber on `PayrollPaymentRecorded` exists. Payroll payments never produce an `Expense` automatically.

**Expected:**

A subscriber on `PayrollPaymentRecorded` that creates the corresponding `Expense` row.

**Evidence:**

- `docs/specs/finance/workflows.md:171-182`:
    ```text
    3. If the expense is for a payroll, PayrollPaymentRecorded subscribes
       and an Expense is automatically created.
    ```
  - `grep -rn 'PayrollPaymentRecorded\|bus.subscribe' crates/domains/finance/src/` returns zero matches outside the `events.rs` definition and `services.rs` import.

---

### FINDING 19 (id: `WF-019`)

- **Source:** `docs/audit_reports/findings/wave7-workflows.md`
- **Severity:** Critical
- **Area:** workflows
- **Location:** `crates/domains/*/src/commands.rs` (all 10 crates) and `docs/ports/storage.md:100-110`.

**Description:**

Across all 10 domain crates, `commands.rs` files contain only command **struct** definitions and helper methods (`pub fn school_id(&self)`, `pub fn range(&self) -> Result`). There is no `handle_*`, `dispatch_*`, or `execute_*` function in any of them, and no reference to `audit_log`, `idempotency`, or `tx.outbox().append(...)`. This means the entire transactional write path (audit row + idempotency record + outbox append + aggregate update) is not wired. Every spec-mandated idempotency requirement (e.g. `AdmitStudent` is idempotent on `(admission_no, school_id)` per `docs/specs/academic/workflows.md:147-148`) is unenforceable because there is no code path that checks the idempotency store before mutating the aggregate.

**Expected:**

Per-aggregate command-handler functions that take a `&mut Transaction`, check `idempotency().exists(key)`, perform the domain mutation via the service factory, append the envelope to the outbox, write the audit row, and record the idempotency key — all in one transaction.

**Evidence:**

- `crates/domains/academic/src/commands.rs:151`:
    ```rust
    pub fn school_id(&self) -> SchoolId { ... }
    ```
    (the only `pub fn` in the file; verified via `grep -n 'pub fn\|pub async fn' crates/domains/academic/src/commands.rs`).
  - `crates/domains/finance/src/commands.rs:1243` total lines, `grep -n 'pub fn\|pub async fn' crates/domains/finance/src/commands.rs` returns only `pub fn` field-accessor lines (no handler functions).
  - `grep -rn 'audit_log\|idempotency\|tx.outbox' crates/domains/*/src/commands.rs` returns zero matches across all 10 crates.
  - Spec mandate: `docs/specs/academic/workflows.md:147-148`:
    ```text
    - `AdmitStudent` is idempotent on `(admission_no, school_id)`. A
      duplicate returns the existing student.
    ```

---

### FINDING 2 (id: `WF-002`)

- **Source:** `docs/audit_reports/findings/wave7-workflows.md`
- **Severity:** Critical
- **Area:** workflows
- **Location:** `docs/specs/library/workflows.md:35-39` and `crates/domains/library/src/` (entire crate).

**Description:**

The library domain's "Member Registration Workflow" step 2 requires "The library subscribes to `StudentAdmitted` (and the HR equivalent for staff) and auto-creates a `LibraryMember` (`RegisterLibraryMember`) for the new member in the current academic year." Cross-domain coordination at `library/workflows.md:201-203` restates this: "The library domain subscribes to `StudentAdmitted` to auto-create a member; the subscriber is idempotent." No code in `crates/domains/library/src/` references `StudentAdmitted`, contains a `bus.subscribe(...)` call, or defines an event-handler function. The student-to-library member fan-out is completely absent.

**Expected:**

A subscriber function in `crates/domains/library/src/` that takes an `EventEnvelope<StudentAdmitted>`, checks idempotency, and returns a `RegisterLibraryMemberCommand` ready to be dispatched (per `library/workflows.md:35-39`).

**Evidence:**

- `docs/specs/library/workflows.md:35-39`:
    ```text
    2. The library subscribes to StudentAdmitted (and the HR
       equivalent for staff) and auto-creates a LibraryMember
       (RegisterLibraryMember) for the new member in the current
       academic year. Auto-creation is idempotent.
    ```
  - `grep -rn 'StudentAdmitted\|bus.subscribe\|EventBus' crates/domains/library/src/` returns zero matches outside the event-type references in `events.rs` and `services.rs` imports (verified — only the `LibraryMemberRegistered` import lines and library's own events are present).

---

### FINDING 20 (id: `WF-020`)

- **Source:** `docs/audit_reports/findings/wave7-workflows.md`
- **Severity:** Critical
- **Area:** workflows
- **Location:** `crates/infra/storage/src/transaction.rs:120-128` (`PostgresTransaction::commit`), `crates/adapters/storage-mysql/src/transaction.rs:120-128`, and the SQLite equivalent.

**Description:**

All three SQL `Transaction::commit` implementations are explicit no-ops: "the sub-port operations have already committed via the `sqlx::Transaction` they each acquired." Per the module-level docs (`storage-postgres/src/transaction.rs:17-21`): "`PostgresTransaction`'s `commit` and `rollback` are no-ops: a `sqlx::Transaction` auto-commits on `Drop`, so each sub-port call commits independently." This breaks the transactional outbox pattern: a `tx.outbox().append(env)` followed by a successful `tx.repositories().students().update(...)` does **not** form a single atomic commit. If the outbox append succeeds and the repository update fails (or vice versa) there is no rollback. The engine rule "the audit row, the outbox envelope, and the event log row are committed atomically with the aggregate state" (`crates/infra/storage/src/transaction.rs:9-15`) is violated by design.

**Expected:**

A `Transaction::commit` that calls `pool.commit()` (or the dialect equivalent) once, after every staged write, with the `sqlx::Transaction` shared across sub-port handles instead of auto-committed per call.

**Evidence:**

- `crates/adapters/storage-postgres/src/transaction.rs:17-21`:
    ```text
    //! `PostgresTransaction`'s `commit` and `rollback` are no-ops:
    //! a `sqlx::Transaction` auto-commits on `Drop`, so each
    //! sub-port call commits independently. The engine's at-least-
    //! once outbox semantics (dedup by `event_id` primary key,
    ```
  - `crates/adapters/storage-postgres/src/transaction.rs:120-128`:
    ```rust
    async fn commit(self: Box<Self>) -> Result<()> {
        // No-op: the sub-port operations have already committed
        // via the `sqlx::Transaction` they each acquired. We
        // only flip the guard flag.
        self.done.store(true, Ordering::SeqCst);
        Ok(())
    }
    ```

---

### FINDING 21 (id: `WF-021`)

- **Source:** `docs/audit_reports/findings/wave7-workflows.md`
- **Severity:** Critical
- **Area:** workflows
- **Location:** `crates/tools/testkit/src/storage.rs:431-452` and the entire `crates/` tree outside `crates/cross-cutting/sync-inprocess/`.

**Description:**

No outbox relay process exists anywhere in the workspace. The testkit's `InMemoryTransaction::commit` drains the outbox into `_pending: Vec<SerializedEnvelope>` and discards it (lines 442-449) with an explicit comment: "the in-memory testkit does not republish envelopes to the bus." The three SQL adapters' `commit` is a no-op (Finding WF-020), so nothing publishes pending envelopes either. The sync-inprocess adapter (`crates/cross-cutting/sync-inprocess/`) is the only place that drains an outbox to the bus, and it operates on its own internal `outbox` channel, not on the storage port's `outbox.pending(limit)` API. There is no production code path that calls `Outbox::pending` followed by `EventBus::publish` for any domain command.

**Expected:**

A relay process (per `docs/ports/event-bus.md` § "Outbox Pattern": "The outbox relay (a separate process) reads pending events from the outbox and publishes them to the event bus") that loops on `outbox.pending(limit)` and calls `bus.publish(env)` per envelope, marking the rows as published.

**Evidence:**

- `crates/tools/testkit/src/storage.rs:431-452`:
    ```rust
    async fn commit(self: Box<Self>) -> Result<()> {
        ...
        // Drain the outbox; the in-memory testkit does not
        // republish envelopes to the bus ...
        let _pending: Vec<SerializedEnvelope> = {
            let mut outbox = self.inner.outbox.lock();
            outbox.drain(..).collect()
        };
        Ok(())
    }
    ```
  - `grep -rn 'bus.publish\|EventBus::publish' crates/cross-cutting/sync-inprocess/src/lib.rs` shows only the test path uses `bus.publish` (line 217 reference); no other crate calls `bus.publish` on the way out of a commit.
  - `docs/ports/event-bus.md` § "Outbox Pattern":
    ```text
    The engine writes events to an outbox table within the same database
    transaction as the domain state change. The outbox relay (a separate
    process) reads pending events from the outbox and publishes them to
    the bus. On success, the relay marks the events as published.
    ```

---

### FINDING 22 (id: `WF-022`)

- **Source:** `docs/audit_reports/findings/wave7-workflows.md`
- **Severity:** Critical
- **Area:** workflows
- **Location:** `docs/specs/facilities/workflows.md:50-55` and `crates/domains/finance/src/`.

**Description:**

"Student Transport Assignment Workflow" step 5: "Finance subscribes to `StudentAssignedToRoute` and applies the transport fee." No subscriber on `StudentAssignedToRoute` exists in the finance crate; no event of that name is emitted by facilities (`grep -n 'StudentAssignedToRoute' crates/domains/facilities/src/events.rs` returns zero matches). The transport-fee fan-out is doubly broken: there is no producer and no consumer.

**Expected:**

A `StudentAssignedToRoute` event in `crates/domains/facilities/src/events.rs` and a subscriber in `crates/domains/finance/src/` that creates the transport-fee `FeesAssign`.

**Evidence:**

- `docs/specs/facilities/workflows.md:50-55`:
    ```text
    5. Finance subscribes to StudentAssignedToRoute and applies
       the transport fee.
    ```
  - `grep -rn 'StudentAssignedToRoute' crates/domains/facilities/src/ crates/domains/finance/src/` returns zero output.

---

### FINDING 28 (id: `WF-028`)

- **Source:** `docs/audit_reports/findings/wave7-workflows.md`
- **Severity:** Critical
- **Area:** workflows
- **Location:** `crates/domains/library/src/commands.rs` (entire file) and `docs/specs/library/workflows.md:35-39`.

**Description:**

The library `commands.rs` defines `AddBookCommand`, `CreateBookCategoryCommand`, `IssueBookCommand`, `RegisterLibraryMemberCommand`, `ReturnBookCommand`, `CalculateFineCommand` etc., but contains **no `RegisterLibraryMemberCommand` variant** flagged as auto-dispatchable from a `StudentAdmitted` envelope. Worse, even if a `RegisterLibraryMember` factory existed in `services.rs`, no dispatch path from the bus exists (see Finding WF-002). The `register_library_member` service factory (`crates/domains/library/src/services.rs:line ~200 region`) is never invoked from outside the crate. The cross-domain auto-create path is dead.

**Expected:**

A subscriber function that maps `EventEnvelope<StudentAdmitted>` → `RegisterLibraryMemberCommand` ready for dispatch.

**Evidence:**

- `crates/domains/library/src/commands.rs:568` total lines (verified via `wc -l`); `grep -n 'pub fn\|pub async fn' crates/domains/library/src/commands.rs` returns only struct field accessors and `pub fn issue_status(...)` helpers.
  - `grep -rn 'StudentAdmitted' crates/domains/library/src/` returns zero matches.

---

### FINDING 3 (id: `WF-003`)

- **Source:** `docs/audit_reports/findings/wave7-workflows.md`
- **Severity:** Critical
- **Area:** workflows
- **Location:** `docs/specs/library/workflows.md:203-204` and `crates/domains/library/src/` (entire crate).

**Description:**

"The library domain subscribes to `StudentWithdrawn` to flag open issues for return; the member is not deleted." No subscriber exists; no code path flags outstanding books when a student is withdrawn. Book issue inventory silently becomes orphaned.

**Expected:**

A subscriber on `EventEnvelope<StudentWithdrawn>` that emits a flag in the library domain (per spec).

**Evidence:**

- `docs/specs/library/workflows.md:203-204`:
    ```text
    - The library domain subscribes to `StudentWithdrawn` to
      flag open issues for return; the member is not deleted.
    ```
  - `grep -rn 'StudentWithdrawn' crates/domains/library/src/` returns zero matches.

---

### FINDING 30 (id: `WF-030`)

- **Source:** `docs/audit_reports/findings/wave7-workflows.md`
- **Severity:** Critical
- **Area:** workflows
- **Location:** `crates/cross-cutting/events/src/event_bus.rs:48` (port trait) and `crates/cross-cutting/sync-inprocess/src/lib.rs`.

**Description:**

The `EventBus` port trait at `crates/cross-cutting/events/src/event_bus.rs:48` exposes `subscribe` that returns `Box<dyn EventSubscription>`. The only place that calls `bus.subscribe(...)` in any domain or cross-cutting crate outside the testkit and event-bus adapter tests is `crates/cross-cutting/sync-inprocess/src/lib.rs:219` (a sync-only consumer). No domain crate ever invokes `bus.subscribe` to wire up its own handlers. The handler functions that exist (only `form_uploaded_public_indexing_subscriber` in `crates/domains/cms/src/services.rs:819`) are not registered. This makes the entire bus port dead code from the perspective of domain handlers.

**Expected:**

A `bootstrap.rs` per domain crate that calls `bus.subscribe(SubscribeOptions { topic: Topic::EventType("..."), consumer: ConsumerId::new("..."), .. })` for each subscriber, executed at server start-up.

**Evidence:**

- `crates/cross-cutting/events/src/event_bus.rs:48`:
    ```rust
    async fn subscribe(&self, options: SubscribeOptions) -> Result<Box<dyn EventSubscription>>;
    ```
  - `grep -rn 'bus\.subscribe\|\.subscribe(' crates/domains/` returns zero matches (only matches in `crates/cross-cutting/sync-inprocess/src/lib.rs:219`, `crates/adapters/event-bus/tests/in_process_e2e.rs`, `crates/tools/testkit/src/event_bus.rs`, and `crates/tools/storage-parity/tests/*`).

---

### FINDING 4 (id: `WF-004`)

- **Source:** `docs/audit_reports/findings/wave7-workflows.md`
- **Severity:** Critical
- **Area:** workflows
- **Location:** `docs/specs/library/workflows.md:207-209` and `crates/domains/finance/src/` (entire crate).

**Description:**

"Finance subscribes to `FineCalculated` (and the implicit loss event in `BookMarkedLost`) to post receivables." No subscriber on `FineCalculated` or `BookMarkedLost` exists in the finance crate. Late-return fines and lost-book replacement costs are never posted to receivables.

**Expected:**

Subscribers on both `FineCalculated` and `BookMarkedLost` envelopes that post `FeesPayment` or `IncomeRecorded`/`ExpenseRecorded` per school policy.

**Evidence:**

- `docs/specs/library/workflows.md:207-209`:
    ```text
    - Finance subscribes to `FineCalculated` (and the implicit
      loss event in `BookMarkedLost`) to post receivables.
    ```
  - `grep -rn 'FineCalculated\|BookMarkedLost' crates/domains/finance/src/` returns zero matches (only library-internal references in `services.rs` factory imports).

---

### FINDING 5 (id: `WF-005`)

- **Source:** `docs/audit_reports/findings/wave7-workflows.md`
- **Severity:** Critical
- **Area:** workflows
- **Location:** `docs/specs/finance/workflows.md:342-350` and `crates/domains/finance/src/services.rs` (entire file).

**Description:**

"Cross-Workflow Order" mandates finance reacts to academic + HR events: "`StudentAdmitted` (academic) → `FeesAssign` is created"; "`StudentPromoted` (academic) → prior balance is closed, new `FeesAssign` is created in the new year, carry-forward is applied"; "`StudentWithdrawn` (academic) → open `FeesAssign` is closed"; "`StaffRegistered` (HR) → `SalaryTemplate` is bound to the staff". "The finance domain never calls the academic or HR domains directly. It only subscribes to their events and reacts through its own commands." None of these four subscribers exist anywhere in the finance crate.

**Expected:**

Four subscriber functions / event-handler entry points in `crates/domains/finance/src/services.rs` (or a new `crates/domains/finance/src/subscribers.rs`).

**Evidence:**

- `docs/specs/finance/workflows.md:342-350`:
    ```text
    The finance domain observes the following order to keep state
    coherent:

    1. `StudentAdmitted` (academic) → `FeesAssign` is created.
    2. `StudentPromoted` (academic) → prior balance is closed, new
       `FeesAssign` is created in the new year, carry-forward is
       applied.
    3. `StudentWithdrawn` (academic) → open `FeesAssign` is closed;
       unpaid balance becomes a carry-forward or a refund, per policy.
    4. `StaffRegistered` (HR) → `SalaryTemplate` is bound to the staff;
       payroll becomes available.
    ```
  - `grep -rn 'StudentAdmitted\|StudentPromoted\|StudentWithdrawn\|StaffRegistered' crates/domains/finance/src/` returns zero matches.

---

### FINDING 6 (id: `WF-006`)

- **Source:** `docs/audit_reports/findings/wave7-workflows.md`
- **Severity:** Critical
- **Area:** workflows
- **Location:** `docs/specs/hr/workflows.md:163-172` and `crates/domains/hr/src/` (entire crate).

**Description:**

"Payroll Disbursement (Cross-Domain)" mandates "The HR domain subscribes to `PayrollPaid` and triggers `MarkPayrollPaid` on the `PayrollGenerate`." The HR crate has no subscriber on `PayrollPaid`, no `on_payroll_paid` handler, and `MarkPayrollPaidCommand` (`crates/domains/hr/src/commands.rs:155`) has no producer wired to the finance event bus. The HR aggregate's payroll status is therefore never advanced to `paid` by the actual payment event.

**Expected:**

A subscriber on `EventEnvelope<PayrollPaid>` that calls `MarkPayrollPaidCommand` on the local `PayrollGenerate` aggregate.

**Evidence:**

- `docs/specs/hr/workflows.md:163-172`:
    ```text
    3. The HR domain subscribes to PayrollPaid and triggers
       MarkPayrollPaid on the PayrollGenerate.
    4. The HR domain emits PayrollPaid (a separate event with the HR
       aggregate id).
    ```
  - `grep -rn 'PayrollPaid\|bus.subscribe\|EventBus' crates/domains/hr/src/` returns only library-internal matches (event-type imports in `events.rs`); no `bus.subscribe` call exists in any HR file.

---

### FINDING 7 (id: `WF-007`)

- **Source:** `docs/audit_reports/findings/wave7-workflows.md`
- **Severity:** Critical
- **Area:** workflows
- **Location:** `docs/specs/hr/workflows.md:72-76` and `crates/domains/academic/src/` (entire crate).

**Description:**

"Subject Teacher Assignment" workflow step 2: "The system emits `SubjectTeacherAssigned`; the academic domain subscribes and creates the corresponding class-subject row." The academic crate defines no subscriber on `SubjectTeacherAssigned`. The class-subject row referenced in `ClassRoutine` workflows therefore never materializes.

**Expected:**

A subscriber on `SubjectTeacherAssigned` that creates/updates the class-subject binding.

**Evidence:**

- `docs/specs/hr/workflows.md:72-76`:
    ```text
    2. HR triggers AssignSubjectTeacher with a class, optional
       section, subject, staff, and academic year.
    3. The system emits SubjectTeacherAssigned; the academic domain
       subscribes and creates the corresponding class-subject row.
    ```
  - `grep -rn 'SubjectTeacherAssigned\|bus.subscribe' crates/domains/academic/src/` returns zero matches.

---

### FINDING 8 (id: `WF-008`)

- **Source:** `docs/audit_reports/findings/wave7-workflows.md`
- **Severity:** Critical
- **Area:** workflows
- **Location:** `docs/specs/academic/workflows.md:11-17` and `crates/domains/communication/src/` (entire crate).

**Description:**

"Admission Workflow" step 6: "Send welcome SMS/email to guardians (Communication subscribes)". The communication crate does not contain a subscriber on `StudentAdmitted` (verified: `grep -rn 'StudentAdmitted' crates/domains/communication/src/` returns zero matches). No welcome message is ever dispatched on admission.

**Expected:**

A subscriber on `EventEnvelope<StudentAdmitted>` that calls `DispatchSendMessage` or `CreateNotice` + `PublishNotice` per guardian audience.

**Evidence:**

- `docs/specs/academic/workflows.md:11-17`:
    ```text
    1. Receive admission inquiry (RegisterAdmissionQuery)
    ...
    6. Send welcome SMS/email to guardians (Communication subscribes)
    ```
  - `grep -rn 'StudentAdmitted' crates/domains/communication/src/` returns zero output (exit code 1).

---

### FINDING 9 (id: `WF-009`)

- **Source:** `docs/audit_reports/findings/wave7-workflows.md`
- **Severity:** Critical
- **Area:** workflows
- **Location:** `docs/specs/attendance/workflows.md:14-19` and `crates/domains/communication/src/` (entire crate).

**Description:**

"Daily Attendance Capture" step 7: "Communication subscribes to `StudentAbsentForDay` and sends notifications to guardians (when notify=true)." No subscriber exists in the communication crate; absence notifications never fire.

**Expected:**

A subscriber on `StudentAbsentForDay` filtered by `notify=true` that dispatches via `SmsGateway` or `EmailSetting` per the school's `NotificationSetting`.

**Evidence:**

- `docs/specs/attendance/workflows.md:14-19`:
    ```text
    7. Communication subscribes to StudentAbsentForDay and sends
       notifications to guardians (when notify=true).
    ```
  - `grep -rn 'StudentAbsentForDay' crates/domains/communication/src/` returns zero output.

---

### FINDING 16 (id: `WF-016`)

- **Source:** `docs/audit_reports/findings/wave7-workflows.md`
- **Severity:** High
- **Area:** workflows
- **Location:** `docs/specs/documents/workflows.md:9-15` and `crates/domains/cms/src/services.rs:819-862` (and the rest of the crate).

**Description:**

"Form Download Lifecycle" step 2: "The CMS domain subscribes to `FormUploaded` and: a. Surfaces the form on the public site when `show_public = true`. b. Surfaces the form on the parent portal otherwise." CMS defines the `form_uploaded_public_indexing_subscriber` function (lines 819-862), but no code calls `bus.subscribe(...)` to register it on the event bus. The function is a pure mapper from `EventEnvelope → FormIndexAction`; it is never invoked. This is a **phantom subscriber** — the wire-up code is missing.

**Expected:**

A subscription registration call (e.g. inside a process bootstrap function or test) that wires `form_uploaded_public_indexing_subscriber` to `Topic::EventType("documents.form_download.uploaded")` on the in-process bus.

**Evidence:**

- `docs/specs/documents/workflows.md:9-15`:
    ```text
    2. The CMS domain subscribes to FormUploaded and:
       a. Surfaces the form on the public site when show_public = true.
       b. Surfaces the form on the parent portal otherwise.
    ```
  - `crates/domains/cms/src/services.rs:819-862`:
    ```rust
    pub fn form_uploaded_public_indexing_subscriber(
        envelope: educore_events::envelope::EventEnvelope,
    ) -> FormIndexAction {
        let show_public = envelope
            .payload
            .get("show_public")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        if show_public {
            FormIndexAction::Index
        } else {
            FormIndexAction::Ignore
        }
    }
    ```
  - `grep -rn 'form_uploaded_public_indexing_subscriber\|bus.subscribe' crates/domains/cms/src/ crates/educore/src/` returns the definition and three `#[test]` callers, but zero production callers that wire the function to `bus.subscribe(...)` (`grep -v 'fn form_uploaded\|#\[test\]\|fn .*_test' produces zero non-test matches`).

---

### FINDING 23 (id: `WF-023`)

- **Source:** `docs/audit_reports/findings/wave7-workflows.md`
- **Severity:** High
- **Area:** workflows
- **Location:** `docs/specs/facilities/workflows.md:69-71` and `crates/domains/facilities/src/`.

**Description:**

"Student Hostel Assignment Workflow" step 4: "The route assignment is released automatically as a subscriber to `StudentWithdrawn`." No subscriber on `StudentWithdrawn` exists in the facilities crate. Withdrawn students retain hostel + transport assignments.

**Expected:**

A subscriber on `StudentWithdrawn` that releases the route assignment.

**Evidence:**

- `docs/specs/facilities/workflows.md:69-71`:
    ```text
    4. The route assignment is released automatically as a subscriber
       to `StudentWithdrawn`.
    ```
  - `grep -rn 'StudentWithdrawn\|bus.subscribe' crates/domains/facilities/src/` returns zero matches.

---

### FINDING 24 (id: `WF-024`)

- **Source:** `docs/audit_reports/findings/wave7-workflows.md`
- **Severity:** High
- **Area:** workflows
- **Location:** `docs/specs/facilities/workflows.md:82-87` and `crates/domains/finance/src/`.

**Description:**

"Hostel Room Assignment" step 5: "Finance subscribes to `StudentAssignedToRoom` and applies hostel fee." No event `StudentAssignedToRoom` exists in any crate's `events.rs` (verified via `grep -rn 'StudentAssignedToRoom' crates/`), and no subscriber exists in finance.

**Expected:**

A `StudentAssignedToRoom` event + a finance subscriber that creates the hostel-fee `FeesAssign`.

**Evidence:**

- `docs/specs/facilities/workflows.md:82-87`:
    ```text
    5. Finance subscribes to StudentAssignedToRoom and applies hostel
       fee.
    ```
  - `grep -rn 'StudentAssignedToRoom' crates/` returns zero output.

---

### FINDING 25 (id: `WF-025`)

- **Source:** `docs/audit_reports/findings/wave7-workflows.md`
- **Severity:** High
- **Area:** workflows
- **Location:** `docs/specs/facilities/workflows.md:115-119`, `:170-175`, `:195-200` and `crates/domains/finance/src/`.

**Description:**

Three further spec-mandated finance subscribers are entirely absent: "Finance subscribes to `ItemReceived` and posts a payable" (line 119), "Finance subscribes to `ItemSold` and posts the income" (line 175), "Finance subscribes to `SupplierCreated` and registers the supplier in the payables ledger" (line 200). No event of any of these three names is emitted by facilities (verified via `grep -n`).

**Expected:**

Three events in `crates/domains/facilities/src/events.rs` and three subscribers in `crates/domains/finance/src/`.

**Evidence:**

- `docs/specs/facilities/workflows.md:115-119`:
    ```text
    3. Finance subscribes to ItemReceived and posts a payable
       ...
    ```
  - `docs/specs/facilities/workflows.md:170-175`:
    ```text
    5. Finance subscribes to ItemSold and posts the income and any
       ...
    ```
  - `docs/specs/facilities/workflows.md:195-200`:
    ```text
    2. Finance subscribes to SupplierCreated and registers the
       ...
    ```
  - `grep -rn 'ItemReceived\|ItemSold\|SupplierCreated' crates/` returns zero matches outside the spec file.

---

### FINDING 26 (id: `WF-026`)

- **Source:** `docs/audit_reports/findings/wave7-workflows.md`
- **Severity:** High
- **Area:** workflows
- **Location:** `docs/specs/sync/overview.md` (entire file) and the rest of `docs/specs/sync/`.

**Description:**

The sync subsystem spec directory at `docs/specs/sync/` contains only `overview.md`. Of the eleven expected per-domain spec files (matching the layout documented in `AGENTS.md` § "Module Layout (per domain)": `overview.md`, `aggregates.md`, `commands.md`, `events.md`, `entities.md`, `errors.md`, `permissions.md`, `repositories.md`, `services.md`, `tables.md`, `value-objects.md`, `workflows.md`), nine are missing: `aggregates.md`, `commands.md`, `entities.md`, `events.md`, `permissions.md`, `repositories.md`, `services.md`, `tables.md`, `value-objects.md`, `workflows.md`. The overview references (e.g. `## Aggregates`, `## Events`, `## Commands`, `## Tables`, `## Services`, `## Permissions`, `## Workflows`) cannot be cross-referenced against spec files that do not exist.

**Expected:**

Ten additional spec files under `docs/specs/sync/` matching the per-domain 11-file layout.

**Evidence:**

- `ls /home/beznet/Workspace/smscore/docs/specs/sync/` returns only `overview.md`.
  - `find /home/beznet/Workspace/smscore/docs/specs/sync -name '*.md'` returns one line: `docs/specs/sync/overview.md`.

---

### FINDING 29 (id: `WF-029`)

- **Source:** `docs/audit_reports/findings/wave7-workflows.md`
- **Severity:** High
- **Area:** workflows
- **Location:** `crates/domains/communication/src/commands.rs` (entire file) and `docs/specs/communication/workflows.md:36-78` (multiple subscribers).

**Description:**

The communication crate has zero subscribers across nine spec-mandated event triggers (`BookIssued`, `BookRenewed`, `StudentAbsentForDay`, `ExamScheduled`, `ResultPublished`, `StudentAdmitted`, `FeesAssignedToClass`, `StudentAssignedToRoute`, `HolidayCreated`, `IncidentResolved`, etc.). The crate is purely a producer of communication events; it never reacts to events from other domains.

**Expected:**

A `subscribers.rs` (or an extension of `services.rs`) containing ~9 subscriber functions plus a registration bootstrap.

**Evidence:**

- `wc -l crates/domains/communication/src/commands.rs` is the only file; `grep -rn 'subscriber\|on_book_issued\|on_.*_event\|bus.subscribe' crates/domains/communication/src/` returns zero matches.
  - `docs/specs/communication/workflows.md` lists subscriber-mandated workflows at lines 36-78, 85-95 (Absent Notification), and 110-120 (Bulk Messaging receipt confirmation).

---

### FINDING 27 (id: `WF-027`)

- **Source:** `docs/audit_reports/findings/wave7-workflows.md`
- **Severity:** Medium
- **Area:** workflows
- **Location:** `docs/specs/academic/workflows.md` (entire file) and `crates/domains/*/src/`.

**Description:**

"Promotion Workflow" step 5: "Finance subscribes and: a. Closes prior fees balance. b. Assigns new fees master for the new year." "Withdrawal Workflow" steps 3-4: "Library domain receives the event and flags outstanding books. Finance domain receives the event and finalizes balances." "Transfer Workflow" step 5: "Both schools reconcile financial ledgers." No compensating action (refund, rollback, or carry-forward) is implemented for any of these transitions. The spec contains no definition of a saga, no `CompensatingAction` value object, and no `undo_*` command. A failed promotion mid-flight (e.g. finance carry-forward rejects because the next-year `FeesMaster` is missing) leaves the student in the new academic year with no fees assignment and no recorded failure.

**Expected:**

Per-aggregate compensating commands and a saga orchestrator (e.g. `PromotionSaga`, `WithdrawalSaga`) defined in the academic crate.

**Evidence:**

- `grep -rn 'compensat\|Compensat\|saga\|Saga\|undo' crates/domains/` returns zero matches in any domain crate's `services.rs` or `commands.rs`.
  - `docs/specs/academic/workflows.md:24-42` (Promotion Workflow) lists five finance-side steps but no rollback path; `docs/specs/academic/workflows.md:49-58` (Withdrawal Workflow) lists finance + library fan-out with no compensation.

---

### FINDING 31 (id: `WF-031`)

- **Source:** `docs/audit_reports/findings/wave7-workflows.md`
- **Severity:** Low
- **Area:** workflows
- **Location:** `docs/specs/academic/workflows.md:9-17` and `docs/specs/academic/aggregates.md`.

**Description:**

"Admission Workflow" step 1 references `RegisterAdmissionQuery` and step 3 references `ConvertAdmissionQuery`; neither command is in `crates/domains/academic/src/commands.rs` (verified: `grep -n 'RegisterAdmissionQuery\|ConvertAdmissionQuery' crates/domains/academic/src/commands.rs` returns zero output). The Phase 3 hand-off (per `crates/domains/academic/src/lib.rs:13-19`) explicitly defers these aggregates to "later phases": "the remaining 27 academic aggregates in `docs/specs/academic/aggregates.md` (Guardian, ClassSection, ClassSubject, ClassRoutine, Homework, Lesson, LessonTopic, LessonPlan, StudentRecord, StudentPromotion, StudentCategory, StudentGroup, RegistrationField, Certificate, IdCard, AdmissionQuery, etc.) land in later phases." The workflows.md spec, however, is the full spec — the Phase 3 deferral is not documented inline, so a consumer reading only `workflows.md` would expect `RegisterAdmissionQuery` to be callable. The spec file does not mark these workflow steps as "Phase X+".

**Expected:**

Each deferred workflow step annotated with the phase it lands in, or a top-of-file note: "Workflows marked with `[phase=N+]` depend on aggregates that ship in Phase N+."

**Evidence:**

- `docs/specs/academic/workflows.md:11-17`:
    ```text
    1. Receive admission inquiry (RegisterAdmissionQuery)
    2. Schedule follow-up calls (FollowUpAdmissionQuery)
    3. Convert inquiry into student (ConvertAdmissionQuery)
    ```
  - `crates/domains/academic/src/lib.rs:13-19`:
    ```text
    The remaining 27 academic aggregates in
    `docs/specs/academic/aggregates.md` (Guardian,
    ClassSection, ClassSubject, ClassRoutine, Homework,
    Lesson, LessonTopic, LessonPlan, StudentRecord,
    StudentPromotion, StudentCategory, StudentGroup,
    RegistrationField, Certificate, IdCard, AdmissionQuery,
    etc.) land in later phases.
    ```

---

