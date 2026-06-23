## Wave 3 Adapter Audit Report — `educore-payment`

**Scope:** `crates/adapters/payment/` (5 src files: `lib.rs`,
`port.rs`, `errors.rs`, `stripe.rs`, `services.rs`; 1 test file
`tests/payment_integration.rs`); `docs/ports/payments.md`;
`docs/handoff/PHASE-15-HANDOFF.md`; `AGENTS.md` (the payment row
in the Crate Inventory); `Cargo.toml` workspace deps.

**Total findings:** 24 (12 Critical / High from Phase A, 12 more High / Medium / Low from Phase B)

---

### FINDING 1

- **id:** ADAPT-PAY-001
- **area:** adapters-payment
- **severity:** Critical
- **location:** `crates/adapters/payment/src/port.rs:521-527` (`PaymentMethod::Wallet`)
- **description:** The port replaces the spec's
  `secrecy::SecretString` wallet PIN with a plain `pub pin: String`.
  Every adapter that materialises the value (log, audit, error
  message, persistence) sees a raw PIN. The crate's own deviation
  note at `lib.rs:18-26` documents this and admits "the adapter
  MUST redact any string whose field name ends in `_pin`,
  `_secret`, or `_card` before the value reaches the audit log" —
  but no code path enforces this. `StripeProvider::charge` rejects
  `Wallet` outright, so the field never reaches a `String` log
  line in the shipped adapter, but a future wallet adapter (or
  any logging middleware) would write the PIN to stdout/log files.
  PCI-DSS scope is unaffected (wallet PIN ≠ PAN), but the design
  is a textbook secret-handling anti-pattern.
- **expected:** `docs/ports/payments.md:65-71` — `Wallet { wallet_id: WalletId, pin: SecretString }`.
  `docs/code-standards.md` § "Code Standards" — "use `secrecy`
  for secrets".
- **evidence:**
  ```rust
  // crates/adapters/payment/src/port.rs:521-527
  /// A wallet payment (school issued or third party). The PIN
  /// is captured at the consumer's frontend and forwarded only
  /// to the wallet adapter; the engine stores it transiently.
  Wallet {
      /// The wallet identifier.
      wallet_id: WalletId,
      /// The wallet's PIN (already redacted in the audit log).
      pin: String,
  },
  ```
  No `secrecy::SecretString` exists in the crate; `pin` is `String`
  end-to-end with a comment-only redaction contract.

---

### FINDING 2

- **id:** ADAPT-PAY-002
- **area:** adapters-payment
- **severity:** Critical
- **location:** `crates/adapters/payment/src/stripe.rs:357-440`
  (`PaymentProvider for StripeProvider` — `charge`, `refund`,
  `settlement`) and `crates/adapters/payment/src/lib.rs:21-26`
- **description:** The crate ships only one reference impl
  (`StripeProvider`) and that single impl returns
  `PaymentError::Provider(...)` for **5 of the 7 `PaymentMethod`
  variants** (`Cash`, `Cheque`, `BankTransfer`, `Wallet`,
  `ManualAdjustment`) plus `Gateway`. The port spec
  (`docs/ports/payments.md` § "Offline Mode") requires adapters
  for `Cash`, `Cheque`, `BankSlip`, and `Wallet`. Without an
  offline cash-book adapter the engine cannot accept any payment
  in offline mode (the documented default for schools without a
  card-processing merchant account) and cannot record any cash /
  cheque / manual adjustment at the school office. The handoff
  (PHASE-15-HANDOFF.md:103-113) confirms "1 reference impl"
  without documenting the missing offline coverage.
- **expected:** `docs/ports/payments.md` § "Offline Mode": "In
  offline mode, the consumer uses the `Cash`, `Cheque`,
  `BankSlip`, or `Wallet` methods. Online gateway methods are
  unavailable."
- **evidence:**
  ```rust
  // crates/adapters/payment/src/stripe.rs:369-410 (charge body)
  PaymentMethod::Cash => {
      return Err(PaymentError::Provider(
          "Stripe does not support offline cash payments".into(),
      ));
  }
  PaymentMethod::Cheque { .. } => {
      return Err(PaymentError::Provider(
          "Stripe does not support offline cheque payments".into(),
      ));
  }
  PaymentMethod::BankTransfer { .. } => { ... }
  PaymentMethod::Wallet { .. } => { ... }
  PaymentMethod::ManualAdjustment { .. } => { ... }
  PaymentMethod::Gateway { gateway, .. } => {
      return Err(PaymentError::Provider(format!(
          "gateway flow not supported in v1 (gateway={})",
          gateway.as_str()
      )));
  }
  ```

---

### FINDING 3

- **id:** ADAPT-PAY-003
- **area:** adapters-payment
- **severity:** Critical
- **location:** `crates/adapters/payment/src/stripe.rs:444-450`
  (`settlement`) and `crates/adapters/payment/src/port.rs:1099-1108`
  (`settlement` trait method)
