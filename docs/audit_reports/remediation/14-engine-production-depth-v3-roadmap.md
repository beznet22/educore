# Engine Production Depth v3 — Comprehensive Implementation Roadmap

**Generated:** 2026-07-02, after Engine Production Depth ferment closed at grade D.
**Purpose:** Move **every** deferred item from prior ferment into a per-aggregate focused implementation plan.
**Honesty rule:** This roadmap is **exhaustive** — if any deferred work is missing, this is a planning failure.

---

## Background

Prior ferment closed at **grade D**. Net invariants promoted: **~9 of ~700+** spec invariants (~1%).

**Key insight:** Sub-agents succeed extending existing aggregates, fail building placeholder-stub aggregates from scratch (5+ of last 6 attempts aborted).
**New approach:** 1 step per placeholder aggregate. No multi-aggregate batches.

---

## Baseline (post-prior-ferment)

| Domain | Total inv | [x] | [~] | [ ] | [N/A] |
|---|---|---|---|---|---|
| Academic | 72 | 17 | 0 | 51 | 4 |
| Finance | 174 | 19 | 31 | 124 | 0 |
| HR | 110 | 1 | 1 | 108 | 0 |
| Attendance | 16 fns | TBD | TBD | TBD | TBD |
| Communication | 104 fns | TBD | TBD | TBD | TBD |
| Documents | 18 fns | TBD | TBD | TBD | TBD |
| Facilities | 60 fns | TBD | TBD | TBD | TBD |
| Library | 48 fns | TBD | TBD | TBD | TBD |
| CMS | 37 fns | TBD | TBD | TBD | TBD |
| Events-Domain | TBD | TBD | TBD | TBD | TBD |

**RBAC:** 540 capability mappings (0 spec-validated).
**Dispatcher wrappers:** ~509 needed.
**CI cross-compile:** not verified (env-bound).

---

## PART 1 — Academic placeholder aggregates (15 steps)

Each = 1 step. Source: `docs/audit_reports/academic-invariant-checklist.md` + `docs/specs/academic/aggregates.md`.

### Step A1 — Guardian (5 missing)
- Spec: `docs/specs/academic/aggregates.md` § Guardian
- Code: `crates/domains/academic/src/aggregate.rs:325-329` (placeholder `pub struct Guardian { id, school_id }`)
- Invariants to enforce: I-1 (at most one phone/email), I-2 (multi-student link), I-3 (Relation + IsPrimary), I-4 (at most one IsPrimary per student), I-5 (soft-delete cascade)
- Tests: 5 invariant-violation tests in `tests/guardian.rs`
- New value objects: `Relation` enum (Father/Mother/Guardian/Other)

### Step A2 — ClassSection (3 missing + 1 N/A)
- Spec: `docs/specs/academic/aggregates.md` § ClassSection
- Code: `crates/domains/academic/src/aggregate.rs:330-333` (placeholder)
- Invariants: I-1 (unique per class/section/year), I-3 (one+ classrooms), I-4 (cannot delete while StudentRecord refs)
- Reuses: `ClassRoomId` (Wave 48 value object)

### Step A3 — ClassSubject (2 missing + 1 N/A)
- Spec: `docs/specs/academic/aggregates.md` § ClassSubject
- Code: `crates/domains/academic/src/aggregate.rs:335-338` (placeholder)
- Invariants: I-1 (class or class-section scope), I-3 (PassMark override)
- Reuses: `ClassSubjectScope` (Wave 48 value object)

### Step A4 — ClassRoutine (5 missing)
- Spec: `docs/specs/academic/aggregates.md` § ClassRoutine
- Code: `crates/domains/academic/src/aggregate.rs:340-343` (placeholder)
- Invariants: I-1 (full week), I-2 (ClassTime periods), I-3 (room+teacher per period), I-4 (no teacher conflict), I-5 (no room conflict)
- New: DayOfWeek enum, ClassPeriod struct, teacher/room conflict UniquenessChecker methods

### Step A5 — Homework (5 missing)
- Spec: `docs/specs/academic/aggregates.md` § Homework
- Code: `crates/domains/academic/src/aggregate.rs:345-348` (placeholder)
- Invariants: I-1 (teacher-created), I-2 (submission > homework date), I-3 (evaluation >= submission), I-4 (optional attachment), I-5 (marks immutable once evaluated)

