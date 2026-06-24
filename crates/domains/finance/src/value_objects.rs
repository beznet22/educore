//! # Finance value objects
//!
//! The typed ids (every aggregate is keyed by one), the validated
//! value objects, and the closed enums the finance aggregates depend
//! on. Per `docs/specs/finance/value-objects.md`:
//!
//! - Every id is `Id { school_id, value }` — a typed wrapper that
//!   carries the school anchor so the type system catches
//!   cross-tenant confusion at compile time.
//! - Money is `MinorUnits` (i64 cents/paisa) per the build-plan §
//!   "Risks" — no floats, no `as` casts; all conversions use
//!   `TryFrom` / `TryInto`.
//! - Foreign-key typed ids (`StudentId`, `ClassId`, `AcademicYearId`,
//!   `StaffId`, `PayrollGenerateId`, `PayrollEarnDeducId`,
//!   `SalaryTemplateId`) are **re-exported** from
//!   [`educore_academic`](::educore_academic) and
//!   [`educore_hr`](::educore_hr); the finance crate owns only the
//!   finance-specific ids.
//!
//! Phase 7 ships 51 finance-defined typed ids (plus 3 HR re-exports
//! for `PayrollGenerateId`, `PayrollEarnDeducId`, `SalaryTemplateId`),
//! the closed enums from the spec, and the `Money` value object.

#![allow(missing_docs)]
#![allow(unused_imports)]

use std::fmt;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::error::{DomainError, Result};
use educore_core::ids::SchoolId;

pub use educore_academic::AcademicYearId;
pub use educore_academic::ClassId;
pub use educore_academic::SectionId;
pub use educore_academic::StudentId;
pub use educore_academic::StudentRecordId;
pub use educore_academic::SubjectId;
pub use educore_hr::value_objects::PayrollEarnDeducId;
pub use educore_hr::value_objects::PayrollGenerateId;
pub use educore_hr::value_objects::RoleId;
pub use educore_hr::value_objects::SalaryTemplateId;
pub use educore_hr::value_objects::StaffId;
pub use educore_rbac::ids::RoleId as RbacRoleId;

// =============================================================================
// Macro: typed finance id
// =============================================================================

/// Macro to define the per-aggregate typed id wrapper. Every
/// finance id follows the same shape: a `school_id` anchor plus a
/// local `Uuid`. The wrapper implements
/// [`Clone`], [`Copy`], [`PartialEq`], [`Eq`], [`Hash`], and
/// the `Display` format `"{school_id}/{value}"`.
///
/// The pattern matches `educore-hr::value_objects::*` and
/// `educore-academic::value_objects::*` so the engine's id types
/// stay consistent across crates.
macro_rules! finance_typed_id {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident;
    ) => {
        $(#[$attr])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        $vis struct $name {
            /// The owning school (tenant anchor).
            pub school_id: SchoolId,
            /// The local id (UUIDv7).
            pub value: Uuid,
        }

        impl $name {
            /// Constructs a new typed id from its parts.
            #[must_use]
            pub const fn new(school_id: SchoolId, value: Uuid) -> Self {
                Self { school_id, value }
            }

            /// Returns the local UUID.
            #[must_use]
            pub const fn as_uuid(&self) -> Uuid {
                self.value
            }

            /// Returns the owning school id.
            #[must_use]
            pub const fn school_id(&self) -> SchoolId {
                self.school_id
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}/{}", self.school_id, self.value)
            }
        }
    };
}

// =============================================================================
// Typed ids: 44 aggregate roots
// =============================================================================

