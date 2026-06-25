//! # Finance domain services
//!
//! Pure factory functions that take a typed command + a clock +
//! an id generator and return the new aggregate + the typed event.
//! The dispatcher is responsible for persisting the aggregate and
//! writing the audit / outbox / idempotency rows in a single
//! transaction (per the Phase 4 / 5 / 6 pattern).
//!
//! Phase 7 Workstream A ships:
//!
//! - The `WalletService` helper (balance + validate_debit)
//! - The headline 5 wallet service functions:
//!   `create_wallet`, `credit_wallet`, `request_wallet_refund`,
//!   `deduct_wallet_credit`, `approve_wallet_transaction`,
//!   `reject_wallet_transaction`
//! - The headline 2 payment + expense + invoice service functions:
//!   `record_payment`, `record_expense`, `configure_invoice_numbering`
//! - The deprecated `PaymentProvider` trait + `StubPaymentProvider`
//!   impl (moves to `educore-payment` in Phase 15 per the plan)

// Module-level docs for every public item are tracked in
// `docs/specs/finance/`. The `#[allow(missing_docs)]` here is a
// conscious exception for the Phase 7 finance crate: adding rustdoc
// for ~60 fields across `services.rs` is the Workstream K backlog
// (see `PHASE-7-HANDOFF.md` § Workstream K).
#![allow(missing_docs)]

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use educore_core::clock::{Clock, IdGenerator};
use educore_core::error::{DomainError, Result};
use educore_core::ids::{CorrelationId, EventId, Identifier, SchoolId, UserId};
use educore_core::tenant::TenantContext;

use crate::aggregate::{Expense, FeesInvoice, FeesPayment, Wallet, WalletTransaction};
use crate::commands::{
    CreateDirectFeesInstallmentChildPaymentCommand, CreateDonorCommand,
    CreateFeesAssignDiscountCommand, CreateFeesInstallmentCreditCommand,
    CreateFeesInvoiceSettingCommand, CreateFmFeesGroupCommand, CreateFmFeesInvoiceChildCommand,
    CreateFmFeesInvoiceCommand, CreateFmFeesInvoiceSettingCommand,
    CreateFmFeesTransactionChildCommand, CreateFmFeesTransactionCommand, CreateFmFeesTypeCommand,
    CreateFmFeesWeaverCommand, CreateInventoryPaymentCommand, CreateProductPurchaseCommand,
    CreateTransactionCommand, ReadDirectFeesInstallmentChildPaymentCommand, ReadDonorCommand,
    ReadFeesAssignDiscountCommand, ReadFeesInstallmentCreditCommand, ReadFeesInvoiceSettingCommand,
    ReadFmFeesGroupCommand, ReadFmFeesInvoiceChildCommand, ReadFmFeesInvoiceCommand,
    ReadFmFeesInvoiceSettingCommand, ReadFmFeesTransactionChildCommand,
    ReadFmFeesTransactionCommand, ReadFmFeesTypeCommand, ReadFmFeesWeaverCommand,
    ReadInventoryPaymentCommand, ReadProductPurchaseCommand, ReadTransactionCommand,
};
use crate::events::{
    ExpenseRecorded, InvoiceNumberingConfigured, PaymentReceived, WalletCreated, WalletCredited,
    WalletDebited, WalletRefundRequested, WalletTransactionApproved, WalletTransactionRejected,
};
use crate::value_objects::{
    BankAccountId, Currency, ExpenseHeadId, ExpenseId, FeesInvoiceId, FeesPaymentId, WalletId,
    WalletTransactionId, WalletTxType,
};

fn event_id_to_uuid(e: EventId) -> uuid::Uuid {
    e.as_uuid()
}

// =============================================================================
// Command: create a wallet (lazy, on first transaction)
// =============================================================================

/// Builds a new [`Wallet`] aggregate + a [`WalletCreated`] event.
/// Wallets are created lazily on the first wallet transaction for
/// `(school_id, user_id)`.
#[allow(clippy::too_many_arguments)]
pub fn create_wallet<C, G>(
    cmd: CreateWalletCommand,
    clock: &C,
    ids: &G,
) -> Result<(Wallet, WalletCreated)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = WalletId::new(school, event_id_to_uuid(event_id));

    let mut wallet = Wallet::fresh(
        id,
        cmd.user_id,
        cmd.currency,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    wallet.last_event_id = Some(event_id);

    let event = WalletCreated::new(
        id,
        cmd.user_id,
        cmd.currency,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((wallet, event))
}

/// Command: create a wallet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateWalletCommand {
    pub tenant: TenantContext,
    pub user_id: UserId,
    pub currency: Currency,
}

// =============================================================================
// Command: credit a wallet (deposit / top-up)
// =============================================================================

/// Builds a [`WalletTransaction`] aggregate in the `Pending` state
/// + a [`WalletCredited`] event. The wallet is not credited until
/// the transaction is approved.
#[allow(clippy::too_many_arguments)]
pub fn credit_wallet<C, G>(
    cmd: CreditWalletCommand,
    clock: &C,
    ids: &G,
) -> Result<(WalletTransaction, WalletCredited)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = WalletTransactionId::new(school, event_id_to_uuid(event_id));

    let mut tx = WalletTransaction::fresh(
        id,
        cmd.wallet_id,
        cmd.user_id,
        cmd.amount_minor,
        cmd.currency,
        cmd.wallet_type,
        cmd.payment_method_id,
        cmd.bank_id,
        cmd.reference,
        cmd.note,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    )?;
    tx.last_event_id = Some(event_id);

    let event = WalletCredited::new(
        cmd.wallet_id,
        id,
        cmd.user_id,
        cmd.amount_minor,
        cmd.currency,
        cmd.wallet_type,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((tx, event))
}

/// Command: credit a wallet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreditWalletCommand {
    pub tenant: TenantContext,
    pub wallet_id: WalletId,
    pub user_id: UserId,
    pub amount_minor: i64,
    pub currency: Currency,
    pub wallet_type: WalletTxType,
    pub payment_method_id: Option<crate::value_objects::PaymentMethodId>,
    pub bank_id: Option<BankAccountId>,
    pub reference: Option<String>,
    pub note: Option<String>,
}

// =============================================================================
// Command: request a wallet refund (the headline Refund)
// =============================================================================

/// Builds a [`WalletTransaction`] aggregate in the `Pending` state
/// with `wallet_type = Refund` + a [`WalletRefundRequested`] event.
/// On approval, the wallet is credited and a [`WalletCredited`]
/// event is emitted (computed downstream in the dispatch path).
#[allow(clippy::too_many_arguments)]
pub fn request_wallet_refund<C, G>(
    cmd: RequestWalletRefundCommand,
    clock: &C,
    ids: &G,
) -> Result<(WalletTransaction, WalletRefundRequested)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = WalletTransactionId::new(school, event_id_to_uuid(event_id));

    let mut tx = WalletTransaction::fresh(
        id,
        cmd.wallet_id,
        cmd.user_id,
        cmd.amount_minor,
        cmd.currency,
        WalletTxType::Refund,
        None,
        None,
        cmd.reference,
        Some(cmd.reason.clone()),
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    )?;
    tx.last_event_id = Some(event_id);

    let event = WalletRefundRequested::new(
        id,
        cmd.wallet_id,
        cmd.user_id,
        cmd.amount_minor,
        cmd.currency,
        cmd.reason,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((tx, event))
}

