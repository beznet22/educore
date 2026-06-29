//! The PII / secret redactor port.
//!
//! The [`Redactor`] trait is the engine's port for stripping
//! sensitive values out of strings before they are persisted to the
//! audit log, emitted in events, or written to operator logs. The
//! [`DefaultRedactor`] implementation is a regex- and
//! keyword-driven redactor that handles the engine's baseline set
//! of sensitive value categories (passwords, API secrets, bearer
//! tokens, email addresses, phone numbers, and JWT-shaped blobs).
//!
//! Wiring into [`crate::writer::AuditWriter`] lands in a later
//! phase; this module ships only the port and the default
//! implementation so the call sites in command handlers and
//! background jobs can be migrated incrementally.
//!
//! ## Design notes
//!
//! - The trait is **object-safe** (no generics, no `Self` in
//!   return position) so consumers can hold it as
//!   `Arc<dyn Redactor>` and swap implementations (default,
//!   tenant-specific, external PII-detection service) without
//!   rewriting call sites.
//! - All fallible initialization (regex compilation) is funneled
//!   through a [`OnceLock<Option<Regex>>`] helper. If a regex
//!   pattern fails to compile — a programmer error in the
//!   constant, not a runtime input error — the helper logs a
//!   `tracing::error!` and disables redaction for that kind
//!   rather than panicking. The engine's no-panic / no-`unwrap` /
//!   no-`expect` invariant (per `docs/code-standards.md`) is
//!   preserved.
//! - Each redaction rule is a [`regex::Regex`] matched against the
//!   full string; matched spans are replaced with a redacted
//!   placeholder (typically `"***"`) while the surrounding
//!   structure (e.g. the `password=` keyword, the email's domain
//!   part, the phone's last four digits) is preserved so the
//!   redacted text remains useful for triage without leaking the
//!   original value.
//!
//! ## Examples
//!
//! ```text
//! use educore_audit::redactor::{DefaultRedactor, Redactor};
//!
//! let r = DefaultRedactor::new();
//! assert_eq!(r.redact("password=hunter2"), "password=***");
//! assert_eq!(r.redact("user: alice@example.com"), "user: ***@example.com");
//! ```

use std::sync::OnceLock;

use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};

/// The kind of redaction to apply.
///
/// Each variant maps to a dedicated redaction rule (or set of
/// rules) on the [`DefaultRedactor`]. The kind is the public
/// handle consumers use when they only want to redact one
/// category — for example, an email-only gateway that wants to
/// scrub `@`-addresses but leave everything else alone should
/// call [`Redactor::redact_kind`] with
/// [`RedactionKind::Email`].
///
/// The variants are `#[non_exhaustive]` so the engine can add
/// new kinds (e.g. `IpAddress`, `CreditCard`) in later phases
/// without a breaking change. The [`RedactionKind::ALL`] slice
/// is the canonical list of built-in kinds as of this writing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum RedactionKind {
    /// A password value (`password=...`, `pwd: ...`).
    Password,
    /// A generic secret (`secret=...`, `client_secret=...`,
    /// `api_secret=...`).
    Secret,
    /// A bearer / access / API token, including `Bearer xxx` and
    /// `Basic xxx` prefixes.
    Token,
    /// An email address (`local@domain.tld`).
    Email,
    /// A phone number in international or domestic format.
    Phone,
    /// A JWT-shaped string (three base64url segments separated
    /// by dots).
    Jwt,
}

impl RedactionKind {
    /// The canonical built-in redaction kinds, in the order they
    /// are applied by [`DefaultRedactor::redact`]. The order is
    /// significant: password and secret rules are applied first
    /// so their keyword prefixes do not collide with the token
    /// rule's `token=` keyword.
    pub const ALL: &'static [Self] = &[
        Self::Password,
        Self::Secret,
        Self::Token,
        Self::Email,
        Self::Phone,
        Self::Jwt,
    ];

    /// Returns the canonical snake_case wire string for the
    /// redaction kind. Stable across versions; safe to use in
    /// log keys, metric labels, and event metadata.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Password => "password",
            Self::Secret => "secret",
            Self::Token => "token",
            Self::Email => "email",
            Self::Phone => "phone",
            Self::Jwt => "jwt",
        }
    }
}