### Step A6 — LessonPlan (4 missing)
- Spec: `docs/specs/academic/aggregates.md` § LessonPlan
- Code: `crates/domains/academic/src/aggregate.rs:351-354` (placeholder)
- Invariants: I-1 (anchored), I-2 (sub-topics), I-3 (CompletedStatus enum), I-4 (multiple teachers)
- New: CompletedStatus enum (Pending/InProgress/Completed/Skipped)

### Step A7 — Lesson (3 missing)
- Spec: `docs/specs/academic/aggregates.md` § Lesson
- Code: `crates/domains/academic/src/aggregate.rs:357-360` (placeholder)
- Invariants: I-1 (unique title), I-2 (zero+ topics), I-3 (creation user+timestamp)

### Step A8 — LessonTopic (2 missing)
- Spec: `docs/specs/academic/aggregates.md` § LessonTopic
- Code: `crates/domains/academic/src/aggregate.rs:363-366` (placeholder)
- Invariants: I-1 (belongs to one lesson), I-2 (CompletedStatus + CompletedDate)

### Step A9 — StudentRecord (6 missing)
- Spec: `docs/specs/academic/aggregates.md` § StudentRecord
- Code: `crates/domains/academic/src/aggregate.rs:445-449` (placeholder `pub struct StudentRecord { id, school_id }`)
- Invariants: I-1 (one non-graduate per year), I-2 (roll uniqueness), I-3 (IsDefault), I-4 (IsPromote=false), I-5 (IsGraduate=true), I-6 (AdmissionNumber carry-over)
- **Critical:** Blocks assessment, finance, attendance downstream

### Step A10 — StudentPromotion (3 missing)
- Spec: `docs/specs/academic/aggregates.md` § StudentPromotion
- Code: `crates/domains/academic/src/aggregate.rs:369-372` (placeholder)
- Invariants: I-1 (From/To references), I-2 (ResultStatus enum), I-3 (immutable once written)

### Step A11 — StudentCategory (1 missing)
- Spec: `docs/specs/academic/aggregates.md` § StudentCategory
- Code: `crates/domains/academic/src/aggregate.rs:375-378` (placeholder)
- Invariant: I-1 (unique name within school)

### Step A12 — StudentGroup (1 missing + 1 N/A)
- Spec: `docs/specs/academic/aggregates.md` § StudentGroup
- Code: `crates/domains/academic/src/aggregate.rs:381-384` (placeholder)
- Invariant: I-1 (unique name within school)

### Step A13 — RegistrationField (3 missing)
- Spec: `docs/specs/academic/aggregates.md` § RegistrationField
- Code: `crates/domains/academic/src/aggregate.rs:387-390` (placeholder)
- Invariants: I-1 (FieldName + LabelName + Type), I-2 (IsRequired + IsVisible + editability), I-3 (AdminSection)

### Step A14 — Certificate (3 missing)
- Spec: `docs/specs/academic/aggregates.md` § Certificate
- Code: `crates/domains/academic/src/aggregate.rs:393-396` (placeholder)
- Invariants: I-1 (Layout + body + footer + photo), I-2 (optional attachment), I-3 (DefaultFor flag)

### Step A15 — IdCard (2 missing)
- Spec: `docs/specs/academic/aggregates.md` § IdCard
- Code: `crates/domains/academic/src/aggregate.rs:399-402` (placeholder)
- Invariants: I-1 (Boolean display flags), I-2 (Layout dimensions + spacing)

### Step A16 — Student I-6 partial fix
- Student I-6 (withdrawn/graduated has no active StudentRecord) — depends on A9 (StudentRecord aggregate built first)

---

## PART 2 — Finance placeholder-stub aggregates (~30 steps + 9 partial-closure steps)

Source: `docs/audit_reports/finance-invariant-checklist.md` + `docs/audit_reports/stub_vs_implementation.md` § "finance — Deep Invariant Audit".

### Partial invariants on existing aggregates (9 steps)

#### Step F1 — Wallet cross-aggregate balance cache reconciliation
- Audit § B: "WalletTransaction balance invariant not enforced in aggregate" (partial)
- Add `recompute_balance` method on Wallet; have `WalletTransaction::approve` call it

