//! # Service helpers
//!
//! Pure helper structs that sit alongside the port and its
//! adapters. None of them perform I/O; they exist so the adapter
//! crates (Stripe, offline cash-book, …) can keep their hot paths
//! tight and so the engine's idempotency, webhook, bank-slip, and
//! settlement logic can be unit-tested without spinning up a
//! database or a network.
//!
//! The four services are:
//!
//! - [`IdempotencyService`] — derives a deterministic charge key
//!   from `(command_id, invoice_ids, amount_minor)` and tracks
//!   seen keys to detect replays.
//! - [`WebhookSignatureService`] — computes and verifies
//!   `sha256=<hex>` HMAC-SHA256 signatures using a per-adapter
//!   webhook secret.
//! - [`BankSlipService`] — validates slip numbers and slip amounts
//!   against the invoice, and mints slip ids.
//! - [`SettlementService`] — matches settlement lines back to
//!   payment receipts by `provider_payment_id`, and computes
//!   per-batch net totals.
//!
//! # Deviations from the spec
//!
//! - The webhook secret is held as a `String`, not a
//!   `secrecy::SecretString`. The `secrecy` crate is not in
//!   `educore-payment`'s dependency set (the port explicitly opts
//!   out in `port.rs` § "Deviations from `docs/ports/payments.md`").
//!   The `Debug` impl on [`WebhookSignatureService`] redacts the
//!   secret, so the helper is still safe to print.
//! - Slip ids are minted from a process-local `AtomicU64` counter
//!   (`SLIP-00000001`, `SLIP-00000002`, …). The spec's suggested
//!   `uuid::Uuid::new_v4()` would require a new dependency;
//!   the counter approach is unique within a process and is what
//!   the task description explicitly authorises.

use std::collections::HashSet;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

use crate::errors::PaymentError;
use crate::port::{PaymentId, PaymentReceipt, SettlementLine};

/// The HMAC-SHA256 typed alias used throughout this module.
type HmacSha256 = Hmac<Sha256>;

// ---------------------------------------------------------------------------
// 1. IdempotencyService
// ---------------------------------------------------------------------------

/// Derives a deterministic charge key and tracks seen keys so the
/// adapter can detect replays of the same charge without issuing
/// a second authorisation.
///
/// The service holds no state of its own; the seen-key set is
/// passed in by the caller (typically a per-tenant
/// `HashMap<SchoolId, HashSet<String>>` held by the adapter).
#[derive(Debug, Default, Clone, Copy)]
pub struct IdempotencyService;

impl IdempotencyService {
    /// Constructs a new `IdempotencyService`. The struct is a
    /// zero-sized marker; the constructor is provided so the
    /// adapter can write `IdempotencyService::new()` and read
    /// identically to the other services in this module.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Derives a deterministic SHA-256 hex charge key from the
    /// command id, the sorted list of invoice ids, and the charge
    /// amount in minor units.
    ///
    /// The canonical input form is:
    ///
    /// ```text
    /// <command_id>\x1f<inv_1>\x1f<inv_2>\x1f…\x1f<inv_n>\x1f<amount_minor as 8 little-endian bytes>
    /// ```
    ///
    /// Invoice ids are sorted lexicographically before hashing so
    /// that a charge submitted as `["inv_a", "inv_b"]` and the
    /// same charge submitted as `["inv_b", "inv_a"]` collapse to
    /// the same key.
    #[must_use]
    pub fn derive_charge_key(
        command_id: &str,
        invoice_ids: &[String],
        amount_minor: i64,
    ) -> String {
        let mut sorted: Vec<&str> = invoice_ids.iter().map(String::as_str).collect();
        sorted.sort_unstable();

        let mut hasher = Sha256::new();
        hasher.update(command_id.as_bytes());
        hasher.update([0x1f_u8]);
        for inv in sorted {
            hasher.update(inv.as_bytes());
            hasher.update([0x1f_u8]);
        }
        hasher.update(amount_minor.to_le_bytes());
        hex_encode(&hasher.finalize())
    }