finance_typed_id! {
    /// A typed id for a [`FeesGroup`](crate::aggregate::FeesGroup).
    pub struct FeesGroupId;
}
finance_typed_id! {
    /// A typed id for a [`FeesType`](crate::aggregate::FeesType).
    pub struct FeesTypeId;
}
finance_typed_id! {
    /// A typed id for a [`FeesMaster`](crate::aggregate::FeesMaster).
    pub struct FeesMasterId;
}
finance_typed_id! {
    /// A typed id for a [`FeesAssign`](crate::aggregate::FeesAssign).
    pub struct FeesAssignId;
}
finance_typed_id! {
    /// A typed id for a [`FeesAssignDiscount`](crate::aggregate::FeesAssignDiscount).
    pub struct FeesAssignDiscountId;
}
finance_typed_id! {
    /// A typed id for a [`FeesDiscount`](crate::aggregate::FeesDiscount).
    pub struct FeesDiscountId;
}
finance_typed_id! {
    /// A typed id for a [`FeesInvoice`](crate::aggregate::FeesInvoice).
    pub struct FeesInvoiceId;
}
finance_typed_id! {
    /// A typed id for a [`FeesInstallment`](crate::aggregate::FeesInstallment).
    pub struct FeesInstallmentId;
}
finance_typed_id! {
    /// A typed id for a [`FeesInstallmentAssign`](crate::aggregate::FeesInstallmentAssign).
    pub struct FeesInstallmentAssignId;
}
finance_typed_id! {
    /// A typed id for a [`FeesPayment`](crate::aggregate::FeesPayment).
    pub struct FeesPaymentId;
}
finance_typed_id! {
    /// A typed id for a [`FeesPaymentSlip`](crate::entities::FeesPaymentSlip).
    pub struct FeesPaymentSlipId;
}
finance_typed_id! {
    /// A typed id for a [`FeesPaymentFine`](crate::entities::FeesPaymentFine).
    pub struct FeesPaymentFineId;
}
finance_typed_id! {
    /// A typed id for a [`FeesCarryForward`](crate::aggregate::FeesCarryForward).
    pub struct FeesCarryForwardId;
}
finance_typed_id! {
    /// A typed id for a [`FeesCarryForwardLog`](crate::aggregate::FeesCarryForwardLog).
    pub struct FeesCarryForwardLogId;
}
finance_typed_id! {
    /// A typed id for a [`FeesCarryForwardSetting`](crate::aggregate::FeesCarryForwardSetting).
    pub struct FeesCarryForwardSettingId;
}
finance_typed_id! {
    /// A typed id for a [`FeesInstallmentCredit`](crate::aggregate::FeesInstallmentCredit).
    pub struct FeesInstallmentCreditId;
}
finance_typed_id! {
    /// A typed id for a [`DirectFeesInstallment`](crate::aggregate::DirectFeesInstallment).
    pub struct DirectFeesInstallmentId;
}
finance_typed_id! {
    /// A typed id for a [`DirectFeesInstallmentAssign`](crate::aggregate::DirectFeesInstallmentAssign).
    pub struct DirectFeesInstallmentAssignId;
}
finance_typed_id! {
    /// A typed id for a [`DirectFeesInstallmentChildPayment`](crate::aggregate::DirectFeesInstallmentChildPayment).
    pub struct DirectFeesInstallmentChildPaymentId;
}
finance_typed_id! {
    /// A typed id for a [`DirectFeesReminder`](crate::aggregate::DirectFeesReminder).
    pub struct DirectFeesReminderId;
}
finance_typed_id! {
    /// A typed id for a [`DirectFeesSetting`](crate::aggregate::DirectFeesSetting).
    pub struct DirectFeesSettingId;
}
finance_typed_id! {
    /// A typed id for a [`FmFeesGroup`](crate::aggregate::FmFeesGroup).
    pub struct FmFeesGroupId;
}
finance_typed_id! {
    /// A typed id for a [`FmFeesType`](crate::aggregate::FmFeesType).
    pub struct FmFeesTypeId;
}
finance_typed_id! {
    /// A typed id for a [`FmFeesInvoice`](crate::aggregate::FmFeesInvoice).
    pub struct FmFeesInvoiceId;
}
finance_typed_id! {
    /// A typed id for a [`FmFeesInvoiceChild`](crate::aggregate::FmFeesInvoiceChild).
    pub struct FmFeesInvoiceChildId;
}
finance_typed_id! {
    /// A typed id for a [`FmFeesInvoiceSetting`](crate::aggregate::FmFeesInvoiceSetting).
    pub struct FmFeesInvoiceSettingId;
}
finance_typed_id! {
    /// A typed id for a [`FmFeesTransaction`](crate::aggregate::FmFeesTransaction).
    pub struct FmFeesTransactionId;
}
finance_typed_id! {
    /// A typed id for a [`FmFeesTransactionChild`](crate::aggregate::FmFeesTransactionChild).
    pub struct FmFeesTransactionChildId;
}
finance_typed_id! {
    /// A typed id for a [`FmFeesWeaver`](crate::aggregate::FmFeesWeaver).
    pub struct FmFeesWeaverId;
}
finance_typed_id! {
    /// A typed id for a [`FeesInvoiceSetting`](crate::aggregate::FeesInvoiceSetting).
    pub struct FeesInvoiceSettingId;
}
finance_typed_id! {
    /// A typed id for an [`InvoiceSetting`](crate::aggregate::InvoiceSetting).
    pub struct InvoiceSettingId;
}
finance_typed_id! {
    /// A typed id for a [`BankAccount`](crate::aggregate::BankAccount).
    pub struct BankAccountId;
}
finance_typed_id! {
    /// A typed id for a [`BankStatement`](crate::aggregate::BankStatement).
    pub struct BankStatementId;
}
finance_typed_id! {
    /// A typed id for a [`BankPaymentSlip`](crate::aggregate::BankPaymentSlip).
    pub struct BankPaymentSlipId;
}
finance_typed_id! {
    /// A typed id for an [`Expense`](crate::aggregate::Expense).
    pub struct ExpenseId;
}
finance_typed_id! {
    /// A typed id for an [`Income`](crate::aggregate::Income).
    pub struct IncomeId;
}
finance_typed_id! {
    /// A typed id for a [`Donor`](crate::aggregate::Donor).
    pub struct DonorId;
}
finance_typed_id! {
    /// A typed id for an [`ExpenseHead`](crate::aggregate::ExpenseHead).
    pub struct ExpenseHeadId;
}
finance_typed_id! {
    /// A typed id for an [`IncomeHead`](crate::aggregate::IncomeHead).
    pub struct IncomeHeadId;
}
finance_typed_id! {
    /// A typed id for a [`Wallet`](crate::aggregate::Wallet). The
    /// user-balance projection per Phase 7.
    pub struct WalletId;
}
finance_typed_id! {
    /// A typed id for a [`WalletTransaction`](crate::aggregate::WalletTransaction).
    /// The `Refund` headline from the Phase 7 prompt is a
    /// `WalletTransaction` with `wallet_type = Refund`.
    pub struct WalletTransactionId;
}
finance_typed_id! {
    /// A typed id for a [`Transaction`](crate::aggregate::Transaction) — the
    /// double-entry journal line.
    pub struct TransactionId;
}
finance_typed_id! {
    /// A typed id for a [`PayrollPayment`](crate::aggregate::PayrollPayment).
    /// The finance-side accounting record, distinct from HR's
    /// `PayrollGenerate` aggregate.
    pub struct PayrollPaymentId;
}
finance_typed_id! {
    /// A typed id for a [`ProductPurchase`](crate::aggregate::ProductPurchase).
    pub struct ProductPurchaseId;
}
finance_typed_id! {
    /// A typed id for an [`InventoryPayment`](crate::aggregate::InventoryPayment).
    pub struct InventoryPaymentId;
}
finance_typed_id! {
    /// A typed id for an [`AmountTransfer`](crate::aggregate::AmountTransfer).
    pub struct AmountTransferId;
}
finance_typed_id! {
    /// A typed id for a [`ChartOfAccount`](crate::aggregate::ChartOfAccount).
    pub struct ChartOfAccountId;
}
finance_typed_id! {
    /// A typed id for a [`QuestionBankFee`](crate::aggregate::QuestionBankFee).
    pub struct QuestionBankFeeId;
}
finance_typed_id! {
    /// A typed id for a [`PaymentGatewaySetting`](crate::aggregate::PaymentGatewaySetting).
    pub struct PaymentGatewaySettingId;
}
finance_typed_id! {
    /// A typed id for a [`PaymentMethod`](crate::aggregate::PaymentMethod).
    pub struct PaymentMethodId;
}
finance_typed_id! {
    /// A typed id for a [`DueFeesLoginPrevent`](crate::aggregate::DueFeesLoginPrevent).
    pub struct DueFeesLoginPreventId;
}

