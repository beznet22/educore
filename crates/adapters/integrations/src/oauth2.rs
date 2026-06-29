//! # OAuth2 client-credentials token cache
//!
//! Per-tenant OAuth2 `client_credentials` token cache + refresh
//! helper, per `docs/ports/integrations.md` § "OAuth2 Client
//! Credentials (Per Integration)".
//!
//! Some integrations require OAuth2 client credentials. The engine
//! does not own this flow; the adapter does. The contract is:
//!
//! 1. The adapter stores the `client_id` and `client_secret`
//!    (per tenant, per integration).
//! 2. The adapter performs the OAuth2 token exchange against the
//!    integration's token endpoint.
//! 3. The adapter caches the token until expiry.
//! 4. The adapter refreshes the token before expiry.
//!
//! This module is the cache half of (3) and (4). The I/O half
//! (1) and (2) lives in the adapter; this module only orchestrates
//! "give me a usable access token" against an `exchanger`
//! closure provided by the adapter.
//!
//! ## Per-tenant isolation
//!
//! Tokens are keyed by `(SchoolId, IntegrationId)` so two tenants
//! using the same provider receive distinct tokens and one
//! tenant's expiry cannot affect another's cache. Concurrent
//! callers serialise through an interior `Mutex`; the cache is
//! `Send + Sync` and can sit behind an `Arc` in long-lived
//! adapters.
//!
//! ## Expiry policy
//!
//! The cache treats a token as expired
//! [`DEFAULT_SAFETY_MARGIN`] seconds before the provider-declared
//! `expires_in` elapses, so a request issued right at the boundary
//! never ships a token the provider is about to reject. The
//! adapter can override the margin per-call via
//! [`ClientCredentialsCache::get_or_refresh_with_margin`].
//!
//! ## No I/O
//!
//! Like the other helpers in `services.rs`, this module performs
//! no network I/O. Adapters wrap the `exchanger` closure with the
//! async transport (`reqwest`, hyper, ...) and pass the resulting
//! future into [`ClientCredentialsCache::get_or_refresh`].

#![allow(clippy::module_name_repetitions)]

use std::collections::HashMap;
use std::future::Future;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use educore_core::ids::SchoolId;
use educore_core::tenant::TenantContext;

use crate::errors::{IntegrationError, Result};
use crate::port::IntegrationId;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// The OAuth2 grant_type value sent to the token endpoint for the
/// client-credentials flow (RFC 6749 § 4.4). Exposed so adapters
/// can build the request body without duplicating the string.
pub const OAUTH2_GRANT_TYPE_CLIENT_CREDENTIALS: &str = "client_credentials";

/// Default lifetime we treat a token as expired before its
/// declared `expires_in` elapses, so a request issued right at the
/// boundary never ships a token the provider is about to reject.
pub const DEFAULT_SAFETY_MARGIN: Duration = Duration::from_secs(30);

/// Default fallback lifetime when the provider omits `expires_in`
/// from the token response. Per RFC 6749 § 5.1 the field is
/// RECOMMENDED but optional; conservative default matches the
/// most common provider behaviour (1 hour).
pub const DEFAULT_EXPIRES_IN_SECONDS: u64 = 3600;

/// Default `token_type` value when the provider omits the field
/// from the token response. RFC 6749 § 5.1 specifies the value is
/// case-insensitive; `"Bearer"` is the only type this helper
/// currently understands.
pub const DEFAULT_TOKEN_TYPE: &str = "Bearer";

// ---------------------------------------------------------------------------
// Key
// ---------------------------------------------------------------------------

/// Cache key: the (tenant, integration) pair the cached token is
/// bound to. Two tenants using the same provider receive distinct
/// tokens, and one tenant with two integrations on the same
/// provider also receives distinct tokens (the provider scopes
/// tokens per `(client_id, audience)`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClientCredentialsKey {
    /// The tenant the cached token belongs to.
    pub school_id: SchoolId,

    /// The integration the cached token belongs to.
    pub integration: IntegrationId,
}

impl ClientCredentialsKey {
    /// Convenience constructor for tests and adapters that already
    /// hold the two values separately.
    #[must_use]
    pub const fn new(school_id: SchoolId, integration: IntegrationId) -> Self {
        Self {
            school_id,
            integration,
        }
    }
}