    /// Returns `true` if the key has been seen before (a replay);
    /// otherwise inserts the key and returns `false`.
    pub fn is_replay(key: &str, seen_keys: &mut HashSet<String>) -> bool {
        if seen_keys.contains(key) {
            true
        } else {
            seen_keys.insert(key.to_owned());
            false
        }
    }
}

// ---------------------------------------------------------------------------
// 2. WebhookSignatureService
// ---------------------------------------------------------------------------

/// Computes and verifies `sha256=<hex>` HMAC-SHA256 webhook
/// signatures against a per-adapter secret.
///
/// The wire format is identical to the
/// `X-Educore-Signature` header documented in
/// `docs/ports/integrations.md`: the literal prefix `sha256=`
/// followed by lower-case hex. The `Debug` impl redacts the
/// secret so the helper is safe to print in logs.
pub struct WebhookSignatureService {
    secret: String,
}

impl WebhookSignatureService {
    /// Constructs a new `WebhookSignatureService` from a raw
    /// secret string. The string is moved into the service and
    /// redacted on `Debug`.
    #[must_use]
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
        }
    }

    /// Returns the `sha256=<hex>` HMAC-SHA256 signature of the
    /// payload using the configured secret.
    ///
    /// The error variant is theoretically unreachable: HMAC-SHA256
    /// accepts keys of any length and `hmac::Hmac::new_from_slice`
    /// only fails on fixed-key-length variants. The function still
    /// returns `Result` so the engine's "no `expect`/`unwrap` in
    /// production paths" rule is preserved — see
    /// `docs/code-standards.md` § "Type Safety". The deviation from
    /// the task spec (`-> String` instead of `-> Result<_, _>`) is
    /// logged in the commit message.
    pub fn compute_signature(&self, payload: &[u8]) -> Result<String, PaymentError> {
        let mut mac = HmacSha256::new_from_slice(self.secret.as_bytes()).map_err(|e| {
            PaymentError::Provider(format!("hmac key rejected (unreachable for HMAC-SHA256): {e}"))
        })?;
        mac.update(payload);
        Ok(format!("sha256={}", hex_encode(&mac.finalize().into_bytes())))
    }

    /// Returns `Ok(true)` iff the provided `sha256=<hex>` header
    /// value matches the recomputed signature for the payload.
    /// Returns `Ok(false)` on a signature mismatch.
    ///
    /// Returns `Err(PaymentError::Provider)` only on the
    /// theoretically-unreachable `hmac::InvalidLength` error from
    /// `compute_signature` — see that method's docstring.
    ///
    /// The comparison is constant-time over the length of the
    /// expected signature, so a timing-side-channel attacker
    /// cannot recover the secret byte-by-byte.
    pub fn verify_signature(&self, payload: &[u8], provided: &str) -> Result<bool, PaymentError> {
        let expected = self.compute_signature(payload)?;
        Ok(constant_time_eq_str(&expected, provided))
    }

    /// Parses an `X-Educore-Signature`-style header value and
    /// returns the hex portion. Accepts any case for the `sha256=`
    /// prefix; returns `None` if the prefix is absent.
    #[must_use]
    pub fn extract_signature_header(header_value: &str) -> Option<&str> {
        let lower = header_value.to_ascii_lowercase();
        if lower.starts_with("sha256=") {
            Some(&header_value["sha256=".len()..])
        } else {
            None
        }
    }
}

impl fmt::Debug for WebhookSignatureService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebhookSignatureService")
            .field("secret", &"<redacted>")
            .finish()
    }
}

// ---------------------------------------------------------------------------
// 3. BankSlipService
// ---------------------------------------------------------------------------

/// Process-local counter backing [`BankSlipService::generate_slip_id`].
static SLIP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Validates Brazilian-style "boleto" bank-slip inputs and mints
/// unique slip ids for the offline cash-book adapter.
#[derive(Debug, Default, Clone, Copy)]
pub struct BankSlipService;

