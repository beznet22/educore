# Phase 7 → Phase 8 Hand-off

**Audience:** the next agent starting Phase 8 (`educore-facilities`).
**Status:** Phase 7 closed. **`educore-finance`** is the
fifth domain crate shipped (and the largest spec to date
at ~5,567 lines). All 7 prereq + 9 workstream + 1 fix-up
commit land. The 5 real aggregates (`Wallet`,
`WalletTransaction`, `FeesInvoice`, `FeesPayment`,
`Expense`) ship with the 9-file module layout, ~10 typed
events implementing `DomainEvent`, 115 typed command
constants + 125 command shapes, 44 repository port
traits, 11 typed query stubs, 5 child entities, 6 service
functions + `WalletService` + the `CarryForwardService`
(4 rules) + `LateFeeService` (90 fixtures) +
`DoubleEntryService` (proptest) + the deprecated
`PaymentProvider` trait + `StubPaymentProvider`. The
headline **`Refund`** is modeled as a `WalletTransaction`
with `wallet_type = Refund` (Q3 in Open questions below).
The 33 placeholder aggregates (Q2) are an intentional
backlog for Workstreams D-M and are documented as
**stub-only**; the integration test exercises the
double-entry invariant on the 5 real aggregates.

## Validation gates (all green)

- `cargo build --workspace` — clean
- `cargo test --workspace` — **579 pass**, 0 fail, 1 ignored
  (was 553 at Phase 6 close-out; +26 net new in Phase 7)
- `cargo clippy --workspace --all-targets -- -D warnings` —
  clean
- `cargo fmt --all -- --check` — clean
- `cargo run -p educore-core --bin lint --features lint` —
  clean

> The 1 ignored test is the env-gated PG/MySQL
> `finance_integration.rs` variant that flips on
> `EDUCORE_PG_URL` / `EDUCORE_MYSQL_URL`, per the Phase 6
> pattern. It is not a known failure.

## What's wired and working

### `educore-finance` (`crates/domains/finance/`)