#### Step F2 — FeesPayment I-1 (FK to FeesAssign/Student)
- Add `assign_id: FeesAssignId`, `student_id: StudentId` to FeesPayment struct
- Constructor requires non-None for both

#### Step F3 — FeesPayment I-3 (gateway consistency)
- Add `payment_method_id: Option<PaymentMethodId>`; validate gateway match

#### Step F4 — FeesPayment I-4 (gateway tx id required if Gateway)
- In `record_payment` service, validate reference.is_some() if payment_method == Gateway

#### Step F5 — FeesInvoice I-3 (IncrementInvoiceCounter arithmetic)
- Add `increment_counter(&mut self, n: u32, ...)` method
- Add `IncrementInvoiceCounterCommand` if missing
- Add service function

#### Step F6 — Expense I-2 (payment_method + account_id compatible)
- Add constructor check for cash vs bank consistency

#### Step F7 — WalletTransaction cross-aggregate balance update on approve
- When `WalletTransaction::approve` fires, update `Wallet.balance_minor` in same transaction

#### Step F8 — BankReconciliationService match_transaction (already real, needs test coverage)
#### Step F9 — DoubleEntryService check_invariant (already real, needs test coverage)

### Placeholder-stub aggregates (~30 steps, one per aggregate)

| Step | Aggregate | Spec section | Spec invariants | Stub location |
|---|---|---|---|---|
| F10 | AmountTransfer | aggregates.md § AmountTransfer | 3 (I-1 double-entry, I-2 debit source, I-3 idempotency) | aggregate.rs:851-854 |
| F11 | BankAccount | § BankAccount | 3 (account_number unique, current_balance derived, account_type) | aggregate.rs:816-819 |
| F12 | BankPaymentSlip | § BankPaymentSlip | 4 (payment_mode, approve_status, promote to BankStatement+Payment, cannot reject after approval) | placeholder |
| F13 | BankPaymentSlipAudit | § BankPaymentSlipAudit | 2 (append-only, timestamps) | placeholder |
| F14 | BankStatement | § BankStatement | 4 (amount ≥ 0, type, after_balance, append-only) | aggregate.rs:825-828 |
| F15 | BankStatementAttachment | § BankStatementAttachment | 2 (attachment ref valid, orphan cleanup) | placeholder |
| F16 | ChartOfAccount | § ChartOfAccount | 2 (unique name, cannot delete while referenced) | aggregate.rs:858-861 |
| F17 | DirectFeesInstallment | § DirectFeesInstallment | 4 (percentage ∈ [0,100], amount ≥ 0, percentage sum ≤ 100, non-overlapping windows) | placeholder |
| F18 | DirectFeesInstallmentAssign | § DirectFeesInstallmentAssign | 3 (unique per student+installment, amount ≥ 0, balance ≥ 0) | placeholder |
| F19 | DirectFeesInstallmentAssignChild | § DirectFeesInstallmentAssignChild | 2 (append-only, timestamps monotonic) | placeholder |
| F20 | DirectFeesInstallmentChildPayment | § DirectFeesInstallmentChildPayment | 2 (paid+balance=amount+discount, paid monotonic) | aggregate.rs:710-713 |
| F21 | DirectFeesReminder | § DirectFeesReminder | 1 (due_date_before ≥ 0) | placeholder |
| F22 | DirectFeesSetting | § DirectFeesSetting | 2 (reminder_before ≥ 0, due_date_from_sem ∈ 1..=28) | placeholder |
| F23 | Donor | § Donor | 2 (show_public boolean, email unique) | aggregate.rs:840-843 |
| F24 | DueFeesLoginPrevent | § DueFeesLoginPrevent | 2 (unique per school+academic+user+role, auto-pruned when balance=0) | placeholder |
| F25 | ExpenseApproval | § ExpenseApproval | 2 (state machine, timestamps) | placeholder |
| F26 | ExpenseHead | § ExpenseHead | 1 (unique name within school) | aggregate.rs:845-848 |
| F27 | FeesAssign | § FeesAssign | 5 (amount ≥ 0, applied_discount ≤ fees, payment cap, active_status, unique) | aggregate.rs:673-676 |
| F28 | FeesAssignDiscount | § FeesAssignDiscount | 3 (applied/unapplied ≥ 0, applied+unapplied constant, timestamp) | aggregate.rs:678-681 |
| F29 | FeesCarryForward | § FeesCarryForward | 3 (balance ≥ 0, balance_type, unique per school+student+academic) | aggregate.rs:890-893 |
| F30 | FeesCarryForwardLog | § FeesCarryForwardLog | 2 (append-only, amount ≥ 0) | aggregate.rs:895-898 |
| F31 | FeesCarryForwardSetting | § FeesCarryForwardSetting | 2 (per-school config, threshold ≥ 0) | placeholder |
| F32 | FeesDiscount | § FeesDiscount | 4 (amount ≥ 0, type valid, once-per-master, once-per-year) | aggregate.rs:684-687 |
| F33 | FeesGroup | § FeesGroup | 4 (unique name, non-empty, cascade to FeesMaster, cannot delete while referenced) | placeholder |
| F34 | FeesInstallment | § FeesInstallment | 5 (percentage ∈ [0,100], amount ≥ 0, percentage sum ≤ 100, due_date ordering, non-overlapping) | aggregate.rs:689-692 |
| F35 | FeesInstallmentAssign | § FeesInstallmentAssign | 3 (unique per assign+installment, paid ≤ amount+discount, active_status) | aggregate.rs:694-697 |
| F36 | FeesInstallmentAssignDiscount | § FeesInstallmentAssignDiscount | 2 (applied_amount ≥ 0, timestamps) | placeholder |
| F37 | FeesInstallmentCredit | § FeesInstallmentCredit | 3 (amount ≥ 0, source valid, append-only) | aggregate.rs:899-902 |
| F38 | FeesInvoiceSetting | § FeesInvoiceSetting | 2 (prefix format, per_th ≥ 0) | aggregate.rs:808-811 |
| F39 | FeesMaster | § FeesMaster | 3 (amount ≥ 0, unique per school+name+group, cannot delete while FeesAssign refs) | aggregate.rs:664-667 |
| F40 | FmFeesGroup | § FmFeesGroup | 1 (unique name) | placeholder |
| F41 | FmFeesInvoice | § FmFeesInvoice | 3 (amount ≥ 0, due_date ≥ invoice_date, state machine) | placeholder |
| F42 | FmFeesInvoiceChild | § FmFeesInvoiceChild | 3 (amount ≥ 0, sub_total = amount+weaver+fine, paid ≤ sub_total+service_charge) | aggregate.rs:741-744 |
| F43 | FmFeesInvoiceLineNote | § FmFeesInvoiceLineNote | 2 (non-empty, append-only) | placeholder |
| F44 | FmFeesInvoiceSetting | § FmFeesInvoiceSetting | 3 (per_th ≥ 0, due_date config, prefix format) | placeholder |
| F45 | FmFeesTransaction | § FmFeesTransaction | 3 (amount ≥ 0, total_paid ≥ 0, state machine) | aggregate.rs:761-764 |
| F46 | FmFeesTransactionChild | § FmFeesTransactionChild | 2 (amount ≥ 0, parent ref valid) | placeholder |
| F47 | FmFeesTransactionLineNote | § FmFeesTransactionLineNote | 2 (non-empty, append-only) | placeholder |
| F48 | FmFeesType | § FmFeesType | 3 (type ∈ {fee,discount,fine}, amount ≥ 0, unique) | placeholder |
| F49 | FmFeesWeaver | § FmFeesWeaver | 2 (percentage ∈ [0,100], sum ≤ child subtotals) | aggregate.rs:772-775 |
| F50 | Income | § Income | 3 (amount ≥ 0, account+payment_method compatible, timestamps) | aggregate.rs:835-838 |
| F51 | IncomeApproval | § IncomeApproval | 2 (state machine, timestamps) | placeholder |
| F52 | IncomeHead | § IncomeHead | 1 (unique name) | placeholder |
| F53 | InventoryPayment | § InventoryPayment | 3 (amount ≥ 0, payment+account compatible, append-only) | placeholder |
| F54 | InvoiceSetting | § InvoiceSetting | 1 (prefix format) | placeholder |
| F55 | PaymentGatewaySetting | § PaymentGatewaySetting | 4 (per-school unique, mode ∈ {sandbox,live}, charge ≥ 0+type ∈ {P,F}, credentials encrypted) | placeholder |
| F56 | PaymentMethod | § PaymentMethod | 3 (method unique, gateway_id required, account_id compatible) | placeholder |
| F57 | PayrollEarnDeduc | § PayrollEarnDeduc | 3 (amount ≥ 0, earn_dedc_type ∈ {e,d}, sum invariants) | placeholder |
| F58 | PayrollGenerate | § PayrollGenerate | 4 (gross = basic+total_earning, net = gross-total_deduction-tax, status FSM, paid ≤ net) | aggregate.rs:933-936 |
| F59 | PayrollPayment | § PayrollPayment | 3 (sum vs unpaid net, payment+bank compatible, creates Expense+BankStatement) | aggregate.rs:874-877 |
| F60 | PayrollPaymentApproval | § PayrollPaymentApproval | 2 (state machine, timestamps) | placeholder |
| F61 | ProductPurchase | § ProductPurchase | 3 (amount ≥ 0, vendor ref, state machine) | placeholder |
| F62 | QuestionBankFee | § QuestionBankFee | 1 (amount ≥ 0) | placeholder |
| F63 | SalaryTemplate | § SalaryTemplate | 4 (gross = sum earnings, net = gross-total_deduction, name unique, append-only) | placeholder |
| F64 | Transaction | § Transaction | 3 (sum debits = sum credits, append-only, state machine) | placeholder |
| F65 | WalletTransactionApproval | § WalletTransactionApproval | 2 (state machine, timestamps+reason) | placeholder |
| F66 | Donor, ExpenseHead, ChartOfAccount, DueFeesLoginPrevent (carry-forward + login prevention) | spec § | various | placeholder |

