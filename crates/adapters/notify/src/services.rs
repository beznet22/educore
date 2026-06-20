//! # Notify service helpers
//!
//! Pure helper structs that sit alongside the
//! [`NotificationProvider`](crate::port::NotificationProvider) port.
//! None of them perform I/O; they exist so the adapter crates
//! (email, SMS, push, …) can keep their hot paths tight and so
//! the engine's template, channel-fan-out, idempotency, and
//! rate-limit logic can be unit-tested without spinning up an
//! SMTP / SMS / FCM backend.
//!
//! The four services are:
//!
//! - [`TemplateService`] — substitutes `{var_name}` placeholders
//!   in a template body, validates that every variable referenced
//!   by the body has been supplied, and extracts the variable
//!   names referenced by a body. Pure string-in / string-out; no
//!   knowledge of [`TemplateValue`](crate::port::TemplateValue).
//! - [`ChannelService`] — classifies a [`Channel`](crate::port::Channel)
//!   (async vs sync, authentication requirements) and computes the
//!   per-channel fan-out list for a single-channel request.
//! - [`IdempotencyService`] — derives a deterministic SHA-256 hex
//!   idempotency key from `(command_id, recipient, template_version)`
//!   and tracks seen keys to detect replays.
//! - [`RateLimitService`] — in-memory token-bucket rate limiter,
//!   one bucket per [`Channel`](crate::port::Channel). Refills at
//!   one token per second up to `max_per_second`. Useful for
//!   tests and for short-lived single-tenant adapters; production
//!   deployments should back the limiter with a shared store.
//!
//! # Deviations from the spec
//!
//! - **SHA-256 is hand-rolled, not pulled from the `sha2` crate.**
//!   The crate's `Cargo.toml` does not declare `sha2` and the task
//!   spec for this file (`Phase 15: educore-notify services (B)`)
//!   explicitly lists `crates/adapters/notify/Cargo.toml` under
//!   "DO NOT TOUCH". The hand-rolled implementation is small
//!   (~100 lines), FIPS 180-4 §6.2 compliant, and follows the
//!   same pattern already in use at
//!   `crates/adapters/files/src/local.rs`. A follow-up commit
//!   by the port+types owner can move it behind `use sha2::{...}`
//!   once the orchestrator re-balances file ownership.
//! - **`RateLimitService` uses `HashMap<String, RateState>` rather
//!   than `HashMap<Channel, RateState>`.** [`Channel`](crate::port::Channel)
//!   does not currently derive `Hash`, and the spec also lists
//!   `crates/adapters/notify/src/port.rs` under "DO NOT TOUCH".
//!   The map key is produced by [`channel_key`], a small
//!   discriminant-based matcher. The lookup semantics are
//!   unchanged from the spec.
//! - **Token-bucket refill uses millisecond precision.** The spec
//!   says "1 token per 1000ms"; the implementation reads
//!   `Instant::elapsed().as_millis() / 1000` so sub-second
//!   carry-over is discarded (a 500ms pause refills zero tokens).
//!   This matches the spec's literal "1 token per 1000ms" wording.

#![allow(clippy::missing_docs_in_private_items)]

use std::collections::{BTreeMap, HashMap, HashSet};
use std::time::{Duration, Instant};

use crate::errors::NotificationError;
use crate::port::Channel;

// ---------------------------------------------------------------------------
// SHA-256 (FIPS 180-4) — hand-rolled, stdlib only.
//
// The crate's `Cargo.toml` does not declare the `sha2` crate; the
// task spec for this file lists the manifest under "DO NOT TOUCH".
// We follow the same pattern as `crates/adapters/files/src/local.rs`:
// a small, FIPS 180-4 §6.2 compliant SHA-256 plus a hex-encode
// helper. Audit-friendly, no new dependencies.
// ---------------------------------------------------------------------------

/// SHA-256 round constants (FIPS 180-4 §4.2.2).
const SHA256_K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a735b, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// Initial hash values for SHA-256 (FIPS 180-4 §5.3.3): the
/// first 32 bits of the fractional parts of the square roots of
/// the first 8 primes.
const SHA256_H0: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527c, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

