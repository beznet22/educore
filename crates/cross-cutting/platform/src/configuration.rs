//! Platform-domain per-tenant configuration port.
//!
//! Per `docs/schemas/tenancy-schema.md` § 9 ("Per-Tenant
//! Configuration"), each school owns its own configuration
//! values — settings, capabilities, modules, payment gateways,
//! notification providers, and any other tenant-scoped knob.
//! Every domain reads its configuration from the active
//! `SchoolId` via this port; there is no global "default
//! school" configuration.
//!
//! This module exposes:
//!
//! - [`SettingName`] / [`ConfigurationKey`] — the typed setting
//!   name used as the lookup key.
//! - [`ConfigurationValue`] — the four canonical value shapes
//!   (`String`, `Integer`, `Bool`, `Json`).
//! - [`ConfigurationEntry`] — one row in the configuration table
//!   (key + value + audit fields).
//! - [`ConfigurationService`] — the port trait every storage
//!   adapter implements.
//! - [`InMemoryConfigurationService`] — the default
//!   in-memory implementation used by tests, the SDK, and any
//!   consumer that does not need persistence.
//!
//! # Why a port?
//!
//! The platform crate is a `cross-cutting` member and may not
//! depend on `adapters` (see `docs/build-plan.md` § The
//! No-Gaps Gates). The port trait lives here; storage
//! adapters (`educore-storage-*`) implement it against their
//! native key-value or relational substrate. The engine's
//! `ConfigurationService` rows are stored in the
//! `platform_configuration` table emitted by the adapter at
//! `create_schema()` time (see `docs/specs/platform/tables.md`).
//!
//! # Storage wiring
//!
//! Full storage-adapter wiring lands in a later phase. The
//! in-memory implementation is the only one shipped with this
//! commit; downstream phases will add SurrealDB, PostgreSQL,
//! MySQL, and SQLite backends behind the same trait.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use educore_core::clock::Clock;
use educore_core::error::{DomainError, Result};
use educore_core::ids::{SchoolId, UserId};
use educore_core::tenant::TenantContext;
use educore_core::value_objects::Timestamp;

/// The maximum length of a configuration key, in bytes.
///
/// Matches the engine's 191-byte `VARCHAR` ceiling used by the
/// cross-cutting schema for typed identifier columns. The cap
/// is intentionally conservative so the configuration table
/// can be indexed in MySQL/InnoDB without falling off the
/// prefix-index length limit.
pub const MAX_KEY_LEN: usize = 191;

/// Typed setting name used as the lookup key for
/// [`ConfigurationService`].
///
/// A `SettingName` is a non-empty string with at most
/// [`MAX_KEY_LEN`] bytes. The engine does not constrain the
/// character set beyond that (the engine uses dotted, kebab,
/// and snake-case keys interchangeably across domains), but
/// empty keys are rejected at construction so a downstream
/// adapter never has to defend against them.
///
/// # Wire format
///
/// `SettingName` serialises transparently as a JSON string
/// (the `#[serde(transparent)]` attribute on the newtype).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SettingName(String);

impl SettingName {
    /// Constructs a new `SettingName` from a raw string.
    ///
    /// # Errors
    ///
    /// - `Validation` if `raw` is empty or longer than
    ///   [`MAX_KEY_LEN`] bytes.
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let s = raw.into();
        validate_key(&s)?;
        Ok(Self(s))
    }

    /// Returns the setting name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SettingName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for SettingName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<SettingName> for String {
    fn from(k: SettingName) -> Self {
        k.0
    }
}

fn validate_key(s: &str) -> Result<()> {
    if s.is_empty() {
        return Err(DomainError::Validation(
            "configuration key must not be empty".to_owned(),
        ));
    }
    if s.len() > MAX_KEY_LEN {
        return Err(DomainError::Validation(format!(
            "configuration key length {} exceeds {}",
            s.len(),
            MAX_KEY_LEN
        )));
    }
    Ok(())
}

/// Conceptual alias for [`SettingName`] when the value object
/// is used as the lookup key of [`ConfigurationService`].
///
/// The two names exist for documentation clarity: the trait
/// signature reads `&ConfigurationKey`, while the value-object
/// constructor is `SettingName::new(...)`. They are the same
/// type; this alias is purely cosmetic.
pub type ConfigurationKey = SettingName;