/// The redactor port trait.
///
/// Object-safe: every method takes `&self` and returns
/// `String` (or a borrowed slice tied to `self`). Implementors
/// can be stored as `Arc<dyn Redactor>` and shared across
/// command handlers, background jobs, and HTTP middleware
/// without generic-type plumbing.
///
/// All implementations must be `Send + Sync` because redactors
/// are shared state.
pub trait Redactor: Send + Sync {
    /// Apply **all** of this redactor's rules in a fixed order
    /// and return the redacted string. The input is consumed
    /// immutably; the output is always a freshly allocated
    /// `String` (callers may freely mutate or re-redact it).
    fn redact(&self, value: &str) -> String;

    /// Apply **only** the redactions for the specified kind.
    /// Use this when a call site has a narrow redaction need
    /// (e.g. a webhook signature only wants to scrub JWTs from
    /// the payload, not email addresses).
    fn redact_kind(&self, kind: RedactionKind, value: &str) -> String;

    /// Returns the list of [`RedactionKind`] variants this
    /// redactor knows how to handle. Used by introspection in
    /// health-check endpoints and by tests that assert the
    /// redactor's coverage matrix.
    fn kinds(&self) -> &[RedactionKind];
}

/// The engine's default redactor: regex- and keyword-driven
/// redaction across all [`RedactionKind`] variants.
///
/// Zero-sized — there are no per-instance knobs in the engine
/// baseline. Consumers that need tenant-specific rules can
/// implement [`Redactor`] directly and wrap the default with
/// their own delegating implementation.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefaultRedactor;

impl DefaultRedactor {
    /// Constructs a new `DefaultRedactor`. Equivalent to
    /// `Default::default()` but `const`-callable so the
    /// redactor can be embedded in a `static` configuration.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Redactor for DefaultRedactor {
    fn redact(&self, value: &str) -> String {
        // Apply each kind in the canonical order. We re-borrow
        // `self` and `value` for each pass so the type signature
        // stays `&str -> String` without intermediate state.
        let mut out = value.to_owned();
        for kind in RedactionKind::ALL {
            out = self.redact_kind(*kind, &out);
        }
        out
    }

    fn redact_kind(&self, kind: RedactionKind, value: &str) -> String {
        match kind {
            RedactionKind::Password => redact_password(value),
            RedactionKind::Secret => redact_secret(value),
            RedactionKind::Token => redact_token(value),
            RedactionKind::Email => redact_email(value),
            RedactionKind::Phone => redact_phone(value),
            RedactionKind::Jwt => redact_jwt(value),
        }
    }

    fn kinds(&self) -> &[RedactionKind] {
        RedactionKind::ALL
    }
}

// ============================================================================
// Per-kind helpers
// ============================================================================
//
// Each helper follows the same shape:
//   1. Look up the compiled regex from a `OnceLock`-backed cache.
//   2. If the regex is unavailable (compile failed at startup),
//      return the input unchanged and let the upstream tracing
//      log carry the diagnostic. No panic, no `unwrap`, no
//      `expect`.
//   3. Otherwise apply `replace_all` with a closure that
//      preserves the structural prefix (keyword, scheme, last-4
//      digits) and substitutes the value with `***`.
//
// The closure uses `caps.get(N).map_or(default, |m| m.as_str())`
// rather than `unwrap_or` / `unwrap()` — every capture group is
// either a documented structural prefix or a non-optional
// matched value, so the `map_or` default is purely defensive.

/// Pattern for password-style `key=value` pairs.
///
/// Group 1: the keyword (`password`, `passwd`, or `pwd`).
/// Group 2: the separator (`:` or `=`).
/// Group 3: the whitespace between the separator and the value (preserved in the redacted form so `password: foo` redacts to `password: ***` and not `password:***`).
/// Group 4: the value (everything up to whitespace, quote, or `&`).
const PASSWORD_PATTERN: &str = r#"(?i)\b(password|passwd|pwd)\s*([:=])(\s*)["']?([^"'\s,&]+)["']?"#;