- **description:** `StripeProvider::settlement` unconditionally
  returns `PaymentError::Provider("settlement is not implemented
  in the Stripe reference adapter; use a dedicated payouts
  adapter")` and no dedicated payouts adapter ships. The port
  spec defines `settlement` as the engine's mechanism for matching
  captured payments against actual bank deposits (the "engine
  consumes settlement events" flow) and the port's
  `PaymentStatus` enum includes `Captured` (not `Settled`) — so
  without settlement the engine can never confirm a charge has
  actually landed in the school's bank account. Reconciliation
  against the bank ledger is the single most important
  accounting control in any payment system; without it the
  engine reports "captured" for charges that may have been
  reversed, refunded-out-of-band, or stuck in a gateway dispute.
- **expected:** `docs/ports/payments.md` § "Settlement &
  Reconciliation" — "`settlement` returns a `Settlement` batch of
  captured payments that have settled into the school's bank
  account." § "Testing": "A test of settlement matching."
- **evidence:**
  ```rust
  // crates/adapters/payment/src/stripe.rs:444-450
  async fn settlement(&self, _request: SettlementRequest) -> Result<Settlement, PaymentError> {
      Err(PaymentError::Provider(
          "settlement is not implemented in the Stripe reference adapter; \
           use a dedicated payouts adapter".into(),
      ))
  }
  ```

---

### FINDING 4

- **id:** ADAPT-PAY-004
- **area:** adapters-payment
- **severity:** Critical
- **location:** `crates/adapters/payment/src/stripe.rs:155-205`
  (`verify_webhook_signature`) and `crates/adapters/payment/src/stripe.rs:798-812`
  (`parse_stripe_signature`)
- **description:** The webhook signature verifier parses the
  `t=<unix>,v1=<hex>` header and computes the HMAC correctly, but
  **never checks the timestamp tolerance**. Stripe's docs
  recommend a 5-minute (300-second) tolerance window to defend
  against replay; the implementation accepts any signature
  regardless of age. An attacker who captures a webhook payload
  (e.g. via a compromised log archive, a leaked proxy history,
  or a man-in-the-middle that only intercepts the response) can
  replay it indefinitely against the engine's webhook endpoint.
  The handoff does not mention replay protection; the spec
  implies it via "The engine correlates the webhook to the
  originating command via `idempotency_key`" but does not pin
  the tolerance. The cost of adding `now - t < 300` is three
  lines of code.
- **expected:** `docs/ports/payments.md` § "Webhook Flow" and
  `docs/ports/integrations.md` (referenced for signature format).
  Stripe's own docs: "Stripe-Signature tolerance is 300 seconds
  by default."
- **evidence:**
  ```rust
  // crates/adapters/payment/src/stripe.rs:155-205
  pub fn verify_webhook_signature(
      &self,
      payload: &[u8],
      signature: &str,
  ) -> Result<(), PaymentError> {
      ...
      let (timestamp, expected_v1) = parse_stripe_signature(signature)?;
      let payload_str = std::str::from_utf8(payload).map_err(...)?;
      let signed = format!("{timestamp}.{payload_str}");
      let mut mac = HmacSha256::new_from_slice(...)?;
      mac.update(signed.as_bytes());
      let computed = mac.finalize().into_bytes();
      if constant_time_eq_hex(&computed, expected_v1) {
          Ok(())
      } else {
          Err(PaymentError::Provider("webhook signature mismatch".into()))
      }
  }
  ```
  No `now.duration_since(...)` check; `timestamp` is discarded
  after being folded into `signed`.

---

### FINDING 5

- **id:** ADAPT-PAY-005
- **area:** adapters-payment
- **severity:** Critical
- **location:** `crates/adapters/payment/src/stripe.rs:412-418`
  (`refund` — params construction) and `crates/adapters/payment/src/stripe.rs:101-111`
  (module doc § "Refund lookup")
- **description:** `StripeProvider::refund` assumes
  `request.original_payment_id.as_str()` already holds the Stripe
  charge id (`ch_...`), passing it directly as the `charge=`
  form field. The engine's `PaymentId` is opaque (per
  `port.rs:63-90`) and a production deployment that mints its own
  payment ids will pass those engine ids to the refund call,
  producing a `400` from Stripe ("No such charge: `pay_abc...`").
  The crate's own deviation note at `stripe.rs:101-111` flags
  this: "A production adapter would translate via the receipt
  store; this reference impl assumes the engine's
  `PaymentId.as_str()` already holds the Stripe charge id." With
  no receipt store shipping and no translation function, the
  adapter cannot round-trip a refund end-to-end. The
  `PHASE-15-HANDOFF.md:117-121` integration test
  `payment_integration_async_stripe_refund_mock` is `#[ignore]`d
  and only verifies the provider builds, not that a refund
  succeeds.
- **expected:** `docs/ports/payments.md` § "Refund": "The
  adapter is responsible for the actual money movement."
- **evidence:**
  ```rust
  // crates/adapters/payment/src/stripe.rs:412-418
  async fn refund(&self, request: RefundRequest) -> Result<RefundReceipt, PaymentError> {
      let mut params: Vec<(String, String)> = Vec::with_capacity(3);
      params.push((
          "charge".to_owned(),
          request.original_payment_id.as_str().to_owned(),
      ));
      ...
  ```

---

### FINDING 6

- **id:** ADAPT-PAY-006
- **area:** adapters-payment
- **severity:** Critical
- **location:** `crates/adapters/payment/src/stripe.rs:388-411`
  (`charge` for `Card { save: true }`) and `crates/adapters/payment/src/stripe.rs:117-121`
  (module doc § "`save` flag")
- **description:** When a `PaymentMethod::Card { save: true }` is
  passed, the adapter "passes it through verbatim" (per the
  module doc at `stripe.rs:117-121`). Stripe's Charges API has no
  `save` parameter — vaulting requires the SetupIntent +
  Customer flow with the PaymentMethod API. The `save: true` flag
  is silently ignored: the card is charged but never saved, and
  no error is returned. A consumer wiring up recurring fee
  collection will pass `save: true`, observe the one-off charge
  succeed, then have recurring charges fail at the next billing
  cycle when there is no PaymentMethod on file. The deviation
  note acknowledges this but the impl has no path to inform the
  caller.
