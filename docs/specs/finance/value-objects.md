# Finance Domain — Value Objects

Value objects are immutable, validated at construction, and have no
identity. They are compared by value.

## Identifiers

All identifiers in the finance domain are typed and tenant-scoped. The
generic `Id<S, T>` wrapper carries the `SchoolId` of the owning school
and the local id.

| Identifier                              | Backing Type               | Notes                                |
| --------------------------------------- | -------------------------- | ------------------------------------ |
| `FeesGroupId`                           | `Id<FeesGroup>`            | A fees group                         |
| `FeesTypeId`                            | `Id<FeesType>`             | A fees type                          |
| `FeesMasterId`                          | `Id<FeesMaster>`           | A class+type fees master             |
| `FeesAssignId`                          | `Id<FeesAssign>`           | A per-student assignment             |
| `FeesAssignDiscountId`                  | `Id<FeesAssignDiscount>`   | A per-student discount               |
| `FeesDiscountId`                        | `Id<FeesDiscount>`         | A discount catalog entry             |
| `FeesInvoiceId`                         | `Id<FeesInvoice>`          | The classic invoice counter row      |
| `FeesInstallmentId`                     | `Id<FeesInstallment>`      | A classic installment plan           |
| `FeesInstallmentAssignId`               | `Id<FeesInstallmentAssign>`| A per-student installment assignment |
| `FeesPaymentId`                         | `Id<FeesPayment>`          | A payment against an assign          |
| `FeesPaymentSlipId`                     | `Id<FeesPaymentSlip>`      | A scanned slip on a payment          |
| `FeesPaymentFineId`                     | `Id<FeesPaymentFine>`      | A fine on a payment                  |
| `FeesCarryForwardId`                    | `Id<FeesCarryForward>`     | A carry-forward balance              |
| `FeesCarryForwardLogId`                 | `Id<FeesCarryForwardLog>`  | A carry-forward audit row            |
| `FeesCarryForwardSettingId`             | `Id<FeesCarryForwardSetting>` | Carry-forward configuration         |
| `FeesInstallmentCreditId`               | `Id<FeesInstallmentCredit>`| A pre-paid credit                    |
| `DirectFeesInstallmentId`               | `Id<DirectFeesInstallment>`| A direct installment plan            |
| `DirectFeesInstallmentAssignId`         | `Id<DirectFeesInstallmentAssign>` | A per-student direct assignment |
| `DirectFeesInstallmentChildPaymentId`   | `Id<...>`                  | A payment against a direct assignment|
| `DirectFeesReminderId`                  | `Id<DirectFeesReminder>`   | A reminder rule                      |
| `DirectFeesSettingId`                   | `Id<DirectFeesSetting>`    | The direct-fees global setting       |
| `FmFeesGroupId`                         | `Id<FmFeesGroup>`          | An FM fees group                     |
| `FmFeesTypeId`                          | `Id<FmFeesType>`           | An FM fees type                      |
| `FmFeesInvoiceId`                       | `Id<FmFeesInvoice>`        | An FM invoice                        |
| `FmFeesInvoiceChildId`                  | `Id<FmFeesInvoiceChild>`   | An FM invoice line                   |
| `FmFeesInvoiceSettingId`                | `Id<FmFeesInvoiceSetting>` | FM invoice numbering config          |
| `FmFeesTransactionId`                   | `Id<FmFeesTransaction>`    | An FM transaction                    |
| `FmFeesTransactionChildId`              | `Id<FmFeesTransactionChild>`| An FM transaction line              |
| `FmFeesWeaverId`                        | `Id<FmFeesWeaver>`         | An FM weaver                         |
| `FeesInvoiceSettingId`                  | `Id<FeesInvoiceSetting>`   | Classic invoice layout               |
| `InvoiceSettingId`                      | `Id<InvoiceSetting>`       | FM invoice layout                    |
| `BankAccountId`                         | `Id<BankAccount>`          | A bank or cash account               |
| `BankStatementId`                       | `Id<BankStatement>`        | A bank statement                     |
| `BankPaymentSlipId`                     | `Id<BankPaymentSlip>`      | A bank payment slip                  |
| `ExpenseId`                             | `Id<Expense>`              | An expense                           |
| `IncomeId`                              | `Id<Income>`               | An income                            |
| `DonorId`                               | `Id<Donor>`                | A donor profile                      |
| `ExpenseHeadId`                         | `Id<ExpenseHead>`          | An expense category                  |
| `IncomeHeadId`                          | `Id<IncomeHead>`           | An income category                   |
| `WalletTransactionId`                   | `Id<WalletTransaction>`    | A wallet movement                    |
| `TransactionId`                         | `Id<Transaction>`          | A journal line                       |
| `PayrollPaymentId`                      | `Id<PayrollPayment>`       | A payroll payment                    |
| `PayrollGenerateId`                     | `Id<PayrollGenerate>`      | A monthly payroll run                |
| `PayrollEarnDeducId`                    | `Id<PayrollEarnDeduc>`     | A payroll earnings/deductions line   |
| `SalaryTemplateId`                      | `Id<SalaryTemplate>`       | A reusable salary grade              |
| `ProductPurchaseId`                     | `Id<ProductPurchase>`      | A vendor product purchase            |
| `InventoryPaymentId`                    | `Id<InventoryPayment>`     | An inventory payment                 |
| `AmountTransferId`                      | `Id<AmountTransfer>`       | A transfer between accounts          |
| `ChartOfAccountId`                      | `Id<ChartOfAccount>`       | A ledger category                    |
| `QuestionBankFeeId`                     | `Id<QuestionBankFee>`      | A fees mapping for a question bank   |
| `PaymentGatewaySettingId`               | `Id<PaymentGatewaySetting>`| A gateway configuration              |
| `PaymentMethodId`                       | `Id<PaymentMethod>`        | A payment method                     |
| `DueFeesLoginPreventId`                 | `Id<DueFeesLoginPrevent>`  | A login-prevent row                  |