/// The four canonical value shapes a tenant configuration
/// entry can hold.
///
/// Per `docs/specs/platform/value-objects.md` the engine
/// restricts configuration values to one of four scalar
/// shapes — `String`, `Integer`, `Bool`, or a free-form JSON
/// payload (`Json`). The `Json` variant exists for settings
/// that have to carry nested structured data (e.g. an SMTP
/// configuration with host, port, and TLS sub-keys), but the
/// intent is to prefer the typed scalars when possible so
/// adapters can index them in their native type system
/// rather than treating the entire column as `JSONB`.
///
/// # Wire format
///
/// `ConfigurationValue` serialises with the externally-tagged
/// representation `{"type": "<variant>", "value": <payload>}`.
/// Storage adapters round-trip this representation
/// faithfully; a `ConfigurationValue::Json(v)` is stored as a
/// `JSONB` column whose top-level object is `{"type": "Json",
/// "value": <v>}`, so consumers must unwrap one layer to
/// read the inner `v`. This indirection is intentional — it
/// keeps the table schema uniform across all four variants
/// without losing the discriminant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum ConfigurationValue {
    /// A UTF-8 string value.
    String(String),
    /// A signed 64-bit integer.
    Integer(i64),
    /// A boolean.
    Bool(bool),
    /// A free-form JSON value. The `serde_json::Value` is the
    /// explicit type for tenant-config JSON payloads; it is
    /// not used as a stand-in for typed domain data anywhere
    /// else in this crate.
    Json(Value),
}

impl ConfigurationValue {
    /// Returns the canonical wire-name discriminant for the
    /// variant (`"String"`, `"Integer"`, `"Bool"`, `"Json"`).
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::String(_) => "String",
            Self::Integer(_) => "Integer",
            Self::Bool(_) => "Bool",
            Self::Json(_) => "Json",
        }
    }
}

/// One configuration row, as returned by
/// [`ConfigurationService::get`].
///
/// The entry carries the value plus the audit fields
/// (`school_id`, `updated_at`, `updated_by`). `school_id` is
/// duplicated on the entry (the trait methods also take a
/// `TenantContext`) so the entry is self-describing when it
/// is moved across boundaries (e.g. embedded in an event
/// payload).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigurationEntry {
    /// The setting name this entry belongs to.
    pub key: SettingName,
    /// The value held under `key`.
    pub value: ConfigurationValue,
    /// The school that owns this entry. Mirrors
    /// `TenantContext::school_id` for the write that produced
    /// the row.
    pub school_id: SchoolId,
    /// Wall-clock instant of the most recent write to this
    /// entry.
    pub updated_at: Timestamp,
    /// The actor (user or system) that performed the most
    /// recent write. Stored as a [`UserId`]; the platform's
    /// system actor is `SYSTEM_USER_ID`.
    pub updated_by: UserId,
}

impl ConfigurationEntry {
    /// Builds a new [`ConfigurationEntry`] from its parts.
    /// Used by storage adapters and the in-memory
    /// implementation when applying a `set` write.
    #[must_use]
    pub const fn new(
        key: SettingName,
        value: ConfigurationValue,
        school_id: SchoolId,
        updated_at: Timestamp,
        updated_by: UserId,
    ) -> Self {
        Self {
            key,
            value,
            school_id,
            updated_at,
            updated_by,
        }
    }
}

/// Port trait: read and write per-tenant configuration
/// values.
///
/// The trait is `Send + Sync` and object-safe; the engine
/// holds it as `Arc<dyn ConfigurationService>` so a single
/// instance can be shared across the runtime. All methods
/// take a [`TenantContext`] for the read or write so the
/// adapter cannot accidentally surface a cross-tenant row:
/// `get`, `set`, `delete`, and `list_keys` are all scoped to
/// `ctx.school_id`.
///
/// # Contract
///
/// - `get` returns `Ok(None)` if the key is absent for the
///   active school; it never returns `Err(NotFound)` for an
///   absent key (the caller is expected to use `Option`).
/// - `set` is an upsert: it writes the value unconditionally
///   (inserting if the key is absent, overwriting if present)
///   and returns the resulting [`ConfigurationEntry`] so the
///   caller has the new `updated_at` and `updated_by`.
/// - `delete` is idempotent: it returns `Ok(())` whether or
///   not the key was present. Use `get` first if the caller
///   needs to distinguish.
/// - `list_keys` returns every key set for the active school,
///   in deterministic ascending order (adapters are expected
///   to sort).
#[async_trait]
pub trait ConfigurationService: Send + Sync {
    /// Reads the entry for `key` in the active school.
    ///
    /// Returns `Ok(None)` if `key` is not set for
    /// `ctx.school_id`.
    async fn get(
        &self,
        ctx: &TenantContext,
        key: &SettingName,
    ) -> Result<Option<ConfigurationEntry>>;