- **expected:** `docs/ports/payments.md` § "PaymentMethod":
  `Card { token: CardToken, save: bool }` — "save requests that
  the gateway store the card for future recurring charges."
- **evidence:**
  ```rust
  // crates/adapters/payment/src/stripe.rs:362-388 (Card branch)
  PaymentMethod::Card { token, .. } => {
      let mut p = Vec::with_capacity(6 + request.metadata.len());
      p.push(("amount".to_owned(), request.amount.amount_minor.to_string()));
      p.push(("currency".to_owned(), request.amount.currency.as_str().to_ascii_lowercase()));
      p.push(("source".to_owned(), token.as_str().to_owned()));
      ...
      // No `save` handling — `..` discards the `save` field.
  }
  ```
  The `..` pattern at `token, ..` discards `save`; no SetupIntent
  path exists.

---

### FINDING 7

- **id:** ADAPT-PAY-007
- **area:** adapters-payment
- **severity:** Critical
- **location:** `crates/adapters/payment/src/stripe.rs:345-411`
  (`charge` body), `crates/adapters/payment/src/stripe.rs:768-796`
  (`stripe_error_to_payment_error`)
- **description:** The port spec defines
  `PaymentError::ThreeDSRequired` as the contract for handling
  3-D Secure challenges: the adapter returns it on the initial
  call so the engine can redirect the customer to the issuer's
  3DS challenge. The shipped adapter never returns this variant
  — instead, a 3DS-required Stripe response flows through
  `stripe_error_to_payment_error` (lines 768-796) as a generic
  `PaymentError::CardError` (`card_error` type with a
  `authentication_required` decline code falls into the default
  `card_error` arm and surfaces as `Declined("authentication
  required (code=...)")`). The consumer cannot distinguish a
  3DS challenge from a hard decline and has no signal to initiate
  the issuer's challenge flow.
- **expected:** `docs/ports/payments.md` § "PaymentError":
  `ThreeDSRequired` — "The gateway requires 3-D Secure
  authentication before the charge can proceed."
- **evidence:**
  ```rust
  // crates/adapters/payment/src/stripe.rs:768-796
  match (status, err_type) {
      (_, "rate_limit_error") => PaymentError::RateLimited,
      (_, "card_error") if decline_code == Some("insufficient_funds") => {
          PaymentError::InsufficientFunds
      }
      (_, "card_error") => PaymentError::Declined(format!(
          "{}{}",
          message,
          code.map(|c| format!(" (code={c})")).unwrap_or_default()
      )),
      ...
  }
  ```
  No match arm inspects `decline_code == Some("authentication_required")`
  or Stripe's `payment_intent.next_action.use_stripe_sdk` payload.

---

### FINDING 8

- **id:** ADAPT-PAY-008
- **area:** adapters-payment
- **severity:** High
- **location:** `crates/adapters/payment/Cargo.toml:11-22` and
  `crates/adapters/payment/src/lib.rs:11-13`
- **description:** The crate has zero audit-log writes. The port
  spec (`docs/ports/payments.md` § "Audit") states: "Every
  charge, refund, status change, and settlement is recorded in
  the audit log with amount, currency, method, parties, and
  metadata. Card data is never logged." The crate's `Cargo.toml`
  declares `educore-core`, `educore-platform`, `educore-events`,
  `tokio`, `async-trait`, `reqwest`, `serde_json`, `hmac`,
  `sha2` — but no `educore-audit`. Per
  `PHASE-15-HANDOFF.md:46-51` the audit crate added two new
  variants (`PaymentReceipt`, `Refund`) for this purpose, but
  the payment adapter never imports `educore-audit` to use them.
  Every charge, refund, status change, and settlement therefore
  proceeds without an immutable audit trail. The
  `PaymentReceived`, `PaymentRefunded`, `PaymentSettled` events
  the spec references (`port.rs:983-985` for the docstring on
  receipts) likewise do not exist on this crate — `educore-events`
  is declared but only re-exported in the prelude is the trait
  shape, not a publisher call.
- **expected:** `docs/ports/payments.md` § "Audit": "Every
  charge, refund, status change, and settlement is recorded in
  the audit log …" `AGENTS.md` Engine Rule 8: "Audit-first.
  Every state change writes an immutable record."
- **evidence:**
  ```toml
  # crates/adapters/payment/Cargo.toml:11-22
  [dependencies]
  educore-core = { workspace = true }
  educore-platform = { workspace = true }
  educore-events = { workspace = true }
  tokio = { workspace = true }
  async-trait = { workspace = true }
  reqwest = { workspace = true }
  serde_json = { workspace = true }
  hmac = { workspace = true }
  sha2 = { workspace = true }
  ```
  No `educore-audit`. `grep -rn "audit" crates/adapters/payment/`
  returns zero non-docstring hits.

---

### FINDING 9

- **id:** ADAPT-PAY-009
- **area:** adapters-payment
- **severity:** High
- **location:** `crates/adapters/payment/src/services.rs:60-90`
  (`IdempotencyService::derive_charge_key`) and
  `docs/handoff/PHASE-15-HANDOFF.md:118-119`