// =============================================================================
// Typed ids: spec'd child-entity identities (per
// `docs/specs/finance/entities.md`). The corresponding child-entity
// structs live in `entities.rs` (or are slated for later workstreams);
// the ID types must exist now so that downstream aggregates can
// reference them in their event payloads and command shapes.
// =============================================================================

finance_typed_id! {
    /// A typed id for a `FeesInstallmentAssignDiscount` child entity.
    pub struct FeesInstallmentAssignDiscountId;
}
finance_typed_id! {
    /// A typed id for a `DirectFeesInstallmentAssignChild` child entity.
    pub struct DirectFeesInstallmentAssignChildId;
}
finance_typed_id! {
    /// A typed id for an `FmFeesInvoiceLineNote` child entity.
    pub struct FmFeesInvoiceLineNoteId;
}
finance_typed_id! {
    /// A typed id for an `FmFeesTransactionLineNote` child entity.
    pub struct FmFeesTransactionLineNoteId;
}
finance_typed_id! {
    /// A typed id for a `BankStatementAttachment` child entity.
    pub struct BankStatementAttachmentId;
}
finance_typed_id! {
    /// A typed id for a `PayrollPaymentApproval` child entity.
    pub struct PayrollPaymentApprovalId;
}
finance_typed_id! {
    /// A typed id for a `BankPaymentSlipAudit` child entity.
    pub struct BankPaymentSlipAuditId;
}
finance_typed_id! {
    /// A typed id for an `ExpenseApproval` child entity.
    pub struct ExpenseApprovalId;
}
finance_typed_id! {
    /// A typed id for an `IncomeApproval` child entity.
    pub struct IncomeApprovalId;
}
finance_typed_id! {
    /// A typed id for a `WalletTransactionApproval` child entity.
    pub struct WalletTransactionApprovalId;
}