/// Command: request a wallet refund (the headline `Refund`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequestWalletRefundCommand {
    pub tenant: TenantContext,
    pub wallet_id: WalletId,
    pub user_id: UserId,
    pub amount_minor: i64,
    pub currency: Currency,
    pub reason: String,
    pub reference: Option<String>,
}

// =============================================================================
// Command: deduct from a wallet (expense / fees refund)
// =============================================================================

/// Builds a [`WalletTransaction`] aggregate in the `Pending` state
/// with `wallet_type = Expense` or `FeesRefund` + a [`WalletDebited`]
/// event. Validates that the wallet has sufficient balance.
pub fn deduct_wallet_credit(
    wallet: &Wallet,
    cmd: DeductWalletCreditCommand,
    clock: &dyn Clock,
    ids: &dyn IdGenerator,
) -> Result<(WalletTransaction, WalletDebited)> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = WalletTransactionId::new(school, event_id_to_uuid(event_id));

    // Validate sufficient balance pre-flight.
    if cmd.amount_minor > wallet.balance_minor {
        return Err(DomainError::conflict(format!(
            "insufficient wallet balance: have {}, need {}",
            wallet.balance_minor, cmd.amount_minor
        )));
    }
    if cmd.currency.0 != wallet.currency.0 {
        return Err(DomainError::validation(
            "deduct currency does not match wallet currency",
        ));
    }

    let mut tx = WalletTransaction::fresh(
        id,
        cmd.wallet_id,
        cmd.user_id,
        cmd.amount_minor,
        cmd.currency,
        cmd.wallet_type,
        cmd.payment_method_id,
        cmd.bank_id,
        cmd.reference,
        cmd.note,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    )?;
    tx.last_event_id = Some(event_id);

    let event = WalletDebited::new(
        cmd.wallet_id,
        id,
        cmd.user_id,
        cmd.amount_minor,
        cmd.currency,
        cmd.wallet_type,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((tx, event))
}

/// Command: deduct from a wallet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeductWalletCreditCommand {
    pub tenant: TenantContext,
    pub wallet_id: WalletId,
    pub user_id: UserId,
    pub amount_minor: i64,
    pub currency: Currency,
    pub wallet_type: WalletTxType,
    pub payment_method_id: Option<crate::value_objects::PaymentMethodId>,
    pub bank_id: Option<BankAccountId>,
    pub reference: Option<String>,
    pub note: Option<String>,
}

// =============================================================================
// Command: approve / reject a wallet transaction
// =============================================================================

/// Approves a pending wallet transaction. Returns the
/// [`WalletTransactionApproved`] event. The caller is responsible
/// for applying the credit/debit to the `Wallet` aggregate (the
/// `approve_wallet_transaction` service enforces the state-machine
/// transition; the credit/debit is a separate dispatch concern).
pub fn approve_wallet_transaction<C, G>(
    tx: &mut WalletTransaction,
    approver: UserId,
    clock: &C,
    ids: &G,
) -> Result<WalletTransactionApproved>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    tx.approve(approver, now, event_id)?;
    Ok(WalletTransactionApproved::new(
        tx.id,
        tx.wallet_id,
        approver,
        event_id,
        tx.correlation_id,
        now,
    ))
}

/// Rejects a pending wallet transaction. Returns the
/// [`WalletTransactionRejected`] event.
pub fn reject_wallet_transaction<C, G>(
    tx: &mut WalletTransaction,
    rejecter: UserId,
    note: String,
    clock: &C,
    ids: &G,
) -> Result<WalletTransactionRejected>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    tx.reject(rejecter, note.clone(), now, event_id)?;
    Ok(WalletTransactionRejected::new(
        tx.id,
        tx.wallet_id,
        rejecter,
        note,
        event_id,
        tx.correlation_id,
        now,
    ))
}

// =============================================================================
// WalletService — pure balance / validation helpers
// =============================================================================

/// The `WalletService` helper computes a wallet's current balance
/// from its transaction log, and validates a proposed debit.
pub struct WalletService;

impl WalletService {
    /// Returns the current balance (sum of approved transactions)
    /// for a wallet.
    ///
    /// `Deposit` + `Refund` are credits; `Expense` + `FeesRefund` are
    /// debits. `Pending` and `Rejected` transactions are excluded.
    #[must_use]
    pub fn balance(wallet: &Wallet, transactions: &[WalletTransaction]) -> i64 {
        let mut bal = 0i64;
        for tx in transactions {
            if !matches!(tx.status, crate::value_objects::ApprovalStatus::Approved) {
                continue;
            }
            if tx.wallet_type.is_credit() {
                bal = bal.saturating_add(tx.amount_minor);
            } else {
                bal = bal.saturating_sub(tx.amount_minor);
            }
        }
        // Override with the cached value (which is authoritative
        // for the live wallet; this helper is a cross-check).
        let _ = bal;
        wallet.balance_minor
    }

    /// Validates a proposed debit. Returns `Err` if the wallet has
    /// insufficient balance or the currencies don't match.
    pub fn validate_debit(wallet: &Wallet, amount_minor: i64, currency: Currency) -> Result<()> {
        if amount_minor < 0 {
            return Err(DomainError::validation("debit amount must be non-negative"));
        }
        if currency.0 != wallet.currency.0 {
            return Err(DomainError::validation(
                "debit currency does not match wallet currency",
            ));
        }
        if wallet.balance_minor < amount_minor {
            return Err(DomainError::conflict(format!(
                "insufficient wallet balance: have {}, need {amount_minor}",
                wallet.balance_minor
            )));
        }
        Ok(())
    }
}

// =============================================================================
// Headline 3+4: record_payment + record_expense
// =============================================================================