- **description:** `IdempotencyService::derive_charge_key` hashes
  only `command_id | invoice_ids | amount_minor` — no
  `tenant_school_id`, no `user_id`, no `currency`, no
  `payment_method`. The handoff (PHASE-15-HANDOFF.md:118-119)
  asserts the canonical form is
  `SHA-256(tenant|user|amount|currency|method)`, but the
  implementation does not match its own documentation. Two
  schools submitting the same `command_id` (a UUID collision or
  a consumer that mints `command_id`s from a per-school sequence)
  collide on the key. Two payments in the same school differing
  only in `currency` collapse to the same key. The
  `payment_integration_idempotency_charge_key` integration test
  at `tests/payment_integration.rs:14-31` only verifies invoice
  ordering and amount differences, not the documented tenant /
  user / currency / method fields.
- **expected:** PHASE-15-HANDOFF.md:118-119 — `SHA-256(tenant |
  user | amount | currency | method)`.
- **evidence:**
  ```rust
  // crates/adapters/payment/src/services.rs:60-90
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
  ```
  No `school_id`, no `user_id`, no `currency`, no `method`.

---

### FINDING 10

- **id:** ADAPT-PAY-010
- **area:** adapters-payment
- **severity:** High
- **location:** `crates/adapters/payment/src/services.rs:281-330`
  (`BankSlipService::validate_slip_number`,
  `BankSlipService::generate_slip_id`) and
  `docs/handoff/PHASE-15-HANDOFF.md:120-122`
- **description:** `BankSlipService::validate_slip_number` only
  enforces length (6-20 chars) and ASCII-alphanumeric
  (services.rs:282-298). The handoff (PHASE-15-HANDOFF.md:120-122)
  asserts "mod-11 check" is performed. Brazilian "boleto" slip
  numbers (the documented target per `services.rs:215`) carry a
  mod-11 check digit; without the check, a typo in a slip number
  silently matches an invoice and a parent / accountant can
  approve a slip that does not exist in the bank's ledger. The
  integration test at `tests/payment_integration.rs:65-77`
  asserts only length and alphanumeric (`"AB-123"` rejected for
  the dash) but no checksum.
- **expected:** PHASE-15-HANDOFF.md:120-122 — "mod-11 check";
  `services.rs:215` — "Brazilian-style 'boleto' bank-slip
  inputs".
- **evidence:**
  ```rust
  // crates/adapters/payment/src/services.rs:282-298
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
  ```
  No checksum digit verification; `"ABC123"` and `"ABC124"`
  (typo) both pass.

---

### FINDING 11

- **id:** ADAPT-PAY-011
- **area:** adapters-payment
- **severity:** High
- **location:** `crates/adapters/payment/src/services.rs:307-321`
  (`BankSlipService::generate_slip_id`) and
  `crates/adapters/payment/src/services.rs:209-210`
  (the static `SLIP_COUNTER`)
- **description:** `BankSlipService::generate_slip_id` mints ids
  from a process-local `AtomicU64` counter (`SLIP-00000001`,
  `SLIP-00000002`, …). After a process restart the counter resets
  to 0, so two distinct physical slips generated by two
  consecutive process runs can collide on `SLIP-00000001`. The
  module-level doc at `services.rs:51-54` documents this: "The
  id is unique within a process; it is not a global UUID." The
  spec (`docs/ports/payments.md` § "Bank Slip Flow") treats slip
  ids as durable identifiers that are stored in the file storage
  port and referenced across approval and reconciliation. With
  counter-reset collisions the engine can mis-attribute a slip
  photo to a previous slip. The deviation note acknowledges the
  limitation but ships it as the only id-minting path; no
  `uuid::Uuid::new_v4()` fallback or startup-rng seed is
  available.
- **expected:** `docs/ports/payments.md` § "Bank Slip Flow" —
  slip id is durable and stable across process restarts.
- **evidence:**
  ```rust
  // crates/adapters/payment/src/services.rs:209-210, 320-322
  static SLIP_COUNTER: AtomicU64 = AtomicU64::new(0);
  ...
  #[must_use]
  pub fn generate_slip_id() -> String {
      let n = SLIP_COUNTER.fetch_add(1, Ordering::SeqCst);
      format!("SLIP-{n:08}")
  }
  ```
  No `AtomicU64::compare_exchange` against a persisted offset;
  no UUID; counter starts at 0 on every process boot.

---

### FINDING 12

- **id:** ADAPT-PAY-012
- **area:** adapters-payment
- **severity:** High
- **location:** `crates/adapters/payment/src/services.rs:355-361`
  (`SettlementService::compute_net_settlement`) and
  `crates/adapters/payment/src/services.rs:362-365` (docstring
  § "SettlementService")
- **description:** `compute_net_settlement` uses `sum` rather
  than `saturating_add` over `i64` minor units. A settlement
  batch with enough high-value lines (or a stuck test fixture
  with `i64::MAX` per line) silently wraps to a negative number.
  The docstring at `services.rs:362-365` acknowledges this: "a
  settlement batch that genuinely overflows `i64` is a
  programming error and the wrap is a loud signal." In a
  production payment system, a wrapped total net means the
  reconciliation engine records a *negative* batch deposit —
  triggering a refund flow on what should have been a positive
  settlement. The spec (`docs/ports/payments.md` § "Settlement")
  requires the engine to consume settlement events and emit
  `PaymentSettled`; a wrapped total would silently propagate
  through events and audit log.
- **expected:** `docs/code-standards.md` § "Numeric
  conversions": "Numeric conversions use `TryFrom`/`TryInto`;
  `as` on numerics is forbidden." Per the same standards, a
  saturating-or-checked sum is the only acceptable aggregation.
- **evidence:**
  ```rust
  // crates/adapters/payment/src/services.rs:355-361
  #[must_use]
  pub fn compute_net_settlement(lines: &[SettlementLine]) -> i64 {
      lines.iter().map(|l| l.net.amount_minor).sum()
  }
  ```