// =============================================================================
// Money (MinorUnits) — the headline correctness primitive
// =============================================================================

/// An ISO-4217 currency code (e.g. `"USD"`, `"INR"`, `"EUR"`, `"GBP"`).
/// Validated to be 3 uppercase ASCII letters per ISO-4217.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Currency(
    /// The 3-letter ISO-4217 code, stored as raw bytes.
    pub [u8; 3],
);

impl Currency {
    /// The `INR` (Indian Rupee) currency — the engine's default.
    pub const INR: Currency = Currency(*b"INR");
    /// The `USD` (US Dollar) currency.
    pub const USD: Currency = Currency(*b"USD");
    /// The `EUR` (Euro) currency.
    pub const EUR: Currency = Currency(*b"EUR");
    /// The `GBP` (Pound Sterling) currency.
    pub const GBP: Currency = Currency(*b"GBP");

    /// Constructs a `Currency` from a 3-letter ISO-4217 code.
    /// Returns `Err` if the input is not 3 uppercase ASCII letters.
    pub fn new(code: &str) -> Result<Self> {
        let bytes = code.as_bytes();
        if bytes.len() != 3 {
            return Err(DomainError::validation(format!(
                "currency code must be exactly 3 letters, got {} chars",
                bytes.len()
            )));
        }
        for &b in bytes {
            if !b.is_ascii_uppercase() {
                return Err(DomainError::validation(format!(
                    "currency code must be uppercase ASCII, got byte {b}"
                )));
            }
        }
        Ok(Currency([bytes[0], bytes[1], bytes[2]]))
    }

    /// Returns the currency code as a `&str`.
    #[must_use]
    pub fn as_str(&self) -> &str {
        // The bytes are validated as 3 uppercase ASCII letters
        // in `new`, so this is always valid UTF-8. Using
        // `from_utf8` (returning `Result`) here is safe because
        // the bytes are guaranteed valid; the `expect` is
        // unavoidable without `unsafe` (which is forbidden by
        // `AGENTS.md`).
        std::str::from_utf8(&self.0).ok().unwrap_or("XXX")
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A monetary amount expressed in `MinorUnits` (i64 cents/paisa)
/// with an associated [`Currency`]. All amounts in the engine are
/// `Money`; raw floats are forbidden per `AGENTS.md` and the
/// Phase 7 build-plan § "Risks".
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Money {
    /// The amount in minor units (cents, paisa, etc.).
    pub amount_minor: i64,
    /// The currency.
    pub currency: Currency,
}

impl Money {
    /// The zero amount in the given currency.
    #[must_use]
    pub const fn zero(currency: Currency) -> Self {
        Self {
            amount_minor: 0,
            currency,
        }
    }

    /// Constructs a `Money` value, validating `amount_minor >= 0`.
    pub fn new(currency: Currency, amount_minor: i64) -> Result<Self> {
        if amount_minor < 0 {
            return Err(DomainError::validation(format!(
                "money amount must be non-negative, got {amount_minor}"
            )));
        }
        Ok(Self {
            amount_minor,
            currency,
        })
    }

    /// Returns `true` if this is the zero amount.
    #[must_use]
    pub const fn is_zero(&self) -> bool {
        self.amount_minor == 0
    }

    /// Returns `true` if the two `Money` values share the same currency.
    #[must_use]
    pub const fn same_currency(&self, other: &Self) -> bool {
        self.currency.0[0] == other.currency.0[0]
            && self.currency.0[1] == other.currency.0[1]
            && self.currency.0[2] == other.currency.0[2]
    }
}

/// An `Amount` is a non-negative `Money` constrained to the school's
/// default currency. Used for all fees, payments, and balances in
/// the finance domain.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Amount {
    /// The inner `Money`.
    pub money: Money,
}

impl Amount {
    /// Constructs an `Amount` in the default currency.
    pub fn new(currency: Currency, amount_minor: i64) -> Result<Self> {
        Ok(Self {
            money: Money::new(currency, amount_minor)?,
        })
    }

    /// Constructs the zero amount.
    #[must_use]
    pub const fn zero(currency: Currency) -> Self {
        Self {
            money: Money {
                amount_minor: 0,
                currency,
            },
        }
    }