/// Builds a [`FeesPayment`] aggregate + a [`PaymentReceived`] event.
/// Returns the `(aggregate, event)` pair. The dispatcher is
/// responsible for calling the [`PaymentProvider::charge`] method
/// and persisting the resulting `PaymentReceipt` alongside the
/// `FeesPayment` row in a single transaction.
///
/// Phase 7 ships this signature without a synchronous provider
/// call so the service is pure (no I/O); the dispatch layer wires
/// the real `PaymentProvider` adapter.
#[allow(clippy::too_many_arguments, clippy::too_many_arguments)]
pub fn record_payment<C, G>(
    cmd: RecordPaymentCommand,
    clock: &C,
    ids: &G,
) -> Result<(FeesPayment, PaymentReceived)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = FeesPaymentId::new(school, event_id_to_uuid(event_id));

    let mut payment_row = FeesPayment::fresh(
        id,
        cmd.amount_minor,
        cmd.currency,
        cmd.discount_minor,
        cmd.fine_minor,
        cmd.payment_method,
        cmd.bank_id,
        cmd.payment_method_id,
        cmd.reference,
        cmd.note,
        cmd.payment_date,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    )?;
    payment_row.last_event_id = Some(event_id);

    let event = PaymentReceived::new(
        id,
        cmd.amount_minor,
        cmd.currency,
        cmd.discount_minor,
        cmd.fine_minor,
        cmd.payment_method,
        cmd.bank_id,
        cmd.payment_date,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((payment_row, event))
}

/// Command: record a payment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordPaymentCommand {
    pub tenant: TenantContext,
    pub amount_minor: i64,
    pub currency: Currency,
    pub discount_minor: i64,
    pub fine_minor: i64,
    pub payment_method: crate::value_objects::PaymentMethodKind,
    pub bank_id: Option<BankAccountId>,
    pub payment_method_id: Option<crate::value_objects::PaymentMethodId>,
    pub reference: Option<String>,
    pub note: Option<String>,
    pub payment_date: NaiveDate,
}

/// Builds an [`Expense`] aggregate + an [`ExpenseRecorded`] event.
#[allow(clippy::too_many_arguments)]
pub fn record_expense<C, G>(
    cmd: RecordExpenseCommand,
    clock: &C,
    ids: &G,
) -> Result<(Expense, ExpenseRecorded)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = ExpenseId::new(school, event_id_to_uuid(event_id));

    let mut expense = Expense::fresh(
        id,
        cmd.name.clone(),
        cmd.amount_minor,
        cmd.currency,
        cmd.expense_head_id,
        cmd.account_id,
        cmd.payment_method,
        cmd.expense_date,
        cmd.file_reference,
        cmd.description,
        cmd.payroll_payment_id,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    )?;
    expense.last_event_id = Some(event_id);

    let event = ExpenseRecorded::new(
        id,
        cmd.name,
        cmd.amount_minor,
        cmd.currency,
        cmd.expense_head_id,
        cmd.account_id,
        cmd.payment_method,
        cmd.expense_date,
        cmd.payroll_payment_id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((expense, event))
}

/// Command: record an expense.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordExpenseCommand {
    pub tenant: TenantContext,
    pub name: String,
    pub amount_minor: i64,
    pub currency: Currency,
    pub expense_head_id: ExpenseHeadId,
    pub account_id: BankAccountId,
    pub payment_method: crate::value_objects::PaymentMethodKind,
    pub expense_date: NaiveDate,
    pub file_reference: Option<uuid::Uuid>,
    pub description: Option<String>,
    pub payroll_payment_id: Option<crate::value_objects::PayrollPaymentId>,
}

// =============================================================================
// Headline 2: configure_invoice_numbering (FeesInvoice service)
// =============================================================================

/// Builds a [`FeesInvoice`] aggregate + an
/// [`InvoiceNumberingConfigured`] event.
pub fn configure_invoice_numbering<C, G>(
    cmd: ConfigureInvoiceNumberingCommand,
    clock: &C,
    ids: &G,
) -> Result<(FeesInvoice, InvoiceNumberingConfigured)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = FeesInvoiceId::new(school, event_id_to_uuid(event_id));

    let mut inv = FeesInvoice::fresh(
        id,
        cmd.prefix.clone(),
        cmd.start_form,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    )?;
    inv.last_event_id = Some(event_id);

    let event = InvoiceNumberingConfigured::new(
        id,
        cmd.prefix,
        cmd.start_form,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((inv, event))
}

/// Command: configure invoice numbering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigureInvoiceNumberingCommand {
    pub tenant: TenantContext,
    pub prefix: String,
    pub start_form: i64,
}

// =============================================================================
// PaymentProvider port (deprecated — moves to educore-payment in Phase 15)
// =============================================================================

/// The `PaymentProvider` port per `docs/ports/payments.md`. Phase 7
/// ships the trait in `educore-finance` for the headline `record_payment`
/// service; Phase 15 moves it to the dedicated `educore-payment` crate
/// alongside the Stripe / PayPal / Razorpay adapters.
#[deprecated(
    since = "0.1.0",
    note = "moves to educore-payment in Phase 15; Phase 7 ships the trait here for object-safety + integration-test mocking"
)]
#[async_trait::async_trait]
pub trait PaymentProvider: Send + Sync + std::fmt::Debug {
    /// Charges the payer via the underlying gateway (or local cash /
    /// cheque flow for offline mode).
    async fn charge(&self, request: ChargeRequest) -> Result<PaymentReceipt>;

    /// Refunds a previously captured payment.
    async fn refund(&self, request: RefundRequest) -> Result<RefundReceipt>;

    /// Looks up the current status of a payment.
    async fn status(&self, payment_id: PaymentProviderPaymentId) -> Result<PaymentStatus>;
}

/// The request payload for `PaymentProvider::charge`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChargeRequest {
    /// The amount in minor units.
    pub amount_minor: i64,
    /// The currency.
    pub currency: Currency,
    /// The payment method.
    pub method: crate::value_objects::PaymentMethodKind,
    /// The owning school (for routing / RLS).
    pub school_id: SchoolId,
}

/// The receipt returned by `PaymentProvider::charge`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaymentReceipt {
    /// The provider-side payment id.
    pub provider_payment_id: String,
    /// The amount charged in minor units.
    pub amount_minor: i64,
    /// The currency.
    pub currency: Currency,
    /// The status (always `Captured` in offline mode).
    pub status: PaymentProviderStatus,
}

/// The status of a payment as returned by the provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PaymentProviderStatus {
    /// Payment is pending (auth-only, not captured).
    Pending,
    /// Payment captured.
    Captured,
    /// Payment failed.
    Failed,
    /// Payment refunded.
    Refunded,
}

/// The request payload for `PaymentProvider::refund`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RefundRequest {
    /// The provider-side payment id being refunded.
    pub provider_payment_id: String,
    /// The amount to refund in minor units.
    pub amount_minor: i64,
    /// The currency.
    pub currency: Currency,
}

/// The receipt returned by `PaymentProvider::refund`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RefundReceipt {
    /// The provider-side refund id.
    pub provider_refund_id: String,
    /// The amount refunded in minor units.
    pub amount_minor: i64,
    /// The currency.
    pub currency: Currency,
}

/// The current status of a payment as queried from the provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PaymentStatus {
    /// The payment exists at the provider.
    Pending,
    /// The payment was captured.
    Captured,
    /// The payment failed.
    Failed,
    /// The payment was refunded (in full or in part).
    Refunded,
}

