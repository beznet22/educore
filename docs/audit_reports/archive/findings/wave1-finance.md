## Wave 1 Finance Domain Audit Report

**Scope:** `crates/domains/finance/`, `docs/specs/finance/`, `docs/commands/finance.md`, `docs/events/finance.md`, `docs/handoff/PHASE-7-HANDOFF.md`, `AGENTS.md` (the finance row).

**Total findings:** 85

### FINDING DOMAIN-FIN-001

- **id:** DOMAIN-FIN-001
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/finance/src/aggregate.rs:650-829` (39 macro stubs)
- **description:** 47 of the 52 aggregates declared in `docs/specs/finance/aggregates.md` are emitted as empty placeholder stubs via the `finance_aggregate_stub!` macro and contain only a `_id: ()` field plus `school_id`. Only 5 aggregates (`Wallet`, `WalletTransaction`, `FeesInvoice`, `FeesPayment`, `Expense`) are real.
- **expected:** "Every aggregate in `docs/specs/finance/aggregates.md` has a Rust struct + tests" (Phase 7 exit criterion #1 in `docs/build-plan.md:914-915`). All 38 root aggregates (e.g. `FeesGroup`, `FeesMaster`, `BankAccount`, `BankStatement`, `Donor`, `Income`, `PayrollPayment`, `ChartOfAccount`, `QuestionBankFee`, `FmFeesInvoice`) should be first-class structs with fields, state machines, and tests.
- **evidence:** Spec lists 38 aggregates in `docs/specs/finance/aggregates.md:3-1569`. Code emits 39 macro stubs at `aggregate.rs:649-829`, e.g. `pub struct FeesGroup { _id: () }`, `pub struct BankAccount { _id: () }`, etc. The handoff acknowledges this as "the intentional Workstreams D-M backlog" (`PHASE-7-HANDOFF.md:500-507`).

### FINDING DOMAIN-FIN-002

- **id:** DOMAIN-FIN-002
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/finance/src/aggregate.rs:424-460` (`FeesPayment` struct)
- **description:** `FeesPayment` is missing the `assign_id` (or `fees_assign_id`), `student_id`, `record_id`, `slip_id`, `receipt_number`, and `is_reversed` fields that the spec mandates. The spec defines a payment as "A single payment against a `FeesAssign` (or a `FeesInstallmentAssign`) ... captures the amount, mode, slip reference, discount applied, and fine paid at the time of payment" (`docs/specs/finance/aggregates.md:316-318`).
- **expected:** Spec mandates `fees_assign_id: FeesAssignId`, `student_id: StudentId`, `record_id: StudentRecordId` per `docs/specs/finance/events.md:166-182` (`PaymentReceived` event payload). Spec also requires `payment_mode` (PaymentMethodId), `slip` (Option<SlipReference>), and an `is_reversed` flag for reversal tracking.
- **evidence:** `aggregate.rs:424-460` defines only `id, school_id, amount_minor, currency, discount_minor, fine_minor, payment_method, bank_id, payment_method_id, reference, note, payment_date, version, etag, ...`. `events.rs:423-435` (`PaymentReceived`) is missing `assign_id`, `student_id`, `record_id`, `slip`, `transaction_id`, `note`. The `record_payment` service (`services.rs:435-481`) takes no assignment reference.

### FINDING DOMAIN-FIN-003

- **id:** DOMAIN-FIN-003
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/commands.rs:267-1243` (event section absent)
- **description:** Approximately 105 of the 110+ events declared in `docs/specs/finance/events.md` and `docs/events/finance.md` are missing from `events.rs`. Only 10 of ~110+ spec events are implemented.
- **expected:** All event types from the spec table in `docs/events/finance.md:17-188` (e.g. `FeesGroupCreated`, `FeesAssignedToClass`, `PaymentReversed`, `BankAccountOpened`, `BankStatementRecorded`, `FundsTransferred`, `FmFeesInvoiceGenerated`, `PayrollGenerated`, `DonorRegistered`, `TransactionRecorded`, etc.) must be defined.
- **evidence:** `events.rs:1-700` defines only `WalletCreated`, `WalletCredited`, `WalletDebited`, `WalletRefundRequested`, `WalletTransactionApproved`, `WalletTransactionRejected`, `InvoiceNumberingConfigured`, `PaymentReceived`, `ExpenseRecorded`, `PayrollPaymentRecorded`. The spec catalog (`docs/events/finance.md`) lists 110 events.

### FINDING DOMAIN-FIN-004

- **id:** DOMAIN-FIN-004
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/commands.rs` (multiple sections)
- **description:** ~30 command shapes specified in `docs/specs/finance/commands.md` are missing from `commands.rs`. Critical missing commands include `AssignFeesToClassCommand`, `AssignFeesToStudentCommand`, `UpdateFeesAssignDiscountCommand`, `CloseFeesAssignCommand`, `PayInvoiceCommand`, `PayInstallmentCommand`, `ConfigureDirectFeesInstallmentCommand`, `AssignDirectInstallmentCommand`, `PayDirectInstallmentCommand`, `ConfigureDirectFeesCommand`, `ConfigureFeesReminderCommand`, `RecordBankStatementCommand`, `GenerateBankPaymentSlipCommand`, `ApproveBankPaymentCommand`, `RejectBankPaymentCommand`, `TransferFundsCommand`, `RecordIncomeCommand`, `RegisterDonorCommand`, `UpdateDonorCommand`, `DeleteDonorCommand`, `AddWalletCreditCommand`, `RecordPayrollPaymentCommand`, `RecordInventoryPaymentCommand`, `RecordProductPurchaseCommand`, `RecordProductPaymentCommand`, `ConfigureInvoiceSettingsCommand`, `ConfigurePaymentGatewayCommand`, `AttachFeesToQuestionBankCommand`, `CreateChartOfAccountCommand`, `CreateSalaryTemplateCommand`, `SetHourlyRateCommand`, `AddFeesInstallmentCreditCommand`, `ConsumeFeesInstallmentCreditCommand`.
- **expected:** Spec defines 65+ commands in `docs/specs/finance/commands.md:10-988` (e.g. `CreateFeesGroupCommand`, `AssignFeesToClassCommand`, `PayInvoiceCommand`, etc.). The commands catalog at `docs/commands/finance.md:15-128` lists the full set.
- **evidence:** Code defines ~50 command structs in `commands.rs:267-1243`. Many spec commands (`AssignFeesToClassCommand`, `PayInvoiceCommand`, `TransferFundsCommand`, etc.) have no matching struct.

### FINDING DOMAIN-FIN-005

- **id:** DOMAIN-FIN-005
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/value_objects.rs` (missing types)
- **description:** ~50 value object types declared in `docs/specs/finance/value-objects.md` are missing from `value_objects.rs`. Examples: `WeaverAmount`, `ServiceCharge`, `TotalEarning`, `TotalDeduction`, `GrossSalary`, `NetSalary`, `BasicSalary`, `Tax`, `HourlyRate`, `OvertimeRate`, `FeePercentage`, `DiscountPercentage`, `TaxPercentage`, `ServiceChargeType`, `PerThousand`, `RoundingPolicy`, `InvoiceNumber`, `InvoicePrefix`, `InvoiceStartForm`, `ReceiptNumber`, `ReferenceNumber`, `SlipReference`, `InvoiceType`, `InvoicePosition`, `InvoiceCopy`, `SignatureSlot`, `PayrollStatus`, `GatewayName`, `PaymentDirection`, `BankAccountNumber`, `IfscCode`, `ChequeNumber`, `TransactionId`, `BankName`, `BranchName`, `AccountHolderName`, `OpeningBalance`, `CurrentBalance`, `DiscountCode`, `DiscountName`, `PayPeriod`, `EarnDeducType`, `PayrollNote`, `SalaryGrade`, `HouseRent`, `ProvidentFund`, `CarryForwardAmount`, `FeesDueDays`, `CarryForwardTitle`, `DaysBeforeDue`, `NotificationChannel`, `ReminderTitle`, `BlockSource`, `CreditAmount`, `CreditStatus`, `QuestionBankType`, `QuestionBankStatus`, `PaymentType`, `AccountDirection`, `FmPaymentType`, `DonorName`, `DonorProfession`, `DonorAddress`, `ShowPublic`, `ProductPackage`, `ExpiryDate`, `PurchaseDate`, `ItemReceiveId`, `ItemSellId`, `StatementDetails`, `AfterBalance`.
- **expected:** All value objects in spec `docs/specs/finance/value-objects.md:12-258` must be implemented as typed wrappers (per AGENTS.md "Compile-time safety over strings").
- **evidence:** `value_objects.rs:1-1225` implements only `Currency`, `Money`, `Amount`, `FeeAmount`, `FineAmount`, `DiscountAmount`, `Balance`, 10 enums, and 6 validator functions. Spec table lists ~80 value objects total.

### FINDING DOMAIN-FIN-006

- **id:** DOMAIN-FIN-006
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/entities.rs:1-453`
- **description:** 32 of 37 child entities declared in `docs/specs/finance/entities.md` are missing. Only 5 are implemented (`WalletTransactionApproval`, `FeesPaymentSlip`, `PayrollPaymentApproval`, `AmountTransferLeg`, `BankStatementAttachment`).
- **expected:** Spec defines 37 child entities at `docs/specs/finance/entities.md:5-303` (e.g. `FeesPaymentFine`, `FeesInstallmentAssignDiscount`, `DirectFeesInstallmentAssignChild`, `FmFeesInvoiceLineNote`, `FmFeesTransactionLineNote`, `BankPaymentSlipAudit`, `ExpenseApproval`, `IncomeApproval`, `DirectFeesInstallmentDueLog`, `FeesAssignClosure`, `FeesAssignDiscountApplication`, `DonorPhoto`, `DonorCustomField`, `ProductPurchasePayment`, `InventoryPaymentReference`, `ChartOfAccountBalance`, `QuestionBankFeeMapping`, `PaymentGatewayMode`, `FeesReminderDispatch`, `DirectFeesSettingOverride`, `FeesInstallmentCreditApplication`, `BankPaymentSlipCounter`, `FeesDiscountEligibility`, `PayrollEarningType`, `PayrollDeductionType`, `LeaveDeductionInfo`, `HourlyRateRow`, `InventoryItemReceive`, `InventoryItemSell`, `BankStatementGroup`, `PayrollPaymentReceipt`, `QuestionBankMuOption`).
- **evidence:** `entities.rs:1-453` implements only 5 child entities. The spec lists 37. 32 are missing.