    /// Returns the inner `Money`.
    #[must_use]
    pub const fn money(&self) -> &Money {
        &self.money
    }

    /// Returns the amount in minor units.
    #[must_use]
    pub const fn amount_minor(&self) -> i64 {
        self.money.amount_minor
    }

    /// Returns the currency.
    #[must_use]
    pub const fn currency(&self) -> Currency {
        self.money.currency
    }
}

/// A `FeeAmount` is constrained to `0..=1_000_000.00` (per
/// `docs/specs/finance/value-objects.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FeeAmount(Amount);

impl FeeAmount {
    /// Maximum minor units (1,000,000.00 = 100 million minor units).
    pub const MAX_MINOR: i64 = 100_000_000;

    /// Constructs a `FeeAmount`, validating the upper bound.
    pub fn new(currency: Currency, amount_minor: i64) -> Result<Self> {
        if amount_minor < 0 {
            return Err(DomainError::validation(format!(
                "fee amount must be non-negative, got {amount_minor}"
            )));
        }
        if amount_minor > Self::MAX_MINOR {
            return Err(DomainError::validation(format!(
                "fee amount must be at most {} minor units, got {amount_minor}",
                Self::MAX_MINOR
            )));
        }
        Ok(Self(Amount::new(currency, amount_minor)?))
    }

    /// Returns the inner `Amount`.
    #[must_use]
    pub const fn amount(&self) -> Amount {
        self.0
    }

    /// Returns the amount in minor units.
    #[must_use]
    pub const fn amount_minor(&self) -> i64 {
        self.0.amount_minor()
    }
}

/// A `FineAmount` is constrained to `0..=100_000.00` (per
/// `docs/specs/finance/value-objects.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FineAmount(Amount);

impl FineAmount {
    /// Maximum minor units (100,000.00 = 10 million minor units).
    pub const MAX_MINOR: i64 = 10_000_000;

    /// Constructs a `FineAmount`, validating the upper bound.
    pub fn new(currency: Currency, amount_minor: i64) -> Result<Self> {
        if amount_minor < 0 {
            return Err(DomainError::validation(format!(
                "fine amount must be non-negative, got {amount_minor}"
            )));
        }
        if amount_minor > Self::MAX_MINOR {
            return Err(DomainError::validation(format!(
                "fine amount must be at most {} minor units, got {amount_minor}",
                Self::MAX_MINOR
            )));
        }
        Ok(Self(Amount::new(currency, amount_minor)?))
    }
}

/// A `DiscountAmount` is constrained to `0..=1_000_000.00` (per
/// `docs/specs/finance/value-objects.md`).
pub type DiscountAmount = FeeAmount;

/// A `Balance` is an `Amount` that may be zero but not negative.
pub type Balance = Amount;

// =============================================================================
// Closed status enums (Copy + Eq + Hash + Serialize)
// =============================================================================

/// Invoice lifecycle status. `Issued` is terminal. `Cancelled` is
/// terminal. `Pending` is the initial state.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FeesInvoiceStatus {
    /// The invoice has been generated but not yet issued.
    #[default]
    Pending,
    /// The invoice has been issued (printed / sent).
    Issued,
    /// The invoice has been cancelled.
    Cancelled,
}

impl FeesInvoiceStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Issued => "issued",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "pending" => Ok(Self::Pending),
            "issued" => Ok(Self::Issued),
            "cancelled" => Ok(Self::Cancelled),
            other => Err(DomainError::validation(format!(
                "unknown fees invoice status: {other:?}"
            ))),
        }
    }

    /// Returns `true` if the state is terminal.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Issued | Self::Cancelled)
    }
}

/// Payment lifecycle status (per the `FeesPayment` aggregate).
/// `Paid` is terminal. `Overpaid` is terminal (the school owes the
/// payer a refund or wallet credit). `Unpaid` -> `Partial` -> `Paid`
/// is the happy path.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FeesPaymentStatus {
    /// No payment received.
    #[default]
    Unpaid,
    /// Partial payment received.
    Partial,
    /// Fully paid.
    Paid,
    /// Overpaid (cumulative payment exceeds the open balance).
    Overpaid,
}

impl FeesPaymentStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unpaid => "unpaid",
            Self::Partial => "partial",
            Self::Paid => "paid",
            Self::Overpaid => "overpaid",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "unpaid" => Ok(Self::Unpaid),
            "partial" => Ok(Self::Partial),
            "paid" => Ok(Self::Paid),
            "overpaid" => Ok(Self::Overpaid),
            other => Err(DomainError::validation(format!(
                "unknown payment status: {other:?}"
            ))),
        }
    }
}