impl BankSlipService {
    /// Constructs a new `BankSlipService`. The struct is a
    /// zero-sized marker; the constructor is provided for API
    /// symmetry with the other services in this module.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Validates a slip number: must be 6-20 ASCII alphanumeric
    /// characters. Returns `Ok(())` on success or
    /// [`PaymentError::InvalidAmount`] (the port's catch-all
    /// validation variant) on a malformed input.
    pub fn validate_slip_number(number: &str) -> Result<(), PaymentError> {
        if number.len() < 6 || number.len() > 20 {
            return Err(PaymentError::InvalidAmount(format!(
                "slip number must be 6-20 chars, got {} chars",
                number.len()
            )));
        }
        if !number.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(PaymentError::InvalidAmount(format!(
                "slip number must be ASCII alphanumeric, got {number:?}"
            )));
        }
        Ok(())
    }

    /// Validates that the slip amount matches the invoice amount
    /// within ±1 minor unit (for FX rounding tolerance).
    ///
    /// Uses `i64::abs_diff` to avoid the `i64::MIN.abs()` overflow
    /// trap while staying within the engine's "no `as` on numerics"
    /// rule.
    pub fn validate_slip_amount(
        amount_minor: i64,
        invoice_amount_minor: i64,
    ) -> Result<(), PaymentError> {
        let diff = amount_minor.abs_diff(invoice_amount_minor);
        if diff > 1 {
            return Err(PaymentError::InvalidAmount(format!(
                "slip amount {amount_minor} does not match invoice amount \
                 {invoice_amount_minor} (diff {diff} > 1)"
            )));
        }
        Ok(())
    }

    /// Returns a unique slip id of the form `SLIP-00000001`,
    /// `SLIP-00000002`, … backed by a process-local
    /// [`AtomicU64`]. The id is unique within a process; it is
    /// not a global UUID.
    #[must_use]
    pub fn generate_slip_id() -> String {
        let n = SLIP_COUNTER.fetch_add(1, Ordering::SeqCst);
        format!("SLIP-{n:08}")
    }
}

// ---------------------------------------------------------------------------
// 4. SettlementService
// ---------------------------------------------------------------------------

/// Matches settlement lines back to payment receipts and
/// computes per-batch net totals.
///
/// The service holds no state; every method is a pure function
/// over the inputs.
#[derive(Debug, Default, Clone, Copy)]
pub struct SettlementService;

impl SettlementService {
    /// Constructs a new `SettlementService`. The struct is a
    /// zero-sized marker; the constructor is provided for API
    /// symmetry with the other services in this module.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Returns the engine payment id of the receipt whose
    /// `provider_payment_id` matches the settlement line's
    /// `provider_payment_id`. Returns `None` when no receipt
    /// matches (e.g. the line is for a payment issued before the
    /// engine started tracking `provider_payment_id`).
    #[must_use]
    pub fn match_settlement_line(
        line: &SettlementLine,
        receipts: &[PaymentReceipt],
    ) -> Option<PaymentId> {
        receipts
            .iter()
            .find(|r| r.provider_payment_id.as_deref() == Some(line.provider_payment_id.as_str()))
            .map(|r| r.payment_id.clone())
    }

    /// Returns the sum of `line.net.amount_minor` across every
    /// line in the batch, in minor units.
    ///
    /// This is a plain `sum` rather than `saturating_add`; a
    /// settlement batch that genuinely overflows `i64` is a
    /// programming error and the wrap is a loud signal.
    #[must_use]
    pub fn compute_net_settlement(lines: &[SettlementLine]) -> i64 {
        lines.iter().map(|l| l.net.amount_minor).sum()
    }

    /// Returns `true` if a settlement line in `lines` matches the
    /// receipt's `provider_payment_id`. A receipt with no
    /// `provider_payment_id` (an offline cash payment, for
    /// example) is never considered settled by a settlement line
    /// — offline settlements are tracked via the cash-book
    /// adapter, not the settlement report.
    #[must_use]
    pub fn is_settled(receipt: &PaymentReceipt, lines: &[SettlementLine]) -> bool {
        match receipt.provider_payment_id.as_deref() {
            Some(provider_id) => lines.iter().any(|l| l.provider_payment_id == provider_id),
            None => false,
        }
    }
}

// ---------------------------------------------------------------------------
// Small helpers
// ---------------------------------------------------------------------------