### FINDING DOMAIN-FIN-007

- **id:** DOMAIN-FIN-007
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/services.rs` (missing services)
- **description:** 11 of the 13 domain services declared in `docs/specs/finance/services.md` are missing. Only `WalletService`, `LateFeeService`, `CarryForwardService`, `DoubleEntryService` exist. Missing: `FeesMasterService`, `InvoiceGenerationService`, `PaymentService`, `InstallmentService`, `PayrollCalculationService`, `BankReconciliationService`, `DiscountService`, `InvoiceNumberingService`, `ReminderDispatchService`, `BankSlipService`, `AccountClosingService`, `ChartOfAccountService`, `FinanceCoordinator`.
- **expected:** All services from `docs/specs/finance/services.md:5-314` (13 service structs: `FeesMasterService`, `InvoiceGenerationService`, `PaymentService`, `InstallmentService`, `CarryForwardService`, `PayrollCalculationService`, `BankReconciliationService`, `DiscountService`, `WalletService`, `InvoiceNumberingService`, `ReminderDispatchService`, `BankSlipService`, `AccountClosingService`, `ChartOfAccountService`).
- **evidence:** `services.rs:1-1307` implements only `WalletService`, `LateFeeService`, `CarryForwardService`, `DoubleEntryService`. The spec mandates 13+ service structs.

### FINDING DOMAIN-FIN-008

- **id:** DOMAIN-FIN-008
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/services.rs:317-365` (`approve_wallet_transaction`)
- **description:** The `approve_wallet_transaction` service approves the `WalletTransaction` state machine but does NOT apply the credit/debit to the actual `Wallet` aggregate. The doc comment at lines 313-316 explicitly says "The caller is responsible for applying the credit/debit to the `Wallet` aggregate". This means the engine does not provide a helper to atomically approve-and-apply, creating a window between approval and balance update where double-spend can occur.
- **expected:** Per `docs/specs/finance/workflows.md:283-294` ("Wallet Credit" workflow), step 4 should be atomic: "The system credits the wallet and emits WalletTransactionApproved." The service should perform the credit/debit on the wallet alongside the state transition.
- **evidence:** `services.rs:317-338` only calls `tx.approve(approver, now, event_id)?` and returns the event. It does not call `wallet.apply_credit()` or `wallet.apply_debit()`. Aggregate comment at line 313-316 confirms this gap: "The caller is responsible for applying the credit/debit to the `Wallet` aggregate".

### FINDING DOMAIN-FIN-009

- **id:** DOMAIN-FIN-009
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/finance/src/` (entire crate — no subscriber)
- **description:** The academic domain's Promotion workflow requires Finance to subscribe to `StudentAdmitted` and `StudentPromoted` to create `FeesAssign` rows and carry-forward prior-year balances (`docs/specs/academic/workflows.md:13-14`: "Auto-create fees assignment (Finance subscribes to StudentAdmitted)"). No such subscriber exists in the finance crate or anywhere in the codebase.
- **expected:** `docs/specs/academic/workflows.md:13-14` mandates "Auto-create fees assignment (Finance subscribes to StudentAdmitted)". `docs/specs/finance/overview.md:156-175` requires Finance to handle `StudentAdmitted`, `StudentPromoted`, `StudentWithdrawn` events.
- **evidence:** `grep -rn "StudentAdmitted\|StudentPromoted" crates/domains/finance/src/` returns no matches. The `educore-event-bus` dependency is declared in `Cargo.toml:17` but never used (`grep "educore_event_bus" crates/domains/finance/src/` returns nothing). The handoff (`PHASE-7-HANDOFF.md:387-394`) mentions "HR→finance payroll bridge subscribes to `hr.payroll.paid` on the bus" but no actual subscriber code is present.

### FINDING DOMAIN-FIN-010

- **id:** DOMAIN-FIN-010
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/query.rs` (entire file)
- **description:** All 11 query stubs return `Err(DomainError::not_supported(...))`. No actual query execution is implemented. Per the file's doc comment at line 4, this is "a Phase 7 stub" that defers to Phase 17, but no test exercises a successful query path.
- **expected:** Queries should compile and execute against a repository; the `#[derive(DomainQuery)]` macro is documented as the path for production queries (`docs/specs/finance/repositories.md:5-7`).
- **evidence:** `query.rs:46-52` (`WalletQuery::execute`), `82-86` (`WalletTransactionQuery::execute`), `111-115` (`FeesPaymentQuery::execute`), `184-188` (`FeesInvoiceQuery::execute`), `258-263` (`ExpenseQuery::execute`), `333-338` (`IncomeQuery::execute`), `407-412` (`BankStatementQuery::execute`), `475-479` (`PayrollPaymentQuery::execute`), `541-547` (`FeesCarryForwardQuery::execute`), `601-606` (`BankAccountQuery::execute`), `676-681` (`TransactionQuery::execute`) all return `Err(DomainError::not_supported(...))`.

### FINDING DOMAIN-FIN-011

- **id:** DOMAIN-FIN-011
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/commands.rs:267-289` (`CreateFeesGroupCommand`)
- **description:** `CreateFeesGroupCommand` is missing the `start_date`, `end_date`, and `due_date` fields mandated by the spec. The spec requires all four fields, with pre-conditions "start_date <= due_date <= end_date".
- **expected:** `docs/specs/finance/commands.md:14-22`: `pub struct CreateFeesGroupCommand { pub tenant: TenantContext, pub name: String, pub description: Option<String>, pub start_date: NaiveDate, pub end_date: NaiveDate, pub due_date: NaiveDate }`.
- **evidence:** `commands.rs:267-272`: only has `tenant`, `name`, `description`. The fields are present in the duplicate `ConfigureFeesGroupCommand` at `commands.rs:1196-1204` but the canonical `CreateFeesGroupCommand` is missing them.

### FINDING DOMAIN-FIN-012

- **id:** DOMAIN-FIN-012
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/commands.rs:299-307` (`CreateFeesTypeCommand`)
- **description:** `CreateFeesTypeCommand` includes extra fields `amount_minor: i64` and `currency: Currency` that are not in the spec, and the spec's `fees_type_id` reference is missing.
- **expected:** `docs/specs/finance/commands.md:42-47`: `pub struct CreateFeesTypeCommand { pub tenant: TenantContext, pub fees_group_id: FeesGroupId, pub name: String, pub description: Option<String> }`.
- **evidence:** `commands.rs:299-307` has 6 fields instead of 5; the extra `amount_minor` and `currency` are not spec-defined.

### FINDING DOMAIN-FIN-013

- **id:** DOMAIN-FIN-013
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/commands.rs:332-340` (`CreateFeesMasterCommand`)
- **description:** `CreateFeesMasterCommand` is missing the `fees_type_id`, `section_id`, and `academic_id` fields mandated by the spec. The spec requires a master to be uniquely keyed by `(fees_group_id, fees_type_id, class_id, section_id?, academic_id)`.
- **expected:** `docs/specs/finance/commands.md:55-66`: `pub struct CreateFeesMasterCommand { pub tenant: TenantContext, pub fees_group_id: FeesGroupId, pub fees_type_id: FeesTypeId, pub class_id: ClassId, pub section_id: Option<SectionId>, pub academic_id: AcademicYearId, pub amount: FeeAmount, pub due_date: Option<NaiveDate> }`. The invariant at `docs/specs/finance/aggregates.md:87-88` requires all 5 identity fields.
- **evidence:** `commands.rs:332-340` has only `fees_group_id`, `class_id`, `amount_minor`, `currency`, `due_date`. Missing `fees_type_id`, `section_id`, `academic_id`.

### FINDING DOMAIN-FIN-014

- **id:** DOMAIN-FIN-014
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/finance/src/commands.rs:341-348` (`UpdateFeesMasterCommand`)
- **description:** `UpdateFeesMasterCommand` is missing the `new_amount: FeeAmount` semantics — the spec mandates a distinct `UpdateFeesMasterAmountCommand` with that field.
- **expected:** `docs/specs/finance/commands.md:74-82`: `pub struct UpdateFeesMasterAmountCommand { pub tenant: TenantContext, pub fees_master_id: FeesMasterId, pub new_amount: FeeAmount }` with effects "Emits `FeesMasterAmountUpdated`."
- **evidence:** `commands.rs:341-348` defines a generic `UpdateFeesMasterCommand` without the `new_amount` field. The amount-update command is not present.

### FINDING DOMAIN-FIN-015

- **id:** DOMAIN-FIN-015
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/finance/src/aggregate.rs:530-618` (`Expense` aggregate)
- **description:** `Expense` aggregate stores an `Option<PayrollPaymentId>` field (`aggregate.rs:556`) but has no referential integrity check that the payroll payment exists or is paid. Per spec invariant 3 of `docs/specs/finance/aggregates.md:1065-1083`, "A payment creates a corresponding `Expense` and `BankStatement` on approval." No code enforces this.
- **expected:** Spec requires the expense to be derived from a real, approved payroll payment; the invariant should be enforced at construction.
- **evidence:** `aggregate.rs:588-595` validates only `validate_ledger_name(&name)` and `amount_minor < 0`. No check on `payroll_payment_id`.

### FINDING DOMAIN-FIN-016

- **id:** DOMAIN-FIN-016
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/aggregate.rs:818` (test stub marker)
- **description:** The 39 placeholder stub aggregates are explicitly marked as a "backlog" (`aggregate.rs:988-1008`) and the single test at line 1010 is `#[ignore = "backlog: 33 placeholder aggregates need Workstreams D-M"]`. No tests exist for the placeholder aggregates.
- **expected:** Phase 7 exit criterion #1 (`docs/build-plan.md:914-915`): "Every aggregate in `docs/specs/finance/aggregates.md` has a Rust struct + tests."
- **evidence:** `aggregate.rs:1010-1016`: `#[test] #[ignore = "backlog: 33 placeholder aggregates need Workstreams D-M"] fn unimplemented_placeholder_aggregates_backlog()` — body is empty. 47 stub aggregates have zero tests.