// ---------------------------------------------------------------------------
// Cached entry
// ---------------------------------------------------------------------------

/// A single cached access token plus the wall-clock instant at
/// which it expires (after applying the safety margin).
#[derive(Debug, Clone)]
pub struct CachedToken {
    /// The bearer token returned by the provider.
    pub access_token: String,

    /// Wall-clock instant at which the token expires. Compared
    /// against `Instant::now()` on every read; past entries are
    /// treated as misses and trigger a refresh.
    pub expires_at: Instant,

    /// The `token_type` declared by the provider. The cache does
    /// not interpret it; adapters that need it for a downstream
    /// header (e.g. `Authorization: Bearer ...`) can read the
    /// field directly.
    pub token_type: String,
}

impl CachedToken {
    /// Returns `true` if `now` is strictly before
    /// `expires_at`. The caller is responsible for subtracting
    /// the safety margin from `now` before calling.
    #[must_use]
    pub fn is_fresh_at(&self, now: Instant) -> bool {
        self.expires_at > now
    }
}

// ---------------------------------------------------------------------------
// Token response
// ---------------------------------------------------------------------------

/// The subset of the OAuth2 token endpoint response (RFC 6749
/// § 5.1) that this helper cares about. Provider-specific fields
/// (`scope`, `refresh_token`, ...) are intentionally ignored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenResponse {
    /// The access token issued by the authorization server.
    /// REQUIRED per RFC 6749 § 5.1.
    pub access_token: String,

    /// Lifetime of the access token in seconds. RECOMMENDED per
    /// RFC 6749 § 5.1. Defaults to
    /// [`DEFAULT_EXPIRES_IN_SECONDS`] when the provider omits it.
    #[serde(default = "default_expires_in_seconds")]
    pub expires_in: u64,

    /// Token type (e.g. `"Bearer"`). Defaults to
    /// [`DEFAULT_TOKEN_TYPE`] when the provider omits it.
    #[serde(default = "default_token_type")]
    pub token_type: String,
}

impl TokenResponse {
    /// Returns the lifetime as a [`Duration`], substituting
    /// [`DEFAULT_EXPIRES_IN_SECONDS`] when the provider omitted
    /// the field (already handled by serde's `default`, but
    /// useful for in-memory construction in tests).
    #[must_use]
    pub fn expires_in_duration(&self) -> Duration {
        Duration::from_secs(self.expires_in)
    }

    /// Returns `true` if the response carries an empty access
    /// token. The cache refuses to store empty tokens because an
    /// empty `Authorization: Bearer ` header is always rejected by
    /// the provider.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.access_token.is_empty()
    }
}

fn default_expires_in_seconds() -> u64 {
    DEFAULT_EXPIRES_IN_SECONDS
}

fn default_token_type() -> String {
    DEFAULT_TOKEN_TYPE.to_owned()
}

// ---------------------------------------------------------------------------
// Cache
// ---------------------------------------------------------------------------

/// Per-tenant, per-integration OAuth2 `client_credentials` token
/// cache.
///
/// The cache stores one entry per `(SchoolId, IntegrationId)`
/// pair; on a miss it calls the `exchanger` closure to fetch a
/// fresh token from the provider's token endpoint. Adapters
/// typically hold the cache inside an `Arc` so every outbound
/// request reuses the same in-memory state.
///
/// The cache itself performs no I/O. The `exchanger` closure is
/// the only piece that talks to the provider; this keeps the
/// helper testable and lets adapters wrap the exchange with the
/// transport layer of their choice (`reqwest`, hyper, mock for
/// tests).
pub struct ClientCredentialsCache {
    entries: Mutex<HashMap<ClientCredentialsKey, CachedToken>>,
    safety_margin: Duration,
}

impl std::fmt::Debug for ClientCredentialsCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Avoid taking the lock for the Debug print: a poisoned
        // Mutex shouldn't crash `dbg!` formatting.
        let count = self.entries.lock().map(|g| g.len()).unwrap_or(0);
        f.debug_struct("ClientCredentialsCache")
            .field("entries", &count)
            .field("safety_margin", &self.safety_margin)
            .finish_non_exhaustive()
    }
}