/// Generic approval status used by slip approvals, wallet
/// transactions, and expense approvals.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ApprovalStatus {
    /// Awaiting review.
    #[default]
    Pending,
    /// Approved.
    Approved,
    /// Rejected.
    Rejected,
}

impl ApprovalStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "pending" => Ok(Self::Pending),
            "approved" => Ok(Self::Approved),
            "rejected" => Ok(Self::Rejected),
            other => Err(DomainError::validation(format!(
                "unknown approval status: {other:?}"
            ))),
        }
    }

    /// Returns `true` if the state is terminal.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Approved | Self::Rejected)
    }

    /// Returns `true` if a `from -> to` transition is allowed.
    /// `Pending -> Approved` and `Pending -> Rejected` are the
    /// happy paths. `Approved` and `Rejected` are terminal.
    #[must_use]
    pub const fn can_transition_to(self, to: Self) -> bool {
        match (self, to) {
            (Self::Pending, Self::Approved) | (Self::Pending, Self::Rejected) => true,
            _ => false,
        }
    }
}

/// Wallet-transaction status. The state machine is the same as
/// `ApprovalStatus` but with a distinct name per the spec.
pub type WalletTxStatus = ApprovalStatus;

/// Wallet transaction kind. The `Refund` variant is the headline
/// `Refund` aggregate per the Phase 7 prompt (modeled as a
/// `WalletTransaction` with `wallet_type = Refund`).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WalletTxType {
    /// A wallet credit (deposit / top-up).
    #[default]
    Deposit,
    /// A wallet refund (the Phase 7 headline `Refund`).
    Refund,
    /// A wallet debit for an expense.
    Expense,
    /// A wallet debit for a fees refund.
    FeesRefund,
}

impl WalletTxType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Deposit => "deposit",
            Self::Refund => "refund",
            Self::Expense => "expense",
            Self::FeesRefund => "fees_refund",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "deposit" => Ok(Self::Deposit),
            "refund" => Ok(Self::Refund),
            "expense" => Ok(Self::Expense),
            "fees_refund" => Ok(Self::FeesRefund),
            other => Err(DomainError::validation(format!(
                "unknown wallet tx type: {other:?}"
            ))),
        }
    }

    /// Returns `true` if the transaction type credits the wallet.
    #[must_use]
    pub const fn is_credit(self) -> bool {
        matches!(self, Self::Deposit | Self::Refund)
    }

    /// Returns `true` if the transaction type debits the wallet.
    #[must_use]
    pub const fn is_debit(self) -> bool {
        matches!(self, Self::Expense | Self::FeesRefund)
    }
}

/// Payment method kind (cash / bank / cheque / card / mobile / gateway).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PaymentMethodKind {
    /// Cash payment.
    #[default]
    Cash,
    /// Bank transfer.
    Bank,
    /// Cheque.
    Cheque,
    /// Card payment.
    Card,
    /// Mobile payment.
    Mobile,
    /// Gateway-backed online payment.
    Gateway,
}

impl PaymentMethodKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cash => "cash",
            Self::Bank => "bank",
            Self::Cheque => "cheque",
            Self::Card => "card",
            Self::Mobile => "mobile",
            Self::Gateway => "gateway",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "cash" => Ok(Self::Cash),
            "bank" => Ok(Self::Bank),
            "cheque" => Ok(Self::Cheque),
            "card" => Ok(Self::Card),
            "mobile" => Ok(Self::Mobile),
            "gateway" => Ok(Self::Gateway),
            other => Err(DomainError::validation(format!(
                "unknown payment method kind: {other:?}"
            ))),
        }
    }
}

/// Payment-gateway sandbox-vs-live mode.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GatewayMode {
    /// Sandbox mode (test gateway).
    #[default]
    Sandbox,
    /// Live mode (production gateway).
    Live,
}

impl GatewayMode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sandbox => "sandbox",
            Self::Live => "live",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "sandbox" => Ok(Self::Sandbox),
            "live" => Ok(Self::Live),
            other => Err(DomainError::validation(format!(
                "unknown gateway mode: {other:?}"
            ))),
        }
    }

    /// Returns `true` if the gateway is in production mode.
    #[must_use]
    pub const fn is_live(self) -> bool {
        matches!(self, Self::Live)
    }
}

/// Account type for a `BankAccount` (or cash drawer).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AccountType {
    /// A bank account.
    #[default]
    Bank,
    /// A cash drawer.
    Cash,
}