---

## PART 3 — HR placeholder-stub aggregates (~40 steps)

Source: `docs/audit_reports/hr-invariant-checklist.md` (107 missing invariants, 110 total).

### Step H0 — Build HR invariant checklist (already done in prior ferment)
- File: `docs/audit_reports/hr-invariant-checklist.md` (110 bullets)

### Real-aggregate partial closure (1 step)

#### Step H1 — Close 1 partial invariant
- Audit shows 1 [~] partial on existing aggregates — close it

### Placeholder-stub aggregates (~42 steps, one per aggregate)

| Step | Aggregate | Spec section | Stub location |
|---|---|---|---|
| H2 | Staff | aggregates.md § Staff | placeholder (8 invariants) |
| H3 | Department | § Department | placeholder |
| H4 | Designation | § Designation | placeholder |
| H5 | DesignationGrade | § DesignationGrade | placeholder |
| H6 | DepartmentHead | § DepartmentHead | placeholder |
| H7 | StaffRoleAssignment | § StaffRoleAssignment | placeholder |
| H8 | LeaveType | § LeaveType | placeholder |
| H9 | LeaveDefine | § LeaveDefine | placeholder |
| H10 | LeaveDefineAdjustment | § LeaveDefineAdjustment | placeholder |
| H11 | LeaveDeductionInfo | § LeaveDeductionInfo | placeholder |
| H12 | LeaveRequest | § LeaveRequest | placeholder |
| H13 | LeaveRequestApproval | § LeaveRequestApproval | placeholder |
| H14 | LeaveRequestAttachment | § LeaveRequestAttachment | placeholder |
| H15 | StaffLeaveBalance | § StaffLeaveBalance | placeholder |
| H16 | StaffLeaveHistory | § StaffLeaveHistory | placeholder |
| H17 | StaffAttendance | § StaffAttendance | placeholder |
| H18 | StaffAttendanceImport | § StaffAttendanceImport | placeholder |
| H19 | StaffAttendanceImportBatch | § StaffAttendanceImportBatch | placeholder |
| H20 | StaffAttendancePunch | § StaffAttendancePunch | placeholder |
| H21 | BulkImportJob | § BulkImportJob | placeholder |
| H22 | StaffImportBulkTemporary | § StaffImportBulkTemporary | placeholder |
| H23 | StaffImportResolution | § StaffImportResolution | placeholder |
| H24 | AssignClassTeacher | § AssignClassTeacher | placeholder |
| H25 | AssignClassTeacherScope | § AssignClassTeacherScope | placeholder |
| H26 | HourlyRate | § HourlyRate | placeholder |
| H27 | HourlyRateOverride | § HourlyRateOverride | placeholder |
| H28 | SalaryTemplate | § SalaryTemplate | placeholder |
| H29 | PayrollGenerate | § PayrollGenerate | placeholder |
| H30 | PayrollGenerateAudit | § PayrollGenerateAudit | placeholder |
| H31 | PayrollEarnDeduc | § PayrollEarnDeduc | placeholder |
| H32 | PayrollPaymentLink | § PayrollPaymentLink | placeholder |
| H33 | StaffAddress | § StaffAddress | placeholder |
| H34 | StaffBankDetail | § StaffBankDetail | placeholder |
| H35 | StaffCustomField | § StaffCustomField | placeholder |
| H36 | StaffDocument | § StaffDocument | placeholder |
| H37 | StaffDrivingLicense | § StaffDrivingLicense | placeholder |
| H38 | StaffPayrollHistory | § StaffPayrollHistory | placeholder |
| H39 | StaffProfilePhoto | § StaffProfilePhoto | placeholder |
| H40 | StaffRegistrationField | § StaffRegistrationField | placeholder |
| H41 | StaffRegistrationFieldOption | § StaffRegistrationFieldOption | placeholder |
| H42 | StaffSocialLink | § StaffSocialLink | placeholder |
| H43 | StaffTimeline | § StaffTimeline | placeholder |