---

### FINDING 13

- **id:** ADAPT-PAY-013
- **area:** adapters-payment
- **severity:** High
- **location:** `crates/adapters/payment/src/stripe.rs:724-744`
  (`map_charge_status` — `partially_refunded`, `authorized`,
  `disputed`, `refunded` arms)
- **description:** `map_charge_status` synthesises four
  `PaymentStatus` variants with **empty / wrong / fabricated**
  inner data when Stripe reports them. Specifically:
  - `partially_refunded` (stripe.rs:733-738) hard-codes
    `remaining: Money::zero(amount.currency.clone())` instead
    of computing `gross - refunded`; the engine's
    `PartiallyRefunded { refunded, remaining }` invariant
    (per `port.rs:741-746`) is broken — every partially-refunded
    charge reports zero remaining, so the engine will allow
    refunds up to the full original amount (double-refund risk).
  - `Authorized` (stripe.rs:722-726) sets `auth_code: String::new()`
    and `expires_at: Timestamp::now()`; the auth code is empty
    and the expiry is "right now", so any auth-then-capture flow
    fails immediately.
  - `Disputed` (stripe.rs:740-744) sets `dispute_id: String::new()`,
    `reason: String::new()`; the dispute id is the only handle
    the support team has to escalate.
  - `Refunded` (stripe.rs:728-732) sets `reason: String::new()`
    and reuses the request `amount` rather than reading the
    actual refunded amount from Stripe.
- **expected:** `docs/ports/payments.md` § "PaymentStatus" — all
  five variants carry the spec'd inner fields with the data
  Stripe actually returned.
- **evidence:**
  ```rust
  // crates/adapters/payment/src/stripe.rs:728-744
  "refunded" => PaymentStatus::Refunded {
      amount: amount.clone(),
      at: Timestamp::now(),
      reason: String::new(),
  },
  "partially_refunded" => PaymentStatus::PartiallyRefunded {
      refunded: amount.clone(),
      remaining: Money::zero(amount.currency.clone()),
  },
  "disputed" => PaymentStatus::Disputed {
      dispute_id: String::new(),
      reason: String::new(),
      opened_at: Timestamp::now(),
  },
  ```

---

### FINDING 14

- **id:** ADAPT-PAY-014
- **area:** adapters-payment
- **severity:** High
- **location:** `crates/adapters/payment/src/stripe.rs:413-444`
  (`refund` body — no `AlreadyRefunded` / `RefundExceedsOriginal`
  check) and `crates/adapters/payment/src/errors.rs:69-78`
  (`PaymentError::AlreadyRefunded`, `PaymentError::RefundExceedsOriginal`)
- **description:** The error enum defines
  `PaymentError::AlreadyRefunded` and
  `PaymentError::RefundExceedsOriginal` (errors.rs:69-78) but
  the `StripeProvider::refund` implementation never returns
  either. A consumer that retries a refund past the original
  amount (or refunds an already-refunded receipt) receives a
  generic `PaymentError::Provider("stripe error (400): ...")`
  rather than the typed variant. The test list in
  `docs/ports/payments.md` § "Testing" requires "A test of
  partial refund" — without the typed errors the engine cannot
  detect the double-refund attempt at the domain layer and
  instead retries a 4xx, hitting Stripe rate limits.
- **expected:** `docs/ports/payments.md` § "PaymentError" —
  `AlreadyRefunded` and `RefundExceedsOriginal` are the
  contract for double-refund detection.
- **evidence:**
  ```rust
  // crates/adapters/payment/src/stripe.rs:413-444 (no check)
  async fn refund(&self, request: RefundRequest) -> Result<RefundReceipt, PaymentError> {
      let mut params: Vec<(String, String)> = Vec::with_capacity(3);
      params.push((
          "charge".to_owned(),
          request.original_payment_id.as_str().to_owned(),
      ));
      params.push(("amount".to_owned(), request.amount.amount_minor.to_string()));
      if !request.reason.is_empty() {
          params.push(("reason".to_owned(), stripe_refund_reason(&request.reason)));
      }
      let body = self.post_form("refunds", &params, &request.idempotency_key.to_string()).await?;
      refund_receipt_from_refund(&body, request)
  }
  ```
  No comparison against the original `amount`; no `status()`
  precheck; no Stripe-side `charge.already_refunded` lookup.

---

### FINDING 15

- **id:** ADAPT-PAY-015
- **area:** adapters-payment
- **severity:** High
- **location:** `crates/adapters/payment/src/services.rs:351-360`
  (`SettlementService::compute_net_settlement`,
  `SettlementRequest` validation) and
  `crates/adapters/payment/src/port.rs:967-982`
  (`SettlementRequest`)
- **description:** `SettlementService` does not validate that
  `request.period_start <= request.period_end` or that the
  window is non-empty. A consumer that constructs a
  `SettlementRequest` with `period_start > period_end` (typo,
  clock skew, off-by-one) silently gets a settlement report
  covering a zero-length or backwards window. The Stripe
  reference impl returns `PaymentError::Provider("settlement is
  not implemented ...")` so the bug is currently masked, but a
  real payouts adapter will hand the window to Stripe as-is.
  The docstring on `SettlementRequest` (`port.rs:967-982`)
  describes the window as "Inclusive start of the settlement
  window" / "Inclusive end of the settlement window" but
  provides no validation entry point.
- **expected:** `docs/ports/payments.md` § "Settlement &
  Reconciliation" — the window is well-formed.