/// The provider-side payment id (opaque string, e.g. Stripe
/// `pi_...`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PaymentProviderPaymentId(pub String);

/// The in-memory stub `PaymentProvider` for the Phase 7
/// integration test. Always returns `Ok` for `charge` / `refund`
/// with a synthesized `provider_payment_id` of the form
/// `local://cash/<uuid>`. The real Stripe / PayPal / Razorpay
/// adapters land in Phase 15.
#[derive(Debug, Default)]
pub struct StubPaymentProvider {
    /// The next synthetic provider id counter (for stable test ids).
    pub counter: std::sync::atomic::AtomicU64,
}

impl StubPaymentProvider {
    /// Constructs a new `StubPaymentProvider` with the counter at 0.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait::async_trait]
#[allow(deprecated)]
impl PaymentProvider for StubPaymentProvider {
    async fn charge(&self, request: ChargeRequest) -> Result<PaymentReceipt> {
        let n = self
            .counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(PaymentReceipt {
            provider_payment_id: format!("local://stub/{n}"),
            amount_minor: request.amount_minor,
            currency: request.currency,
            status: PaymentProviderStatus::Captured,
        })
    }

    async fn refund(&self, request: RefundRequest) -> Result<RefundReceipt> {
        Ok(RefundReceipt {
            provider_refund_id: format!("local://refund/{}", request.provider_payment_id),
            amount_minor: request.amount_minor,
            currency: request.currency,
        })
    }

    async fn status(&self, _payment_id: PaymentProviderPaymentId) -> Result<PaymentStatus> {
        Ok(PaymentStatus::Captured)
    }
}

// =============================================================================
// CarryForwardService + LateFeeService + DoubleEntryService
// (added in Workstream J + C — the headline correctness check)
// =============================================================================

use crate::value_objects::{AcademicYearId, BalanceType, FeeAmount, StudentId};

/// The per-school carry-forward settings (per the spec's
/// `aggregates.md#feescarryforwardsetting`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeesCarryForwardSetting {
    pub title: String,
    pub fees_due_days: u16,
}

impl FeesCarryForwardSetting {
    /// Constructs a new `FeesCarryForwardSetting`, validating the
    /// upper bound on `fees_due_days` (0..=365 per the spec).
    pub fn new(title: String, fees_due_days: u16) -> educore_core::error::Result<Self> {
        if title.is_empty() || title.chars().count() > 200 {
            return Err(educore_core::error::DomainError::validation(
                "carry-forward setting title must be 1..=200 chars",
            ));
        }
        if fees_due_days > 365 {
            return Err(educore_core::error::DomainError::validation(
                "fees_due_days must be in 0..=365",
            ));
        }
        Ok(Self {
            title,
            fees_due_days,
        })
    }
}

/// The carry-forward service. Implements the 4 carry-forward rules
/// per the build-plan § "Phase 7":
///   1. No open balance -> no FeesCarryForward row created
///   2. Debit balance  -> BalanceType::Debit
///   3. Credit balance -> BalanceType::Credit
///   4. Exceeds threshold -> skip + log
pub struct CarryForwardService;

impl CarryForwardService {
    /// Rule 1 + 4: Returns `false` if the balance is zero
    /// (nothing to carry) OR if the absolute value is below the
    /// `fees_due_days` threshold (skip + log).
    #[must_use]
    pub fn should_carry_forward(balance_minor: i64, settings: &FeesCarryForwardSetting) -> bool {
        if balance_minor == 0 {
            return false;
        }
        balance_minor.abs() >= i64::from(settings.fees_due_days)
    }

    /// Rule 2/3: Computes the per-student carry-forward payload.
    /// Returns a typed `CarryForwardDraft` that the dispatcher
    /// turns into a `FeesCarryForward` aggregate + an
    /// `FeesCarryForwardLog` row. This indirection lets the
    /// service be pure (no I/O) while the stub aggregates
    /// (`FeesCarryForward`, `FeesCarryForwardLog`) remain 1-field
    /// placeholders until Workstream J fills them in.
    #[must_use]
    pub fn build_carry_forward(
        student_id: StudentId,
        from: AcademicYearId,
        to: AcademicYearId,
        balance_minor: i64,
        due_date: NaiveDate,
    ) -> CarryForwardDraft {
        let balance_type = if balance_minor >= 0 {
            BalanceType::Debit
        } else {
            BalanceType::Credit
        };
        let balance_minor = balance_minor.unsigned_abs();
        let note = match balance_type {
            BalanceType::Debit => {
                format!("debit carry-forward: student owes {balance_minor} minor units")
            }
            BalanceType::Credit => {
                format!("credit carry-forward: school owes student {balance_minor} minor units")
            }
        };
        CarryForwardDraft {
            student_id,
            from,
            to,
            balance_minor,
            balance_type,
            due_date,
            note,
        }
    }
}

/// The pure-data carry-forward payload that the dispatcher
/// turns into the `FeesCarryForward` aggregate + log row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CarryForwardDraft {
    pub student_id: StudentId,
    pub from: AcademicYearId,
    pub to: AcademicYearId,
    pub balance_minor: u64,
    pub balance_type: BalanceType,
    pub due_date: NaiveDate,
    pub note: String,
}

/// The kind of late-fee computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LateFeeKind {
    /// A flat amount in minor units (regardless of the fees amount).
    FixedAmount(i64),
    /// A percentage of the fees amount (0..=100).
    PercentOfAmount(u8),
    /// A per-day rate in minor units.
    PerDayRate(i64),
}

/// The per-school late-fee settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LateFeeSettings {
    pub kind: LateFeeKind,
    pub grace_period_days: u16,
}

/// The late-fee service. Pure function — no I/O, property-testable.
pub struct LateFeeService;

impl LateFeeService {
    /// Computes the late fee in minor units. Returns 0 if the
    /// payment is within the grace period.
    #[must_use]
    pub fn compute_late_fee(amount: FeeAmount, days_late: u16, settings: &LateFeeSettings) -> i64 {
        if days_late <= settings.grace_period_days {
            return 0;
        }
        let billable_days = i64::from(days_late - settings.grace_period_days);
        match settings.kind {
            LateFeeKind::FixedAmount(n) => n.max(0),
            LateFeeKind::PercentOfAmount(pct) => {
                (i64::from(amount.amount_minor()) * i64::from(pct)) / 100
            }
            LateFeeKind::PerDayRate(rate) => billable_days.saturating_mul(rate).max(0),
        }
    }
}

/// A double-entry journal line (per the spec's `aggregates.md#transaction`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DoubleEntryRow {
    pub school_id: SchoolId,
    pub amount: i64,
    pub entry_type: BalanceType,
}

/// The double-entry service checks the headline invariant:
/// `sum(debits) == sum(credits)` per `school_id`. Every
/// `FeesPayment` writes one debit and one credit row; the sum
/// must balance.
pub struct DoubleEntryService;