impl AccountType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bank => "bank",
            Self::Cash => "cash",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "bank" => Ok(Self::Bank),
            "cash" => Ok(Self::Cash),
            other => Err(DomainError::validation(format!(
                "unknown account type: {other:?}"
            ))),
        }
    }
}

/// Bank payment slip mode (bank transfer vs cheque).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BankMode {
    /// Bank transfer.
    #[default]
    Bk,
    /// Cheque.
    Cq,
}

impl BankMode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bk => "Bk",
            Self::Cq => "Cq",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "Bk" => Ok(Self::Bk),
            "Cq" => Ok(Self::Cq),
            other => Err(DomainError::validation(format!(
                "unknown bank mode: {other:?}"
            ))),
        }
    }
}

/// Bank statement type (income vs expense).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatementType {
    /// Money flowing in (e.g. fees payment received).
    #[default]
    Income,
    /// Money flowing out (e.g. expense paid).
    Expense,
}

impl StatementType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Income => "income",
            Self::Expense => "expense",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "income" => Ok(Self::Income),
            "expense" => Ok(Self::Expense),
            other => Err(DomainError::validation(format!(
                "unknown statement type: {other:?}"
            ))),
        }
    }
}

/// Discount type (once-per-master vs once-per-year).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiscountType {
    /// Apply once per fees master per student.
    #[default]
    Once,
    /// Apply once per student per year across all masters.
    Year,
}

impl DiscountType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Once => "once",
            Self::Year => "year",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "once" => Ok(Self::Once),
            "year" => Ok(Self::Year),
            other => Err(DomainError::validation(format!(
                "unknown discount type: {other:?}"
            ))),
        }
    }
}

/// FM invoice type (fees or LMS).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FmInvoiceType {
    /// A regular fees invoice.
    #[default]
    Fees,
    /// An LMS (Learning Management System) invoice.
    Lms,
}

impl FmInvoiceType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fees => "fees",
            Self::Lms => "lms",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "fees" => Ok(Self::Fees),
            "lms" => Ok(Self::Lms),
            other => Err(DomainError::validation(format!(
                "unknown fm invoice type: {other:?}"
            ))),
        }
    }
}

/// Carry-forward balance type. A debit balance means the student
/// owes money; a credit balance means the school owes the student
/// (overpayment).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BalanceType {
    /// The student owes the school.
    #[default]
    Debit,
    /// The school owes the student.
    Credit,
}

impl BalanceType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Debit => "debit",
            Self::Credit => "credit",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "debit" => Ok(Self::Debit),
            "credit" => Ok(Self::Credit),
            other => Err(DomainError::validation(format!(
                "unknown balance type: {other:?}"
            ))),
        }
    }
}

/// The reason a login is blocked (per the due-fees login
/// prevention spec, `docs/specs/finance/aggregates.md#duefeesloginprevent`).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PreventReason {
    /// The user has an overdue fees balance.
    #[default]
    OverdueFees,
}

impl PreventReason {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OverdueFees => "overdue_fees",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "overdue_fees" => Ok(Self::OverdueFees),
            other => Err(DomainError::validation(format!(
                "unknown prevent reason: {other:?}"
            ))),
        }
    }
}

// =============================================================================
// Validators
// =============================================================================

/// Validates that a discount name is 1..=200 chars.
pub fn validate_discount_name(name: &str) -> Result<()> {
    if name.is_empty() || name.chars().count() > 200 {
        return Err(DomainError::validation(format!(
            "discount name must be 1..=200 chars, got {}",
            name.chars().count()
        )));
    }
    Ok(())
}

/// Validates that a donor name is 1..=200 chars.
pub fn validate_donor_name(name: &str) -> Result<()> {
    if name.is_empty() || name.chars().count() > 200 {
        return Err(DomainError::validation(format!(
            "donor name must be 1..=200 chars, got {}",
            name.chars().count()
        )));
    }
    Ok(())
}

/// Validates that an expense / income name is 1..=200 chars.
pub fn validate_ledger_name(name: &str) -> Result<()> {
    if name.is_empty() || name.chars().count() > 200 {
        return Err(DomainError::validation(format!(
            "expense/income name must be 1..=200 chars, got {}",
            name.chars().count()
        )));
    }
    Ok(())
}

/// Validates that a bank account number is 6..=34 alphanumeric chars.
pub fn validate_bank_account_number(account_number: &str) -> Result<()> {
    if account_number.len() < 6 || account_number.len() > 34 {
        return Err(DomainError::validation(format!(
            "bank account number must be 6..=34 chars, got {}",
            account_number.len()
        )));
    }
    if !account_number.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(DomainError::validation(
            "bank account number must be alphanumeric",
        ));
    }
    Ok(())
}