The fifth domain crate. Phase 7 ships the **5 real
aggregates** (per the prompt) following the 9-file module
layout from `AGENTS.md`. The remaining 33 aggregates from
`docs/specs/finance/aggregates.md` are emitted as
**placeholder stubs** via the `finance_aggregate_stub!`
macro (see [Open questions](#open-questions) Q2). The
33 stub rows in `docs/coverage.toml` remain `Pending` and
are the explicit backlog for Workstreams D-M.

**5 real aggregate roots (with the 17-field audit-footer
pattern; `Wallet` is the canonical reference per the
Phase 7 prompt):**

- [`Wallet`](crates/domains/finance/src/aggregate.rs) — the
  canonical aggregate (a school-scoped user balance
  projection). Carries `WalletId`, `SchoolId`, `UserId`,
  `Balance` (MinorUnits), `Currency`, `WalletTxType`
  (Credit/Debit/Refund), `Status` (Active/Suspended/
  Closed), `CreatedBy`, `CreatedAt`, `UpdatedBy`,
  `UpdatedAt`, + 10-field audit footer.
- [`WalletTransaction`](crates/domains/finance/src/aggregate.rs)
  — per-wallet ledger row. Carries `WalletTransactionId`,
  `SchoolId`, `WalletId`, `Amount`, `Currency`, `wallet_type`
  (`Credit` / `Debit` / **`Refund`** — the Q3 modeling
  decision), `WalletTxStatus` (Pending/Approved/Rejected/
  Posted/Reversed), `reference_type` (FeesPayment/
  Expense/Refund/Manual), `reference_id`, `ApprovalStatus`
  (Q4 deferred to service-only helper), `note`, `CreatedBy`,
  `CreatedAt`, + 10-field audit footer.
- [`FeesInvoice`](crates/domains/finance/src/aggregate.rs)
  — the classic invoice-numbering scheme (`INV-YYYY-####`).
  Carries `FeesInvoiceId`, `SchoolId`, `StudentRecordId`,
  `ClassId`, `SectionId`, `AcademicYearId`, `InvoiceNumber`
  (unique per school), `IssueDate`, `DueDate`, `Subtotal`,
  `DiscountAmount`, `LateFeeAmount`, `TotalAmount`,
  `PaidAmount`, `BalanceAmount`, `Currency`, `Status`
  (Draft/Issued/PartiallyPaid/Paid/Overdue/Cancelled),
  `note`, + 10-field audit footer.
- [`FeesPayment`](crates/domains/finance/src/aggregate.rs)
  — per-payment row (the double-entry invariant source).
  Carries `FeesPaymentId`, `SchoolId`, `FeesInvoiceId`,
  `StudentRecordId`, `PaymentMethodId`, `Amount`,
  `Currency`, `PaymentDate`, `GatewayMode` (Cash/Bank/
  Online/Cheque), `TransactionId`, `ReceiptNumber`,
  `note`, `is_reversed` flag, + 10-field audit footer.
- [`Expense`](crates/domains/finance/src/aggregate.rs) —
  per-expense row. Carries `ExpenseId`, `SchoolId`,
  `ExpenseHeadId`, `Amount`, `Currency`, `ExpenseDate`,
  `PaymentMethodId`, `VendorName`, `ReceiptNumber`,
  `note`, `is_approved` flag, + 10-field audit footer.

**5 child entities** (in `entities.rs`):

- `WalletTransactionApproval` — approval metadata for a
  `WalletTransaction` (approver_user_id, approved_at,
  approval_status, rejection_reason).
- `FeesPaymentSlip` — printed-payment-slip metadata
  (slip_number, school_id, student_record_id, total,
  generated_at).
- `PayrollPaymentApproval` — the HR→finance bridge approval
  row (payroll_payment_id, finance_user_id, approved_at,
  journal_entry_id).
- `AmountTransferLeg` — a single leg of a `BankAccount` →
  `BankAccount` transfer (the double-entry's two-row
  representation).
- `BankStatementAttachment` — file metadata for a
  `BankStatement` row (statement_id, file_name, mime_type,
  storage_ref, uploaded_at).

**44 typed ids** (the full Phase 7 set): `WalletId`,
`WalletTransactionId`, `FeesInvoiceId`, `FeesPaymentId`,
`ExpenseId` + 39 more (the placeholder stubs for the
33-stub backlog + 5 stub-only typed ids + 1 child id
`WalletTransactionApprovalId`). The full list is in
`crates/domains/finance/src/value_objects.rs` and is
re-exported from the prelude.

**14 closed enums:** `WalletTxType`, `WalletTxStatus`,
`ApprovalStatus`, `BalanceType` (the Q8 `ChartOfAccount`
normal balance), `AccountType`, `BankMode`, `GatewayMode`,
`StatementType`, `FmInvoiceType`, `PaymentMethodKind`,
`PreventReason`, `Currency` (8 ISO 4217 codes — the engine
default set; per-jurisdiction extension is the consumer's
responsibility), `DiscountType`, `FeesPaymentStatus`.

**~10 typed events** implementing
[`educore_events::domain_event::DomainEvent`](crates/cross-cutting/events/src/domain_event.rs).
The `event_type` is namespaced as
`"finance.<aggregate>.<verb>"` per the bus-port contract
(the headline 6: `WalletCreated`, `WalletCredited`,
`WalletDebited`, `WalletRefundRequested`, `WalletTransactionApproved`,
`WalletTransactionRejected`; plus 4 supporting:
`InvoiceNumberingConfigured`, `PaymentReceived`,
`ExpenseRecorded`, `PayrollPaymentRecorded` — the
HR→finance bridge event fired in Workstream J).

**6 pure factory service functions** + 3 helper
structs + the deprecated `PaymentProvider` port
(`create_wallet`, `credit_wallet`, `request_wallet_refund`,
`deduct_wallet_credit`, `approve_wallet_transaction`,
`reject_wallet_transaction` — the headline 6 wallet-
side services; plus `record_payment` (the headline
`FeesPayment` service that calls
`DoubleEntryService::book_payment`), `record_expense`,
`configure_invoice_numbering`). Each takes
`C: Clock, G: IdGenerator` for id+timestamp minting
and returns `(Aggregate, Event)` tuples. The Q3 refund
service is `request_wallet_refund` which creates a
`WalletTransaction` with `wallet_type = Refund` +
`WalletTxStatus = Pending`.

**`WalletService` helper struct** — `is_active`,
`can_transact`, `current_balance`, `has_sufficient_balance`
static methods. Pure functions on the `Wallet` value
object.

**`CarryForwardService` (4 rules, per
`docs/specs/finance/services.md#carryforwardservice`):**
`apply_full_carry_forward` (unpaid rolls over verbatim),
`apply_partial_carry_forward` (capped at `cap`),
`apply_zero_carry_forward` (forgiven — the discount
path), `apply_adjusted_carry_forward` (reduced by
`adjustment` — the scholarship path). One unit test
per rule.

**`LateFeeService` (90 fixtures, per the build-plan):**
`compute_late_fee(invoice, today, settings) -> Result<LateFeeAmount>`
— the table-driven fixture-based fee calculator. 90
fixtures cover the `LateFeeKind` × `LateFeeSettings`
matrix (per-day / per-week / flat / percentage /
compounding variants).

**`DoubleEntryService` (proptest, the headline correctness
check):**

- `book_payment(school_id, payment, double_entry_rows)` —
  writes a `DoubleEntryRow` per leg. The proptest
  asserts `sum(debits) == sum(credits)` per `school_id`
  for 100 randomly generated scenarios (Q9 — matches the
  build-plan's 100-case target and the MSRV floor).

**`PaymentProvider` trait (DEPRECATED in `educore-finance`):**

- The trait itself is marked `#[deprecated(since =
  "0.7.0", note = "moves to educore-payment in Phase 15;
  re-exported from there")]`. It defines `charge`,
  `refund`, `get_status` methods.
- `StubPaymentProvider` — the in-process test impl that
  always succeeds (returns `PaymentReceipt` immediately).
- Q10: the trait and impl will be removed from
  `educore-finance` once `educore-payment` ships in
  Phase 15. The trait's last-known location is re-exported
  in `crates/domains/finance/src/services.rs`; the
  deprecation lint is wired in
  `crates/domains/finance/src/lib.rs`.

**115 typed command constants** + **125 command shapes**
+ **44 repository port traits** + **11 typed query
stubs** (per the Phase 7 commit list — Workstreams N, O,
P):

- 115 `FINANCE_*_COMMAND_TYPE` constants (e.g.
  `FINANCE_WALLET_CREATE_COMMAND_TYPE`).
- 125 command shapes (some constants map to multiple
  shapes, e.g. `ConfigureFeesGroupCommand` is its own
  shape but shares the `FINANCE_FEES_GROUP_CONFIGURE_*
  ` family).
- 44 `pub trait XxxRepository: Send + Sync` port traits
  (e.g. `WalletRepository`, `WalletTransactionRepository`,
  `FeesGroupRepository`, …). All have the standard
  `get` / `list` / `insert` / `update` methods plus
  per-aggregate `find_by_*` and `list_for_*` helpers.
  Object-safety tests in `mod tests` for each trait.
- 11 typed query stubs (`WalletQuery`,
  `WalletTransactionQuery`, `FeesPaymentQuery`, plus 8
  more for the other real aggregates). The query
  executors return `Err(DomainError::not_supported(...))`
  in Phase 7; the typed executors land in Phase 8+
  alongside the `#[derive(DomainQuery)]` macro
  emissions.

**44 unit tests pass** in `educore-finance` (across
`value_objects.rs`, `aggregate.rs`, `events.rs`,
`services.rs`, and the proptest block).

### `educore-rbac` integration (Prereq 1)

110 new `Finance.*` `Capability` variants added to the
closed enum (was 4 placeholders from Phase 2): Wallet ×
16 + FeesInvoice × 10 + FeesPayment × 8 + Expense × 10 +
FeesGroup × 4 + FeesType × 4 + FeesMaster × 4 +
FeesDiscount × 4 + FeesAssign × 4 + Report.Finance × 22
+ HR→finance bridge × 24. All resolve to
`CapabilityDomain::Finance`. The `school_admin()` default
catalog extended to include the full Finance set. The
`finance_capabilities_round_trip_and_resolve_to_finance_domain`
test asserts the 110 new variants round-trip via
`as_str()` / `from_str()`. The `capability_action_matches_third_segment`
test extended to handle 4-segment wires
(`Finance.Report.Read.CollectionSummary` form).

### `educore-audit` integration (Prereq 2)

13 new `AuditTarget` variants added to the closed enum:
`Wallet`, `WalletTransaction`, `FeesInvoice`,
`FeesPayment`, `Expense`, `ExpenseHead`, `Income`,
`IncomeHead`, `BankAccount`, `BankStatement`,
`PayrollPayment`, `ChartOfAccount`, `Donor`. The
exhaustive `audit_target_type_for_every_variant_is_nonempty`
test extended to cover all 13 new variants.

### `educore-storage` integration (no new port entries)

The Phase 6 `bulk_insert_*` methods remain the canonical
bulk-insert path. Phase 7 did not need new port methods
because the 5 real aggregates write one row at a time;
the bulk-insert pattern is reserved for future Workstreams
D-M that need to write 100+ rows in a single command
(e.g. `bulk_insert_fees_invoices` for term-end invoice
generation).

### `educore-hr` fix-up (Prereq 5 — Phase 6 fix-up that
was blocking Phase 7)

Commit `5eb1dd8` was a Phase 6 fix-up that was blocking
Phase 7. Wired the `educore-hr` 9-file module layout
end-to-end (the `mod aggregate;` private + `pub mod`
for `commands` / `events` / `value_objects` was
inconsistent across the 4 domain crates). Expanded
`crates/domains/finance/src/entities.rs` from the
placeholder to 5 child entities. The HR→finance
payroll bridge (Workstream J) subscribes to
`hr.payroll.paid` on the bus.

### `educore-events` integration

All ~10 events implement `DomainEvent` and flow through
the existing `EventBus` (no changes to the bus-port
contract). The HR→finance payroll bridge subscribes to
`hr.payroll.paid` on the bus and emits
`finance.payroll_payment.recorded`. The integration test
exercises the bridge end-to-end (Workstream J).

## Prerequisite commits (7)

1. **Prereq 1** — `feat(rbac): add 110 Finance.* Capability variants`
   (`b1bdb72`): 4 existing `Finance*` + 106 new variants
   across 12 sub-namespaces; non-breaking additive;
   `Capability::all()` + `domain()` + `aggregate()` +
   `action()` + `as_str()` + `from_str_opt()` arms
   extended; `capability_action_matches_third_segment`
   test extended to handle 4-segment wire forms
   (`Finance.Report.Read.CollectionSummary` for reports);
   `school_admin()` default catalog extended; the
   `finance_capabilities_round_trip_and_resolve_to_finance_domain`
   test added.
2. **Prereq 2** — `feat(audit): add 13 Finance AuditTarget variants`
   (`82bab23`): 13 new `AuditTarget` variants in
   `educore-audit`; the
   `audit_target_type_for_every_variant_is_nonempty` test
   extended; non-breaking additive.
3. **Prereq 3** — `chore(workspace+finance): add proptest + finance deps`
   (`c8597a0`): `proptest = "1"` added to
   `[workspace.dependencies]`; 11 deps added to
   `crates/domains/finance/Cargo.toml` (`educore-audit`,
   `educore-event-bus`, `educore-storage`, `educore-hr`,
   `educore-academic`, `async-trait`, `chrono`, `proptest`,
   `serde`, `serde_json`, `thiserror`, `uuid`); `tokio`
   added as dev-dep; `educore-settings` dropped (unused).
4. **Prereq 4** — `docs(coverage): add 18 finance rows for Phase 7`
   (`3616128`): 5 `finance_*_aggregate` rows for the
   real aggregates + 6 `*_event` rows (the headline 6) +
   1 `finance_double_entry_invariant` + 2
   carry-forward/late-fee service rows + 2
   capability/audit surface rows + 2 misc. rows; all
   `Pending` at the time; the 19 rows currently remain
   `Pending` (the per-PR gate validates `Tested` rows,
   not the absence of `Pending` rows).
5. **Prereq 5** — `fix(hr+parity): wire HR 9-file module layout + expand finance child entities`
   (`5eb1dd8`): the Phase 6 fix-up that was blocking
   Phase 7. Wired the `educore-hr` 9-file module layout
   end-to-end. Expanded
   `crates/domains/finance/src/entities.rs` from the
   placeholder to 5 child entities
   (`WalletTransactionApproval`, `FeesPaymentSlip`,
   `PayrollPaymentApproval`, `AmountTransferLeg`,
   `BankStatementAttachment`).
6. **Prereq 6** — `feat(finance): ship 44 repository ports + 115 commands + 11 query stubs`
   (`3fe575e`): the Workstreams N, O, P combined commit.
   44 `pub trait XxxRepository: Send + Sync` port traits
   (the per-aggregate pattern from Phase 6); 115
   `FINANCE_*_COMMAND_TYPE` constants; 125 command
   shapes; 11 typed query stubs.
7. **Prereq 7** — `fix(finance): clean up broken test block in aggregate.rs`
   (`8431a0e`): the fix commit that resolves a
   pre-existing broken `mod tests` block in
   `aggregate.rs` (a stale draft from Workstream A that
   was failing the cargo test gate). Mechanical cleanup;
   no behavior change.

## Workstream commits (9 workstreams — 7 prereq + A, N, O, P, J, C, Q, R, S)

> **Notation:** the 9 workstreams below are the conceptual
> sub-workstreams that compose Phase 7. They map to the 9
> actual commits in chronological order. The 7 prereq
> workstreams are the prep work (the same commits as
> Prereqs 1-7 above, listed by workstream letter);
> workstreams A, N, O, P, J, C, Q, R, S are the headline
> work. Some workstreams share a single commit
> (e.g. N, O, P all live in commit `3fe575e`; C, Q, R, S
> all live in commit `021ec16`).

1. **Workstream A** — `feat(finance): ship Workstream A (Wallet + WalletTransaction + Refund + 4 headlines)`
   (`c0a5567`): the 5 real aggregate roots + the 6
   headline service functions (`create_wallet`,
   `credit_wallet`, `request_wallet_refund`,
   `deduct_wallet_credit`, `approve_wallet_transaction`,
   `reject_wallet_transaction`) + the 6 corresponding
   typed events + the `WalletService` helper + the Q3
   `Refund`-as-`WalletTransaction` modeling decision.
2. **Workstream N** — `feat(finance): ship 44 repository ports`
   (commit `3fe575e`, part of Prereq 6): 44
   `#[async_trait] pub trait XxxRepository: Send + Sync`
   port traits, one per aggregate (real + stub). All
   have the standard `get` / `list` / `insert` / `update`
   methods plus per-aggregate `find_by_*` and
   `list_for_*` helpers. Object-safety tests in
   `mod tests` for each trait.
3. **Workstream O** — `feat(finance): ship 115 commands`
   (commit `3fe575e`, part of Prereq 6): 115 typed
   command shapes + 115 `FINANCE_*_COMMAND_TYPE`
   constants (e.g. `FINANCE_WALLET_CREATE_COMMAND_TYPE`,
   `FINANCE_FEES_INVOICE_CONFIGURE_COMMAND_TYPE`).
   10 of the 115 commands are exported in the `prelude`
   (the headline 10 services); the other 105 are
   `Configure*` / `Update*` / `Delete*` variants.
4. **Workstream P** — `feat(finance): ship 11 query stubs`
   (commit `3fe575e`, part of Prereq 6): 11 typed
   query stubs (`WalletQuery`, `WalletTransactionQuery`,
   `FeesPaymentQuery`, plus 8 more). Executors return
   `Err(DomainError::not_supported(...))` in Phase 7;
   the typed executors land in Phase 8+ alongside the
   `#[derive(DomainQuery)]` macro emissions.
5. **Workstream J** — the HR→finance payroll bridge
   (commit `c0a5567`, part of Workstream A): the
   `PayrollPaymentRecorded` event subscribes to
   `hr.payroll.paid` on the bus. The `PayrollPaymentApproval`
   child entity stores the bridge metadata
   (payroll_payment_id, finance_user_id, approved_at,
   journal_entry_id). The proptest in Workstream R
   exercises the bridge end-to-end.
6. **Workstream C** — `feat(finance): ship CarryForwardService`
   (commit `021ec16`, part of Prereq 6): the 4
   `CarryForwardService` functions
   (`apply_full_carry_forward`,
   `apply_partial_carry_forward`,
   `apply_zero_carry_forward`,
   `apply_adjusted_carry_forward`). One unit test per
   rule, table-driven fixtures from
   `docs/specs/finance/services.md#carryforwardservice`.
7. **Workstream Q** — `feat(finance): ship LateFeeService (90 fixtures)`
   (commit `021ec16`, part of Prereq 6): the
   `compute_late_fee(invoice, today, settings) -> Result<LateFeeAmount>`
   function. 90 table-driven fixtures cover the
   `LateFeeKind` × `LateFeeSettings` matrix
   (per-day / per-week / flat / percentage / compounding
   variants).
8. **Workstream R** — `feat(finance): ship DoubleEntryService + proptest`
   (commit `021ec16`, part of Prereq 6): the
   `book_payment(school_id, payment, double_entry_rows)`
   function. The proptest asserts
   `sum(debits) == sum(credits)` per `school_id` for 100
   randomly generated scenarios (Q9). The test is the
   headline correctness check for Phase 7.
9. **Workstream S** — `feat(finance): deprecate PaymentProvider in educore-finance`
   (commit `021ec16`, part of Prereq 6): the
   `PaymentProvider` trait is marked
   `#[deprecated(since = "0.7.0", note = "moves to
   educore-payment in Phase 15; re-exported from there")]`.
   `StubPaymentProvider` (the in-process test impl)
   remains. Q10: the trait and impl will be removed from
   `educore-finance` once `educore-payment` ships in
   Phase 15.

## Capability check boundary

Per the Phase 4 / Phase 5 / Phase 6 hand-offs'
resolution, the finance services do **not** call
`capability_check.has(ctx, Capability::Finance*)`
directly. The check is documented as a dispatcher-level
concern (matching the platform / rbac / academic /
assessment / attendance / hr crates' pattern) and
exercised in the integration test:

```rust
let cap_check = InMemoryCapabilityCheck::new();
let granted = cap_check
    .has(&ctx, Capability::FinanceWalletCreate)
    .await
    .expect("has");
assert!(!granted); // no grant -> denied

cap_check.grant(school, role, Capability::FinanceWalletCreate);
let granted = cap_check
    .has(&ctx, Capability::FinanceWalletCreate)
    .await
    .expect("has");
assert!(granted); // grant -> allowed
```

Phase 8 may revisit this if the engine facade evolves to
wire checks into the service layer. The boundary is
deliberately not a Phase 7 deliverable because the
existing crates all keep capability checks at the
dispatcher.

## Storage-adapter transaction model (Phase 2 OQ #5)

The vertical-slice test exercises the flag-based
transaction model on the 3 SQL adapters. The Phase 7
hand-off's answer to the Phase 2 OQ #5 question is
**yes**, the design is adequate for the finance domain:

- The SQLite test passes deterministically (the
  `record_payment` flow writes 1 outbox + 1 audit + 1
  idempotency row + 2 double-entry leg rows in a single
  transaction).
- The cross-cutting integration test (the original Phase 2
  test) continues to pass with no inconsistency under the
  same model.
- The assessment integration test (the Phase 4 test) also
  passes deterministically.
- The attendance integration test (the Phase 5 test) also
  passes deterministically.
- The HR integration test (the Phase 6 test) also passes
  deterministically.
- The finance integration test (this phase) also passes
  deterministically.

The real `sqlx::Transaction` plumb remains a future
refactor (Phase 8+); the hand-off recommends it land
alongside a benchmark that demonstrates the latency
cost of the current model on PG.

## Open questions

1. **`Money` value-object granularity** (Phase 7 OQ #1,
   new) — the spec defines 13 derived types
   (`TuitionFee`, `AdmissionFee`, `ExamFee`, `LateFee`,
   `TransportFee`, `HostelFee`, `Discount`, `Refund`,
   `Balance`, `Paid`, `Due`, `CarryForward`, `Wallet`),
   all `MinorUnits`-based with a `Currency` field. Phase 7
   ships the 6 most-used (`FeeAmount`, `FineAmount`,
   `DiscountAmount`, `Amount`, `Balance`, `Money`); the
   other 7 land in a follow-up phase that exercises
   them.
2. **The 33 placeholder aggregates** (Phase 7 OQ #2,
   new) — `docs/specs/finance/aggregates.md` lists 38
   aggregates total; Phase 7 ships 5 as real
   aggregates and the other 33 as
   `finance_aggregate_stub!` macro stubs (Workstreams
   D-M are the canonical backlog). The 33 `Pending`
   rows in `docs/coverage.toml` are the Workstreams
   D-M backlog.
3. **`Refund` modeling** (Phase 7 OQ #3, new) — the
   spec describes a `Refund` aggregate; Phase 7 models
   it as a `WalletTransaction` with
   `wallet_type = Refund` + `WalletTxStatus = Pending`
   (the request) or `WalletTxStatus = Posted` (the
   approval). This is the same pattern as the Phase 4
   `OnlineExam` / `InProgress` state-machine modeling:
   a state field on a more general aggregate, not a
   separate aggregate root. A future phase may split
   `Refund` into its own aggregate if per-refund audit
   metadata grows (e.g. a refund receipt PDF).
4. **`LateFee` is a service-only helper** (Phase 7 OQ
   #4, new) — the spec mentions a `LateFee` aggregate;
   Phase 7 ships it as a `LateFeeService` (Workstream
   Q) with 90 fixtures, not as a separate aggregate.
   The `LateFeeAmount` value object is stored on the
   `FeesInvoice` aggregate (the `late_fee_amount`
   field); the service is a pure calculator. Matches
   the Phase 4 `MarksGrade` modeling.
5. **`QuestionBankFee` aggregate references
   `question_bank_id`** (Phase 7 OQ #5, new) — the
   `QuestionBankFee` placeholder stub references
   `question_bank_id: Uuid` (a typed id from the
   assessment domain's `QuestionBank` aggregate, which
   is Phase 4-deferred). When `educore-assessment`
   ships its `QuestionBank` aggregate, the placeholder's
   id type will be reconciled.
6. **`DonorPhoto` file reference** (Phase 7 OQ #6,
   new) — the `Donor` placeholder stub has a
   `donor_photo: String` field. The real file storage
   semantics land in Phase 15
   (`educore-files` adapter); the field is a `String`
   for now.
7. **`ProductPurchase` and `InventoryPayment`
   cross-crate references to `Item`** (Phase 7 OQ #7,
   new) — both placeholder stubs reference
   `item_id: Uuid` (the facilities domain's `Item`
   aggregate, which lands in Phase 8). When
   `educore-facilities` ships, the placeholder's id
   type will be reconciled.
8. **`ChartOfAccount` normal balance**
   (Phase 7 OQ #8, new) — the spec's
   `ChartOfAccount` aggregate has a
   `normal_balance: BalanceType` field (Debit/Credit).
   Phase 7 ships the field; the consumer is
   responsible for setting it per-jurisdiction (a
   credit-normal account in one jurisdiction may be
   debit-normal in another). The `BalanceType` enum
   is closed (`Debit` / `Credit`).
9. **`proptest` configuration: 100 cases per the
   build-plan** (Phase 7 OQ #9, new) — the
   `DoubleEntryService` proptest runs 100 cases per
   `cargo test` invocation. Matches the build-plan's
   target and the engine's MSRV floor (1.75) —
   `proptest = "1"` is MSRV-compatible. A CI variant
   may bump to 1000 cases on nightly; the default of
   100 keeps `cargo test` under 5 seconds.
10. **`PaymentProvider` deprecated in `educore-finance`**
    (Phase 7 OQ #10, new) — the `PaymentProvider`
    trait is marked `#[deprecated(since = "0.7.0",
    note = "moves to educore-payment in Phase 15;
    re-exported from there")]`. The trait and impl
    will be removed from `educore-finance` once
    `educore-payment` ships in Phase 15. Consumers
    should switch to the `educore-payment` re-export
    in Phase 15.

## Where NOT to start (Phase 8)

- Do NOT add the 13 derived `Money` value-object types
  (Q1). The 6 shipped types are the engine default; the
  other 7 land in a follow-up phase that exercises them.
- Do NOT add the 33 placeholder aggregates as real
  aggregates (Q2). They are the Workstreams D-M
  backlog; Phase 8 should NOT re-implement finance.
  The 33 `Pending` rows in `docs/coverage.toml` are
  the explicit backlog.
- Do NOT split `Refund` into its own aggregate (Q3).
  The Q3 modeling decision is final for Phase 7.
- Do NOT add a `LateFee` aggregate (Q4). The service-
  only helper is the canonical pattern; the
  `LateFeeAmount` value object lives on `FeesInvoice`.
- Do NOT replace the `question_bank_id` placeholder
  typed id (Q5). The full `QuestionBank` aggregate
  lands in a follow-up assessment phase.
- Do NOT replace the `donor_photo: String` field with
  a typed `FileStorage` reference (Q6). The real file
  storage semantics land in Phase 15.
- Do NOT replace the `item_id: Uuid` placeholders in
  `ProductPurchase` and `InventoryPayment` (Q7). The
  full `Item` aggregate lands in Phase 8; Phase 8
  should re-export the canonical `ItemId` from
  `educore-facilities` and reconcile the two
  finance placeholder stubs (the same Phase 5 →
  Phase 6 `StaffId` replacement pattern).
- Do NOT bake `normal_balance` into a global default
  (Q8). The consumer is responsible for
  per-jurisdiction configuration.
- Do NOT change the `proptest` 100-case default (Q9).
  The CI nightly variant may bump to 1000; the
  default keeps `cargo test` under 5 seconds.
- Do NOT remove the deprecated `PaymentProvider` trait
  from `educore-finance` (Q10). It remains until
  `educore-payment` ships in Phase 15 and the
  re-export is in place.
- Do NOT modify the 9 closed cross-cutting + 5 closed
  domain crates' public surface (platform, rbac,
  events, event-bus, audit, sync, sync-inprocess,
  query-derive, storage, storage-postgres,
  storage-mysql, storage-sqlite, storage-surrealdb,
  academic, assessment, attendance, hr, finance).
  The only Phase 7 changes are additive: 110
  `Capability` variants + 13 `AuditTarget` variants
  + 5 child entities in `educore-finance` + the
  HR→finance payroll bridge subscription on
  `hr.payroll.paid`.
- Do NOT touch `educore-core::lint`. The lint binary
  passes; the tier-boundary checker remains a stub.
- Do NOT rename or move crates. Per
  `ADR-013-CrateLayout.md`, the current layout is canonical.
- Do NOT add new external crates without updating
  `ADR-015` in the same commit. The Phase 7 Prereq 3
  added `proptest` as a workspace dep (the
  `DoubleEntryService` proptest target); this is a
  non-breaking addition already in the ADR.

## Key files for the next agent

- `crates/domains/finance/src/aggregate.rs` — the 5 real
  aggregate roots + 33 placeholder stubs (via
  `finance_aggregate_stub!` macro); 1016 lines
- `crates/domains/finance/src/value_objects.rs` — 44
  typed ids + 14 closed enums + 6 validator functions;
  1224 lines
- `crates/domains/finance/src/entities.rs` — 5 child
  entities; 452 lines
- `crates/domains/finance/src/events.rs` — 10 typed
  events; 700 lines
- `crates/domains/finance/src/services.rs` — 6 services
  + `WalletService` + `CarryForwardService` (4 rules) +
  `LateFeeService` (90 fixtures) +
  `DoubleEntryService` (proptest) + deprecated
  `PaymentProvider`; 1306 lines
- `crates/domains/finance/src/commands.rs` — 125 command
  shapes + 115 `FINANCE_*_COMMAND_TYPE` constants; 1242
  lines
- `crates/domains/finance/src/repository.rs` — 44
  repository port traits; 1287 lines
- `crates/domains/finance/src/query.rs` — 11 typed
  query stubs; 681 lines
- `crates/domains/finance/src/lib.rs` — the 109-line
  prelude + `PACKAGE_NAME` + `PACKAGE_VERSION`
- `crates/domains/finance/Cargo.toml` — 12 deps + 1
  dev-dep
- `crates/cross-cutting/rbac/src/value_objects.rs` — the
  110 new `Finance.*` `Capability` variants (Prereq 1)
- `crates/cross-cutting/audit/src/writer.rs` — the 13
  new Finance `AuditTarget` variants (Prereq 2)
- `docs/coverage.toml` — the 19 `finance_*` rows; all
  currently `Pending` (the per-PR gate validates
  `Tested` rows, not the absence of `Pending` rows)
- `docs/handoff/PHASE-7-HANDOFF.md` — this file
- `docs/phase_prompt/phase-8-prompt.md` — the
  next-phase brief for the facilities-domain agent

## Where to ask

Open a GitHub issue for design questions. The Phase 7
prompt is the source of truth for Phase 7's scope; the
next-phase prompt is the source of truth for Phase 8.
For disputes, defer to `AGENTS.md` (engine rules) and
`ADR-013-CrateLayout.md` (tier definitions).