---

## PART 4 — Remaining 7 domains (~90 steps)

For each: Step 0 = produce invariant checklist, Steps 1+ = per-aggregate.

### D-Attendance (~9 steps)
- Step Att-0: Produce `docs/audit_reports/attendance-invariant-checklist.md`
- Aggregates: StudentAttendance, StaffAttendance, SubjectAttendance, ExamAttendance, BulkAttendanceImport, ClassAttendance, AttendanceBulk, StudentAttendanceImport, StaffAttendanceImport

### D-Communication (~25 steps)
- Step Com-0: Produce `docs/audit_reports/communication-invariant-checklist.md`
- Aggregates: Notice, Complaint, ComplaintType, Notification, EmailLog, SmsLog, SmsTemplate, EmailSetting, SmsGateway, NotificationSetting, AbsentNotificationTimeSetup, ChatMessage, ChatConversation, ChatGroup, ChatGroupUser, ChatGroupMessageRecipient, ChatGroupMessageRemove, ChatBlockUser, ChatInvitation, ChatInvitationType, ChatStatusRecord, SendMessage, ContactMessage, SpeechSlider, PhoneCallLog

### D-Documents (~12 steps)
- Step Doc-0: Produce `docs/audit_reports/documents-invariant-checklist.md`
- Aggregates: NewFormDownload, UpdateFormDownload, FormDownload, FormDownloadFile, FormDownloadLink, NewPostalDispatch, UpdatePostalDispatch, PostalDispatch, PostalDispatchAttachment, NewPostalReceive, UpdatePostalReceive, PostalReceive, PostalReceiveAttachment

