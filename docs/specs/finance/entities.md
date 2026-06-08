# Finance Domain — Entities

Entities have identity and lifecycle but are not aggregate roots. They
are loaded and persisted only through their aggregate root.

## FeesPaymentSlip

**Identity:** `FeesPaymentSlipId(SchoolId, Uuid)`
**Owner:** `FeesPayment`

A reference to a scanned bank slip attached to a payment. Has
`SlipReference` (file storage pointer) and a `Note`.

## FeesPaymentFine

**Identity:** `FeesPaymentFineId(SchoolId, Uuid)`
**Owner:** `FeesPayment`

A fine captured at the time of payment. Has `FineTitle`, `FineAmount`,
`Note`, and a reference to a `FeesType`.

## FeesInstallmentAssignDiscount

**Identity:** `FeesInstallmentAssignDiscountId(SchoolId, Uuid)`
**Owner:** `FeesInstallmentAssign`

A discount applied to a specific installment assignment. Has
`DiscountAmount` and a reference to a `FeesDiscount`.

## DirectFeesInstallmentAssignChild

**Identity:** `DirectFeesInstallmentAssignChildId(SchoolId, Uuid)`
**Owner:** `DirectFeesInstallmentAssign`

A child line on a direct installment assignment, e.g. for tracking
slip, mode, and partial-paid status. Has `ActiveStatus`, `Slip`,
`PaymentMode`, `PaymentDate`, `BankId`, `DiscountAmount`,
`FeesTypeId`.

## FmFeesInvoiceLineNote

**Identity:** `FmFeesInvoiceLineNoteId(SchoolId, Uuid)`
**Owner:** `FmFeesInvoiceChild`

A free-text note attached to a child invoice line.

## FmFeesTransactionLineNote

**Identity:** `FmFeesTransactionLineNoteId(SchoolId, Uuid)`
**Owner:** `FmFeesTransactionChild`

A free-text note attached to a transaction child line.

## BankStatementAttachment

**Identity:** `BankStatementAttachmentId(SchoolId, Uuid)`
**Owner:** `BankStatement`

A receipt or file attached to a bank statement.

## PayrollPaymentApproval

**Identity:** `PayrollPaymentApprovalId(SchoolId, Uuid)`
**Owner:** `PayrollPayment`

The approval state of a payroll payment. Has `ApprovedAt`,
`ApprovedBy`, `RejectionReason`.

## BankPaymentSlipAudit

**Identity:** `BankPaymentSlipAuditId(SchoolId, Uuid)`
**Owner:** `BankPaymentSlip`

Audit log for a slip: `CreatedAt`, `CreatedBy`, `ApprovedBy`,
`RejectedBy`, `Reason`.

## ExpenseApproval

**Identity:** `ExpenseApprovalId(SchoolId, Uuid)`
**Owner:** `Expense`

The approval state of an expense. Has `ApproverId`, `Decision`,
`DecisionAt`, `Note`.

## IncomeApproval

**Identity:** `IncomeApprovalId(SchoolId, Uuid)`
**Owner:** `Income`

The approval state of an income. Has `ApproverId`, `Decision`,
`DecisionAt`, `Note`.

## WalletTransactionApproval

**Identity:** `WalletTransactionApprovalId(SchoolId, Uuid)`
**Owner:** `WalletTransaction`

The approval state of a wallet transaction. Has `ApproverId`,
`Decision`, `DecisionAt`, `RejectNote`.

## DirectFeesInstallmentDueLog

**Identity:** `DirectFeesInstallmentDueLogId(SchoolId, Uuid)`
**Owner:** `DirectFeesInstallment`

A log of the most recent notification dispatch for a due installment.
Has `Channel`, `DispatchedAt`, `RecipientUserId`, `MessageReference`.

## FeesAssignClosure

**Identity:** `FeesAssignClosureId(SchoolId, Uuid)`
**Owner:** `FeesAssign`

The closure state of an assignment. Has `ClosedAt`, `ClosedBy`,
`ClosureReason`, `FinalBalance`.

## FeesAssignDiscountApplication

**Identity:** `FeesAssignDiscountApplicationId(SchoolId, Uuid)`
**Owner:** `FeesAssignDiscount`

A record of a single discount application. Has `AppliedAmount`,
`AppliedAt`, `Reference` (invoice id, payment id, or installment id).

## DonorPhoto

**Identity:** `DonorPhotoId(SchoolId, Uuid)`
**Owner:** `Donor`

The stored photo of a donor. Has `FileReference`, `UploadedAt`.

## DonorCustomField

**Identity:** `DonorCustomFieldId(SchoolId, Uuid)`
**Owner:** `Donor`

A user-defined field on a donor profile. Has `Key`, `Value`,
`FormName`.

## ProductPurchasePayment

**Identity:** `ProductPurchasePaymentId(SchoolId, Uuid)`
**Owner:** `ProductPurchase`

A payment row against a product purchase. Has `Amount`,
`PaymentDate`, `PaymentMethod`, `Reference`, `BankId`, `Note`.

## InventoryPaymentReference

**Identity:** `InventoryPaymentReferenceId(SchoolId, Uuid)`
**Owner:** `InventoryPayment`

A cross-reference from an inventory payment to the underlying
item-receive or item-sell row.