/// Computes the SHA-256 digest of `message` (FIPS 180-4 §6.2).
fn sha256(message: &[u8]) -> [u8; 32] {
    // 1. Padding: append `0x80`, pad with zeros to a multiple of
    //    64 bytes minus 8, then append the big-endian bit length.
    let bit_len = u64::try_from(message.len())
        .ok()
        .and_then(|n| n.checked_mul(8))
        .unwrap_or(0);
    let mut buf = Vec::with_capacity(message.len() + 1 + 8);
    buf.extend_from_slice(message);
    buf.push(0x80);
    while buf.len() % 64 != 56 {
        buf.push(0x00);
    }
    buf.extend_from_slice(&bit_len.to_be_bytes());

    // 2. Initial hash value.
    let mut h = SHA256_H0;

    // 3. Process each 512-bit (64-byte) chunk.
    for chunk in buf.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, word) in chunk.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut hh = h[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(SHA256_K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);

            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    let mut out = [0u8; 32];
    for (i, &v) in h.iter().enumerate() {
        let bytes = v.to_be_bytes();
        out[i * 4] = bytes[0];
        out[i * 4 + 1] = bytes[1];
        out[i * 4 + 2] = bytes[2];
        out[i * 4 + 3] = bytes[3];
    }
    out
}

/// Returns the lowercase hex digit for a 4-bit value.
#[inline]
fn hex_digit(n: u8) -> char {
    match n & 0x0f {
        0..=9 => (b'0' + (n & 0x0f)) as char,
        10..=15 => (b'a' + ((n & 0x0f) - 10)) as char,
        _ => '0',
    }
}

/// Lowercase hex encoding of a byte slice.
fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(hex_digit(b >> 4));
        out.push(hex_digit(b & 0x0f));
    }
    out
}

/// Lowercase hex SHA-256 digest of `message`.
fn sha256_hex(message: &[u8]) -> String {
    hex_encode(&sha256(message))
}

/// Computes a stable map key for a [`Channel`]. Used by
/// [`RateLimitService`] in lieu of `Channel: Hash` (the
/// [`Channel`] enum does not currently derive `Hash` and the
/// spec lists `port.rs` under "DO NOT TOUCH"). The format is a
/// short, fixed-width discriminant that uniquely identifies the
/// channel variant and is stable across processes.
fn channel_key(channel: &Channel) -> String {
    match channel {
        Channel::Email { from, reply_to } => {
            let from = from.as_ref().map_or("-", EmailAddress::as_str);
            let reply = reply_to.as_ref().map_or("-", EmailAddress::as_str);
            format!("email:{from}:{reply}")
        }
        Channel::Sms { from, unicode } => {
            let from = from.as_ref().map_or("-", PhoneNumber::as_str);
            format!("sms:{from}:{unicode}")
        }
        Channel::Push { topic, ttl, collapse_key } => {
            let topic = topic.as_deref().unwrap_or("-");
            let collapse = collapse_key.as_deref().unwrap_or("-");
            let ttl_ms = ttl.map_or(0, |d| {
                u64::try_from(d.as_millis()).unwrap_or(u64::MAX)
            });
            format!("push:{topic}:{ttl_ms}:{collapse}")
        }
        Channel::InApp => "inapp".to_owned(),
        Channel::Chat { provider } => format!("chat:{}", provider.as_str()),
        Channel::Voice { voice_id, language } => {
            let voice = voice_id.as_deref().unwrap_or("-");
            format!("voice:{voice}:{}", language.as_str())
        }
        Channel::Webhook { url, secret } => {
            let signed = if secret.is_some() { "1" } else { "0" };
            format!("webhook:{signed}:{}", url.as_str())
        }
    }
}

// Use the typed accessors from port.rs so `channel_key` does not
// depend on field-private details.
use crate::port::{EmailAddress, PhoneNumber};

// ---------------------------------------------------------------------------
// 1. TemplateService
// ---------------------------------------------------------------------------

/// Substitutes `{var_name}` placeholders in template bodies,
/// validates that every variable referenced by a body has been
/// supplied, and extracts the variable names referenced by a body.
///
/// The variable-name grammar is `[A-Za-z_][A-Za-z0-9_]*`. Any
/// `{...}` sequence whose body does not match the grammar is left
/// untouched (this is conservative — it lets email subjects
/// contain stray `{` characters without breaking).
#[derive(Debug, Default, Clone, Copy)]
pub struct TemplateService;