/// Lower-case hex encoder. Stays in this module rather than
/// pulling in a `hex` dependency that the rest of the crate does
/// not need.
fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

/// Constant-time string equality. Iterates over the full length
/// of the expected string regardless of where the first mismatch
/// is found, so the comparison does not leak the position of the
/// first differing byte.
fn constant_time_eq_str(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.as_bytes().iter().zip(b.as_bytes().iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::*;
    use crate::port::{CurrencyCode, Money, PaymentStatus};

    fn usd() -> CurrencyCode {
        CurrencyCode::new("USD").unwrap()
    }

    fn receipt(provider_payment_id: Option<&str>, amount_minor: i64) -> PaymentReceipt {
        PaymentReceipt {
            payment_id: PaymentId::new(format!("pay_{amount_minor}")),
            provider_payment_id: provider_payment_id.map(str::to_owned),
            status: PaymentStatus::Captured {
                at: educore_core::value_objects::Timestamp::now(),
            },
            amount: Money::new(usd(), amount_minor).unwrap(),
            method: crate::port::PaymentMethodKind::Card,
            authorized_at: None,
            captured_at: None,
            fees: Vec::new(),
            net: Money::new(usd(), amount_minor).unwrap(),
            receipt_url: None,
            metadata: std::collections::BTreeMap::new(),
        }
    }

    fn settlement_line(provider_payment_id: &str, net_minor: i64) -> SettlementLine {
        SettlementLine {
            provider_payment_id: provider_payment_id.to_owned(),
            payment_id: PaymentId::new(format!("pay_{net_minor}")),
            gross: Money::new(usd(), net_minor).unwrap(),
            fee: Money::new(usd(), 0).unwrap(),
            net: Money::new(usd(), net_minor).unwrap(),
            settled_at: educore_core::value_objects::Timestamp::now(),
        }
    }

    // ---- 1. IdempotencyService --------------------------------------

    #[test]
    fn test_idempotency_service_derive_charge_key() {
        let a = IdempotencyService::derive_charge_key("cmd-1", &["inv_a".into(), "inv_b".into()], 1500);
        let b = IdempotencyService::derive_charge_key("cmd-1", &["inv_b".into(), "inv_a".into()], 1500);
        // Invoice order is normalised: same key for any permutation.
        assert_eq!(a, b);
        // Different amount → different key.
        let c = IdempotencyService::derive_charge_key("cmd-1", &["inv_a".into()], 1501);
        assert_ne!(a, c);
        // Different command_id → different key.
        let d = IdempotencyService::derive_charge_key("cmd-2", &["inv_a".into()], 1500);
        assert_ne!(a, d);
        // 64 hex chars (32 bytes) — SHA-256 output length.
        assert_eq!(a.len(), 64);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn idempotency_is_replay_detects_duplicates() {
        let mut seen = HashSet::new();
        assert!(!IdempotencyService::is_replay("k1", &mut seen));
        assert!(IdempotencyService::is_replay("k1", &mut seen));
        assert!(!IdempotencyService::is_replay("k2", &mut seen));
        assert!(IdempotencyService::is_replay("k2", &mut seen));
    }

    // ---- 2. WebhookSignatureService ---------------------------------

    #[test]
    fn test_webhook_signature_service_compute_and_verify() {
        let svc = WebhookSignatureService::new("super-secret-key");
        let payload = br#"{"event":"charge.succeeded","id":"ch_123"}"#;

        let sig = svc.compute_signature(payload).unwrap();
        assert!(sig.starts_with("sha256="));
        assert_eq!(sig.len(), 7 + 64); // "sha256=" + 32-byte hex

        // Round-trip: same payload + same signature verifies.
        assert!(svc.verify_signature(payload, &sig).unwrap());

        // Wrong payload fails.
        let other = svc.compute_signature(b"other").unwrap();
        assert!(!svc.verify_signature(payload, &other).unwrap());

        // Wrong length fails (constant-time guard).
        assert!(!svc.verify_signature(payload, "sha256=00").unwrap());

        // Debug redacts the secret.
        let dbg = format!("{svc:?}");
        assert!(dbg.contains("<redacted>"));
        assert!(!dbg.contains("super-secret-key"));
    }

    #[test]
    fn webhook_extract_signature_header() {
        assert_eq!(
            WebhookSignatureService::extract_signature_header("sha256=deadbeef"),
            Some("deadbeef")
        );
        // Case-insensitive prefix match.
        assert_eq!(
            WebhookSignatureService::extract_signature_header("SHA256=deadbeef"),
            Some("deadbeef")
        );
        assert_eq!(
            WebhookSignatureService::extract_signature_header("md5=00"),
            None
        );
        assert_eq!(WebhookSignatureService::extract_signature_header(""), None);
    }

    // ---- 3. BankSlipService -----------------------------------------

    #[test]
    fn test_bank_slip_service_validate_slip_number() {
        // 6-char alphanumeric: ok.
        assert!(BankSlipService::validate_slip_number("ABC123").is_ok());
        // 20-char alphanumeric: ok.
        assert!(BankSlipService::validate_slip_number("A".repeat(20).as_str()).is_ok());

        // Too short.
        let err = BankSlipService::validate_slip_number("AB12").unwrap_err();
        assert!(matches!(err, PaymentError::InvalidAmount(_)));

        // Too long.
        let err = BankSlipService::validate_slip_number("A".repeat(21).as_str()).unwrap_err();
        assert!(matches!(err, PaymentError::InvalidAmount(_)));

        // Non-alphanumeric.
        let err = BankSlipService::validate_slip_number("AB-123").unwrap_err();
        assert!(matches!(err, PaymentError::InvalidAmount(_)));
    }

    #[test]
    fn bank_slip_validate_amount_allows_one_minor_unit_rounding() {
        assert!(BankSlipService::validate_slip_amount(1500, 1500).is_ok());
        assert!(BankSlipService::validate_slip_amount(1501, 1500).is_ok());
        assert!(BankSlipService::validate_slip_amount(1499, 1500).is_ok());
        // diff > 1: error.
        let err = BankSlipService::validate_slip_amount(1502, 1500).unwrap_err();
        assert!(matches!(err, PaymentError::InvalidAmount(_)));
    }

    #[test]
    fn bank_slip_generate_id_is_unique_within_process() {
        let a = BankSlipService::generate_slip_id();
        let b = BankSlipService::generate_slip_id();
        assert_ne!(a, b);
        assert!(a.starts_with("SLIP-"));
        assert!(b.starts_with("SLIP-"));
    }

    // ---- 4. SettlementService ---------------------------------------

    #[test]
    fn test_settlement_service_match_settlement_line() {
        let line = settlement_line("ch_abc", 1500);
        let r1 = receipt(Some("ch_abc"), 1500);
        let r2 = receipt(Some("ch_xyz"), 9999);
        let matched = SettlementService::match_settlement_line(&line, &[r1.clone(), r2]);
        assert_eq!(matched, Some(r1.payment_id));

        // No matching receipt.
        let line2 = settlement_line("ch_missing", 100);
        let only = receipt(Some("ch_xyz"), 100);
        assert_eq!(
            SettlementService::match_settlement_line(&line2, &[only]),
            None
        );
    }

    #[test]
    fn settlement_compute_net_settlement_sums_net_amounts() {
        let lines = vec![
            settlement_line("ch_a", 1000),
            settlement_line("ch_b", 2500),
            settlement_line("ch_c", 500),
        ];
        assert_eq!(SettlementService::compute_net_settlement(&lines), 4000);
        assert_eq!(SettlementService::compute_net_settlement(&[]), 0);
    }

    #[test]
    fn settlement_is_settled_requires_provider_payment_id() {
        let line = settlement_line("ch_abc", 1500);
        let r = receipt(Some("ch_abc"), 1500);
        assert!(SettlementService::is_settled(&r, std::slice::from_ref(&line)));
        assert!(!SettlementService::is_settled(&r, &[]));

        // Offline receipt with no provider id: never settled by a line.
        let offline = receipt(None, 1500);
        assert!(!SettlementService::is_settled(&offline, &[line]));
    }
}