### D-Facilities (~14 steps)
- Step Fac-0: Produce `docs/audit_reports/facilities-invariant-checklist.md`
- Aggregates: Vehicle, Route, AssignVehicle, Dormitory, RoomType, Room, ItemCategory, Item, ItemStore, ItemReceive, ItemReceiveChild, ItemIssue, ItemSell, ItemSellChild, Supplier

### D-Library (~9 steps)
- Step Lib-0: Produce `docs/audit_reports/library-invariant-checklist.md`
- Aggregates: BookCategory, Book, LibraryMember, BookIssue, BookReturn, Fine, BookAcquisition, BookCatalogEntry, LibraryMemberNote

### D-CMS (~22 steps)
- Step Cms-0: Produce `docs/audit_reports/cms-invariant-checklist.md`
- Aggregates: NewPage, UpdatePage, Page, NewPageRevision, NewNews, UpdateNews, News, NewNewsCategory, NewsCategory, NewNewsComment, NewsComment, NewNewsPage, NewsPage, NewNoticeBoard, NoticeBoard, NewTestimonial, Testimonial, NewHomeSlider, HomeSlider, NewSpeechSlider, SpeechSlider, NewContent, UpdateContent, Content, NewContentType

### D-Events-Domain (~TBD steps)
- Step Evt-0: Produce `docs/audit_reports/events-domain-invariant-checklist.md`
- Aggregate list TBD (read `crates/cross-cutting/events-domain/src/aggregate.rs`)

---

## PART 5 — RBAC spec validation (11 steps)