impl TemplateService {
    /// Constructs a new `TemplateService`. The struct is a
    /// zero-sized marker; the constructor exists so the call site
    /// reads identically to the other services in this module.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Substitutes every `{var_name}` placeholder in `body` with
    /// the corresponding value from `variables`. Placeholders
    /// whose name is not present in `variables` are left as-is
    /// (the caller is expected to have run
    /// [`validate_required_variables`](Self::validate_required_variables)
    /// first).
    #[must_use]
    pub fn substitute_variables(
        body: &str,
        variables: &BTreeMap<String, String>,
    ) -> String {
        let mut out = String::with_capacity(body.len());
        let bytes = body.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'{' {
                if let Some((end, name)) = scan_variable(&bytes[i + 1..]) {
                    if let Some(value) = variables.get(name) {
                        out.push_str(value);
                        i += 1 + end;
                        continue;
                    }
                }
            }
            // Push the byte as a UTF-8 char boundary. The body is
            // required to be UTF-8 by the type system, so this
            // push is safe; the worst case is pushing a
            // continuation byte that immediately becomes the
            // start of the next placeholder scan (no false
            // positives because `{` is never a UTF-8 continuation
            // byte).
            let ch_start = i;
            // Advance to the next char boundary so multi-byte
            // sequences don't get split.
            let ch_end = next_char_boundary(body, ch_start);
            out.push_str(&body[ch_start..ch_end]);
            i = ch_end;
        }
        out
    }

    /// Validates that every `{var_name}` placeholder in `body`
    /// has a corresponding entry in `provided`. Returns the first
    /// missing variable wrapped in a
    /// [`NotificationError::MissingVariable`] error.
    pub fn validate_required_variables(
        body: &str,
        provided: &BTreeMap<String, String>,
    ) -> Result<(), NotificationError> {
        for name in Self::extract_variables(body) {
            if !provided.contains_key(&name) {
                return Err(NotificationError::MissingVariable(name));
            }
        }
        Ok(())
    }

    /// Extracts the unique `{var_name}` placeholder names from
    /// `body`. The result is in first-appearance order.
    #[must_use]
    pub fn extract_variables(body: &str) -> Vec<String> {
        let mut seen: Vec<String> = Vec::new();
        let bytes = body.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'{' {
                if let Some((end, name)) = scan_variable(&bytes[i + 1..]) {
                    if !seen.iter().any(|n| n == name) {
                        seen.push(name.to_owned());
                    }
                    i += 1 + end;
                    continue;
                }
            }
            i += 1;
        }
        seen
    }
}

/// Scans `bytes` (which starts immediately after the opening
/// `{`) for a closing `}` whose interior matches the variable
/// grammar `[A-Za-z_][A-Za-z0-9_]*`. Returns `(1 + name_len,
/// &name)` on success, where the `+1` accounts for the closing
/// `}` and `name` is the slice (no allocation).
fn scan_variable(bytes: &[u8]) -> Option<(usize, &str)> {
    let name_start = 0;
    let name_end = bytes
        .iter()
        .take_while(|&&b| b.is_ascii_alphanumeric() || b == b'_')
        .count();
    if name_end == 0 {
        return None;
    }
    let first = bytes[name_start];
    if !(first.is_ascii_alphabetic() || first == b'_') {
        return None;
    }
    if bytes.get(name_end) != Some(&b'}') {
        return None;
    }
    let name = std::str::from_utf8(&bytes[..name_end]).ok()?;
    Some((name_end + 1, name))
}

/// Returns the byte index of the next UTF-8 char boundary at or
/// after `i`. `i` must already be on a char boundary.
fn next_char_boundary(s: &str, i: usize) -> usize {
    let mut j = i + 1;
    while j < s.len() && !s.is_char_boundary(j) {
        j += 1;
    }
    j
}

// ---------------------------------------------------------------------------
// 2. ChannelService
// ---------------------------------------------------------------------------

/// Classifies a [`Channel`] (sync vs async, authentication
/// requirements) and computes the per-channel fan-out list.
#[derive(Debug, Default, Clone, Copy)]
pub struct ChannelService;

impl ChannelService {
    /// Constructs a new `ChannelService`. Zero-sized.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Returns `true` if the channel is delivered asynchronously
    /// (push, in-app, webhook). Email, SMS, voice, and chat are
    /// all treated as synchronous for purposes of this
    /// classification.
    #[must_use]
    pub fn is_async(channel: &Channel) -> bool {
        matches!(channel, Channel::Push { .. } | Channel::InApp | Channel::Webhook { .. })
    }

    /// Returns the list of channels to dispatch to for a single
    /// channel input. The notify port currently routes each
    /// request through exactly one channel, so this returns a
    /// single-element vector; the method exists so a future
    /// "multi-channel request" feature can reuse the same helper
    /// without changing adapter call sites.
    #[must_use]
    pub fn fan_out_targets(channel: &Channel) -> Vec<Channel> {
        vec![channel.clone()]
    }