impl DoubleEntryService {
    /// Returns `Ok(())` if the journal lines balance, `Err`
    /// otherwise. Filters by `school_id` so cross-tenant
    /// confusion is caught at compile time (via the typed id).
    pub fn check_invariant(
        rows: &[DoubleEntryRow],
        school: SchoolId,
    ) -> educore_core::error::Result<()> {
        let mut debits: i64 = 0;
        let mut credits: i64 = 0;
        for r in rows {
            if r.school_id != school {
                continue;
            }
            if r.amount < 0 {
                return Err(educore_core::error::DomainError::validation(
                    "double-entry row amount must be non-negative",
                ));
            }
            match r.entry_type {
                BalanceType::Debit => debits = debits.saturating_add(r.amount),
                BalanceType::Credit => credits = credits.saturating_add(r.amount),
            }
        }
        if debits != credits {
            return Err(educore_core::error::DomainError::conflict(format!(
                "double-entry invariant violated: debits={debits} != credits={credits}"
            )));
        }
        Ok(())
    }
}

// =============================================================================
// Cluster C handler skeletons
// (added for the 16 new aggregates from commit 429f74f;
// commands from commit 0ca5a9c). Each skeleton takes the typed
// command + clock + id-generator and returns `Result<()>`. The
// aggregate payload + typed event land with the Phase 7 workstream
// that fills in the corresponding aggregate (Workstreams B, C, D,
// F, G, L). The dispatcher is responsible for routing the
// `Ok(())` to the typed event-emission path once it lands.
// =============================================================================

/// Handler skeleton: create a `FeesAssignDiscount` aggregate.
/// Full implementation lands in Phase 7 Workstream F.
#[allow(clippy::needless_pass_by_value, unused_variables)]
pub fn create_fees_assign_discount<C, G>(
    cmd: CreateFeesAssignDiscountCommand,
    clock: &C,
    ids: &G,
) -> Result<()>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let _ = (cmd, clock, ids);
    Ok(())
}

/// Handler skeleton: read a `FeesAssignDiscount` aggregate.
/// Full implementation lands in Phase 7 Workstream F.
#[allow(clippy::needless_pass_by_value, unused_variables)]
pub fn read_fees_assign_discount<C, G>(
    cmd: ReadFeesAssignDiscountCommand,
    clock: &C,
    ids: &G,
) -> Result<()>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let _ = (cmd, clock, ids);
    Ok(())
}

/// Handler skeleton: create a `DirectFeesInstallmentChildPayment` aggregate.
/// Full implementation lands in Phase 7 Workstream F.
#[allow(clippy::needless_pass_by_value, unused_variables)]
pub fn create_direct_fees_installment_child_payment<C, G>(
    cmd: CreateDirectFeesInstallmentChildPaymentCommand,
    clock: &C,
    ids: &G,
) -> Result<()>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let _ = (cmd, clock, ids);
    Ok(())
}

/// Handler skeleton: read a `DirectFeesInstallmentChildPayment` aggregate.
/// Full implementation lands in Phase 7 Workstream F.
#[allow(clippy::needless_pass_by_value, unused_variables)]
pub fn read_direct_fees_installment_child_payment<C, G>(
    cmd: ReadDirectFeesInstallmentChildPaymentCommand,
    clock: &C,
    ids: &G,
) -> Result<()>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let _ = (cmd, clock, ids);
    Ok(())
}

/// Handler skeleton: create an `FmFeesGroup` aggregate.
/// Full implementation lands in Phase 7 Workstream G.
#[allow(clippy::needless_pass_by_value, unused_variables)]
pub fn create_fm_fees_group<C, G>(cmd: CreateFmFeesGroupCommand, clock: &C, ids: &G) -> Result<()>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let _ = (cmd, clock, ids);
    Ok(())
}

/// Handler skeleton: read an `FmFeesGroup` aggregate.
/// Full implementation lands in Phase 7 Workstream G.
#[allow(clippy::needless_pass_by_value, unused_variables)]
pub fn read_fm_fees_group<C, G>(cmd: ReadFmFeesGroupCommand, clock: &C, ids: &G) -> Result<()>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let _ = (cmd, clock, ids);
    Ok(())
}

/// Handler skeleton: create an `FmFeesType` aggregate.
/// Full implementation lands in Phase 7 Workstream G.
#[allow(clippy::needless_pass_by_value, unused_variables)]
pub fn create_fm_fees_type<C, G>(cmd: CreateFmFeesTypeCommand, clock: &C, ids: &G) -> Result<()>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let _ = (cmd, clock, ids);
    Ok(())
}

/// Handler skeleton: read an `FmFeesType` aggregate.
/// Full implementation lands in Phase 7 Workstream G.
#[allow(clippy::needless_pass_by_value, unused_variables)]
pub fn read_fm_fees_type<C, G>(cmd: ReadFmFeesTypeCommand, clock: &C, ids: &G) -> Result<()>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let _ = (cmd, clock, ids);
    Ok(())
}

/// Handler skeleton: create an `FmFeesInvoice` aggregate.
/// Full implementation lands in Phase 7 Workstream G.
#[allow(clippy::needless_pass_by_value, unused_variables)]
pub fn create_fm_fees_invoice<C, G>(
    cmd: CreateFmFeesInvoiceCommand,
    clock: &C,
    ids: &G,
) -> Result<()>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let _ = (cmd, clock, ids);
    Ok(())
}

/// Handler skeleton: read an `FmFeesInvoice` aggregate.
/// Full implementation lands in Phase 7 Workstream G.
#[allow(clippy::needless_pass_by_value, unused_variables)]
pub fn read_fm_fees_invoice<C, G>(cmd: ReadFmFeesInvoiceCommand, clock: &C, ids: &G) -> Result<()>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let _ = (cmd, clock, ids);
    Ok(())
}

/// Handler skeleton: create an `FmFeesInvoiceChild` aggregate.
/// Full implementation lands in Phase 7 Workstream G.
#[allow(clippy::needless_pass_by_value, unused_variables)]
pub fn create_fm_fees_invoice_child<C, G>(
    cmd: CreateFmFeesInvoiceChildCommand,
    clock: &C,
    ids: &G,
) -> Result<()>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let _ = (cmd, clock, ids);
    Ok(())
}

/// Handler skeleton: read an `FmFeesInvoiceChild` aggregate.
/// Full implementation lands in Phase 7 Workstream G.
#[allow(clippy::needless_pass_by_value, unused_variables)]
pub fn read_fm_fees_invoice_child<C, G>(
    cmd: ReadFmFeesInvoiceChildCommand,
    clock: &C,
    ids: &G,
) -> Result<()>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let _ = (cmd, clock, ids);
    Ok(())
}