fn redact_password(s: &str) -> String {
    match password_regex() {
        Some(re) => re
            .replace_all(s, |caps: &Captures<'_>| {
                let key = caps.get(1).map_or("password", |m| m.as_str());
                let sep = caps.get(2).map_or("=", |m| m.as_str());
                let ws = caps.get(3).map_or("", |m| m.as_str());
                format!("{key}{sep}{ws}***")
            })
            .into_owned(),
        None => s.to_owned(),
    }
}

fn password_regex() -> Option<&'static Regex> {
    static CACHE: OnceLock<Option<Regex>> = OnceLock::new();
    CACHE
        .get_or_init(|| compile_or_log("password", PASSWORD_PATTERN))
        .as_ref()
}

/// Pattern for generic secret-style `key=value` pairs.
///
/// Keywords: `secret`, `client_secret`, `api_secret`,
/// `app_secret`, `secret_key`. Group 1 is the keyword, group 2
/// the separator, group 3 the whitespace between the separator
/// and the value, group 4 the value itself.
const SECRET_PATTERN: &str = r#"(?i)\b(secret|client_secret|api_secret|app_secret|secret_key)\s*([:=])(\s*)["']?([^"'\s,&]+)["']?"#;

fn redact_secret(s: &str) -> String {
    match secret_regex() {
        Some(re) => re
            .replace_all(s, |caps: &Captures<'_>| {
                let key = caps.get(1).map_or("secret", |m| m.as_str());
                let sep = caps.get(2).map_or("=", |m| m.as_str());
                let ws = caps.get(3).map_or("", |m| m.as_str());
                format!("{key}{sep}{ws}***")
            })
            .into_owned(),
        None => s.to_owned(),
    }
}

fn secret_regex() -> Option<&'static Regex> {
    static CACHE: OnceLock<Option<Regex>> = OnceLock::new();
    CACHE
        .get_or_init(|| compile_or_log("secret", SECRET_PATTERN))
        .as_ref()
}

/// Pattern for token-style `key=value` pairs.
///
/// Keywords: `token`, `access_token`, `auth_token`, `api_token`,
/// `refresh_token`, `id_token`. Group 1 is the keyword, group 2
/// the separator (optional because `Bearer xxx` has no
/// separator), group 3 the whitespace between the separator and
/// the value, group 4 the value itself.
const TOKEN_KEYWORD_PATTERN: &str = r#"(?i)\b(token|access_token|auth_token|api_token|refresh_token|id_token)\s*([:=]?)(\s*)["']?([^"'\s,&]+)["']?"#;

/// Pattern for `Bearer xxx` and `Basic xxx` prefixes.
///
/// Group 1 is the prefix (including the trailing space). The
/// trailing token is replaced wholesale with `***`.
const BEARER_PATTERN: &str = r"(?i)(Bearer\s+|Basic\s+)[A-Za-z0-9._\-+/=]+";

fn redact_token(s: &str) -> String {
    let mut out = s.to_owned();
    if let Some(re) = token_keyword_regex() {
        out = re
            .replace_all(&out, |caps: &Captures<'_>| {
                let key = caps.get(1).map_or("token", |m| m.as_str());
                let sep_raw = caps.get(2).map_or("", |m| m.as_str());
                let ws = caps.get(3).map_or("", |m| m.as_str());
                // When the regex matches a bare keyword (no separator),
                // the matched prefix is `token` itself; preserve it as
                // `token=***` so the structure stays readable.
                let sep = if sep_raw.is_empty() { "=" } else { sep_raw };
                format!("{key}{sep}{ws}***")
            })
            .into_owned();
    }
    if let Some(re) = bearer_regex() {
        out = re
            .replace_all(&out, |caps: &Captures<'_>| {
                let prefix = caps.get(1).map_or("", |m| m.as_str());
                format!("{prefix}***")
            })
            .into_owned();
    }
    out
}