### FINDING DOMAIN-FIN-017

- **id:** DOMAIN-FIN-017
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/finance/tests/` (directory absent)
- **description:** The crate has no integration test directory at `crates/domains/finance/tests/`. AGENTS.md mandates "At least one integration test added for new behavior" per PR. The finance integration test lives at `crates/tools/storage-parity/tests/finance_integration.rs`, not in the domain crate.
- **expected:** Per AGENTS.md Validation Checklist: "At least one integration test added for new behavior". Domain crate should have its own `tests/` directory.
- **evidence:** `ls /home/beznet/Workspace/smscore/crates/domains/finance/tests/` → "No such file or directory". The integration test is at `crates/tools/storage-parity/tests/finance_integration.rs`.

### FINDING DOMAIN-FIN-018

- **id:** DOMAIN-FIN-018
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/finance/src/aggregate.rs:124-180` (`Wallet::apply_credit`, `Wallet::apply_debit`)
- **description:** `Wallet::apply_credit` and `Wallet::apply_debit` use `saturating_add`/`saturating_sub` (lines 141, 174) which can silently swallow integer overflow. With `balance_minor: i64` and a `WalletTxType::Deposit` of `i64::MAX`, the balance would saturate at `i64::MAX`. This is a financial correctness issue — a real production system must not silently cap balances.
- **expected:** Money must be `MinorUnits` (i64 cents/paisa) per `docs/build-plan.md:924-927` "Risks" — "All amounts are `MinorUnits` (i64 cents/paisa). The `as` ban (per `AGENTS.md`) is enforced." Overflow handling must be explicit, not silent.
- **evidence:** `aggregate.rs:141`: `self.balance_minor = self.balance_minor.saturating_add(amount_minor);` — silent overflow. `aggregate.rs:174`: `self.balance_minor = self.balance_minor.saturating_sub(amount_minor);` — silent underflow. No upper-bound check before arithmetic; no `Result` returned on overflow.

### FINDING DOMAIN-FIN-019

- **id:** DOMAIN-FIN-019
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/value_objects.rs:361-370` (`Currency::as_str`)
- **description:** `Currency::as_str` uses `std::str::from_utf8(&self.0).ok().unwrap_or("XXX")` with a misleading comment. The comment at lines 364-369 says "the `expect` is unavoidable without `unsafe`" but the code does NOT use `expect` — it uses `unwrap_or("XXX")`. The fallback `"XXX"` is silently wrong for any non-UTF8 bytes (which the constructor prevents, but the doc lies about the implementation).
- **expected:** Documentation/comments must accurately reflect behavior. Use `expect` or return a `Result`.
- **evidence:** `value_objects.rs:369`: `std::str::from_utf8(&self.0).ok().unwrap_or("XXX")` — comment at lines 362-368 says "the `expect` is unavoidable without `unsafe`" but no `expect` is present.

### FINDING DOMAIN-FIN-020

- **id:** DOMAIN-FIN-020
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/finance/src/value_objects.rs:421-426` (`Money::same_currency`)
- **description:** `same_currency` manually compares 3 bytes instead of comparing the `[u8; 3]` arrays directly. The `Currency` inner type is `pub [u8; 3]` which already implements `PartialEq`, so the comparison could be `self.currency.0 == other.currency.0`.
- **expected:** Idiomatic Rust code; the manual byte comparison is unnecessarily verbose and a maintenance hazard.
- **evidence:** `value_objects.rs:422-426`: `self.currency.0[0] == other.currency.0[0] && self.currency.0[1] == other.currency.0[1] && self.currency.0[2] == other.currency.0[2]` — manual byte comparison.

### FINDING DOMAIN-FIN-021

- **id:** DOMAIN-FIN-021
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/services.rs:882-892` (`LateFeeSettings`)
- **description:** `LateFeeSettings` has no `new()` constructor and no validation. `LateFeeKind::FixedAmount(i64)` can be constructed with negative values; `LateFeeKind::PerDayRate(i64)` can be negative; `LateFeeKind::PercentOfAmount(u8)` allows 0-255 (spec mandates 0-100). The service silently masks negatives with `.max(0)` at lines 907-911.
- **expected:** Per `docs/specs/finance/value-objects.md:91-100`: `FeePercentage` is `f32` in `[0, 100]`; `RoundingPolicy` is `HalfUp | HalfEven | Truncate`. Per spec rules, late-fee values must be validated at construction.
- **evidence:** `services.rs:887-892`: `pub struct LateFeeSettings { pub kind: LateFeeKind, pub grace_period_days: u16 }` — fields are pub, no constructor. `LateFeeKind::PercentOfAmount(u8)` at line 882 takes `u8` (0-255), exceeding spec's 0-100 range. `services.rs:907-911`: `n.max(0)` and `billable_days.saturating_mul(rate).max(0)` silently mask negatives.

### FINDING DOMAIN-FIN-022

- **id:** DOMAIN-FIN-022
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/finance/src/services.rs:809-861` (`CarryForwardService`)
- **description:** `CarryForwardService::should_carry_forward` at lines 815-820 uses `balance_minor.abs() >= i64::from(settings.fees_due_days)` which silently coerces `i64::MIN` (whose `.abs()` returns `i64::MIN` in Rust, not a panic). For a `balance_minor = i64::MIN` carrying a massive debit, the function returns `false` (because `i64::MIN < 0`). The spec's invariant 7 in `docs/specs/finance/overview.md:78-79` says "Carry-forward never overwrites ... it adds to the existing balance" — but this corner case silently swallows the largest possible debit balance.
- **expected:** Explicit handling or rejection of `i64::MIN` balance values; the spec invariant must be enforced, not silently bypassed.
- **evidence:** `services.rs:815-820`: `if balance_minor == 0 { return false; } balance_minor.abs() >= i64::from(settings.fees_due_days)`. `i64::MIN.abs()` in Rust returns `i64::MIN` itself (does not panic), and `i64::MIN < 0`, so the function returns `false` for the largest possible debit.

### FINDING DOMAIN-FIN-023

- **id:** DOMAIN-FIN-023
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/commands.rs:38-249` (command-type constants)
- **description:** The command-type constants in `commands.rs` use a flattened `finance.<aggregate>.<action>` wire form, but the spec mandates a hierarchical form. For example, `FINANCE_FEES_GROUP_CONFIGURE_COMMAND_TYPE` is not present; instead the code uses `FINANCE_FEES_GROUP_CREATE_COMMAND_TYPE` for what the spec calls `FeesGroup.Create`. Spec uses dotted names like `FeesGroup.Create`.
- **expected:** Per `docs/commands/finance.md` and `docs/specs/finance/permissions.md`, capability strings are `<Domain>.<Aggregate>.<Action>` (e.g. `FeesGroup.Create`). The constant naming should mirror.
- **evidence:** `commands.rs:68-71`: `FINANCE_FEES_GROUP_CREATE_COMMAND_TYPE = "finance.fees_group.create"` etc. The spec's `docs/specs/finance/permissions.md:22-24` lists `FeesGroup.Create`, `FeesType.Create`, etc.

### FINDING DOMAIN-FIN-024

- **id:** DOMAIN-FIN-024
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/value_objects.rs:553-562` (`FeesInvoiceStatus`)
- **description:** `FeesInvoiceStatus` enum defines `Pending`, `Issued`, `Cancelled`, but the spec's payment status from `docs/specs/finance/value-objects.md:118-126` mandates `PaymentStatus` with values `Unpaid`, `Partial`, `Paid`, `Overpaid`. The code instead uses `FeesPaymentStatus` with the same values but a different name. Spec lists these as separate concepts.
- **expected:** Spec requires both `PaymentStatus` and `PayrollStatus` and `FeesPaymentStatus` as distinct types per `docs/specs/finance/value-objects.md:118-126`.
- **evidence:** `value_objects.rs:553-562`: `pub enum FeesInvoiceStatus { Pending, Issued, Cancelled }`. `value_objects.rs:596-606`: `pub enum FeesPaymentStatus { Unpaid, Partial, Paid, Overpaid }` — but this is `FeesPaymentStatus`, not `PaymentStatus`. The spec expects `PaymentStatus` as a generic concept.

### FINDING DOMAIN-FIN-025

- **id:** DOMAIN-FIN-025
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/repository.rs` (multiple traits)
- **description:** The 44 repository port traits in `repository.rs` import the placeholder stub aggregates (`aggregate.rs:649-829`). These traits cannot be implemented against real aggregate structs because the placeholders have only `_id: ()` and `school_id`. For example, `FeesGroupRepository::insert` takes `&FeesGroup`, but `FeesGroup` is a stub.
- **expected:** Repository port traits must operate on real aggregate types so adapters can be implemented.
- **evidence:** `repository.rs:122-137` defines `FeesGroupRepository` with `async fn insert(&self, ctx: &TenantContext, agg: &FeesGroup) -> Result<()>`, but `FeesGroup` at `aggregate.rs:651-652` is `pub struct FeesGroup { _id: () }` — a stub.

### FINDING DOMAIN-FIN-026

- **id:** DOMAIN-FIN-026
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/finance/src/aggregate.rs:1019-1225` (lib-level `#[allow(missing_docs)]`)
- **description:** Every src file has `#![allow(missing_docs)]` (aggregate.rs:19, value_objects.rs:23, entities.rs:8, events.rs:16, services.rs:21, commands.rs:16, repository.rs:19, query.rs:8, errors.rs:7, lib.rs has `#![deny(missing_docs)]` at line 14 but every module overrides). This blanket allowance suppresses the `#![deny(missing_docs)]` from lib.rs:14 and from AGENTS.md which mandates public rustdoc.
- **expected:** AGENTS.md mandates "All public APIs are documented with rustdoc; `#![deny(missing_docs)]`". The lib.rs has the deny, but every module file opts out with `#![allow(missing_docs)]`.
- **evidence:** `lib.rs:14`: `#![deny(missing_docs)]`. But `aggregate.rs:19`: `#![allow(missing_docs)]`. Same pattern in all other src files (lines cited above).