    /// Writes `value` under `key` in the active school.
    ///
    /// The write is an upsert: it inserts if `key` is absent
    /// and overwrites if present. The `actor` is recorded as
    /// the entry's `updated_by`; the `Timestamp` is the
    /// adapter's clock at the moment of the write.
    ///
    /// Returns the resulting [`ConfigurationEntry`] so the
    /// caller has the new `updated_at` and `updated_by`.
    async fn set(
        &self,
        ctx: &TenantContext,
        key: SettingName,
        value: ConfigurationValue,
        actor: UserId,
    ) -> Result<ConfigurationEntry>;

    /// Removes the entry for `key` in the active school.
    ///
    /// Idempotent: returns `Ok(())` whether or not the key
    /// was present. Use [`ConfigurationService::get`] first if
    /// the caller needs to distinguish the two cases.
    async fn delete(&self, ctx: &TenantContext, key: &SettingName) -> Result<()>;

    /// Lists every key set for the active school.
    ///
    /// Returns the keys in deterministic ascending order
    /// (adapters must sort before returning). The list is
    /// scoped to `ctx.school_id`; keys belonging to other
    /// schools are never returned.
    async fn list_keys(&self, ctx: &TenantContext) -> Result<Vec<SettingName>>;
}

/// Default in-memory implementation of
/// [`ConfigurationService`].
///
/// The store is keyed by `(school_id, setting_name)`. All
/// writes go through an interior `RwLock<BTreeMap<...>>` and
/// read methods take a read lock. The clock is supplied at
/// construction so the implementation is deterministic in
/// tests (use [`TestClock`](educore_core::clock::TestClock))
/// and live in production (use
/// [`SystemClock`](educore_core::clock::SystemClock)).
///
/// # Thread-safety
///
/// `InMemoryConfigurationService` is `Clone` (cheap, via the
/// inner `Arc`s) and `Send + Sync`. Multiple clones share the
/// same underlying store.
#[derive(Clone)]
pub struct InMemoryConfigurationService {
    /// School → (SettingName → (value, updated_at, updated_by)).
    inner: Arc<RwLock<BTreeMap<SchoolId, BTreeMap<SettingName, StoredRow>>>>,
    /// Clock used to stamp `updated_at` on every write.
    clock: Arc<dyn Clock>,
}

/// One row in the in-memory store.
#[derive(Debug, Clone, PartialEq)]
struct StoredRow {
    value: ConfigurationValue,
    updated_at: Timestamp,
    updated_by: UserId,
}

impl fmt::Debug for InMemoryConfigurationService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let count = self
            .inner
            .read()
            .map(|g| g.values().map(BTreeMap::len).sum::<usize>())
            .unwrap_or(0);
        f.debug_struct("InMemoryConfigurationService")
            .field("entries", &count)
            .finish_non_exhaustive()
    }
}

impl InMemoryConfigurationService {
    /// Creates a new, empty in-memory configuration store
    /// backed by `clock`.
    #[must_use]
    pub fn new(clock: Arc<dyn Clock>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(BTreeMap::new())),
            clock,
        }
    }

    /// Returns the total number of entries across every
    /// school. Intended for tests and observability; not part
    /// of the [`ConfigurationService`] port.
    #[must_use]
    pub fn total_entries(&self) -> usize {
        match self.inner.read() {
            Ok(g) => g.values().map(BTreeMap::len).sum(),
            Err(poisoned) => poisoned.into_inner().values().map(BTreeMap::len).sum(),
        }
    }
}

#[async_trait]
impl ConfigurationService for InMemoryConfigurationService {
    async fn get(
        &self,
        ctx: &TenantContext,
        key: &SettingName,
    ) -> Result<Option<ConfigurationEntry>> {
        let g = match self.inner.read() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        let Some(by_school) = g.get(&ctx.school_id) else {
            return Ok(None);
        };
        let Some(row) = by_school.get(key) else {
            return Ok(None);
        };
        Ok(Some(ConfigurationEntry::new(
            key.clone(),
            row.value.clone(),
            ctx.school_id,
            row.updated_at,
            row.updated_by,
        )))
    }