fn token_keyword_regex() -> Option<&'static Regex> {
    static CACHE: OnceLock<Option<Regex>> = OnceLock::new();
    CACHE
        .get_or_init(|| compile_or_log("token-keyword", TOKEN_KEYWORD_PATTERN))
        .as_ref()
}

fn bearer_regex() -> Option<&'static Regex> {
    static CACHE: OnceLock<Option<Regex>> = OnceLock::new();
    CACHE
        .get_or_init(|| compile_or_log("bearer", BEARER_PATTERN))
        .as_ref()
}

/// Pattern for email addresses.
///
/// Group 1 is the domain part (after the `@`), preserved so the
/// redacted address (`***@example.com`) still tells the operator
/// which domain leaked.
const EMAIL_PATTERN: &str = r"\b([A-Za-z0-9._%+\-]+)@([A-Za-z0-9.\-]+\.[A-Za-z]{2,})\b";

fn redact_email(s: &str) -> String {
    match email_regex() {
        Some(re) => re
            .replace_all(s, |caps: &Captures<'_>| {
                let domain = caps.get(2).map_or("", |m| m.as_str());
                format!("***@{domain}")
            })
            .into_owned(),
        None => s.to_owned(),
    }
}

fn email_regex() -> Option<&'static Regex> {
    static CACHE: OnceLock<Option<Regex>> = OnceLock::new();
    CACHE
        .get_or_init(|| compile_or_log("email", EMAIL_PATTERN))
        .as_ref()
}

/// Pattern for phone numbers (international or domestic).
///
/// Captures the prefix (country code + area code + trunk) in
/// group 1 and the last four digits in group 2, so the redacted
/// form `***-***-1234` preserves enough context for triage
/// without leaking the full number.
///
/// The pattern allows an optional leading `(`, the international
/// `+`, and any of `space`, `-`, `.`, `+`, `(`, `)` as inner
/// separators. The body is 6–18 of those separator chars between
/// a leading digit and the trailing 4 digits — wide enough to
/// match common formats (US 10-digit, Indian 10-digit, European
/// with parens) without dragging in unrelated digit runs.
const PHONE_PATTERN: &str = r"([(]?\+?\d[\d\s.+\-()]{6,18})(\d{4})\b";

fn redact_phone(s: &str) -> String {
    match phone_regex() {
        Some(re) => re
            .replace_all(s, |caps: &Captures<'_>| {
                let _prefix = caps.get(1).map_or("", |m| m.as_str());
                let last_four = caps.get(2).map_or("", |m| m.as_str());
                // The prefix is intentionally discarded to avoid
                // leaking the area code; the last four digits give
                // just enough context for call-back correlation.
                format!("***-***-{last_four}")
            })
            .into_owned(),
        None => s.to_owned(),
    }
}

fn phone_regex() -> Option<&'static Regex> {
    static CACHE: OnceLock<Option<Regex>> = OnceLock::new();
    CACHE
        .get_or_init(|| compile_or_log("phone", PHONE_PATTERN))
        .as_ref()
}

/// Pattern for JWT-shaped strings: three base64url segments
/// separated by dots, each at least 8 characters long. The
/// 8-character floor avoids matching short dotted strings like
/// `a.b.c` that are clearly not JWTs.
const JWT_PATTERN: &str = r"\b[A-Za-z0-9_\-]{8,}\.[A-Za-z0-9_\-]{8,}\.[A-Za-z0-9_\-]{8,}\b";

fn redact_jwt(s: &str) -> String {
    match jwt_regex() {
        Some(re) => re.replace_all(s, "***").into_owned(),
        None => s.to_owned(),
    }
}

fn jwt_regex() -> Option<&'static Regex> {
    static CACHE: OnceLock<Option<Regex>> = OnceLock::new();
    CACHE
        .get_or_init(|| compile_or_log("jwt", JWT_PATTERN))
        .as_ref()
}

// ============================================================================
// Regex compilation cache
// ============================================================================