### FINDING DOMAIN-FIN-027

- **id:** DOMAIN-FIN-027
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/aggregate.rs:13` (`#![allow(unused_imports)]`)
- **description:** Every src file uses `#![allow(unused_imports)]` (aggregate.rs:13, value_objects.rs:13 implicit, entities.rs:9, events.rs:17, services.rs:22, commands.rs:17, repository.rs:20, query.rs:9, lib.rs:13). This blanket allowance hides unused-import warnings that would otherwise indicate dead code or refactoring needs.
- **expected:** Per AGENTS.md: "No `#[allow(dead_code)]` or `_var` prefixes to silence the compiler. Delete unused code, wire it in, or open a follow-up issue."
- **evidence:** `aggregate.rs:13`: `#![allow(unused_imports)]`. `commands.rs:18`: also `#![allow(dead_code)]` at line 18.

### FINDING DOMAIN-FIN-028

- **id:** DOMAIN-FIN-028
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/commands.rs:18`
- **description:** `commands.rs:18` has `#![allow(dead_code)]` which hides unused code. Per AGENTS.md, dead code should be deleted or wired in, not silenced.
- **expected:** AGENTS.md: "No `#[allow(dead_code)]` or `_var` prefixes to silence the compiler."
- **evidence:** `commands.rs:18`: `#![allow(dead_code)]`. Also `entities.rs:10`: `#![allow(dead_code)]`. Also `repository.rs:21`: `#![allow(dead_code)]`. Also `query.rs:10`: `#![allow(dead_code)]`.

### FINDING DOMAIN-FIN-029

- **id:** DOMAIN-FIN-029
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/commands.rs:38-249` (command-type constants) and `commands.rs:267-1243` (command shapes)
- **description:** The spec defines 65+ commands in `docs/specs/finance/commands.md`. The code defines ~50 command shapes plus 90+ command-type constants, but the canonical spec names are not followed. For example, the spec mandates `PayInvoiceCommand`, `PayInstallmentCommand`, `RecordBankStatementCommand`, `GenerateBankPaymentSlipCommand`, `ApproveBankPaymentCommand`, `RejectBankPaymentCommand`, `TransferFundsCommand`, `RecordIncomeCommand`, `RegisterDonorCommand`, `AddWalletCreditCommand`, `RecordPayrollPaymentCommand`, `AddFeesInstallmentCreditCommand`, `ConsumeFeesInstallmentCreditCommand`, etc. — all missing.
- **expected:** Spec mandates these commands at `docs/specs/finance/commands.md` and `docs/commands/finance.md`.
- **evidence:** `grep` confirms these command names are not in `commands.rs`. See `docs/specs/finance/commands.md:233-988` for full list of missing commands.

### FINDING DOMAIN-FIN-030

- **id:** DOMAIN-FIN-030
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/services.rs:621-638` (`PaymentProvider`)
- **description:** `PaymentProvider` trait is marked `#[deprecated(since = "0.1.0", ...)]` in finance crate, but the actual deprecation reason says `since = "0.7.0"` in the handoff. The trait was supposed to move to `educore-payment` in Phase 15, but is still defined in finance. The handoff acknowledges this is "Q10" outstanding work.
- **expected:** Per `docs/handoff/PHASE-7-HANDOFF.md:565-573` Q10: "the trait and impl will be removed from `educore-finance` once `educore-payment` ships in Phase 15."
- **evidence:** `services.rs:623-626`: `#[deprecated(since = "0.1.0", note = "moves to educore-payment in Phase 15; ...")]` — the `since` value is `0.1.0`, not `0.7.0` as documented in the handoff. The handoff (line 187-189) says `since = "0.7.0"`.

### FINDING DOMAIN-FIN-031

- **id:** DOMAIN-FIN-031
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/finance/src/errors.rs:1-11`
- **description:** `errors.rs` aliases `DomainError` as `FinanceError` and re-exports `Result`. No domain-specific error variants exist for finance — the spec mandates rich error categories (e.g. `ValidationError::UniqueViolation`, `ValidationError::OutOfRange`, `ValidationError::Inconsistent` per `docs/specs/finance/workflows.md:32-35`).
- **expected:** Spec workflows.md mandates distinct error types: `ValidationError::UniqueViolation`, `ValidationError::OutOfRange`, `ValidationError::Inconsistent`.
- **evidence:** `errors.rs:1-11` has only `pub use educore_core::error::DomainError as FinanceError;` and `pub use educore_core::error::Result;` — no domain-specific error variants.

### FINDING DOMAIN-FIN-032

- **id:** DOMAIN-FIN-032
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/aggregate.rs:424-526` (`FeesPayment`) and `events.rs:422-484` (`PaymentReceived`)
- **description:** The `FeesPayment` aggregate and `PaymentReceived` event are missing critical fields mandated by the spec event schema. Spec event payload requires `assign_id`, `student_id`, `record_id`, `slip`, `transaction_id`, `note`. Code only has `fees_payment_id`, `amount_minor`, `currency`, `discount_minor`, `fine_minor`, `payment_method`, `bank_id`, `payment_date`. This breaks downstream subscribers (`communication`, `hr`, `assessment`) that depend on these fields per `docs/specs/finance/events.md:185-189`.
- **expected:** Per `docs/specs/finance/events.md:166-182`, the `PaymentReceived` event must carry `assign_id: FeesAssignId`, `student_id: StudentId`, `record_id: StudentRecordId`, `slip: Option<SlipReference>`, `transaction_id: Option<TransactionId>`, `note: Option<String>`. The `communication` subscriber at line 186 needs `student_id` to look up the guardian.
- **evidence:** `events.rs:422-435` defines `PaymentReceived` with only 8 fields. Spec requires 12 fields. Subscribers at line 185-188 need the missing fields.

### FINDING DOMAIN-FIN-033

- **id:** DOMAIN-FIN-033
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/services.rs:54-87` (`create_wallet`)
- **description:** The `create_wallet` service does not enforce the lazy-creation invariant. Per spec (`docs/specs/finance/services.md#walletservice`), wallets should be created lazily on the first `WalletTransaction`. The service requires the caller to invoke `create_wallet` explicitly with a `user_id` and `currency`, but there's no helper that detects "no wallet exists for (school_id, user_id)" and creates it.
- **expected:** Spec says "Wallets are created lazily on the first wallet transaction for `(school_id, user_id)`." A `get_or_create_wallet` helper should be available.
- **evidence:** `services.rs:54-87` (`create_wallet`): takes `CreateWalletCommand` and unconditionally creates a new wallet. No lazy creation pattern. Comment at line 51-52: "Wallets are created lazily on the first wallet transaction for `(school_id, user_id)`." is contradicted by the service signature.

### FINDING DOMAIN-FIN-034

- **id:** DOMAIN-FIN-034
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/finance/src/services.rs:237-291` (`deduct_wallet_credit`)
- **description:** `deduct_wallet_credit` validates that the wallet has sufficient balance at Pending-creation time but does NOT reserve the funds. After this returns, the wallet's `balance_minor` is unchanged, so concurrent debit requests can be approved before the first one's approval completes, leading to a balance going negative at approval time.
- **expected:** Per `docs/specs/finance/workflows.md:304-314` (Wallet Debit workflow), step 3: "The system creates a WalletTransaction in `pending` state." Step 4: "Approver approves; the wallet is debited." The Pending transaction must hold a reservation, or concurrent approvals must be serialized.
- **evidence:** `services.rs:238-291`: returns `(WalletTransaction, WalletDebited)` after validating balance. No mutation of `wallet.balance_minor`. `aggregate.rs:151-180` (`apply_debit`) is the only place where balance decreases. The dispatch path is not provided.

### FINDING DOMAIN-FIN-035

- **id:** DOMAIN-FIN-035
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/aggregate.rs:530-618` (`Expense` aggregate)
- **description:** `Expense::fresh` does not validate that `currency` matches the `account_id`'s currency. A school in INR could record an expense in USD against an INR-denominated bank account. Per spec workflow `docs/specs/finance/workflows.md:177-183` ("Expense Recording"): "The system creates a BankStatement (debit) on the chosen account".
- **expected:** Spec invariant: "The expense's `payment_method` and `account` must be compatible (cash payment → cash account; bank → bank account)" per `docs/specs/finance/aggregates.md:863-864`.
- **evidence:** `aggregate.rs:587-617`: only validates `name` and `amount_minor >= 0`. No cross-aggregate integrity check.

### FINDING DOMAIN-FIN-036