## Money

| Type                 | Constraints                                                       |
| -------------------- | ----------------------------------------------------------------- |
| `Money`              | `i64` minor units, currency `Currency`, validated non-negative   |
| `Currency`           | ISO-4217 alpha-3 (e.g. `USD`, `INR`, `EUR`, `GBP`)                |
| `Amount`             | `Money` with explicit currency matching the school's default     |
| `FeeAmount`          | `Amount` constrained `0 <= x <= 1_000_000.00`                     |
| `FineAmount`         | `Amount` constrained `0 <= x <= 100_000.00`                       |
| `DiscountAmount`     | `Amount` constrained `0 <= x <= 1_000_000.00`                     |
| `WeaverAmount`       | `Amount` (negative or positive in some flows; here non-negative) |
| `ServiceCharge`      | `Amount` constrained `0 <= x <= 50_000.00`                       |
| `Balance`            | `Amount` allowed to be zero but not negative                     |
| `BalanceType`        | `Debit`, `Credit`                                                 |
| `TotalEarning`       | `Amount`                                                          |
| `TotalDeduction`     | `Amount`                                                          |
| `GrossSalary`        | `Amount`                                                          |
| `NetSalary`          | `Amount`                                                          |
| `BasicSalary`        | `Amount`                                                          |
| `Tax`                | `Amount` constrained `0 <= x <= 1_000_000.00`                     |
| `HourlyRate`         | `Amount` constrained `0 < x <= 100_000.00`                        |
| `OvertimeRate`       | `Amount`                                                          |

## Percentages & Rounding

| Type                 | Constraints                                                       |
| -------------------- | ----------------------------------------------------------------- |
| `FeePercentage`      | `f32` in `[0, 100]`                                                |
| `DiscountPercentage` | `f32` in `[0, 100]`                                                |
| `TaxPercentage`      | `f32` in `[0, 100]`                                                |
| `ServiceChargeType`  | `Percentage`, `Flat`                                               |
| `PerThousand`        | `u16` in `0..=10`                                                  |
| `RoundingPolicy`     | `HalfUp`, `HalfEven`, `Truncate`                                   |