/// Compiles the given regex pattern and returns `Some(Regex)` on
/// success or `None` on failure (logging a `tracing::error!`).
///
/// This helper does NOT cache the compiled regex itself — each
/// caller (per-kind helper) holds its own `OnceLock<Option<Regex>>`
/// so the static caches stay per-kind and never bleed patterns
/// into the wrong rule.
fn compile_or_log(name: &'static str, pattern: &'static str) -> Option<Regex> {
    match Regex::new(pattern) {
        Ok(re) => Some(re),
        Err(err) => {
            tracing::error!(
                redactor = name,
                pattern = pattern,
                error = %err,
                "redactor regex pattern failed to compile; \
                 redaction rule is disabled for this process"
            );
            None
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::*;

    fn redactor() -> DefaultRedactor {
        DefaultRedactor::new()
    }

    #[test]
    fn password_redaction_replaces_value_with_stars() {
        // The keyword and separator are preserved; the value is
        // collapsed to `***`.
        assert_eq!(
            redactor().redact_kind(RedactionKind::Password, "password=hunter2"),
            "password=***"
        );
        // Space-separated `key: value` form.
        assert_eq!(
            redactor().redact_kind(RedactionKind::Password, "password: hunter2"),
            "password: ***"
        );
        // Aliases `passwd` and `pwd` work too.
        assert_eq!(
            redactor().redact_kind(RedactionKind::Password, "passwd=foo"),
            "passwd=***"
        );
        assert_eq!(
            redactor().redact_kind(RedactionKind::Password, "pwd: bar"),
            "pwd: ***"
        );
        // Case-insensitive.
        assert_eq!(
            redactor().redact_kind(RedactionKind::Password, "PASSWORD=foo"),
            "PASSWORD=***"
        );
        // Non-password strings pass through unchanged.
        assert_eq!(
            redactor().redact_kind(RedactionKind::Password, "hello world"),
            "hello world"
        );
        // `passwordless=true` must NOT match (no separator after
        // `password`).
        assert_eq!(
            redactor().redact_kind(RedactionKind::Password, "passwordless=true"),
            "passwordless=true"
        );
        // Quoted value (key=value URL-encoded style).
        assert_eq!(
            redactor().redact_kind(RedactionKind::Password, "password=\"hunter2\""),
            "password=***"
        );
    }

    #[test]
    fn secret_and_token_redaction_replace_value_with_stars() {
        // Secret keywords (generic).
        assert_eq!(
            redactor().redact_kind(RedactionKind::Secret, "client_secret=abc123"),
            "client_secret=***"
        );
        assert_eq!(
            redactor().redact_kind(RedactionKind::Secret, "api_secret: xyz"),
            "api_secret: ***"
        );

        // Token keywords.
        assert_eq!(
            redactor().redact_kind(RedactionKind::Token, "access_token=tk_abc"),
            "access_token=***"
        );
        assert_eq!(
            redactor().redact_kind(RedactionKind::Token, "refresh_token=tk_xyz"),
            "refresh_token=***"
        );

        // Bearer prefix — the prefix is preserved, the token is
        // collapsed.
        assert_eq!(
            redactor().redact_kind(
                RedactionKind::Token,
                "Authorization: Bearer eyJabc123def456"
            ),
            "Authorization: Bearer ***"
        );
        // Basic prefix.
        assert_eq!(
            redactor().redact_kind(RedactionKind::Token, "Authorization: Basic dXNlcjpwYXNz"),
            "Authorization: Basic ***"
        );
    }

    #[test]
    fn email_redaction_preserves_domain_and_redacts_local_part() {
        // The local part is collapsed to `***`; the domain is
        // preserved so the operator still sees which domain
        // leaked.
        assert_eq!(
            redactor().redact_kind(RedactionKind::Email, "alice@example.com"),
            "***@example.com"
        );
        // Embedded in a longer string.
        assert_eq!(
            redactor().redact_kind(
                RedactionKind::Email,
                "user=alice@example.com;admin=bob@school.org"
            ),
            "user=***@example.com;admin=***@school.org"
        );
        // Non-email strings pass through unchanged.
        assert_eq!(
            redactor().redact_kind(RedactionKind::Email, "no at-sign here"),
            "no at-sign here"
        );
    }

    #[test]
    fn phone_redaction_preserves_last_four_digits() {
        // US domestic format.
        assert_eq!(
            redactor().redact_kind(RedactionKind::Phone, "call 555-123-4567 now"),
            "call ***-***-4567 now"
        );
        // Parenthesized area code.
        assert_eq!(
            redactor().redact_kind(RedactionKind::Phone, "(555) 123-4567"),
            "***-***-4567"
        );
        // International with country code.
        assert_eq!(
            redactor().redact_kind(RedactionKind::Phone, "+1-555-123-4567"),
            "***-***-4567"
        );
        // Indian mobile format (10-digit local number). The
        // last 4 digits of `9876543210` are `3210`.
        assert_eq!(
            redactor().redact_kind(RedactionKind::Phone, "+91 98765 43210"),
            "***-***-3210"
        );
        // Non-phone strings pass through unchanged.
        assert_eq!(
            redactor().redact_kind(RedactionKind::Phone, "hello world"),
            "hello world"
        );
    }

    #[test]
    fn jwt_redaction_replaces_three_segment_base64_with_stars() {
        // A realistic-looking JWT (header.payload.signature).
        let jwt = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        assert_eq!(redactor().redact_kind(RedactionKind::Jwt, jwt), "***");
        // JWT embedded in a longer string.
        assert_eq!(
            redactor().redact_kind(
                RedactionKind::Jwt,
                "Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c"
            ),
            "Authorization: Bearer ***"
        );
        // Short dotted strings (less than 8 chars per segment)
        // do NOT match — they're too short to be JWTs.
        assert_eq!(redactor().redact_kind(RedactionKind::Jwt, "a.b.c"), "a.b.c");
    }

    #[test]
    fn default_redactor_applies_all_kinds_in_one_pass() {
        // A realistic log line containing multiple sensitive
        // categories. The redactor applies each kind in turn and
        // returns a fully scrubbed string.
        let line =
            r#"login attempt: user=alice@example.com pwd=hunter2 token=tk_abc ip=+1-555-123-4567"#;
        let out = redactor().redact(line);
        // Email local part is redacted; domain preserved.
        assert!(out.contains("***@example.com"), "out = {out}");
        // Password value collapsed.
        assert!(out.contains("pwd=***"), "out = {out}");
        // Token keyword value collapsed.
        assert!(out.contains("token=***"), "out = {out}");
        // Phone number preserved only as last-4.
        assert!(out.contains("***-***-4567"), "out = {out}");
        // The word "login" and "attempt" pass through.
        assert!(out.starts_with("login attempt"), "out = {out}");
    }

    #[test]
    fn redactor_kinds_lists_all_six_builtin_kinds() {
        // Sanity check: `DefaultRedactor` advertises the full
        // set of built-in kinds. Future phases that add new
        // kinds MUST update this assertion (and the
        // `RedactionKind::ALL` slice) in lockstep.
        let r = redactor();
        let kinds = r.kinds();
        assert_eq!(kinds.len(), RedactionKind::ALL.len());
        assert!(kinds.contains(&RedactionKind::Password));
        assert!(kinds.contains(&RedactionKind::Secret));
        assert!(kinds.contains(&RedactionKind::Token));
        assert!(kinds.contains(&RedactionKind::Email));
        assert!(kinds.contains(&RedactionKind::Phone));
        assert!(kinds.contains(&RedactionKind::Jwt));
    }

    #[test]
    fn redaction_kind_as_str_is_stable_snake_case() {
        // Wire strings are part of the public API (they appear
        // in log keys, metric labels, and event metadata). They
        // MUST be stable across versions.
        assert_eq!(RedactionKind::Password.as_str(), "password");
        assert_eq!(RedactionKind::Secret.as_str(), "secret");
        assert_eq!(RedactionKind::Token.as_str(), "token");
        assert_eq!(RedactionKind::Email.as_str(), "email");
        assert_eq!(RedactionKind::Phone.as_str(), "phone");
        assert_eq!(RedactionKind::Jwt.as_str(), "jwt");
    }
}