- **id:** DOMAIN-FIN-036
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/services.rs:1040-1064` (test case)
- **description:** The test `deduct_wallet_rejects_insufficient_balance` demonstrates the validation works, but production-grade rounding behavior is not enforced. The `LateFeeService::compute_late_fee` uses integer division which truncates (`(amount * pct) / 100`), losing fractional minor units. No `RoundingPolicy` enum exists to select `HalfUp`, `HalfEven`, or `Truncate`.
- **expected:** Per `docs/specs/finance/value-objects.md:91-100`, `RoundingPolicy` must be `HalfUp`, `HalfEven`, or `Truncate`.
- **evidence:** `services.rs:907-911`: `LateFeeKind::PercentOfAmount(pct) => (i64::from(amount.amount_minor()) * i64::from(pct)) / 100` — truncate-only. No `RoundingPolicy` enum.

### FINDING DOMAIN-FIN-037

- **id:** DOMAIN-FIN-037
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/finance/src/services.rs:435-481` (`record_payment`)
- **description:** The `record_payment` service does not emit a corresponding `BankStatement` event. Per spec workflow `docs/specs/finance/workflows.md:90-100` ("Payment Collection (Cash)") step 4: "System records the payment (PaymentReceived), updates the bank account's cash balance via a BankStatement". The service returns only `(FeesPayment, PaymentReceived)`.
- **expected:** Spec mandates that recording a payment must produce both `PaymentReceived` and `BankStatementRecorded` events, plus a `Transaction` journal line. Per `docs/specs/finance/commands.md:256-258`: "Emits `PaymentReceived` and a corresponding `BankStatementRecorded` (when bank) and a `Transaction` line."
- **evidence:** `services.rs:435-481` returns `(FeesPayment, PaymentReceived)` only. No `BankStatement` event emitted. Spec mandates dual emission.

### FINDING DOMAIN-FIN-038

- **id:** DOMAIN-FIN-038
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/services.rs:801-861` (`CarryForwardService`)
- **description:** `CarryForwardService::build_carry_forward` produces a `CarryForwardDraft` but the corresponding `FeesCarryForward` aggregate is a placeholder stub (`aggregate.rs:815-816`). The dispatcher cannot persist a real `FeesCarryForward` row because the aggregate type has no fields.
- **expected:** Spec requires `FeesCarryForward` aggregate with `student_id`, `academic_id`, `balance_minor`, `balance_type`, `due_date`, `notes` per `docs/specs/finance/aggregates.md:341-368`.
- **evidence:** `services.rs:830-860` returns `CarryForwardDraft`. `aggregate.rs:815-816`: `pub struct FeesCarryForward { _id: () }` — stub.

### FINDING DOMAIN-FIN-039

- **id:** DOMAIN-FIN-039
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/events.rs:1-700` (entire file)
- **description:** The events file does not define any subscriber-side handlers for cross-domain events. The spec requires Finance to subscribe to `StudentAdmitted`, `StudentPromoted`, `StudentWithdrawn`, `StaffRegistered`, `LeaveApproved`, and `hr.payroll.paid`. None are implemented.
- **expected:** Per `docs/specs/finance/overview.md:154-179` ("Cross-Domain Impact"): "When a `Student` is admitted, the academic domain emits `StudentAdmitted`. Finance subscribes and: Creates a `FeesAssign` per active `FeesMaster`...".
- **evidence:** `events.rs:1-700` contains only event definitions, no subscriber functions. `grep "subscribe" crates/domains/finance/src/` returns no matches.

### FINDING DOMAIN-FIN-040

- **id:** DOMAIN-FIN-040
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/finance/src/aggregate.rs:580-583` (`Expense::file_reference`)
- **description:** `Expense::file_reference` is typed as `Option<Uuid>` but per `docs/specs/finance/aggregates.md:850-851` ("A receipt or file attached to a bank statement" / `BankStatementAttachment`), the file should be a typed `FileReference` from the file-storage port (`educore-files`, Phase 15). Using raw `Uuid` loses type safety.
- **expected:** Spec uses typed `FileReference` for receipt/slip attachments.
- **evidence:** `aggregate.rs:551`: `pub file_reference: Option<Uuid>,` — raw UUID, not a typed `FileReference`.

### FINDING DOMAIN-FIN-041

- **id:** DOMAIN-FIN-041
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/value_objects.rs:331-338` (`Currency` constants)
- **description:** `Currency` defines 4 constants: `INR`, `USD`, `EUR`, `GBP`. The spec says "ISO-4217 alpha-3 (e.g. `USD`, `INR`, `EUR`, `GBP`)" with same 4 examples. This is consistent. However, the handoff claims "Currency (8 ISO 4217 codes — the engine default set)" (`PHASE-7-HANDOFF.md:126`), but only 4 are defined.
- **expected:** Per handoff: "Currency (8 ISO 4217 codes — the engine default set)". Code defines only 4 constants.
- **evidence:** `value_objects.rs:332-338`: 4 constants defined (INR, USD, EUR, GBP). Handoff says 8.

### FINDING DOMAIN-FIN-042

- **id:** DOMAIN-FIN-042
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/finance/src/services.rs:816-820` (`should_carry_forward`)
- **description:** The spec mandates that the threshold check uses `fees_due_days` (a count of days) per `docs/specs/finance/value-objects.md:175-178` ("`FeesDueDays` | `u16` in `0..=365`"). The code at `services.rs:819` uses `balance_minor.abs() >= i64::from(settings.fees_due_days)` which compares the minor-unit balance against a day count — this is a unit mismatch (minor units vs. days).
- **expected:** The spec's invariant 4 says "Exceeds threshold → skip + log". The threshold semantics (days vs. minor units) need to match the spec. Per the spec workflow at `docs/specs/finance/workflows.md:145-157`, the carry-forward trigger is days after the due date, not a money threshold.
- **evidence:** `services.rs:815-820`: `balance_minor.abs() >= i64::from(settings.fees_due_days)` — comparing i64 minor units to a day count. The two units are incomparable.

### FINDING DOMAIN-FIN-043

- **id:** DOMAIN-FIN-043
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/services.rs:765-770` (CarryForwardService declaration)
- **description:** The `use` statement `use crate::value_objects::{AcademicYearId, BalanceType, FeeAmount, StudentId};` at line 771 is mid-file (not at the top). Per Rust style and AGENTS.md, imports should be at the top of the file. Additionally, `AcademicYearId`, `BalanceType`, `FeeAmount`, `StudentId` are imported but `LateFeeSettings` and `LateFeeKind` use these without re-import.
- **expected:** Per Rust idiomatic style, all `use` statements should be at the top of the module.
- **evidence:** `services.rs:771`: `use crate::value_objects::{AcademicYearId, BalanceType, FeeAmount, StudentId};` is mid-file, after several `pub fn` declarations.

### FINDING DOMAIN-FIN-044

- **id:** DOMAIN-FIN-044
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/aggregate.rs:266-293` (`WalletTransaction::fresh`)
- **description:** The `WalletTransaction::fresh` constructor takes 14 positional arguments. This makes call sites error-prone (argument order matters). Spec mandates that the engine's Rust style uses typed builders per AGENTS.md, not 14-argument constructors.
- **expected:** Per AGENTS.md: "Domain scopes via extension traits. `.active()`, `.in_class()`, etc. are implemented as extension traits on the macro-generated builder."
- **evidence:** `aggregate.rs:244-258`: 14-argument `fresh` constructor for `WalletTransaction`.

### FINDING DOMAIN-FIN-045

- **id:** DOMAIN-FIN-045
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/aggregate.rs:808-829` (stub aggregates — duplicate `FeesInvoiceSetting`)
- **description:** The macro stub block at `aggregate.rs:736` declares `FeesInvoiceSetting` and the macro stub block at `aggregate.rs:739` also declares `InvoiceSetting`. The spec distinguishes these clearly: `FeesInvoiceSetting` is for the classic scheme, `InvoiceSetting` for the FM scheme. Per the spec, both are required but as separate types. The handoff acknowledges both are stubs.
- **expected:** Both types should be real aggregates, not stubs.
- **evidence:** `aggregate.rs:733-740`: both `FeesInvoiceSetting` and `InvoiceSetting` are 1-field stubs.

### FINDING DOMAIN-FIN-046

- **id:** DOMAIN-FIN-046
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/finance/src/value_objects.rs:540-545` (`DiscountAmount`, `Balance`)
- **description:** `DiscountAmount` is defined as `pub type DiscountAmount = FeeAmount;` (line 542) and `Balance` is `pub type Balance = Amount;` (line 545). Type aliases do NOT provide type safety — a `Balance` and an `Amount` are interchangeable. AGENTS.md mandates "Compile-time safety over strings" and forbids `HashMap<String, T>`; type aliases undermine this.
- **expected:** Spec mandates distinct types. Per `docs/specs/finance/value-objects.md:77-80`: "`DiscountAmount` is `Amount` constrained to `0..=1_000_000.00`" — should be a newtype, not an alias.
- **evidence:** `value_objects.rs:542`: `pub type DiscountAmount = FeeAmount;` — type alias, not newtype.

### FINDING DOMAIN-FIN-047

- **id:** DOMAIN-FIN-047
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/finance/src/value_objects.rs:687` (`WalletTxStatus`)
- **description:** `WalletTxStatus` is `pub type WalletTxStatus = ApprovalStatus;` (line 687). Per `docs/specs/finance/value-objects.md:122-126`, `WalletTxStatus` has values `Pending`, `Approved`, `Rejected` — matching `ApprovalStatus`. However, the spec lists them as separate enum types in the value-object catalog. Using a type alias loses semantic distinction between "wallet transaction approval" and "generic approval".
- **expected:** Per spec, both are independent types in the value-object catalog.
- **evidence:** `value_objects.rs:687`: `pub type WalletTxStatus = ApprovalStatus;` — alias.

### FINDING DOMAIN-FIN-048