- **evidence:**
  ```rust
  // crates/adapters/payment/src/port.rs:967-982
  pub struct SettlementRequest {
      pub tenant: TenantContext,
      pub period_start: Timestamp,
      pub period_end: Timestamp,
      pub currency: CurrencyCode,
  }
  ```
  No constructor / no validator; `SettlementService` exposes
  only `match_settlement_line`, `compute_net_settlement`, and
  `is_settled` — none of which inspect the window.

---

### FINDING 16

- **id:** ADAPT-PAY-016
- **area:** adapters-payment
- **severity:** High
- **location:** `crates/adapters/payment/src/stripe.rs:358-411`
  (`charge` body — `Card` arm) and
  `crates/adapters/payment/src/stripe.rs:432-444`
  (`refund_receipt_from_refund` — `receipt_from_charge` mapping)
- **description:** The Stripe `charge` POST does not request
  expanded fields (`expand[]=balance_transaction`,
  `expand[]=customer`). `extract_stripe_fees`
  (`stripe.rs:551-572`) reads
  `charge["balance_transaction"]` and returns an empty `fees`
  vector when the field is a string id rather than an expanded
  object. Stripe does not expand by default, so every shipped
  charge reports `fees: Vec::new()` and `net: amount` (gross
  equal to net), even though Stripe deducted a processing fee.
  The engine's `PaymentReceipt.net` (`port.rs:874-878`) is
  documented as "gross minus fees"; with `fees = []` the
  finance reconciler sees the gross deposit as the net deposit
  and the school's bank-account reconciliation drifts by the
  2.9% + 30¢ per charge.
- **expected:** `docs/ports/payments.md` § "PaymentReceipt":
  `net: Money` is "The net amount (gross minus fees)
  deposited in the school's account."
- **evidence:**
  ```rust
  // crates/adapters/payment/src/stripe.rs:551-572
  fn extract_stripe_fees(
      charge: &JsonValue,
      currency: &crate::port::CurrencyCode,
  ) -> Result<Vec<PaymentFee>, PaymentError> {
      let Some(balance_txn) = charge.get("balance_transaction") else {
          return Ok(Vec::new());   // <- default path
      };
      if !balance_txn.is_object() {
          return Ok(Vec::new());   // <- default path
      }
      ...
  }
  ```
  And the POST body at `stripe.rs:362-388` has no
  `expand[]=balance_transaction` entry.

---

### FINDING 17

- **id:** ADAPT-PAY-017
- **area:** adapters-payment
- **severity:** High
- **location:** `crates/adapters/payment/src/stripe.rs:897-906`
  (`stripe_refund_reason`) and `crates/adapters/payment/src/stripe.rs:413-444`
  (`refund` body — callsite)
- **description:** `stripe_refund_reason` returns an empty
  string for any reason that does not match Stripe's whitelist
  (`duplicate` / `fraudulent` / `requested_by_customer`) — but
  the empty string is still passed to Stripe as the
  `reason=...` form field. Stripe silently drops the empty
  field and the audit log loses the human-readable refund
  reason entirely. The handoff notes the whitelist but does not
  document that the unmapped reason is destroyed. A consumer
  that records `"customer requested after delivery issue"`
  will see no audit trail entry tying the refund to the
  underlying issue.
- **expected:** `docs/ports/payments.md` § "Refund" — reason is
  "shown on the customer's statement" and persisted in the
  audit log.
- **evidence:**
  ```rust
  // crates/adapters/payment/src/stripe.rs:897-906
  fn stripe_refund_reason(reason: &str) -> String {
      match reason {
          "duplicate" | "fraudulent" | "requested_by_customer" => reason.to_owned(),
          other => match other.to_ascii_lowercase().as_str() {
              "duplicate" => "duplicate".to_owned(),
              "fraud" | "fraudulent" => "fraudulent".to_owned(),
              "customer" | "requested" | "requested_by_customer" => {
                  "requested_by_customer".to_owned()
              }
              _ => String::new(),  // <- silent loss
          },
      }
  }
  // stripe.rs:413-444
  if !request.reason.is_empty() {
      params.push(("reason".to_owned(), stripe_refund_reason(&request.reason)));
  }
  ```
  When `request.reason = "customer changed mind"`,
  `stripe_refund_reason` returns `""`, which is then
  pushed to the form — Stripe drops it, the audit log has no
  reason.

---

### FINDING 18

- **id:** ADAPT-PAY-018
- **area:** adapters-payment
- **severity:** High
- **location:** `crates/adapters/payment/src/stripe.rs:828-855`
  (`parse_stripe_signature`)
- **description:** `parse_stripe_signature` captures only the
  first `v1=` value from the `Stripe-Signature` header and
  silently discards additional `v1=` entries. Stripe's
  documentation recommends emitting **multiple `v1=` values**
  during a webhook secret rotation so a delivery can be
  verified against either the old or new secret. The
  implementation only checks the first `v1=`; if Stripe sends
  `t=...,v1=<new-secret-sig>,v1=<old-secret-sig>` and the
  provider was configured with the old secret, the
  verification fails. Worse, there is no warning that
  additional `v1=` entries were dropped — the log shows a
  clean signature mismatch.
- **expected:** `docs.stripe.com/webhooks#verify-official-libraries`
  — accept any `v1=` entry that verifies.
- **evidence:**
  ```rust
  // crates/adapters/payment/src/stripe.rs:828-855
  for part in signature.split(',') {
      let Some((k, v)) = part.split_once('=') else { continue; };
      match k {
          "t" => { timestamp = v.parse::<i64>().ok(); }
          "v1" if v1.is_none() => { v1 = Some(v); }  // <- first only
          _ => {}
      }
  }
  ```
  No `Vec<&str>` accumulation; no per-`v1` verify loop in
  `verify_webhook_signature` (stripe.rs:155-205).