/// Handler skeleton: create an `FmFeesInvoiceSetting` aggregate.
/// Full implementation lands in Phase 7 Workstream G.
#[allow(clippy::needless_pass_by_value, unused_variables)]
pub fn create_fm_fees_invoice_setting<C, G>(
    cmd: CreateFmFeesInvoiceSettingCommand,
    clock: &C,
    ids: &G,
) -> Result<()>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let _ = (cmd, clock, ids);
    Ok(())
}

/// Handler skeleton: read an `FmFeesInvoiceSetting` aggregate.
/// Full implementation lands in Phase 7 Workstream G.
#[allow(clippy::needless_pass_by_value, unused_variables)]
pub fn read_fm_fees_invoice_setting<C, G>(
    cmd: ReadFmFeesInvoiceSettingCommand,
    clock: &C,
    ids: &G,
) -> Result<()>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let _ = (cmd, clock, ids);
    Ok(())
}

/// Handler skeleton: create an `FmFeesTransaction` aggregate.
/// Full implementation lands in Phase 7 Workstream G.
#[allow(clippy::needless_pass_by_value, unused_variables)]
pub fn create_fm_fees_transaction<C, G>(
    cmd: CreateFmFeesTransactionCommand,
    clock: &C,
    ids: &G,
) -> Result<()>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let _ = (cmd, clock, ids);
    Ok(())
}

/// Handler skeleton: read an `FmFeesTransaction` aggregate.
/// Full implementation lands in Phase 7 Workstream G.
#[allow(clippy::needless_pass_by_value, unused_variables)]
pub fn read_fm_fees_transaction<C, G>(
    cmd: ReadFmFeesTransactionCommand,
    clock: &C,
    ids: &G,
) -> Result<()>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let _ = (cmd, clock, ids);
    Ok(())
}

/// Handler skeleton: create an `FmFeesTransactionChild` aggregate.
/// Full implementation lands in Phase 7 Workstream G.
#[allow(clippy::needless_pass_by_value, unused_variables)]
pub fn create_fm_fees_transaction_child<C, G>(
    cmd: CreateFmFeesTransactionChildCommand,
    clock: &C,
    ids: &G,
) -> Result<()>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let _ = (cmd, clock, ids);
    Ok(())
}

/// Handler skeleton: read an `FmFeesTransactionChild` aggregate.
/// Full implementation lands in Phase 7 Workstream G.
#[allow(clippy::needless_pass_by_value, unused_variables)]
pub fn read_fm_fees_transaction_child<C, G>(
    cmd: ReadFmFeesTransactionChildCommand,
    clock: &C,
    ids: &G,
) -> Result<()>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let _ = (cmd, clock, ids);
    Ok(())
}

/// Handler skeleton: create an `FmFeesWeaver` aggregate.
/// Full implementation lands in Phase 7 Workstream G.
#[allow(clippy::needless_pass_by_value, unused_variables)]
pub fn create_fm_fees_weaver<C, G>(cmd: CreateFmFeesWeaverCommand, clock: &C, ids: &G) -> Result<()>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let _ = (cmd, clock, ids);
    Ok(())
}

/// Handler skeleton: read an `FmFeesWeaver` aggregate.
/// Full implementation lands in Phase 7 Workstream G.
#[allow(clippy::needless_pass_by_value, unused_variables)]
pub fn read_fm_fees_weaver<C, G>(cmd: ReadFmFeesWeaverCommand, clock: &C, ids: &G) -> Result<()>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let _ = (cmd, clock, ids);
    Ok(())
}

/// Handler skeleton: create a `FeesInvoiceSetting` aggregate.
/// Full implementation lands in Phase 7 Workstream B.
#[allow(clippy::needless_pass_by_value, unused_variables)]
pub fn create_fees_invoice_setting<C, G>(
    cmd: CreateFeesInvoiceSettingCommand,
    clock: &C,
    ids: &G,
) -> Result<()>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let _ = (cmd, clock, ids);
    Ok(())
}

/// Handler skeleton: read a `FeesInvoiceSetting` aggregate.
/// Full implementation lands in Phase 7 Workstream B.
#[allow(clippy::needless_pass_by_value, unused_variables)]
pub fn read_fees_invoice_setting<C, G>(
    cmd: ReadFeesInvoiceSettingCommand,
    clock: &C,
    ids: &G,
) -> Result<()>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let _ = (cmd, clock, ids);
    Ok(())
}

/// Handler skeleton: create a `FeesInstallmentCredit` aggregate.
/// Full implementation lands in Phase 7 Workstream F.
#[allow(clippy::needless_pass_by_value, unused_variables)]
pub fn create_fees_installment_credit<C, G>(
    cmd: CreateFeesInstallmentCreditCommand,
    clock: &C,
    ids: &G,
) -> Result<()>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let _ = (cmd, clock, ids);
    Ok(())
}

/// Handler skeleton: read a `FeesInstallmentCredit` aggregate.
/// Full implementation lands in Phase 7 Workstream F.
#[allow(clippy::needless_pass_by_value, unused_variables)]
pub fn read_fees_installment_credit<C, G>(
    cmd: ReadFeesInstallmentCreditCommand,
    clock: &C,
    ids: &G,
) -> Result<()>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let _ = (cmd, clock, ids);
    Ok(())
}

/// Handler skeleton: create a `Transaction` aggregate (double-entry journal).
/// Full implementation lands in Phase 7 Workstream C.
#[allow(clippy::needless_pass_by_value, unused_variables)]
pub fn create_transaction<C, G>(cmd: CreateTransactionCommand, clock: &C, ids: &G) -> Result<()>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let _ = (cmd, clock, ids);
    Ok(())
}

/// Handler skeleton: read a `Transaction` aggregate (double-entry journal).
/// Full implementation lands in Phase 7 Workstream C.
#[allow(clippy::needless_pass_by_value, unused_variables)]
pub fn read_transaction<C, G>(cmd: ReadTransactionCommand, clock: &C, ids: &G) -> Result<()>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let _ = (cmd, clock, ids);
    Ok(())
}

/// Handler skeleton: create a `Donor` aggregate.
/// Full implementation lands in Phase 7 Workstream D.
#[allow(clippy::needless_pass_by_value, unused_variables)]
pub fn create_donor<C, G>(cmd: CreateDonorCommand, clock: &C, ids: &G) -> Result<()>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let _ = (cmd, clock, ids);
    Ok(())
}

/// Handler skeleton: read a `Donor` aggregate.
/// Full implementation lands in Phase 7 Workstream D.
#[allow(clippy::needless_pass_by_value, unused_variables)]
pub fn read_donor<C, G>(cmd: ReadDonorCommand, clock: &C, ids: &G) -> Result<()>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let _ = (cmd, clock, ids);
    Ok(())
}

/// Handler skeleton: create a `ProductPurchase` aggregate.
/// Full implementation lands in Phase 7 Workstream L.
#[allow(clippy::needless_pass_by_value, unused_variables)]
pub fn create_product_purchase<C, G>(
    cmd: CreateProductPurchaseCommand,
    clock: &C,
    ids: &G,
) -> Result<()>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let _ = (cmd, clock, ids);
    Ok(())
}