- **id:** DOMAIN-FIN-048
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/finance/src/value_objects.rs:1069-1083` (`validate_bank_account_number`)
- **description:** `validate_bank_account_number` only checks length (6..=34) and ASCII alphanumeric. The spec at `docs/specs/finance/value-objects.md:140-150` mandates the format "6..34 chars, alphanumeric" — matches. However, the spec also requires `IfscCode` (11 chars, `[A-Z]{4}0[A-Z0-9]{6}`) which IS implemented. But `BankAccountNumber`, `IfscCode`, `ChequeNumber`, `TransactionId`, `BankName`, `BranchName`, `AccountHolderName`, `OpeningBalance`, `CurrentBalance` are typed as raw `String`/`i64` in command/event/aggregate structs instead of dedicated value objects.
- **expected:** Spec mandates typed value objects for each bank-related concept.
- **evidence:** `commands.rs:774-783` (`UpdateBankAccountCommand`): `bank_name: Option<String>, account_number: Option<String>` — raw `String`, no typed wrapper.

### FINDING DOMAIN-FIN-049

- **id:** DOMAIN-FIN-049
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/aggregate.rs:67-68` (`Wallet::balance_minor: i64`)
- **description:** `Wallet::balance_minor: i64` is `pub` and mutable from outside the crate's services. The aggregate's invariants rely on calling `apply_credit`/`apply_debit`, but direct field mutation is possible. Per AGENTS.md, aggregates should be encapsulated.
- **expected:** Per AGENTS.md "Strict eager loading" and aggregate encapsulation patterns, fields should be private with accessors.
- **evidence:** `aggregate.rs:65`: `pub balance_minor: i64,` — public mutable field.

### FINDING DOMAIN-FIN-050

- **id:** DOMAIN-FIN-050
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/services.rs:42-44` (`event_id_to_uuid` helper)
- **description:** The `event_id_to_uuid` helper function at `services.rs:42-44` is defined at module scope and used to construct typed IDs from `EventId`. The same pattern is repeated across all service functions. This is an unsafe-feeling pattern because it relies on `EventId`'s UUID value being the canonical UUID for the new aggregate.
- **expected:** The engine should have a dedicated `IdGenerator` API that mints typed ids directly.
- **evidence:** `services.rs:42-44`: `fn event_id_to_uuid(e: EventId) -> uuid::Uuid { e.as_uuid() }` — used at lines 66, 117, 186, 247, 447, 513, 584.

### FINDING DOMAIN-FIN-051

- **id:** DOMAIN-FIN-051
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/commands.rs:1220-1228` (`OpenBankAccountCommand`)
- **description:** `OpenBankAccountCommand` is missing the `account_name`, `opening_balance` (typed), and `note` fields mandated by the spec. Spec also requires `bank_name` as a typed `BankName` value object.
- **expected:** Per `docs/specs/finance/commands.md:447-457`: `pub struct OpenBankAccountCommand { pub tenant: TenantContext, pub bank_name: String, pub account_name: String, pub account_number: BankAccountNumber, pub account_type: AccountType, pub opening_balance: Amount, pub note: Option<String> }`.
- **evidence:** `commands.rs:1220-1228`: only has `tenant`, `bank_name`, `account_number`, `account_type`, `opening_balance_minor`, `currency` — missing `account_name` and `note`.

### FINDING DOMAIN-FIN-052

- **id:** DOMAIN-FIN-052
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/commands.rs:1230-1235` (`BlockLoginForDueFeesCommand`)
- **description:** `BlockLoginForDueFeesCommand` is missing the `role_id` field mandated by the spec. The spec requires `(user_id, role_id)` to identify a unique block.
- **expected:** Per `docs/specs/finance/commands.md:417-423`: `pub struct BlockLoginForDueFeesCommand { pub tenant: TenantContext, pub user_id: UserId, pub role_id: Option<RoleId>, pub reason: PreventReason }`.
- **evidence:** `commands.rs:1230-1235`: only has `tenant`, `user_id`, `reason`. Missing `role_id`.

### FINDING DOMAIN-FIN-053

- **id:** DOMAIN-FIN-053
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/commands.rs:1237-1243` (`CarryForwardFeesBalanceCommand`)
- **description:** `CarryForwardFeesBalanceCommand` is missing the `notes`, `due_date`, and `payment_gateway` fields mandated by the spec.
- **expected:** Per `docs/specs/finance/commands.md:349-357`: `pub struct CarryForwardFeesBalanceCommand { pub tenant: TenantContext, pub student_id: StudentId, pub academic_id: AcademicYearId, pub target_academic_id: AcademicYearId, pub notes: Option<String>, pub due_date: Option<NaiveDate>, pub payment_gateway: Option<String> }`. Also uses `student_id` and `academic_id`/`target_academic_id` field names, not `from`/`to`.
- **evidence:** `commands.rs:1237-1243`: only has `tenant`, `student_id`, `from` (as `AcademicYearId`), `to` (as `AcademicYearId`). Field names and missing fields diverge from spec.

### FINDING DOMAIN-FIN-054

- **id:** DOMAIN-FIN-054
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/finance/src/services.rs:104-148` (`credit_wallet`)
- **description:** `credit_wallet` creates a `WalletTransaction` in Pending state, but the corresponding `Wallet` is NOT updated until approval. The lazy-creation pattern (creating wallet on first transaction) is not implemented in `credit_wallet`. The service requires `cmd.wallet_id: WalletId` to be already valid; if it doesn't exist, the call fails.
- **expected:** Per spec (`docs/specs/finance/aggregates.md#wallet`), the wallet should be lazily created.
- **evidence:** `services.rs:105-148`: `credit_wallet` requires `cmd.wallet_id` to be pre-existing. No lazy creation.

### FINDING DOMAIN-FIN-055

- **id:** DOMAIN-FIN-055
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/commands.rs:38-249` and `events.rs`
- **description:** The spec defines `docs/commands/finance.md` and `docs/events/finance.md` as the canonical command/event catalogs. These catalogs list 80+ commands and 110+ events. The code only implements ~50 command shapes and 10 events, with many using non-spec names (e.g. `CreditWalletCommand` instead of spec's `AddWalletCreditCommand`, `GenerateBankSlipCommand` instead of `GenerateBankPaymentSlipCommand`, `ApproveBankSlipCommand` instead of `ApproveBankPaymentCommand`, `CreateFeesGroupCommand` instead of `ConfigureFeesGroupCommand`).
- **expected:** Command names should match the spec exactly per `docs/commands/finance.md`.
- **evidence:** `services.rs:151-163` defines `CreditWalletCommand`; spec (`docs/commands/finance.md:106`) names it `AddWalletCredit`. `commands.rs:806-816` defines `GenerateBankSlipCommand`; spec names it `GenerateBankPaymentSlip`.

### FINDING DOMAIN-FIN-056

- **id:** DOMAIN-FIN-056
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/aggregate.rs:1-1017` (test count gap)
- **description:** The handoff claims "44 unit tests pass" but the build-plan exit criterion #1 requires "Every aggregate in `docs/specs/finance/aggregates.md` has a Rust struct + tests". With 47 placeholder aggregates, this criterion is not met. The handoff's "579 tests pass" counts cross-cutting tests, not per-aggregate tests.
- **expected:** Per `docs/build-plan.md:914-915`, all 52 aggregates must have tests. Currently only 5 do.
- **evidence:** `aggregate.rs:1010-1016`: single `#[ignore]` test for the entire 47-stub backlog. Real test count for aggregates: ~10 tests across 5 aggregates.

### FINDING DOMAIN-FIN-057

- **id:** DOMAIN-FIN-057
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/finance/src/services.rs:622-638` (deprecated trait)
- **description:** The `PaymentProvider` trait and `StubPaymentProvider` are marked deprecated but still in the crate. The handoff's "Where NOT to start" list (`PHASE-7-HANDOFF.md:609-612`) instructs Phase 8 NOT to remove it. This is tech debt carried into production.
- **expected:** The trait should be moved to `educore-payment` (Phase 15) before any production release. Currently, this trait's `charge`/`refund` methods are callable from finance code with `#[allow(deprecated)]` suppression.
- **evidence:** `services.rs:738-739`: `#[allow(deprecated)] impl PaymentProvider for StubPaymentProvider` — suppresses the deprecation warning.

### FINDING DOMAIN-FIN-058

- **id:** DOMAIN-FIN-058
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/services.rs:882-914` (`LateFeeService`)
- **description:** `LateFeeKind::PercentOfAmount(u8)` uses `u8` (range 0-255) but spec mandates `0 <= x <= 100` for `FeePercentage`. The type allows `pct = 150` which would silently compute 150% of the amount as a fee, not raise an error.
- **expected:** Per spec value-object constraint (`docs/specs/finance/value-objects.md:91-100`), percentages must be in `[0, 100]`.
- **evidence:** `services.rs:882`: `LateFeeKind::PercentOfAmount(u8)` — `u8` allows 0-255.

### FINDING DOMAIN-FIN-059

- **id:** DOMAIN-FIN-059
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/services.rs:809-861` (`CarryForwardService`)
- **description:** The `CarryForwardService` returns a `CarryForwardDraft` (a non-persistent draft struct) but the corresponding `FeesCarryForward` aggregate is a stub. The draft has `balance_minor: u64` while the spec requires `balance >= 0` per `docs/specs/finance/aggregates.md:357`. The draft also has no `correlation_id`, `created_by`, etc. — not a real aggregate.
- **expected:** A real `FeesCarryForward` aggregate with full audit footer.
- **evidence:** `services.rs:865-874` defines `CarryForwardDraft` with `student_id`, `from`, `to`, `balance_minor`, `balance_type`, `due_date`, `note` — but this is a service-local type, not the spec-mandated aggregate.

### FINDING DOMAIN-FIN-060

- **id:** DOMAIN-FIN-060
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/aggregate.rs:580-618` (`Expense::fresh`)
- **description:** `Expense::fresh` does not validate `payment_method` compatibility with `account_id`. Per spec invariant: "The expense's `payment_method` and `account` must be compatible (cash payment → cash account; bank → bank account)" (`docs/specs/finance/aggregates.md:863-864`).
- **expected:** Construction should reject `Cash` payment method with `Bank` account_type (and vice versa).
- **evidence:** `aggregate.rs:587-617`: only validates `name` and `amount_minor`. No `payment_method` ↔ `account_type` cross-check.

### FINDING DOMAIN-FIN-061

- **id:** DOMAIN-FIN-061
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/finance/src/services.rs:435-481` (`record_payment`)
- **description:** `record_payment` does not enforce the spec's idempotency rule. Per `docs/specs/finance/workflows.md:316-322`: "`PayInvoice` is idempotent on `(fees_assign_id, transaction_id)`. A duplicate payment with the same transaction id is a no-op success."
- **expected:** The service should accept an idempotency key (`transaction_id`) and skip duplicates.
- **evidence:** `services.rs:484-497` defines `RecordPaymentCommand` with `reference: Option<String>` but no idempotency check in `record_payment` body.