---

### FINDING 19

- **id:** ADAPT-PAY-019
- **area:** adapters-payment
- **severity:** Medium
- **location:** `crates/adapters/payment/src/stripe.rs:45-56`
  (`StripeProvider` struct fields `secret_key: String`,
  `webhook_secret: String`) and
  `crates/adapters/payment/src/stripe.rs:268-283`
  (`Debug` impl + `redact_secret`)
- **description:** `StripeProvider` holds `secret_key` and
  `webhook_secret` as `String` (not `secrecy::SecretString`).
  The `Debug` impl redacts them with `redact_secret` returning
  `&'static str` — every configured provider's `Debug` output
  looks identical (`<redacted>`), so log analysis tools
  cannot distinguish two concurrently-wired Stripe providers
  (e.g. `sk_test_` vs `sk_live_`, or two schools with
  different keys). The wallet PIN deviation (FINDING 1) is
  the same shape; the API-key deviation is functionally
  similar but the practical risk is lower because the value
  never leaves the provider's `Debug` impl. The crate's
  deviation note (`port.rs:18-26`) explicitly opts out of
  `secrecy` — the cost is this debugging-blindness.
- **expected:** `docs/ports/payments.md` § "Webhook Flow" —
  secrets must not leak via `Debug` or log; the `SecretString`
  type implements `Debug` as `Secret<String>` so the value
  is never reachable from a `format!("{:?}", provider)`.
- **evidence:**
  ```rust
  // crates/adapters/payment/src/stripe.rs:268-283
  impl fmt::Debug for StripeProvider {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
          f.debug_struct("StripeProvider")
              .field("base_url", &self.base_url)
              .field("secret_key", &redact_secret(&self.secret_key))
              .field("webhook_secret", &redact_secret(&self.webhook_secret))
              .finish()
      }
  }
  // stripe.rs:889-895
  fn redact_secret(s: &str) -> &'static str {
      if s.is_empty() { "<empty>" } else { "<redacted>" }
  }
  ```
  Two providers, two different keys, two identical `Debug`
  outputs.

---

### FINDING 20

- **id:** ADAPT-PAY-020
- **area:** adapters-payment
- **severity:** Medium
- **location:** `crates/adapters/payment/tests/payment_integration.rs:128-145`
  (`payment_integration_async_stripe_charge_mock`,
  `payment_integration_async_stripe_refund_mock`)
- **description:** Both env-gated async integration tests are
  `#[ignore = "requires EDUCORE_PORT_ADAPTER_E2E env var"]` and
  contain only `StripeProviderBuilder::new()...build()` calls —
  they verify that `reqwest::Client::builder()` succeeds, not
  that an actual Stripe round-trip works. No `mockito`,
  `wiremock`, or `httpmock` server is used; no HTTP traffic is
  recorded; no fixture body is parsed. The handoff
  (PHASE-15-HANDOFF.md:131-135) describes the tests as
  "vertical-slice integration tests" but they only verify the
  builder constructs. A `cargo test --workspace` will never run
  them; `cargo test -- --ignored` will pass without exercising
  any Stripe API call.
- **expected:** `docs/ports/payments.md` § "Testing" — charge,
  refund, status, idempotency, webhook reconciliation each
  have a passing test against a mocked Stripe endpoint.
- **evidence:**
  ```rust
  // crates/adapters/payment/tests/payment_integration.rs:128-145
  #[tokio::test]
  #[ignore = "requires EDUCORE_PORT_ADAPTER_E2E env var"]
  async fn payment_integration_async_stripe_charge_mock() {
      let _provider = StripeProviderBuilder::new()
          .secret_key("sk_test_placeholder".to_owned())
          .webhook_secret("whsec_test_placeholder".to_owned())
          .build()
          .expect("reqwest client build");
  }
  #[tokio::test]
  #[ignore = "requires EDUCORE_PORT_ADAPTER_E2E env var"]
  async fn payment_integration_async_stripe_refund_mock() {
      let _provider = StripeProviderBuilder::new()
          .secret_key("sk_test_placeholder".to_owned())
          .build()
          .expect("reqwest client build");
  }
  ```
  No `let body = ...`; no `MockServer::start()`; no assertion
  on response shape.

---

### FINDING 21

- **id:** ADAPT-PAY-021
- **area:** adapters-payment
- **severity:** Medium
- **location:** `crates/adapters/payment/src/port.rs:34-39`
  (deviation note § "URL is represented as `String`") and
  `crates/adapters/payment/src/port.rs:841-846`
  (`ChargeRequest::webhook_url: Option<String>`)
- **description:** The port replaces `url::Url` with `String`
  for `webhook_url`, `return_url`, and `receipt_url`. None of
  these fields is validated at construction time: a malformed
  URL (`"not a url"`, `"javascript:alert(1)"`, empty string)
  flows through to Stripe as the `return_url=` form field and
  is silently accepted by the port. A typo in the return URL
  redirects the customer to a 404; a `javascript:` URL in a
  hosted-page flow is a reflected XSS vector if the gateway's
  page surfaces the return URL verbatim.
- **expected:** `docs/ports/payments.md` § "ChargeRequest" —
  `webhook_url: Option<Url>` (typed, parsed, scheme-checked).
- **evidence:**
  ```rust
  // crates/adapters/payment/src/port.rs:841-846
  /// Optional webhook URL the gateway should POST status
  /// updates to.
  pub webhook_url: Option<String>,
  ```
  No `Url::parse`; no `scheme == "https"` check; bare `String`.