## AmountTransferLeg

**Identity:** `AmountTransferLegId(SchoolId, Uuid)`
**Owner:** `AmountTransfer`

The two legs of a transfer. Each leg has `BankId`, `Direction`
(`debit` or `credit`), `Amount`, and the resulting `BankStatementId`.

## ChartOfAccountBalance

**Identity:** `ChartOfAccountBalanceId(SchoolId, Uuid)`
**Owner:** `ChartOfAccount`

The cached running balance of the chart of account. Has
`AsOf`, `DebitTotal`, `CreditTotal`, `Balance`.

## QuestionBankFeeMapping

**Identity:** `QuestionBankFeeMappingId(SchoolId, Uuid)`
**Owner:** `QuestionBankFee`

The mapping of a question bank to a `fees_type_id` and an optional
`class_id` and `section_id` for scoping.

## PaymentGatewayMode

**Identity:** `PaymentGatewayModeId(SchoolId, Uuid)`
**Owner:** `PaymentGatewaySetting`

The mode toggle and sandbox/live indicator. Has `Mode` (`sandbox` or
`live`), `SwitchedAt`, `SwitchedBy`.

## FeesReminderDispatch

**Identity:** `FeesReminderDispatchId(SchoolId, Uuid)`
**Owner:** `DirectFeesReminder`

A record of a single dispatch. Has `DispatchedAt`,
`NotificationType`, `RecipientUserId`, `MessageReference`,
`Status`.

## DirectFeesSettingOverride

**Identity:** `DirectFeesSettingOverrideId(SchoolId, Uuid)`
**Owner:** `DirectFeesSetting`

A per-class or per-fees-master override of the global direct-fees
setting. Has `ClassId`, `FeesMasterId`, `OverrideNoInstallment`,
`OverrideDueDateFromSem`, `OverrideEndDay`.

## FeesInstallmentCreditApplication

**Identity:** `FeesInstallmentCreditApplicationId(SchoolId, Uuid)`
**Owner:** `FeesInstallmentCredit`

A record of the credit being applied to a specific installment. Has
`InstallmentAssignId`, `AppliedAmount`, `AppliedAt`.

## BankPaymentSlipCounter

**Identity:** `BankPaymentSlipCounterId(SchoolId, Uuid)`
**Owner:** `BankAccount`

The running count of slips for a bank account in an academic year.
Has `Year`, `Count`.

## FeesDiscountEligibility

**Identity:** `FeesDiscountEligibilityId(SchoolId, Uuid)`
**Owner:** `FeesDiscount`

The eligibility rules of a discount. Has `StudentCategoryId`,
`ClassId`, `SectionId`, `MinimumAttendance`, `EffectiveFrom`,
`EffectiveTo`.

## PayrollEarningType

**Identity:** `PayrollEarningTypeId(SchoolId, Uuid)`
**Owner:** `SalaryTemplate`

A named earning type (e.g. "Basic", "HRA", "Travel"). Has `Name`,
`Computation` (fixed, percentage, formula), `Value`.

## PayrollDeductionType

**Identity:** `PayrollDeductionTypeId(SchoolId, Uuid)`
**Owner:** `SalaryTemplate`

A named deduction type (e.g. "PF", "Tax", "Loan"). Has `Name`,
`Computation`, `Value`.

## LeaveDeductionInfo

**Identity:** `LeaveDeductionInfoId(SchoolId, Uuid)`
**Owner:** `PayrollEarnDeduc` (or `PayrollGenerate`)

The leave-deduction line on a payroll. Carries `StaffId`,
`PayrollId`, `ExtraLeave`, `SalaryDeduct`, `PayMonth`, `PayYear`,
`ActiveStatus`. This is the typed projection of the HR-owned
`SmLeaveDeductionInfo` row.

## HourlyRateRow

**Identity:** `HourlyRateId(SchoolId, Uuid)`
**Owner:** `Staff` (HR) — viewed in finance when building payroll

A per-grade hourly rate. Has `Grade`, `Rate`. (The grade is
matched against the staff's `hourly_grade`.)

## InventoryItemReceive

**Identity:** `InventoryItemReceiveId(SchoolId, Uuid)`
**Owner:** `InventoryPayment`

The inventory receive record that the payment is applied to.

## InventoryItemSell

**Identity:** `InventoryItemSellId(SchoolId, Uuid)`
**Owner:** `InventoryPayment`

The inventory sell record that the payment is applied to.

## BankStatementGroup

**Identity:** `BankStatementGroupId(SchoolId, Uuid)`
**Owner:** `BankStatement`

A logical grouping of statements for a single payment event
(e.g. an invoice payment might produce two statements: one fees
credit and one gateway debit).

## PayrollPaymentReceipt

**Identity:** `PayrollPaymentReceiptId(SchoolId, Uuid)`
**Owner:** `PayrollPayment`

A receipt issued for a payroll payment. Has `ReceiptNumber`,
`ReceiptDate`, `ReceiptFile`.

## QuestionBankMuOption

**Identity:** `QuestionBankMuOptionId(SchoolId, Uuid)`
**Owner:** `QuestionBank`

A multiple-choice option for a question bank item. Has `Title`,
`Status` (correct/incorrect). (Treated as a finance-side concern
because the question bank item itself carries a fees amount.)