### FINDING DOMAIN-FIN-062

- **id:** DOMAIN-FIN-062
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/aggregate.rs:11` (`lib.rs:23-32` module structure)
- **description:** `value_objects.rs` is declared `pub mod value_objects;` (lib.rs:23) but `aggregate`, `entities`, `errors`, and `repository` are `mod` (private). Consumers cannot import `educore_finance::aggregate::FeesGroup` etc., even though the prelude re-exports some types. This breaks the 9-file module layout's contract.
- **expected:** Per AGENTS.md Module Layout: every src file should be a `pub mod` or at least re-export its key types.
- **evidence:** `lib.rs:23-32`: `pub mod value_objects; mod aggregate; pub mod commands; mod entities; mod errors; pub mod events; pub mod query; mod repository; pub mod services;`.

### FINDING DOMAIN-FIN-063

- **id:** DOMAIN-FIN-063
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/finance/src/services.rs:42-44` and throughout
- **description:** `event_id_to_uuid` is used to construct typed IDs from event IDs, e.g. `WalletId::new(school, event_id_to_uuid(event_id))`. This means the new aggregate's UUID is identical to the event's UUID. This creates a naming collision risk: a `WalletId` and an `EventId` with the same UUID would be indistinguishable in storage.
- **expected:** IDs and event IDs should have separate UUID namespaces.
- **evidence:** `services.rs:66`: `let id = WalletId::new(school, event_id_to_uuid(event_id));`. Same pattern at lines 117, 186, 247, 447, 513, 584.

### FINDING DOMAIN-FIN-064

- **id:** DOMAIN-FIN-064
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/value_objects.rs:553-590` (`FeesInvoiceStatus`)
- **description:** `FeesInvoiceStatus` enum (Pending, Issued, Cancelled) does not match the spec's required invoice lifecycle. Per spec at `docs/specs/finance/aggregates.md:222-238`, `FeesInvoice` has no state machine (it's a config row). But the `FmFeesInvoice` spec defines states (`Draft/Issued/PartiallyPaid/Paid/Overdue/Cancelled` per the handoff line 79-81). The current `FeesInvoiceStatus` is unused / mismatched.
- **expected:** Spec defines no `FeesInvoiceStatus` enum; this is over-engineering. The `FmFeesInvoice` should have its own status enum.
- **evidence:** `value_objects.rs:553-590` defines `FeesInvoiceStatus` with `Pending/Issued/Cancelled`. The spec's `FeesInvoice` aggregate has no state machine; only `FmFeesInvoice` does.

### FINDING DOMAIN-FIN-065

- **id:** DOMAIN-FIN-065
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/finance/src/services.rs:763-800` (`FeesCarryForwardSetting`)
- **description:** `FeesCarryForwardSetting` is defined in `services.rs` as a service-local type (`pub struct FeesCarryForwardSetting { pub title: String, pub fees_due_days: u16 }`), not as a real aggregate. The corresponding `FeesCarryForwardSetting` aggregate at `aggregate.rs:822-824` is a 1-field stub.
- **expected:** The setting should be a first-class aggregate with the spec's fields (title, fees_due_days, payment_gateway reference) per `docs/specs/finance/aggregates.md:1520-1543`.
- **evidence:** `services.rs:775-800` defines the setting as a service-local struct, separate from the placeholder aggregate at `aggregate.rs:822-824`.

### FINDING DOMAIN-FIN-066

- **id:** DOMAIN-FIN-066
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/aggregate.rs:425-526` (`FeesPayment::net_minor`)
- **description:** `FeesPayment::net_minor` (line 522-525) computes `amount - discount` using `saturating_sub`. If `discount > amount`, this returns 0 silently. The spec at `docs/specs/finance/aggregates.md:323` requires "`amount >= 0` and `discount_amount >= 0` and `fine >= 0`" but does NOT mandate `discount <= amount`. This could mask data entry errors.
- **expected:** Validation at construction should reject `discount > amount`, not silently clamp at query time.
- **evidence:** `aggregate.rs:523-525`: `pub const fn net_minor(&self) -> i64 { self.amount_minor.saturating_sub(self.discount_minor) }`.

### FINDING DOMAIN-FIN-067

- **id:** DOMAIN-FIN-067
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/aggregate.rs:355-356` (comment about stubs)
- **description:** Comment at line 355-356 says "Stubs for the other 4 headline aggregates (FeesInvoice, FeesPayment, Expense) — typed-shape-only; real impl lands in subsequent workstreams per the Phase 7 plan." But FeesInvoice, FeesPayment, and Expense ARE real implementations (lines 360-618). The comment is misleading and contradicts the actual code.
- **expected:** Comments should accurately describe code status.
- **evidence:** `aggregate.rs:352-356`: `// Stubs for the other 4 headline aggregates (FeesInvoice, FeesPayment, // Expense) — typed-shape-only; real impl lands in subsequent // workstreams per the Phase 7 plan.` — but `FeesInvoice` (line 360), `FeesPayment` (line 424), and `Expense` (line 530) are full implementations.

### FINDING DOMAIN-FIN-068

- **id:** DOMAIN-FIN-068
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/finance/src/services.rs:817-820` (`should_carry_forward`)
- **description:** The function returns `false` for `balance == 0` AND for `|balance| < fees_due_days`. The spec's invariant 1 says "No open balance → no `FeesCarryForward` row" (handled) and invariant 4 says "Exceeds threshold → skip + log" — but the spec's invariant compares against the carry-forward days threshold, not a money amount. Per `docs/specs/finance/workflows.md:145-157`, the trigger is "how many days after the due date a balance is carried forward" — a time-based trigger, not a money threshold.
- **expected:** Carry-forward trigger should be time-based (days overdue), not money-based.
- **evidence:** `services.rs:815-820`: `balance_minor.abs() >= i64::from(settings.fees_due_days)` — money threshold comparison.

### FINDING DOMAIN-FIN-069

- **id:** DOMAIN-FIN-069
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/lib.rs:48-100` (prelude re-exports)
- **description:** The prelude re-exports many `aggregate::*` items but does NOT include the `FeesAssignDiscount`, `FeesInstallment`, `BankAccount`, `BankStatement`, `Income`, `Donor`, etc. (the stub aggregates that the spec needs to function). Consumers cannot construct or reference these types even though the spec defines them.
- **expected:** Per the 9-file module layout (`AGENTS.md`), the prelude should re-export all public aggregate types.
- **evidence:** `lib.rs:48-50`: only `Expense`, `FeesInvoice`, `FeesPayment`, `Wallet`, `WalletTransaction` are re-exported. The other 47 aggregates are not re-exported because they are stubs.

### FINDING DOMAIN-FIN-070

- **id:** DOMAIN-FIN-070
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/value_objects.rs:46` (`RbacRoleId`)
- **description:** `RbacRoleId` is re-exported from `educore_rbac::ids::RoleId as RbacRoleId` (line 46), but `RoleId` is also re-exported directly from `educore_hr::value_objects::RoleId` (line 43). This creates two distinct role-id types in the finance crate's namespace, which can cause cross-domain confusion.
- **expected:** Per spec, only one `RoleId` type should exist; consumers should not need both.
- **evidence:** `value_objects.rs:43`: `pub use educore_hr::value_objects::RoleId;` and `value_objects.rs:46`: `pub use educore_rbac::ids::RoleId as RbacRoleId;` — both available in finance crate.

### FINDING DOMAIN-FIN-071

- **id:** DOMAIN-FIN-071
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/services.rs:435-481` (`record_payment`) and `commands.rs` missing `PayInvoiceCommand`
- **description:** The spec mandates a distinct `PayInvoiceCommand` (per `docs/specs/finance/commands.md:233-249`) with `fees_assign_id: FeesAssignId`, `amount: Amount`, `payment_method_id: PaymentMethodId`, `bank_id: Option<BankAccountId>`, `note: Option<String>`, `slip: Option<SlipReference>`, `transaction_id: Option<TransactionId>`, `discount_month: Option<u8>`, `discount_amount: Option<DiscountAmount>`, `fine_amount: Option<FineAmount>`, `fine_title: Option<String>`, `service_charge: Option<ServiceCharge>`. The code only has `RecordPaymentCommand` in `services.rs:484-497` with fewer fields and a different name.
- **expected:** Spec mandates `PayInvoiceCommand` with the full field set including `fees_assign_id` (the link to the assignment being paid).
- **evidence:** `services.rs:484-497` defines `RecordPaymentCommand` with `tenant, amount_minor, currency, discount_minor, fine_minor, payment_method, bank_id, payment_method_id, reference, note, payment_date` — no `fees_assign_id`, no `discount_month`, no `service_charge`, no `fine_title`. Missing spec compliance.

### FINDING DOMAIN-FIN-072