---

### FINDING 22

- **id:** ADAPT-PAY-022
- **area:** adapters-payment
- **severity:** Medium
- **location:** `crates/adapters/payment/src/services.rs:332-360`
  (`SettlementService`) and
  `crates/adapters/payment/src/port.rs:998-1011`
  (`Settlement` struct)
- **description:** `SettlementService` exposes only three
  helpers (`match_settlement_line`, `compute_net_settlement`,
  `is_settled`) — none of them validate the invariants on the
  `Settlement` struct itself:
  - `Settlement.total_gross == sum(line.gross)` is not checked.
  - `Settlement.total_fees == sum(line.fee)` is not checked.
  - `Settlement.total_net == sum(line.net)` is not checked.
  - `Settlement.lines` may be empty while `total_gross` is
    non-zero (no empty-batch invariant).
  - `Settlement.school_id` may not match
    `SettlementRequest.tenant.school_id` (no school-tenant
    cross-check).
  - `Settlement.currency` may not match every `line.net.currency`
    (no per-line currency validation).
  These are exactly the invariants the spec expects the
  settlement adapter to enforce
  (`docs/ports/payments.md` § "Settlement & Reconciliation");
  without them a buggy payouts adapter can produce
  `Settlement { total_gross: 100_000, lines: [] }` and the
  engine has no way to detect the inconsistency.
- **expected:** `docs/ports/payments.md` § "Settlement" — the
  totals agree with the lines.
- **evidence:**
  ```rust
  // crates/adapters/payment/src/port.rs:998-1011
  pub struct Settlement {
      pub settlement_id: String,
      pub school_id: SchoolId,
      pub currency: CurrencyCode,
      pub period_start: Timestamp,
      pub period_end: Timestamp,
      pub lines: Vec<SettlementLine>,
      pub total_gross: Money,
      pub total_fees: Money,
      pub total_net: Money,
  }
  ```
  No `Settlement::validate(&self) -> Result<(), PaymentError>`;
  no constructor.

---

### FINDING 23

- **id:** ADAPT-PAY-023
- **area:** adapters-payment
- **severity:** Medium
- **location:** `crates/adapters/payment/src/port.rs:323-348`
  (`CurrencyCode::new`) and `crates/adapters/payment/src/port.rs:369-388`
  (`Money::new`)
- **description:** `CurrencyCode::new` rejects non-ASCII-uppercase
  input (`port.rs:323-348`), but the `Cargo.toml` does not
  depend on any ISO-4217 source-of-truth list, so `CurrencyCode::new("XBT")`
  succeeds even though "XBT" is not a registered ISO-4217
  alphabetic code (the closest match is XBT — actually XBT is
  the unofficial bitcoin code; ISO reserves it under the
  internal "X" range). Similarly `"USDX"`, `"ZZZ"`, `"AAA"`
  all pass. A consumer charging in a non-ISO currency sends
  it to Stripe, which rejects with `400` — but the typed
  `CurrencyCode` gives the consumer a false sense of validity.
  The spec (`docs/ports/payments.md` § "Multi-Currency") treats
  the code as authoritative; the validator should match.
- **expected:** `docs/ports/payments.md` § "Multi-Currency" —
  `CurrencyCode` is a valid ISO-4217 code (not just
  3 ASCII uppercase).
- **evidence:**
  ```rust
  // crates/adapters/payment/src/port.rs:323-348
  pub fn new(code: &str) -> Result<Self, crate::errors::PaymentError> {
      if code.len() == 3 && code.chars().all(|c| c.is_ascii_uppercase()) {
          Ok(Self(code.to_owned()))
      } else {
          Err(...)
      }
  }
  ```
  No ISO-4217 lookup; `"ZZZ"` is accepted.

---

### FINDING 24

- **id:** ADAPT-PAY-024
- **area:** adapters-payment
- **severity:** Low
- **location:** `crates/adapters/payment/src/stripe.rs:54-56`
  (`HTTP_TIMEOUT_SECS = 30`) and `crates/adapters/payment/src/stripe.rs:332-344`
  (`StripeProviderBuilder::build` — Client builder)
- **description:** `StripeProviderBuilder::build` configures a
  single 30-second request timeout via
  `Client::builder().timeout(Duration::from_secs(30))`. There is
  no `connect_timeout`, no `tcp_keepalive`, no retries
  (`reqwest`'s built-in retry middleware is off by default), no
  request-id propagation, and no user-agent string identifying
  the Educore engine. A consumer that hits Stripe over a
  flaky link (mobile / developing-country 3G) will see every
  charge fail with `Infrastructure` after 30 s. The
  `PaymentError::RateLimited` variant exists but the adapter
  does not back off; it surfaces the rate-limit response
  immediately to the caller, with no hint about how long to
  wait.
- **expected:** AGENTS.md § "TLS/SSL Cross-Compilation"
  requires `rustls` for `reqwest`. The same standards should
  drive a production-grade HTTP client (connect timeout, retry
  policy, user-agent).
- **evidence:**
  ```rust
  // crates/adapters/payment/src/stripe.rs:54-56
  /// HTTP request timeout applied to every outbound call.
  const HTTP_TIMEOUT_SECS: u64 = 30;
  // stripe.rs:332-344
  let http = Client::builder()
      .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
      .build()
      .map_err(|e| PaymentError::Infrastructure(InfrastructureError::new(...)))?;
  ```
  No `.connect_timeout(...)`, no `.user_agent(...)`, no
  `.tcp_keepalive(...)`, no retry config.

---