    /// Returns `true` if the channel requires an authenticated
    /// send path. Email relies on DKIM / SPF; webhook delivery
    /// signs the body with an HMAC secret. Other channels are
    /// considered authenticated by transport (carrier-grade SMS
    /// gateways, push-provider device tokens, etc.).
    #[must_use]
    pub fn requires_authentication(channel: &Channel) -> bool {
        matches!(channel, Channel::Email { .. } | Channel::Webhook { .. })
    }
}

// ---------------------------------------------------------------------------
// 3. IdempotencyService
// ---------------------------------------------------------------------------

/// Derives a deterministic idempotency key from
/// `(command_id, recipient, template_version)` and tracks seen
/// keys so the adapter can detect replays.
///
/// The service holds no state of its own; the seen-key set is
/// passed in by the caller (typically a per-tenant
/// `HashMap<SchoolId, HashSet<String>>` held by the adapter).
#[derive(Debug, Default, Clone, Copy)]
pub struct IdempotencyService;

impl IdempotencyService {
    /// Constructs a new `IdempotencyService`. Zero-sized.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Derives a deterministic SHA-256 hex idempotency key from
    /// `command_id`, `recipient`, and `template_version`.
    ///
    /// The canonical input form is:
    ///
    /// ```text
    /// <command_id>:<recipient>:<template_version as decimal>
    /// ```
    #[must_use]
    pub fn derive_key(
        command_id: &str,
        recipient: &str,
        template_version: u32,
    ) -> String {
        let input = format!("{command_id}:{recipient}:{template_version}");
        sha256_hex(input.as_bytes())
    }

    /// Returns `true` if the key has been seen before (a replay);
    /// otherwise inserts the key and returns `false`.
    pub fn is_duplicate(key: &str, seen_keys: &mut HashSet<String>) -> bool {
        if seen_keys.contains(key) {
            true
        } else {
            seen_keys.insert(key.to_owned());
            false
        }
    }
}

// ---------------------------------------------------------------------------
// 4. RateLimitService
// ---------------------------------------------------------------------------

/// The state of a single channel's token bucket. Returned by
/// [`RateLimitService::current_state`] for observability and tests.
#[derive(Debug, Clone, Copy)]
pub struct RateState {
    /// The number of tokens currently available.
    pub tokens: u32,
    /// The maximum number of tokens the bucket can hold (== the
    /// configured `max_per_second`).
    pub max_tokens: u32,
    /// The wall-clock instant of the last refill.
    pub last_refill: Instant,
}

/// In-memory token-bucket rate limiter, one bucket per channel.
///
/// The bucket refills at one token per 1000 ms, capped at
/// `max_per_second`. The bucket starts full on first use. The
/// service is process-local; production deployments that span
/// multiple processes or pods should back the limiter with a
/// shared store.
#[derive(Debug, Default)]
pub struct RateLimitService {
    state: HashMap<String, RateState>,
}

impl RateLimitService {
    /// Constructs a new `RateLimitService` with no buckets.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Attempts to acquire one token for `channel` with the given
    /// `max_per_second`. Returns `true` if a token was available
    /// (and decrements the bucket); `false` if the bucket was
    /// empty after the refill.
    pub fn try_acquire(&mut self, channel: &Channel, max_per_second: u32) -> bool {
        let key = channel_key(channel);
        let now = Instant::now();
        let cap = max_per_second.max(1);
        let entry = self.state.entry(key).or_insert(RateState {
            tokens: cap,
            max_tokens: cap,
            last_refill: now,
        });

        // Refill: 1 token per 1000 ms, capped at `max_tokens`.
        let elapsed_ms = u64::try_from(now.duration_since(entry.last_refill).as_millis())
            .unwrap_or(u64::MAX);
        let new_tokens = u32::try_from(elapsed_ms / 1000).unwrap_or(u32::MAX);
        if new_tokens > 0 {
            entry.tokens = entry.tokens.saturating_add(new_tokens).min(entry.max_tokens);
            entry.last_refill = entry
                .last_refill
                .checked_add(Duration::from_millis(u64::from(new_tokens) * 1000))
                .unwrap_or(now);
        }

        if entry.tokens > 0 {
            entry.tokens -= 1;
            true
        } else {
            false
        }
    }