- **id:** DOMAIN-FIN-072
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/repository.rs:122-1288`
- **description:** The 44 repository port traits declare `async fn insert` / `async fn update` but never `async fn delete` for the soft-delete aggregates (e.g. `FeesGroup`, `FeesType`, `FeesMaster`, `FeesDiscount`, `FeesInstallment`, etc.). The spec mandates soft-delete commands and events.
- **expected:** Per `docs/specs/finance/aggregates.md`, all CRUD aggregates should have `delete` repository methods.
- **evidence:** `repository.rs:122-137` (`FeesGroupRepository`) has `get`, `list_for_school`, `find_by_name`, `insert`, `update` but no `delete`. Same gap for `FeesTypeRepository`, `FeesMasterRepository`, etc. Spec defines `DeleteFeesGroup`, `DeleteFeesType`, `DeleteFeesMaster`, `DeleteFeesDiscount`, `DeleteFeesInstallment` commands at `docs/specs/finance/commands.md:33-34, 65-66, 95-96`.

### FINDING DOMAIN-FIN-073

- **id:** DOMAIN-FIN-073
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/services.rs:244-291` (`deduct_wallet_credit`)
- **description:** `deduct_wallet_credit` returns `(WalletTransaction, WalletDebited)` but the function takes `wallet: &Wallet` (immutable reference) yet does not return the modified wallet. The caller has no way to know the wallet's state after a Pending debit is created. Per the spec invariant, the wallet balance should reflect the pending debit somehow.
- **expected:** Either return the updated wallet alongside, or document that the wallet is unchanged at Pending time and update it on approval.
- **evidence:** `services.rs:243`: `pub fn deduct_wallet_credit(wallet: &Wallet, ...) -> Result<(WalletTransaction, WalletDebited)>` — takes `&Wallet` (not `&mut`), so the caller cannot get the updated balance.

### FINDING DOMAIN-FIN-074

- **id:** DOMAIN-FIN-074
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/finance/src/aggregate.rs:67` (Wallet import)
- **description:** `Wallet` aggregate has `school_id: SchoolId` (line 61) and `id: WalletId`. Both fields are `pub`. `WalletId.school_id()` already returns the school, so the redundant `school_id` field is denormalized. This violates single-source-of-truth and risks divergence.
- **expected:** Per AGENTS.md and the audit-footer pattern, denormalized fields should be accessor methods, not pub fields.
- **evidence:** `aggregate.rs:60-67`: `pub school_id: SchoolId,` and `pub id: WalletId,` — both pub, with `school_id` derivable from `id.school_id()`.

### FINDING DOMAIN-FIN-075

- **id:** DOMAIN-FIN-075
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/services.rs:50-87` (`create_wallet` / `CreateWalletCommand`)
- **description:** `CreateWalletCommand` does not validate that a wallet for the `(school_id, user_id)` pair does not already exist. Idempotency violation: the spec's lazy-creation invariant says wallets are created lazily on the first transaction. The code unconditionally creates a new `Wallet` aggregate on every call, which would create duplicate wallets if called twice.
- **expected:** Idempotency check at construction time.
- **evidence:** `services.rs:54-87`: `create_wallet` calls `Wallet::fresh(...)` unconditionally without checking for existing wallet.

### FINDING DOMAIN-FIN-076

- **id:** DOMAIN-FIN-076
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/repository.rs:43-50` (imports)
- **description:** `repository.rs:43-52` imports typed IDs from `crate::value_objects::*` but the placeholder stub aggregates from `crate::aggregate::*`. The repository traits cannot be implemented because the stub aggregates have no fields to populate. This means the entire repository layer is unusable for 47 of 52 aggregates.
- **expected:** Repository traits must operate on real aggregates with full field schemas.
- **evidence:** `repository.rs:29-52`: imports `AmountTransfer, BankAccount, BankStatement, ...` from `crate::aggregate` — all are 1-field stubs.

### FINDING DOMAIN-FIN-077

- **id:** DOMAIN-FIN-077
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/finance/src/services.rs:622-638` (`PaymentProvider`)
- **description:** `PaymentProvider::charge` takes a `ChargeRequest` with `method: PaymentMethodKind`. The kind enum has 6 variants (Cash, Bank, Cheque, Card, Mobile, Gateway) but only `Gateway` should trigger external gateway calls. The trait does not discriminate — a Cash charge could be routed to a real gateway.
- **expected:** Spec mandates gateway isolation per `docs/specs/finance/overview.md:192-194` ("Anti-Goals: The finance domain does not connect to any payment gateway. Gateway integration is a port.").
- **evidence:** `services.rs:640-651` (`ChargeRequest`): `method: PaymentMethodKind` — no discrimination. The trait must enforce the routing decision internally.

### FINDING DOMAIN-FIN-078

- **id:** DOMAIN-FIN-078
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/services.rs:925-961` (`DoubleEntryService`)
- **description:** `DoubleEntryService::check_invariant` checks `sum(debits) == sum(credits)`, but the spec invariant 2 says "A fees amount is non-negative; a discount amount is non-negative." The proptest uses random `i64` values (lines 1264-1265) that are positive only because the proptest uses `0i64..10_000`, but the production code has no upper bound on the journal row amounts. A malicious input could overflow `i64`.
- **expected:** Production code should validate journal row amounts at construction.
- **evidence:** `services.rs:944-952`: `if r.amount < 0 { return Err(...) }` — only checks lower bound. No upper bound. `services.rs:950-951`: uses `saturating_add` which silently overflows at `i64::MAX`.

### FINDING DOMAIN-FIN-079

- **id:** DOMAIN-FIN-079
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/finance/src/aggregate.rs:111-119` (`Wallet::balance`)
- **description:** `Wallet::balance()` returns `Amount` which wraps `Money { amount_minor: i64, currency: Currency }`. The `Amount` struct allows `Money::new(...)` to fail with negative amounts. The accessor `balance()` constructs `Amount { money: Money { amount_minor: self.balance_minor, currency: self.currency } }` directly without going through the validating constructor. If `balance_minor` is somehow negative (e.g. due to a bug), an invalid `Amount` is constructed.
- **expected:** Validate the balance before returning an `Amount`.
- **evidence:** `aggregate.rs:111-119`: `pub fn balance(&self) -> Amount { Amount { money: Money { amount_minor: self.balance_minor, currency: self.currency } } }` — bypasses `Money::new` validation.

### FINDING DOMAIN-FIN-080

- **id:** DOMAIN-FIN-080
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/finance/src/services.rs:1240-1306` (proptest)
- **description:** The `prop_double_entry_invariant` proptest uses random `i64` values but uses `debits.clone()` (line 1277) implicitly via the proptest framework. The test only validates 2 cases: balanced journal passes and unbalanced journal fails. No test covers:
  - Mixed-school isolation (a row from school A should not affect school B's invariant check)
  - Empty journal
  - Single row (debit without credit or vice versa)
  - Overflow handling
- **expected:** Production-grade property tests with edge cases.
- **evidence:** `services.rs:1260-1306`: only 2 proptest cases (balanced/unbalanced). No isolation test. No empty test. No overflow test.

### FINDING DOMAIN-FIN-081

- **id:** DOMAIN-FIN-081
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/lib.rs:102-110` (lib-level test)
- **description:** `lib.rs:102-110` has a single `package_metadata_is_set` test asserting the package name and version. This test trivially passes but adds no coverage. No test asserts that the prelude re-exports are correct or that the module layout matches the 9-file spec.
- **expected:** Per AGENTS.md "No dummy tests". Each test must validate real-world behavior.
- **evidence:** `lib.rs:104-110`: `#[test] fn package_metadata_is_set() { assert_eq!(PACKAGE_NAME, "educore-finance"); assert!(!PACKAGE_VERSION.is_empty()); }` — dummy test, always passes.

### FINDING DOMAIN-FIN-082

- **id:** DOMAIN-FIN-082
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/services.rs:381-419` (`WalletService::balance`)
- **description:** `WalletService::balance` computes a balance from transactions but the result is discarded (`let _ = bal;` at line 396). The function returns `wallet.balance_minor` instead. This makes the parameter `transactions: &[WalletTransaction]` meaningless — the function is misnamed; it's not computing a balance from transactions.
- **expected:** The function should compute and return the cross-check balance, or be renamed to clarify its purpose.
- **evidence:** `services.rs:381-398`: `let _ = bal; wallet.balance_minor` — the computed balance is thrown away.

### FINDING DOMAIN-FIN-083

- **id:** DOMAIN-FIN-083
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/services.rs:622-764` (PaymentProvider section)
- **description:** The `PaymentProvider` trait, `ChargeRequest`, `PaymentReceipt`, `PaymentStatus`, `PaymentProviderPaymentId`, `PaymentProviderStatus`, `RefundRequest`, `RefundReceipt`, and `StubPaymentProvider` are all defined inside `services.rs` rather than in a dedicated port module. This violates separation of concerns: the deprecated trait should be moved to `educore-payment` (per `PHASE-7-HANDOFF.md` Q10), and the `StubPaymentProvider` is a test fixture that should be in a `tests/` module or testkit crate.
- **expected:** Per the spec, ports live in dedicated crates (e.g., `educore-payment`).
- **evidence:** `services.rs:622-764`: all these types defined inline. The handoff Q10 (lines 565-573) mandates they be moved.

### FINDING DOMAIN-FIN-084

- **id:** DOMAIN-FIN-084
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/aggregate.rs:649-829` (stub aggregates via macro)
- **description:** The `finance_aggregate_stub!` macro (lines 626-647) generates structs with only `school_id` and `_id: ()`. These stubs are `pub` and can be instantiated by consumers. A consumer could create a `BankAccount { school_id: ..., _id: () }` which is meaningless and breaks domain invariants.
- **expected:** Stubs should be in a `#[cfg(test)]` block or be `pub(crate)` to prevent misuse.
- **evidence:** `aggregate.rs:626-647`: macro definition. `aggregate.rs:649-829`: macro invocations are `pub struct ...` (lines 651, 654, 658, 662, 666, 670, etc.). All 39 stubs are public.

### FINDING DOMAIN-FIN-085

- **id:** DOMAIN-FIN-085
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/finance/src/services.rs:382-398` (`WalletService::balance`)
- **description:** `WalletService::balance` computes `bal = bal.saturating_add(tx.amount_minor)` or `saturating_sub` for each approved transaction. The function name implies it computes the balance, but the computed value is discarded. The `transactions` parameter is essentially unused, making the function a no-op wrapper around `wallet.balance_minor`.
- **expected:** Either compute and return the derived balance, or remove the parameter and rename the method.
- **evidence:** `services.rs:382-398`: function body computes `bal` in a loop, then discards with `let _ = bal;`, and returns `wallet.balance_minor`.

### END FINDINGS
Total Findings: 85