## Invoice & Receipt

| Type                 | Constraints                                                       |
| -------------------- | ----------------------------------------------------------------- |
| `InvoiceNumber`      | `String` 1..50 chars, unique within school; format `prefix + seq` |
| `InvoicePrefix`      | `String` 1..10 chars                                              |
| `InvoiceStartForm`   | `u32`                                                             |
| `ReceiptNumber`      | `String` 1..50 chars, unique within school                        |
| `ReferenceNumber`    | `String` 1..50 chars (bank/cheque reference)                      |
| `SlipReference`      | `FileReference` to a stored slip                                  |
| `InvoiceType`        | `Invoice`, `Slip`                                                 |
| `InvoicePosition`    | `Top`, `Middle`, `Bottom` (legacy ordering)                       |
| `InvoiceCopy`        | `Parent`, `Office`, `Cashier`                                     |
| `SignatureSlot`      | `Parent`, `Cashier`, `Officer`                                    |

## Payment Status

| Type                 | Values                                                            |
| -------------------- | ----------------------------------------------------------------- |
| `PaymentStatus`      | `Unpaid`, `Partial`, `Paid`, `Overpaid`                           |
| `ApprovalStatus`     | `Pending`, `Approved`, `Rejected`                                 |
| `PayrollStatus`      | `NotGenerated`, `Generated`, `Paid`                               |
| `WalletTxStatus`     | `Pending`, `Approved`, `Rejected`                                 |
| `WalletTxType`       | `Deposit`, `Refund`, `Expense`, `FeesRefund`                      |

## Payment Method

| Type                 | Values                                                            |
| -------------------- | ----------------------------------------------------------------- |
| `PaymentMethodKind`  | `Cash`, `Bank`, `Cheque`, `Card`, `Mobile`, `Gateway`             |
| `GatewayMode`        | `Sandbox`, `Live`                                                  |
| `GatewayName`        | `Stripe`, `PayPal`, `Razorpay`, `Paytm`, `Other` (string list)    |
| `BankMode`           | `Bk` (bank transfer), `Cq` (cheque)                              |
| `PaymentDirection`   | `Debit`, `Credit`                                                 |
| `AccountType`        | `Bank`, `Cash`                                                    |

## Bank

| Type                 | Constraints                                                       |
| -------------------- | ----------------------------------------------------------------- |
| `BankAccountNumber`  | 6..34 chars, alphanumeric                                         |
| `IfscCode`           | 11 chars, format `[A-Z]{4}0[A-Z0-9]{6}`                           |
| `ChequeNumber`       | 6 digits                                                          |
| `TransactionId`      | 1..100 chars, unique per gateway                                  |
| `BankName`           | 1..200 chars                                                      |
| `BranchName`         | 1..200 chars                                                      |
| `AccountHolderName`  | 1..200 chars                                                      |
| `OpeningBalance`     | `Amount`                                                          |
| `CurrentBalance`     | `Amount`                                                          |

## Discount

| Type                 | Constraints                                                       |
| -------------------- | ----------------------------------------------------------------- |
| `DiscountType`       | `Once`, `Year`                                                     |
| `DiscountCode`       | 1..50 chars                                                       |
| `DiscountName`       | 1..200 chars                                                      |
| `DiscountAmount`     | `Amount`                                                          |

## Payroll

| Type                 | Constraints                                                       |
| -------------------- | ----------------------------------------------------------------- |
| `PayPeriod`          | `(Month, Year)` with `Month in 1..=12`                           |
| `EarnDeducType`      | `Earning`, `Deduction` (encoded `e` / `d` in storage)              |
| `PayrollNote`        | 0..200 chars                                                      |
| `SalaryGrade`        | 1..200 chars                                                      |
| `HouseRent`          | `Amount`                                                          |
| `ProvidentFund`      | `Amount`                                                          |

## Carry Forward