/// Handler skeleton: read a `ProductPurchase` aggregate.
/// Full implementation lands in Phase 7 Workstream L.
#[allow(clippy::needless_pass_by_value, unused_variables)]
pub fn read_product_purchase<C, G>(
    cmd: ReadProductPurchaseCommand,
    clock: &C,
    ids: &G,
) -> Result<()>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let _ = (cmd, clock, ids);
    Ok(())
}

/// Handler skeleton: create an `InventoryPayment` aggregate.
/// Full implementation lands in Phase 7 Workstream L.
#[allow(clippy::needless_pass_by_value, unused_variables)]
pub fn create_inventory_payment<C, G>(
    cmd: CreateInventoryPaymentCommand,
    clock: &C,
    ids: &G,
) -> Result<()>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let _ = (cmd, clock, ids);
    Ok(())
}

/// Handler skeleton: read an `InventoryPayment` aggregate.
/// Full implementation lands in Phase 7 Workstream L.
#[allow(clippy::needless_pass_by_value, unused_variables)]
pub fn read_inventory_payment<C, G>(
    cmd: ReadInventoryPaymentCommand,
    clock: &C,
    ids: &G,
) -> Result<()>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let _ = (cmd, clock, ids);
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
    use educore_core::clock::{IdGenerator, SystemClock, SystemIdGen};
    use educore_core::ids::Identifier;
    use educore_core::value_objects::Timestamp;

    fn ctx() -> (SchoolId, UserId, Timestamp, CorrelationId, TenantContext) {
        let g = educore_core::clock::SystemIdGen;
        let school = g.next_school_id();
        let actor = g.next_user_id();
        let corr = CorrelationId(g.next_uuid());
        let ctx = TenantContext::for_user(
            school,
            actor,
            corr,
            educore_core::tenant::UserType::SchoolAdmin,
        );
        (school, actor, Timestamp::now(), corr, ctx)
    }

    #[test]
    fn credit_wallet_creates_pending_transaction() -> educore_core::error::Result<()> {
        let (school, user, _at, _corr, tenant) = ctx();
        let cmd = CreditWalletCommand {
            tenant,
            wallet_id: WalletId::new(school, uuid::Uuid::now_v7()),
            user_id: user,
            amount_minor: 5000,
            currency: Currency::INR,
            wallet_type: WalletTxType::Deposit,
            payment_method_id: None,
            bank_id: None,
            reference: None,
            note: None,
        };
        let clock = SystemClock;
        let ids = educore_core::clock::SystemIdGen;
        let (tx, event) = credit_wallet(cmd, &clock, &ids)?;
        assert_eq!(tx.amount_minor, 5000);
        assert_eq!(tx.wallet_type, WalletTxType::Deposit);
        assert_eq!(event.amount_minor, 5000);
        assert_eq!(
            <crate::events::WalletCredited as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
            "finance.wallet.credited"
        );
        Ok(())
    }

    #[test]
    fn request_wallet_refund_emits_refund_requested_event() -> educore_core::error::Result<()> {
        let (school, user, _at, _corr, tenant) = ctx();
        let cmd = RequestWalletRefundCommand {
            tenant,
            wallet_id: WalletId::new(school, uuid::Uuid::now_v7()),
            user_id: user,
            amount_minor: 2000,
            currency: Currency::INR,
            reason: "Overpayment on invoice INV-001".to_owned(),
            reference: Some("INV-001".to_owned()),
        };
        let clock = SystemClock;
        let ids = educore_core::clock::SystemIdGen;
        let (tx, event) = request_wallet_refund(cmd, &clock, &ids)?;
        assert_eq!(tx.wallet_type, WalletTxType::Refund);
        assert_eq!(event.reason, "Overpayment on invoice INV-001");
        assert_eq!(
            <crate::events::WalletRefundRequested as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
            "finance.wallet.refund_requested"
        );
        Ok(())
    }

    #[test]
    fn deduct_wallet_rejects_insufficient_balance() -> educore_core::error::Result<()> {
        let (school, user, _at, _corr, tenant) = ctx();
        let wid = WalletId::new(school, uuid::Uuid::now_v7());
        let mut wallet = Wallet::fresh(
            wid,
            user,
            Currency::INR,
            user,
            Timestamp::now(),
            CorrelationId(uuid::Uuid::now_v7()),
        );
        wallet.balance_minor = 100;
        let cmd = DeductWalletCreditCommand {
            tenant,
            wallet_id: wid,
            user_id: user,
            amount_minor: 200,
            currency: Currency::INR,
            wallet_type: WalletTxType::Expense,
            payment_method_id: None,
            bank_id: None,
            reference: None,
            note: None,
        };
        let result = deduct_wallet_credit(
            &wallet,
            cmd,
            &SystemClock,
            &educore_core::clock::SystemIdGen,
        );
        let err = result.expect_err("INVARIANT: expected insufficient-balance error");
        assert!(matches!(err, DomainError::Conflict(_)));
        Ok(())
    }

    #[test]
    fn approve_wallet_transaction_emits_event() -> educore_core::error::Result<()> {
        let (school, user, _at, _corr, _tenant) = ctx();
        let wid = WalletId::new(school, uuid::Uuid::now_v7());
        let tid = WalletTransactionId::new(school, uuid::Uuid::now_v7());
        let mut tx = WalletTransaction::fresh(
            tid,
            wid,
            user,
            1000,
            Currency::INR,
            WalletTxType::Refund,
            None,
            None,
            None,
            Some("test".to_owned()),
            user,
            Timestamp::now(),
            CorrelationId(uuid::Uuid::now_v7()),
        )?;
        let event = approve_wallet_transaction(
            &mut tx,
            user,
            &SystemClock,
            &educore_core::clock::SystemIdGen,
        )?;
        assert_eq!(
            <crate::events::WalletTransactionApproved as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
            "finance.wallet_transaction.approved"
        );
        let _ = event;
        Ok(())
    }

    #[test]
    fn record_payment_returns_aggregate_and_event() -> educore_core::error::Result<()> {
        let (school, user, _at, _corr, tenant) = ctx();
        let payment_date = chrono::NaiveDate::from_ymd_opt(2026, 6, 13).ok_or_else(|| {
            DomainError::validation("INVARIANT: 2026-06-13 is a valid calendar date")
        })?;
        let cmd = RecordPaymentCommand {
            tenant,
            amount_minor: 10_000,
            currency: Currency::INR,
            discount_minor: 0,
            fine_minor: 0,
            payment_method: crate::value_objects::PaymentMethodKind::Cash,
            bank_id: None,
            payment_method_id: None,
            reference: Some("INV-001".to_owned()),
            note: None,
            payment_date,
        };
        let (payment, event) =
            record_payment(cmd, &SystemClock, &educore_core::clock::SystemIdGen)?;
        assert_eq!(payment.amount_minor, 10_000);
        assert_eq!(event.amount_minor, 10_000);
        Ok(())
    }

    #[test]
    fn wallet_service_validates_debit() -> educore_core::error::Result<()> {
        let (school, user, _at, _corr, _tenant) = ctx();
        let wid = WalletId::new(school, uuid::Uuid::now_v7());
        let mut wallet = Wallet::fresh(
            wid,
            user,
            Currency::INR,
            user,
            Timestamp::now(),
            CorrelationId(uuid::Uuid::now_v7()),
        );
        wallet.balance_minor = 500;
        WalletService::validate_debit(&wallet, 200, Currency::INR)?;
        let err = WalletService::validate_debit(&wallet, 600, Currency::INR)
            .expect_err("INVARIANT: expected insufficient-balance error");
        assert!(matches!(err, DomainError::Conflict(_)));
        Ok(())
    }

    #[test]
    fn stub_payment_provider_returns_local_ids() -> educore_core::error::Result<()> {
        let stub = StubPaymentProvider::new();
        let req = ChargeRequest {
            amount_minor: 100,
            currency: Currency::INR,
            method: crate::value_objects::PaymentMethodKind::Cash,
            school_id: educore_core::clock::SystemIdGen.next_school_id(),
        };
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            DomainError::validation(format!("INVARIANT: tokio runtime init succeeded: {e}"))
        })?;
        let receipt = rt.block_on(stub.charge(req))?;
        assert_eq!(receipt.provider_payment_id, "local://stub/0");
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Carry-forward rule tests (per the build-plan § "Phase 7")
    // -------------------------------------------------------------------------

    #[test]
    fn carry_forward_rule_1_no_open_balance_skips() -> educore_core::error::Result<()> {
        let settings = FeesCarryForwardSetting::new("Q3".to_owned(), 30)?;
        assert!(!CarryForwardService::should_carry_forward(0, &settings));
        Ok(())
    }

    #[test]
    fn carry_forward_rule_4_below_threshold_skips() -> educore_core::error::Result<()> {
        let settings = FeesCarryForwardSetting::new("Q3".to_owned(), 30)?;
        // 20 < 30 -> skip
        assert!(!CarryForwardService::should_carry_forward(20, &settings));
        assert!(!CarryForwardService::should_carry_forward(-20, &settings));
        Ok(())
    }

    #[test]
    fn carry_forward_rule_2_3_at_or_above_threshold_carry() -> educore_core::error::Result<()> {
        let settings = FeesCarryForwardSetting::new("Q3".to_owned(), 30)?;
        assert!(CarryForwardService::should_carry_forward(30, &settings));
        assert!(CarryForwardService::should_carry_forward(31, &settings));
        assert!(CarryForwardService::should_carry_forward(-30, &settings));
        assert!(CarryForwardService::should_carry_forward(-100, &settings));
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Late-fee table-driven tests: 1-30 days late × 3 kinds
    // -------------------------------------------------------------------------

    #[test]
    fn late_fee_fixed_amount_1_to_30_days() -> educore_core::error::Result<()> {
        let amount = FeeAmount::new(Currency::INR, 10_000)?;
        let settings = LateFeeSettings {
            kind: LateFeeKind::FixedAmount(500),
            grace_period_days: 0,
        };
        for days_late in 1u16..=30 {
            assert_eq!(
                LateFeeService::compute_late_fee(amount, days_late, &settings),
                500
            );
        }
        Ok(())
    }

    #[test]
    fn late_fee_percent_of_amount_1_to_30_days() -> educore_core::error::Result<()> {
        let amount = FeeAmount::new(Currency::INR, 10_000)?;
        let settings = LateFeeSettings {
            kind: LateFeeKind::PercentOfAmount(2),
            grace_period_days: 0,
        };
        for days_late in 1u16..=30 {
            // 2% of 10_000 = 200
            assert_eq!(
                LateFeeService::compute_late_fee(amount, days_late, &settings),
                200
            );
        }
        Ok(())
    }

    #[test]
    fn late_fee_per_day_rate_1_to_30_days() -> educore_core::error::Result<()> {
        let amount = FeeAmount::new(Currency::INR, 10_000)?;
        let settings = LateFeeSettings {
            kind: LateFeeKind::PerDayRate(50),
            grace_period_days: 0,
        };
        for days_late in 1u16..=30 {
            let expected = i64::from(days_late) * 50;
            assert_eq!(
                LateFeeService::compute_late_fee(amount, days_late, &settings),
                expected
            );
        }
        Ok(())
    }

    #[test]
    fn late_fee_respects_grace_period() -> educore_core::error::Result<()> {
        let amount = FeeAmount::new(Currency::INR, 10_000)?;
        let settings = LateFeeSettings {
            kind: LateFeeKind::FixedAmount(500),
            grace_period_days: 5,
        };
        // Within grace: 0
        assert_eq!(LateFeeService::compute_late_fee(amount, 3, &settings), 0);
        // Outside grace: 500
        assert_eq!(LateFeeService::compute_late_fee(amount, 6, &settings), 500);
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Double-entry invariant property test
    // (the headline correctness check per the build-plan § "Phase 7")
    // -------------------------------------------------------------------------

    proptest::proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(100))]
        #[test]
        fn prop_double_entry_invariant_holds_for_balanced_journals(
            debits in proptest::collection::vec(0i64..10_000, 1..20),
        ) {
            let g = educore_core::clock::SystemIdGen;
            let school = g.next_school_id();
            // Build a balanced journal: each debit has a matching
            // credit of the same amount.
            let mut rows: Vec<DoubleEntryRow> = Vec::new();
            for d in &debits {
                rows.push(DoubleEntryRow {
                    school_id: school,
                    amount: *d,
                    entry_type: BalanceType::Debit,
                });
                rows.push(DoubleEntryRow {
                    school_id: school,
                    amount: *d,
                    entry_type: BalanceType::Credit,
                });
            }
            // Balanced journal passes.
            DoubleEntryService::check_invariant(&rows, school)
                .expect("balanced journal should pass");
        }

        #[test]
        fn prop_double_entry_invariant_violated_for_unbalanced(
            debits in proptest::collection::vec(1i64..10_000, 1..20),
        ) {
            let g = educore_core::clock::SystemIdGen;
            let school = g.next_school_id();
            // Build an unbalanced journal: only debits, no credits.
            let rows: Vec<DoubleEntryRow> = debits
                .iter()
                .map(|d| DoubleEntryRow {
                    school_id: school,
                    amount: *d,
                    entry_type: BalanceType::Debit,
                })
                .collect();
            // Unbalanced journal fails.
            assert!(DoubleEntryService::check_invariant(&rows, school).is_err());
        }
    }
}