    async fn set(
        &self,
        ctx: &TenantContext,
        key: SettingName,
        value: ConfigurationValue,
        actor: UserId,
    ) -> Result<ConfigurationEntry> {
        let now = self.clock.now();
        let mut g = match self.inner.write() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        let by_school = g.entry(ctx.school_id).or_default();
        let row = StoredRow {
            value: value.clone(),
            updated_at: now,
            updated_by: actor,
        };
        by_school.insert(key.clone(), row);
        Ok(ConfigurationEntry::new(
            key,
            value,
            ctx.school_id,
            now,
            actor,
        ))
    }

    async fn delete(&self, ctx: &TenantContext, key: &SettingName) -> Result<()> {
        let mut g = match self.inner.write() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        if let Some(by_school) = g.get_mut(&ctx.school_id) {
            by_school.remove(key);
        }
        Ok(())
    }

    async fn list_keys(&self, ctx: &TenantContext) -> Result<Vec<SettingName>> {
        let g = match self.inner.read() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        let keys = g
            .get(&ctx.school_id)
            .map(|m| m.keys().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        Ok(keys)
    }
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
    use educore_core::clock::SystemClock;
    use educore_core::ids::CorrelationId;
    use educore_core::tenant::UserType;

    fn school(seed: u8) -> SchoolId {
        // Deterministic per-test SchoolId so failure output is
        // greppable.
        SchoolId(uuid::Uuid::from_bytes([seed; 16]))
    }

    fn user(seed: u8) -> UserId {
        UserId(uuid::Uuid::from_bytes([seed + 100; 16]))
    }

    fn tenant(school_id: SchoolId, actor: UserId) -> TenantContext {
        TenantContext::for_user(
            school_id,
            actor,
            CorrelationId(uuid::Uuid::nil()),
            UserType::SuperAdmin,
        )
    }

    fn svc() -> InMemoryConfigurationService {
        InMemoryConfigurationService::new(Arc::new(SystemClock))
    }

    #[test]
    fn setting_name_rejects_empty() {
        assert!(SettingName::new("").is_err());
    }

    #[test]
    fn setting_name_rejects_overlong() {
        let long = "x".repeat(MAX_KEY_LEN + 1);
        assert!(SettingName::new(long).is_err());
    }

    #[test]
    fn setting_name_accepts_max_len() {
        let s = "x".repeat(MAX_KEY_LEN);
        let k = SettingName::new(s).unwrap();
        assert_eq!(k.as_str().len(), MAX_KEY_LEN);
    }

    #[test]
    fn setting_name_round_trips_through_string() {
        let k = SettingName::new("tenant.theme.primary").unwrap();
        let s: String = k.clone().into();
        assert_eq!(s, "tenant.theme.primary");
        assert_eq!(k.as_str(), s);
    }

    #[test]
    fn configuration_key_is_alias_for_setting_name() {
        // Compile-time check: the alias resolves to the same
        // concrete type.
        fn _alias_check(_: ConfigurationKey) -> SettingName {
            SettingName::new("a.b.c").unwrap()
        }
        let k: ConfigurationKey = SettingName::new("a.b").unwrap();
        let s: SettingName = k;
        assert_eq!(s.as_str(), "a.b");
    }

    #[test]
    fn configuration_value_kind_is_stable() {
        assert_eq!(ConfigurationValue::String("x".into()).kind(), "String");
        assert_eq!(ConfigurationValue::Integer(42).kind(), "Integer");
        assert_eq!(ConfigurationValue::Bool(true).kind(), "Bool");
        assert_eq!(ConfigurationValue::Json(Value::Null).kind(), "Json");
    }

    #[test]
    fn configuration_value_round_trips_via_serde() {
        let v = ConfigurationValue::Json(serde_json::json!({"k": [1, 2, 3]}));
        let s = serde_json::to_string(&v).unwrap();
        let back: ConfigurationValue = serde_json::from_str(&s).unwrap();
        assert_eq!(back, v);
    }

    #[tokio::test]
    async fn in_memory_get_returns_none_for_absent_key() {
        let s = svc();
        let ctx = tenant(school(1), user(1));
        let k = SettingName::new("missing").unwrap();
        let out = s.get(&ctx, &k).await.unwrap();
        assert!(out.is_none());
    }

    #[tokio::test]
    async fn in_memory_set_then_get_round_trips() {
        let s = svc();
        let ctx = tenant(school(1), user(1));
        let k = SettingName::new("tenant.theme.primary").unwrap();
        let v = ConfigurationValue::String("#0a0a0a".into());
        let written = s.set(&ctx, k.clone(), v.clone(), user(1)).await.unwrap();
        assert_eq!(written.key, k);
        assert_eq!(written.value, v);
        assert_eq!(written.school_id, school(1));
        assert_eq!(written.updated_by, user(1));
        let read = s.get(&ctx, &k).await.unwrap();
        assert_eq!(read, Some(written));
    }

    #[tokio::test]
    async fn in_memory_set_is_upsert() {
        let s = svc();
        let ctx = tenant(school(1), user(1));
        let k = SettingName::new("k").unwrap();
        s.set(&ctx, k.clone(), ConfigurationValue::Bool(true), user(1))
            .await
            .unwrap();
        let after = s
            .set(&ctx, k.clone(), ConfigurationValue::Bool(false), user(2))
            .await
            .unwrap();
        assert_eq!(after.updated_by, user(2));
        let read = s.get(&ctx, &k).await.unwrap().unwrap();
        assert_eq!(read.value, ConfigurationValue::Bool(false));
        assert_eq!(read.updated_by, user(2));
    }

    #[tokio::test]
    async fn in_memory_delete_is_idempotent() {
        let s = svc();
        let ctx = tenant(school(1), user(1));
        let k = SettingName::new("k").unwrap();
        // Delete absent: still Ok.
        s.delete(&ctx, &k).await.unwrap();
        s.set(&ctx, k.clone(), ConfigurationValue::Integer(7), user(1))
            .await
            .unwrap();
        assert!(s.get(&ctx, &k).await.unwrap().is_some());
        s.delete(&ctx, &k).await.unwrap();
        assert!(s.get(&ctx, &k).await.unwrap().is_none());
        // Re-delete: still Ok.
        s.delete(&ctx, &k).await.unwrap();
    }

    #[tokio::test]
    async fn in_memory_list_keys_is_sorted_and_scoped_per_school() {
        let s = svc();
        let ctx_a = tenant(school(1), user(1));
        let ctx_b = tenant(school(2), user(1));
        for k in ["zebra", "alpha", "mango"] {
            let key = SettingName::new(k).unwrap();
            s.set(&ctx_a, key.clone(), ConfigurationValue::Bool(true), user(1))
                .await
                .unwrap();
        }
        let list_a = s.list_keys(&ctx_a).await.unwrap();
        let list_b = s.list_keys(&ctx_b).await.unwrap();
        let names_a: Vec<&str> = list_a.iter().map(SettingName::as_str).collect();
        assert_eq!(names_a, vec!["alpha", "mango", "zebra"]);
        assert!(list_b.is_empty());
    }

    #[tokio::test]
    async fn in_memory_entries_are_scoped_per_school() {
        let s = svc();
        let ctx_a = tenant(school(1), user(1));
        let ctx_b = tenant(school(2), user(1));
        let k = SettingName::new("shared.key").unwrap();
        s.set(
            &ctx_a,
            k.clone(),
            ConfigurationValue::String("A".into()),
            user(1),
        )
        .await
        .unwrap();
        s.set(
            &ctx_b,
            k.clone(),
            ConfigurationValue::String("B".into()),
            user(1),
        )
        .await
        .unwrap();
        let read_a = s.get(&ctx_a, &k).await.unwrap().unwrap();
        let read_b = s.get(&ctx_b, &k).await.unwrap().unwrap();
        assert_eq!(read_a.value, ConfigurationValue::String("A".into()));
        assert_eq!(read_b.value, ConfigurationValue::String("B".into()));
        assert_eq!(s.total_entries(), 2);
    }

    #[test]
    fn trait_is_object_safe() {
        // Compile-time check: confirm the trait is object-safe
        // by naming a `Box<dyn ConfigurationService>`.
        fn _is_object_safe(_: Box<dyn ConfigurationService>) {}
    }
}