| Type                 | Constraints                                                       |
| -------------------- | ----------------------------------------------------------------- |
| `CarryForwardAmount` | `Amount`                                                          |
| `FeesDueDays`        | `u16` in `0..=365`                                                |
| `CarryForwardTitle`  | 1..200 chars                                                      |

## Reminder

| Type                 | Constraints                                                       |
| -------------------- | ----------------------------------------------------------------- |
| `DaysBeforeDue`      | `u16` in `0..=365`                                                |
| `NotificationChannel`| `Sms`, `Email`, `Push`, `Portal`, `Whatsapp`                      |
| `ReminderTitle`      | 1..200 chars                                                      |

## Login Prevention

| Type                 | Constraints                                                       |
| -------------------- | ----------------------------------------------------------------- |
| `PreventReason`      | `OverdueFees`                                                     |
| `BlockSource`        | `Manual`, `Auto`                                                  |

## Installment Credit

| Type                 | Constraints                                                       |
| -------------------- | ----------------------------------------------------------------- |
| `CreditAmount`       | `Amount`                                                          |
| `CreditStatus`       | `Active`, `Consumed`, `Cancelled`                                 |

## Question Bank Fees

| Type                 | Constraints                                                       |
| -------------------- | ----------------------------------------------------------------- |
| `QuestionBankType`   | `Multi`, `TrueFalse`, `FillBlanks`                                |
| `QuestionBankStatus` | `Active`, `Inactive`                                              |

## Status Enums

| Type                 | Values                                                            |
| -------------------- | ----------------------------------------------------------------- |
| `ActiveStatus`       | `Active`, `Inactive`                                              |
| `PaymentType`        | `Receive`, `Sell` (encoded `R`, `S` in storage)                  |
| `AccountDirection`   | `Debit`, `Credit`                                                 |
| `FmInvoiceType`      | `Fees`, `Lms`                                                     |
| `FmPaymentType`      | `Fees`, `Lms`                                                     |

## Donor

| Type                 | Constraints                                                       |
| -------------------- | ----------------------------------------------------------------- |
| `DonorName`          | 1..200 chars                                                      |
| `DonorProfession`    | 0..200 chars                                                      |
| `DonorAddress`       | 0..500 chars                                                      |
| `ShowPublic`         | `bool`                                                            |

## Inventory

| Type                 | Constraints                                                       |
| -------------------- | ----------------------------------------------------------------- |
| `ProductPackage`     | 1..10 chars                                                       |
| `ExpiryDate`         | `NaiveDate`                                                       |
| `PurchaseDate`       | `NaiveDate`                                                       |
| `ItemReceiveId`      | `u64`                                                             |
| `ItemSellId`         | `u64`                                                             |

## Bank Statement

| Type                 | Constraints                                                       |
| -------------------- | ----------------------------------------------------------------- |
| `StatementType`      | `Income`, `Expense`                                               |
| `StatementDetails`   | 0..500 chars                                                      |
| `AfterBalance`       | `Amount`                                                          |

## School Identity Bindings

| Type                  | Notes                                                       |
| --------------------- | ----------------------------------------------------------- |
| `SchoolId`            | From `smsengine-platform`                                     |
| `TenantContext`       | `(SchoolId, UserId, ...)` from `smsengine-platform`           |
| `UserId`              | From `smsengine-platform`                                     |
| `AcademicYearId`      | From `smsengine-academic`                                     |
| `StudentId`           | From `smsengine-academic`                                     |
| `StudentRecordId`     | From `smsengine-academic`                                     |
| `ClassId`             | From `smsengine-academic`                                     |
| `SectionId`           | From `smsengine-academic`                                     |
| `StaffId`             | From `smsengine-hr`                                           |
| `PayrollGenerateId`   | From `smsengine-hr` (queried by finance)                      |

## Validation Rules

All value objects implement `Validate` and refuse construction when
validation fails:

```rust
pub trait Validate {
    fn validate(&self) -> Result<(), ValueError>;
}
```

Construction is the only entry point:

```rust
let amount = Amount::new(Money::INR, 1500_00)?;
```

Parsing returns `Result<T, ValueError>`. There are no setters that
bypass validation.