/// Validates that an IFSC code matches the Indian Financial System
/// Code format: `[A-Z]{4}0[A-Z0-9]{6}`.
pub fn validate_ifsc_code(ifsc: &str) -> Result<()> {
    if ifsc.len() != 11 {
        return Err(DomainError::validation(format!(
            "IFSC code must be exactly 11 chars, got {}",
            ifsc.len()
        )));
    }
    let bytes = ifsc.as_bytes();
    for i in 0..4 {
        if !bytes[i].is_ascii_uppercase() {
            return Err(DomainError::validation(format!(
                "IFSC code positions 0..=3 must be uppercase ASCII, got byte {} at pos {i}",
                bytes[i]
            )));
        }
    }
    if bytes[4] != b'0' {
        return Err(DomainError::validation(format!(
            "IFSC code position 4 must be '0', got {:?}",
            bytes[4] as char
        )));
    }
    for i in 5..11 {
        if !bytes[i].is_ascii_uppercase() && !bytes[i].is_ascii_digit() {
            return Err(DomainError::validation(format!(
                "IFSC code positions 5..=10 must be uppercase ASCII or digit, got byte {} at pos {i}",
                bytes[i]
            )));
        }
    }
    Ok(())
}

/// Validates that a percentage is in `[0, 100]`.
pub fn validate_percentage(pct: f32) -> Result<()> {
    if !pct.is_finite() || !(0.0..=100.0).contains(&pct) {
        return Err(DomainError::validation(format!(
            "percentage must be in [0, 100], got {pct}"
        )));
    }
    Ok(())
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::*;

    #[test]
    fn money_rejects_negative() {
        let err = Money::new(Currency::INR, -1).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn money_zero_is_zero() {
        let m = Money::zero(Currency::INR);
        assert!(m.is_zero());
        assert_eq!(m.amount_minor, 0);
    }

    #[test]
    fn currency_rejects_lowercase() {
        let err = Currency::new("inr").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn currency_accepts_uppercase_iso4217() {
        assert!(Currency::new("USD").is_ok());
        assert!(Currency::new("INR").is_ok());
        assert!(Currency::new("EUR").is_ok());
        assert!(Currency::new("GBP").is_ok());
    }

    #[test]
    fn fee_amount_enforces_max() {
        let ok = FeeAmount::new(Currency::INR, FeeAmount::MAX_MINOR);
        assert!(ok.is_ok());
        let err = FeeAmount::new(Currency::INR, FeeAmount::MAX_MINOR + 1);
        assert!(err.is_err());
    }

    #[test]
    fn wallet_tx_type_round_trip() {
        for t in [
            WalletTxType::Deposit,
            WalletTxType::Refund,
            WalletTxType::Expense,
            WalletTxType::FeesRefund,
        ] {
            assert_eq!(WalletTxType::parse(t.as_str()).unwrap(), t);
            assert!(t.is_credit() ^ t.is_debit());
        }
    }

    #[test]
    fn approval_status_state_machine() {
        assert!(ApprovalStatus::Pending.can_transition_to(ApprovalStatus::Approved));
        assert!(ApprovalStatus::Pending.can_transition_to(ApprovalStatus::Rejected));
        assert!(!ApprovalStatus::Approved.can_transition_to(ApprovalStatus::Pending));
        assert!(ApprovalStatus::Approved.is_terminal());
        assert!(ApprovalStatus::Rejected.is_terminal());
    }

    #[test]
    fn ifsc_code_validates_format() {
        assert!(validate_ifsc_code("HDFC0001234").is_ok());
        assert!(validate_ifsc_code("SBIN0001234").is_ok());
        // wrong length
        assert!(validate_ifsc_code("HDFC000123").is_err());
        // lowercase
        assert!(validate_ifsc_code("hdfc0001234").is_err());
        // position 4 not 0
        assert!(validate_ifsc_code("HDFC1001234").is_err());
    }

    #[test]
    fn bank_account_number_validates_format() {
        assert!(validate_bank_account_number("123456").is_ok());
        assert!(validate_bank_account_number("1234567890123456789012345678901234").is_ok());
        assert!(validate_bank_account_number("12345").is_err()); // too short
        assert!(validate_bank_account_number("12345-67890").is_err()); // not alphanumeric
    }

    #[test]
    fn typed_id_display() {
        let school = SchoolId(Uuid::now_v7());
        let id = WalletId::new(school, Uuid::now_v7());
        let s = id.to_string();
        assert!(s.contains('/'));
        assert!(s.starts_with(&school.to_string()));
    }
}