### Step R0 — Build full RBAC map (extend prior ferment's TOML)
- Cross-reference all 681 explicit Capability annotations in `docs/specs/*/commands.md`
- Current TOML has 163 commands — expand to all 681
- Map each `required_capabilities()` in `crates/domains/*/src/commands.rs` against the spec

### Steps R1-R10 — Per-domain spec validation (10 steps)
- R1-academic (32 commands)
- R1-assessment (42)
- R1-attendance (14)
- R1-communication (72)
- R1-documents (10)
- R1-facilities (49)
- R1-finance (184 — largest, split into 3 sub-batches: F-fees 80, F-payment 60, F-payroll 44)
- R1-hr (61)
- R1-library (26)
- R1-cms (50)

### Step R11 — Verify 10+ rejection tests
- Already exist from prior ferment Wave 37 in `crates/cross-cutting/dispatcher/tests/forbidden_rejection.rs`
- Add per-domain coverage tests for any domains not yet covered

---

## PART 6 — Dispatcher wrappers (~509 wrappers, 10 steps)

Pattern: `crates/educore/src/dispatch.rs::dispatch_<verb>(cmd, deps)` per service function.

| Step | Domain | Wrapper count |
|---|---|---|
| W1 | Academic | 38 |
| W2 | Assessment | 74 |
| W3 | Attendance | 16 |
| W4 | Communication | 104 |
| W5 | Documents | 18 |
| W6 | Facilities | 59 |
| W7 | Finance | 66 |
| W8 | HR | 49 |
| W9 | Library | 48 |
| W10 | CMS | 37 |

Each step: build all wrappers + migrate storage-parity tests + verify cargo test --workspace.

---

## PART 7 — CI cross-compile verification (3 steps)

### Step CI1 — Verify aarch64 build in CI
- `.github/workflows/ci.yml` cross-compile job (already exists)
- Add `apt install gcc-aarch64-linux-gnu` if needed in CI image

### Step CI2 — Verify wasm32 build in CI
- `cargo build --target wasm32-unknown-unknown` for `educore-core` + `educore-storage-surrealdb`
- CI must have clang installed (for ring v0.17.14)

### Step CI3 — Load test at 100×10k baseline in CI
- `crates/tools/loadtest/` at full scale
- Publish benchmarks

---

## TOTAL SCOPE

| Part | Steps |
|---|---|
| Part 1: Academic placeholders | 16 |
| Part 2: Finance | ~57 |
| Part 3: HR | ~43 |
| Part 4: 7 remaining domains | ~93 |
| Part 5: RBAC | 11 |
| Part 6: Dispatcher wrappers | 10 |
| Part 7: CI | 3 |
| **TOTAL** | **~233 steps** |

**Realistic timeline:** ~233 focused steps × 30-50 turns each = **months of focused engineering work**, not a single ferment.

---

## Verification (per step)

1. **Spec review:** Read `docs/specs/<domain>/aggregates.md` § <aggregate> BEFORE coding
2. **Implementation:** Real aggregate struct or service function (no placeholder)
3. **Test:** Behavioral integration test that proves invariant violation is REJECTED
4. **Verify:** `cargo test -p <domain> --tests --no-fail-fast` passes
5. **Checklist update:** Flip [ ] → [x] in relevant checklist
6. **Worktree + merge:** Standard worktree pattern, manual merge, worktree removal
7. **Commit:** Single-purpose commit naming the invariant reached

---

## Success criteria (new ferment)

1. Part 1 complete: 16 steps, 51 academic invariants reach [x]
2. Part 2 complete: 57 steps, 155 finance invariants reach [x]
3. Part 3 complete: 43 steps, 108 HR invariants reach [x]
4. Part 4 complete: 93 steps, 7 remaining domains covered
5. Part 5 complete: 11 steps, 540 RBAC mappings spec-validated
6. Part 6 complete: 10 steps, 509 dispatcher wrappers built
7. Part 7 complete: 3 steps, CI cross-compile green

**Total: 233 steps to reach grade A.**

---

## Re-grade target

**Starting grade:** D (from prior ferment)
**Target grade:** A (all spec invariants enforced with behavioral tests)

**No goal erosion:** If a step can't be completed, the gate evidence must show real reasons, not a re-scoped goal. Either the work is done or explicitly deferred.