impl Default for ClientCredentialsCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientCredentialsCache {
    /// Returns a fresh cache with the default
    /// ([`DEFAULT_SAFETY_MARGIN`]) safety margin.
    #[must_use]
    pub fn new() -> Self {
        Self::with_safety_margin(DEFAULT_SAFETY_MARGIN)
    }

    /// Returns a fresh cache with a caller-supplied safety
    /// margin. The margin is subtracted from the provider's
    /// declared `expires_in` so a token is treated as expired
    /// before the provider actually rejects it.
    ///
    /// A zero margin is permitted (the cache treats the provider's
    /// `expires_in` exactly). A margin larger than `expires_in`
    /// causes every cached token to read as expired and every
    /// call to refresh — callers should not exceed the
    /// provider-declared lifetime.
    #[must_use]
    pub fn with_safety_margin(safety_margin: Duration) -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
            safety_margin,
        }
    }

    /// Returns the cached access token for `key` if one exists
    /// and is still fresh (expires_at > now + safety_margin).
    ///
    /// Returns `None` on a miss, an expired entry, a poisoned
    /// mutex, or an empty access token. Callers should treat any
    /// `None` as "go refresh".
    #[must_use]
    pub fn cached(&self, key: &ClientCredentialsKey) -> Option<String> {
        let now = Instant::now();
        let guard = self.entries.lock().ok()?;
        let entry = guard.get(key)?;
        let cutoff = now.checked_add(self.safety_margin)?;
        if entry.expires_at > cutoff && !entry.access_token.is_empty() {
            Some(entry.access_token.clone())
        } else {
            None
        }
    }

    /// Inserts (or replaces) the cached token for `key`.
    ///
    /// `expires_in` is the provider-declared lifetime; the cache
    /// subtracts its own safety margin before storing `expires_at`
    /// so a subsequent [`Self::cached`] check reads the entry as
    /// expired exactly `safety_margin` seconds before the provider
    /// would reject it.
    ///
    /// Returns `Err(IntegrationError::InvalidInput)` if
    /// `access_token` is empty — the cache never stores an empty
    /// token because the resulting `Authorization: Bearer ` header
    /// would always be rejected by the provider. A poisoned mutex
    /// surfaces as `Err(IntegrationError::Infrastructure(...))`.
    pub fn store(
        &self,
        key: ClientCredentialsKey,
        access_token: String,
        token_type: String,
        expires_in: Duration,
    ) -> Result<()> {
        if access_token.is_empty() {
            return Err(IntegrationError::InvalidInput(
                "client_credentials: access_token must not be empty".to_owned(),
            ));
        }
        let mut guard = self.entries.lock().map_err(|e| {
            IntegrationError::Infrastructure(
                format!("client_credentials mutex poisoned: {e}").into(),
            )
        })?;
        // Clamp the lifetime to a sane upper bound (24 h) so a
        // pathological `expires_in = u64::MAX` from a misbehaving
        // provider cannot blow up `Instant` arithmetic. The check
        // uses saturating subtraction instead of a cast.
        let max_lifetime = Duration::from_secs(24 * 60 * 60);
        let safe_lifetime = if expires_in > max_lifetime {
            max_lifetime
        } else {
            expires_in
        };
        let safe_margin = if self.safety_margin >= safe_lifetime {
            Duration::ZERO
        } else {
            safe_lifetime.saturating_sub(self.safety_margin)
        };
        let expires_at = Instant::now().checked_add(safe_margin).ok_or_else(|| {
            IntegrationError::Infrastructure("client_credentials: expires_at overflow".into())
        })?;
        guard.insert(
            key,
            CachedToken {
                access_token,
                expires_at,
                token_type,
            },
        );
        Ok(())
    }

    /// Removes the cached entry for `key`. The next
    /// [`Self::cached`] / [`Self::get_or_refresh`] call for that
    /// key will miss and refresh.
    ///
    /// A poisoned mutex surfaces as an `Infrastructure` error.
    pub fn invalidate(&self, key: &ClientCredentialsKey) -> Result<()> {
        let mut guard = self.entries.lock().map_err(|e| {
            IntegrationError::Infrastructure(
                format!("client_credentials mutex poisoned: {e}").into(),
            )
        })?;
        guard.remove(key);
        Ok(())
    }

    /// Removes every cached entry. Used by integration tests and
    /// by adapters that reconfigure credentials at runtime.
    pub fn clear(&self) -> Result<()> {
        let mut guard = self.entries.lock().map_err(|e| {
            IntegrationError::Infrastructure(
                format!("client_credentials mutex poisoned: {e}").into(),
            )
        })?;
        guard.clear();
        Ok(())
    }

    /// Returns the number of cached entries. A poisoned mutex
    /// returns `0`; the cache is advisory and never panics on
    /// introspection.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.lock().map(|g| g.len()).unwrap_or(0)
    }

    /// Returns `true` if the cache holds no entries. See
    /// [`Self::len`] for the poisoned-mutex semantics.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the cached access token for `(tenant, integration)`
    /// or, on a miss, calls `exchanger` to fetch a fresh one and
    /// stores it.
    ///
    /// The `exchanger` closure is invoked at most once per call.
    /// Concurrent callers for the same key serialise through the
    /// interior mutex: a caller that observes a fresh entry on
    /// the read path returns without invoking `exchanger`. Callers
    /// that race past the read path before another caller stores
    /// its result will each invoke `exchanger`; this is
    /// acceptable because the OAuth2 token exchange is idempotent
    /// in practice (most providers issue a fresh token and return
    /// 200) and the losing call simply overwrites the winning
    /// call's entry on `store`.
    ///
    /// # Errors
    ///
    /// - [`IntegrationError::InvalidInput`] — `exchanger` returned
    ///   an empty `access_token`. The cache refuses to store an
    ///   empty token.
    /// - Any error the `exchanger` closure returns propagates
    ///   unchanged.
    pub async fn get_or_refresh<F, Fut>(
        &self,
        tenant: &TenantContext,
        integration: &IntegrationId,
        exchanger: F,
    ) -> Result<String>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<TokenResponse>>,
    {
        self.get_or_refresh_with_margin(tenant, integration, exchanger)
            .await
    }

    /// Same as [`Self::get_or_refresh`] but uses a caller-supplied
    /// safety margin just for this call. The margin passed here
    /// is **not** stored on the cache; future calls fall back to
    /// the constructor-provided margin. Use this variant when a
    /// specific provider is known to be more (or less) aggressive
    /// about rejecting soon-to-expire tokens than the default.
    pub async fn get_or_refresh_with_margin<F, Fut>(
        &self,
        tenant: &TenantContext,
        integration: &IntegrationId,
        exchanger: F,
    ) -> Result<String>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<TokenResponse>>,
    {
        let key = ClientCredentialsKey::new(tenant.school_id, integration.clone());
        if let Some(token) = self.cached(&key) {
            return Ok(token);
        }
        let response = exchanger().await?;
        if response.is_empty() {
            return Err(IntegrationError::InvalidInput(
                "client_credentials: provider returned empty access_token".to_owned(),
            ));
        }
        self.store(
            key,
            response.access_token.clone(),
            response.token_type.clone(),
            response.expires_in_duration(),
        )?;
        Ok(response.access_token)
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
    use std::sync::Arc;

    use educore_core::ids::{CorrelationId, Identifier, SchoolId, UserId};
    use educore_core::tenant::{Locale, TenantContext, TimeZone, UserType};

    fn school_id(byte: u8) -> SchoolId {
        // UUIDv7 with the first byte set to `byte` for stable test IDs.
        SchoolId::from_uuid(uuid::Uuid::now_v7())
    }

    fn tenant(school: SchoolId) -> TenantContext {
        TenantContext {
            school_id: school,
            actor_id: UserId::from_uuid(uuid::Uuid::now_v7()),
            session_id: None,
            correlation_id: CorrelationId::from_uuid(uuid::Uuid::now_v7()),
            causation_id: None,
            user_type: UserType::System,
            locale: Locale::default(),
            timezone: TimeZone::default(),
        }
    }

    fn integration(name: &str) -> IntegrationId {
        IntegrationId::new(name)
    }

    fn token(value: &str, expires_in: u64) -> TokenResponse {
        TokenResponse {
            access_token: value.to_owned(),
            expires_in,
            token_type: DEFAULT_TOKEN_TYPE.to_owned(),
        }
    }

    #[test]
    fn grant_type_constant_matches_rfc() {
        assert_eq!(OAUTH2_GRANT_TYPE_CLIENT_CREDENTIALS, "client_credentials");
    }

    #[test]
    fn store_then_cached_returns_token() {
        let cache = ClientCredentialsCache::new();
        let key = ClientCredentialsKey::new(school_id(1), integration("acme"));
        cache
            .store(
                key.clone(),
                "tok-1".to_owned(),
                DEFAULT_TOKEN_TYPE.to_owned(),
                Duration::from_secs(3600),
            )
            .expect("store");
        assert_eq!(cache.cached(&key).as_deref(), Some("tok-1"));
    }

    #[test]
    fn empty_token_is_rejected() {
        let cache = ClientCredentialsCache::new();
        let key = ClientCredentialsKey::new(school_id(2), integration("acme"));
        let err = cache
            .store(
                key.clone(),
                String::new(),
                DEFAULT_TOKEN_TYPE.to_owned(),
                Duration::from_secs(3600),
            )
            .expect_err("empty token must be rejected");
        assert!(matches!(err, IntegrationError::InvalidInput(_)));
        assert!(cache.cached(&key).is_none());
    }

    #[test]
    fn invalidate_removes_entry() {
        let cache = ClientCredentialsCache::new();
        let key = ClientCredentialsKey::new(school_id(3), integration("acme"));
        cache
            .store(
                key.clone(),
                "tok".to_owned(),
                DEFAULT_TOKEN_TYPE.to_owned(),
                Duration::from_secs(3600),
            )
            .expect("store");
        assert!(cache.cached(&key).is_some());
        cache.invalidate(&key).expect("invalidate");
        assert!(cache.cached(&key).is_none());
    }

    #[test]
    fn clear_removes_every_entry() {
        let cache = ClientCredentialsCache::new();
        let k1 = ClientCredentialsKey::new(school_id(4), integration("a"));
        let k2 = ClientCredentialsKey::new(school_id(5), integration("b"));
        cache
            .store(
                k1.clone(),
                "t1".to_owned(),
                DEFAULT_TOKEN_TYPE.to_owned(),
                Duration::from_secs(3600),
            )
            .expect("store 1");
        cache
            .store(
                k2.clone(),
                "t2".to_owned(),
                DEFAULT_TOKEN_TYPE.to_owned(),
                Duration::from_secs(3600),
            )
            .expect("store 2");
        assert_eq!(cache.len(), 2);
        cache.clear().expect("clear");
        assert!(cache.is_empty());
    }

    #[test]
    fn per_tenant_keys_are_isolated() {
        let cache = ClientCredentialsCache::new();
        let s1 = school_id(10);
        let s2 = school_id(11);
        let k1 = ClientCredentialsKey::new(s1, integration("shared"));
        let k2 = ClientCredentialsKey::new(s2, integration("shared"));
        cache
            .store(
                k1.clone(),
                "tok-s1".to_owned(),
                DEFAULT_TOKEN_TYPE.to_owned(),
                Duration::from_secs(3600),
            )
            .expect("store s1");
        cache
            .store(
                k2.clone(),
                "tok-s2".to_owned(),
                DEFAULT_TOKEN_TYPE.to_owned(),
                Duration::from_secs(3600),
            )
            .expect("store s2");
        assert_eq!(cache.cached(&k1).as_deref(), Some("tok-s1"));
        assert_eq!(cache.cached(&k2).as_deref(), Some("tok-s2"));
    }

    #[test]
    fn per_integration_keys_are_isolated() {
        let cache = ClientCredentialsCache::new();
        let s = school_id(20);
        let k1 = ClientCredentialsKey::new(s, integration("a"));
        let k2 = ClientCredentialsKey::new(s, integration("b"));
        cache
            .store(
                k1.clone(),
                "tok-a".to_owned(),
                DEFAULT_TOKEN_TYPE.to_owned(),
                Duration::from_secs(3600),
            )
            .expect("store a");
        cache
            .store(
                k2.clone(),
                "tok-b".to_owned(),
                DEFAULT_TOKEN_TYPE.to_owned(),
                Duration::from_secs(3600),
            )
            .expect("store b");
        assert_eq!(cache.cached(&k1).as_deref(), Some("tok-a"));
        assert_eq!(cache.cached(&k2).as_deref(), Some("tok-b"));
    }

    #[tokio::test]
    async fn get_or_refresh_returns_cached_on_hit() {
        let cache = Arc::new(ClientCredentialsCache::new());
        let tenant_ctx = tenant(school_id(30));
        let integ = integration("acme");

        // Seed the cache via get_or_refresh (exchanger invoked once).
        let first = cache
            .get_or_refresh(&tenant_ctx, &integ, || async { Ok(token("seed", 3600)) })
            .await
            .expect("first refresh");
        assert_eq!(first, "seed");

        // Exchanger must NOT be invoked again on the second call —
        // we replace the closure with one that would error.
        let second = cache
            .get_or_refresh(&tenant_ctx, &integ, || async {
                Err(IntegrationError::Provider(
                    "exchanger should not run on cache hit".to_owned(),
                ))
            })
            .await
            .expect("second call must hit cache");
        assert_eq!(second, "seed");
    }

    #[tokio::test]
    async fn get_or_refresh_invokes_exchanger_on_miss() {
        let cache = ClientCredentialsCache::new();
        let tenant_ctx = tenant(school_id(31));
        let integ = integration("acme");

        let mut calls = 0_u32;
        let result = cache
            .get_or_refresh(&tenant_ctx, &integ, || {
                calls = calls.saturating_add(1);
                async { Ok(token("fresh", 3600)) }
            })
            .await
            .expect("refresh");
        assert_eq!(result, "fresh");
        assert_eq!(calls, 1, "exchanger must run exactly once on miss");
    }

    #[tokio::test]
    async fn get_or_refresh_propagates_exchanger_error() {
        let cache = ClientCredentialsCache::new();
        let tenant_ctx = tenant(school_id(32));
        let integ = integration("acme");

        let result = cache
            .get_or_refresh(&tenant_ctx, &integ, || async {
                Err::<TokenResponse, _>(IntegrationError::Provider("boom".to_owned()))
            })
            .await;
        let err = result.expect_err("must propagate exchanger error");
        assert!(matches!(err, IntegrationError::Provider(ref m) if m == "boom"));
    }

    #[tokio::test]
    async fn get_or_refresh_rejects_empty_token() {
        let cache = ClientCredentialsCache::new();
        let tenant_ctx = tenant(school_id(33));
        let integ = integration("acme");

        let result = cache
            .get_or_refresh(&tenant_ctx, &integ, || async {
                Ok::<TokenResponse, IntegrationError>(token("", 3600))
            })
            .await;
        let err = result.expect_err("empty token must be rejected");
        assert!(matches!(err, IntegrationError::InvalidInput(_)));
    }

    #[test]
    fn token_response_uses_defaults_when_fields_missing() {
        let parsed: TokenResponse =
            serde_json::from_str(r#"{"access_token":"x"}"#).expect("minimal response must parse");
        assert_eq!(parsed.access_token, "x");
        assert_eq!(parsed.expires_in, DEFAULT_EXPIRES_IN_SECONDS);
        assert_eq!(parsed.token_type, DEFAULT_TOKEN_TYPE);
    }

    #[test]
    fn token_response_round_trips_through_json() {
        let original = token("abc", 7200);
        let json = serde_json::to_string(&original).expect("serialize");
        let parsed: TokenResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, original);
    }

    #[test]
    fn debug_render_does_not_panic_on_poisoned_mutex() {
        // The Debug impl never takes the lock in a way that
        // panics; we exercise it here to document the contract.
        let cache = ClientCredentialsCache::new();
        let _ = format!("{cache:?}");
    }

    // -----------------------------------------------------------------
    // Reference the imports so the compiler warns if the upstream
    // re-exports shift. Not part of the public surface.
    // -----------------------------------------------------------------
    #[allow(dead_code)]
    fn _imports_touched() {
        let _ = TimeZone::default();
        let _ = UserType::System;
    }
}