    /// Clears the bucket for `channel` (so the next
    /// [`try_acquire`](Self::try_acquire) starts fresh at
    /// `max_per_second`).
    pub fn reset(&mut self, channel: &Channel) {
        self.state.remove(&channel_key(channel));
    }

    /// Returns the current state of the bucket for `channel`,
    /// or `None` if no bucket has been created yet.
    #[must_use]
    pub fn current_state(&self, channel: &Channel) -> Option<RateState> {
        self.state.get(&channel_key(channel)).copied()
    }
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

    fn s<S: Into<String>>(s: S) -> String {
        s.into()
    }

    #[test]
    fn test_template_service_substitute_variables() {
        let mut vars = BTreeMap::new();
        vars.insert(s("name"), s("Alice"));
        vars.insert(s("grade"), s("5A"));
        let body = "Hello {name}, welcome to {grade}!";
        let out = TemplateService::substitute_variables(body, &vars);
        assert_eq!(out, "Hello Alice, welcome to 5A!");
    }

    #[test]
    fn test_template_service_validate_required_variables_missing() {
        let mut provided = BTreeMap::new();
        provided.insert(s("name"), s("Alice"));
        let body = "Hello {name}, your grade is {grade}.";
        let err = TemplateService::validate_required_variables(body, &provided)
            .expect_err("missing {grade} should error");
        match err {
            NotificationError::MissingVariable(name) => assert_eq!(name, "grade"),
            other => panic!("expected MissingVariable, got {other:?}"),
        }
    }

    #[test]
    fn test_channel_service_is_async() {
        let email = Channel::Email {
            from: None,
            reply_to: None,
        };
        assert!(!ChannelService::is_async(&email));

        let sms = Channel::Sms {
            from: None,
            unicode: false,
        };
        assert!(!ChannelService::is_async(&sms));

        let push = Channel::Push {
            topic: None,
            ttl: None,
            collapse_key: None,
        };
        assert!(ChannelService::is_async(&push));

        assert!(ChannelService::is_async(&Channel::InApp));

        let webhook = Channel::Webhook {
            url: crate::port::Url::new("https://example.test/hook"),
            secret: None,
        };
        assert!(ChannelService::is_async(&webhook));

        let voice = Channel::Voice {
            voice_id: None,
            language: crate::port::LanguageTag::default(),
        };
        assert!(!ChannelService::is_async(&voice));

        let chat = Channel::Chat {
            provider: crate::port::ChatProvider::Telegram,
        };
        assert!(!ChannelService::is_async(&chat));

        // Authentication classification.
        assert!(ChannelService::requires_authentication(&email));
        assert!(ChannelService::requires_authentication(&webhook));
        assert!(!ChannelService::requires_authentication(&sms));
    }

    #[test]
    fn test_idempotency_service_derive_key_is_deterministic() {
        let a = IdempotencyService::derive_key("cmd_42", "alice@example.test", 7);
        let b = IdempotencyService::derive_key("cmd_42", "alice@example.test", 7);
        assert_eq!(a, b);
        // 64 lowercase hex characters = 32-byte SHA-256.
        assert_eq!(a.len(), 64);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(a.chars().all(|c| !c.is_ascii_uppercase()));

        // Different inputs must produce different keys.
        let c = IdempotencyService::derive_key("cmd_42", "bob@example.test", 7);
        let d = IdempotencyService::derive_key("cmd_42", "alice@example.test", 8);
        assert_ne!(a, c);
        assert_ne!(a, d);

        // Replay detection.
        let mut seen: HashSet<String> = HashSet::new();
        assert!(!IdempotencyService::is_duplicate(&a, &mut seen));
        assert!(IdempotencyService::is_duplicate(&a, &mut seen));
    }

    #[test]
    fn test_rate_limit_service_token_bucket() {
        let mut svc = RateLimitService::new();
        let email = Channel::Email {
            from: None,
            reply_to: None,
        };

        // Bucket starts full at `max_per_second = 5`.
        for i in 0..5 {
            assert!(
                svc.try_acquire(&email, 5),
                "acquire #{i} of 5 should succeed"
            );
        }
        // Sixth acquire in the same millisecond fails (no time
        // for the bucket to refill).
        assert!(!svc.try_acquire(&email, 5));

        // Reset clears the bucket.
        svc.reset(&email);
        assert!(svc.try_acquire(&email, 5));

        // current_state reflects the bucket.
        let state = svc
            .current_state(&email)
            .expect("bucket exists after acquire");
        assert_eq!(state.max_tokens, 5);
        assert!(state.tokens <= 5);
    }
}


