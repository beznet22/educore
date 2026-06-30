//! # Schema registry port
//!
//! Per `docs/ports/event-bus.md` § "Schema Registry", the engine
//! maintains a schema registry that tracks every event /
//! command / aggregate-state schema by `(kind, name, version)`.
//! Consumers consult the registry before publishing or
//! consuming an envelope so a malformed payload never reaches
//! the bus.
//!
//! This module is the **port** — the trait shape consumers
//! program against. Storage adapters (postgres, mysql, sqlite,
//! surrealdb, in-memory testkit) implement the trait; this
//! module only declares the contract.
//!
//! ## Object safety
//!
//! The trait is object-safe. Bus implementations hold the
//! registry as `Arc<dyn SchemaRegistry>` and pass it across
//! spawn boundaries without generic-type plumbing.

use async_trait::async_trait;
use std::collections::BTreeMap;
use std::sync::Mutex;

use educore_core::error::{DomainError, Result};
use educore_core::ids::{Identifier, SchoolId, UserId};
use educore_core::value_objects::Timestamp;
use uuid::Uuid;

/// The kind of schema a registration describes.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum SchemaKind {
    /// A domain event (`platform.school.created`, etc.).
    DomainEvent,
    /// A command (`academic.student.admit`, etc.).
    Command,
    /// An aggregate-state projection (`Student`, `Invoice`, ...).
    AggregateState,
}

/// The semantic version of a schema. The wire format is a
/// monotonic `u32`; consumers should treat versions as
/// `[SCHEMA_VERSION]` constants in `DomainEvent` impls.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub struct SchemaVersion(pub u32);

impl SchemaVersion {
    /// Mints a fresh v1 version.
    #[must_use]
    pub const fn v1() -> Self {
        Self(1)
    }
}

/// A single registered schema row.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RegisteredSchema {
    /// The schema kind (event / command / aggregate).
    pub kind: SchemaKind,
    /// The schema name (e.g. `"platform.school.created"`).
    pub name: String,
    /// The semantic version.
    pub version: SchemaVersion,
    /// The content-addressable id of the JSON-Schema document
    /// (a CID, sha256 of the canonical schema bytes).
    pub json_schema_cid: String,
    /// The tenant anchor. `SchoolId::PUBLIC` means a global
    /// schema visible to every tenant.
    pub school_id: SchoolId,
    /// When the registration was added.
    pub registered_at: Timestamp,
    /// Who registered the schema (typically the system actor
    /// for engine-bundled schemas, or a developer for ad-hoc
    /// registrations during a migration).
    pub registered_by: UserId,
}

impl RegisteredSchema {
    /// Mints a fresh `RegisteredSchema` with a new UUIDv7-style
    /// timestamp anchor.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        kind: SchemaKind,
        name: String,
        version: SchemaVersion,
        json_schema_cid: String,
        school_id: SchoolId,
        registered_at: Timestamp,
        registered_by: UserId,
    ) -> Self {
        Self {
            kind,
            name,
            version,
            json_schema_cid,
            school_id,
            registered_at,
            registered_by,
        }
    }
}

/// The schema registry port.
///
/// Object-safe; held as `Arc<dyn SchemaRegistry>` by bus
/// implementations. The `register` method MUST be idempotent
/// for the same `(kind, name, version)` triple so consumers
/// can replay registration at startup without producing
/// duplicate rows.
#[async_trait]
pub trait SchemaRegistry: Send + Sync + std::fmt::Debug {
    /// Registers `schema`. Returns `Ok(())` on success or if
    /// the registration already exists with identical fields;
    /// returns `Err(DomainError::Conflict(_))` if a different
    /// registration already exists for the same triple.
    async fn register(&self, schema: RegisteredSchema) -> Result<()>;

    /// Returns the registration for `(kind, name, version)` if
    /// present.
    async fn get(
        &self,
        kind: SchemaKind,
        name: &str,
        version: SchemaVersion,
    ) -> Result<Option<RegisteredSchema>>;

    /// Returns the highest-version registration for `(kind,
    /// name)` if any is present.
    async fn latest(&self, kind: SchemaKind, name: &str) -> Result<Option<RegisteredSchema>>;

    /// Lists every registration of `kind`, ordered by `(name,
    /// version)` ascending.
    async fn list(&self, kind: SchemaKind) -> Result<Vec<RegisteredSchema>>;
}

/// Default `SchemaRegistry` impl backed by an interior
/// `BTreeMap`. Suitable for tests and for adapter
/// configurations where the registry is intentionally
/// in-process (single-node setups, ephemeral local-dev runs).
#[derive(Debug, Default)]
pub struct InMemorySchemaRegistry {
    inner: Mutex<BTreeMap<(SchemaKind, String, SchemaVersion), RegisteredSchema>>,
}

impl InMemorySchemaRegistry {
    /// Constructs a new empty in-memory registry.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(BTreeMap::new()),
        }
    }
}

#[async_trait]
impl SchemaRegistry for InMemorySchemaRegistry {
    async fn register(&self, schema: RegisteredSchema) -> Result<()> {
        let key = (schema.kind, schema.name.clone(), schema.version);
        let mut guard = self
            .inner
            .lock()
            .map_err(|e| DomainError::Validation(format!("schema registry mutex poisoned: {e}")))?;
        if let Some(existing) = guard.get(&key) {
            if existing.json_schema_cid != schema.json_schema_cid
                || existing.school_id != schema.school_id
            {
                return Err(DomainError::Conflict(format!(
                    "schema already registered for {key:?} with different content"
                )));
            }
            // Idempotent re-registration with identical fields: no-op.
            Ok(())
        } else {
            guard.insert(key, schema);
            Ok(())
        }
    }

    async fn get(
        &self,
        kind: SchemaKind,
        name: &str,
        version: SchemaVersion,
    ) -> Result<Option<RegisteredSchema>> {
        let guard = self
            .inner
            .lock()
            .map_err(|e| DomainError::Validation(format!("schema registry mutex poisoned: {e}")))?;
        Ok(guard.get(&(kind, name.to_owned(), version)).cloned())
    }

    async fn latest(&self, kind: SchemaKind, name: &str) -> Result<Option<RegisteredSchema>> {
        let guard = self
            .inner
            .lock()
            .map_err(|e| DomainError::Validation(format!("schema registry mutex poisoned: {e}")))?;
        let mut latest: Option<&RegisteredSchema> = None;
        for ((k, n, _v), schema) in guard.iter() {
            if *k == kind && n == name {
                match latest {
                    None => latest = Some(schema),
                    Some(cur) if schema.version.0 > cur.version.0 => latest = Some(schema),
                    _ => {}
                }
            }
        }
        Ok(latest.cloned())
    }

    async fn list(&self, kind: SchemaKind) -> Result<Vec<RegisteredSchema>> {
        let guard = self
            .inner
            .lock()
            .map_err(|e| DomainError::Validation(format!("schema registry mutex poisoned: {e}")))?;
        let mut out: Vec<RegisteredSchema> = guard
            .iter()
            .filter(|((k, _, _), _)| *k == kind)
            .map(|(_, v)| v.clone())
            .collect();
        out.sort_by(|a, b| a.name.cmp(&b.name).then(a.version.cmp(&b.version)));
        Ok(out)
    }
}

/// Convenience: returns a fresh `Uuid` for the schema's
/// `json_schema_cid` field (caller can hash the schema bytes
/// and store the hex digest instead).
#[must_use]
pub fn fresh_schema_cid() -> String {
    Uuid::now_v7().to_string()
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
    use educore_core::clock::{IdGenerator, SystemIdGen};

    fn sample(kind: SchemaKind, name: &str, version: u32) -> RegisteredSchema {
        let g = SystemIdGen;
        RegisteredSchema::new(
            kind,
            name.to_owned(),
            SchemaVersion(version),
            fresh_schema_cid(),
            g.next_school_id(),
            Timestamp::now(),
            g.next_user_id(),
        )
    }

    #[tokio::test]
    async fn register_and_get_round_trip() {
        let reg = InMemorySchemaRegistry::new();
        let s = sample(SchemaKind::DomainEvent, "platform.school.created", 1);
        reg.register(s.clone()).await.expect("register");
        let fetched = reg
            .get(
                SchemaKind::DomainEvent,
                "platform.school.created",
                SchemaVersion(1),
            )
            .await
            .expect("get");
        assert_eq!(fetched, Some(s));
    }

    #[tokio::test]
    async fn register_is_idempotent_with_identical_content() {
        let reg = InMemorySchemaRegistry::new();
        let s = sample(SchemaKind::Command, "academic.student.admit", 1);
        reg.register(s.clone()).await.expect("first register");
        reg.register(s.clone())
            .await
            .expect("idempotent re-register");
        let count = reg.list(SchemaKind::Command).await.expect("list").len();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn register_conflicts_on_different_content() {
        let reg = InMemorySchemaRegistry::new();
        let mut s1 = sample(SchemaKind::DomainEvent, "x", 1);
        reg.register(s1.clone()).await.expect("first register");
        s1.json_schema_cid = "different-cid".to_owned();
        let err = reg.register(s1).await.expect_err("must conflict");
        assert!(matches!(err, DomainError::Conflict(_)));
    }

    #[tokio::test]
    async fn latest_picks_highest_version() {
        let reg = InMemorySchemaRegistry::new();
        for v in 1..=3 {
            reg.register(sample(SchemaKind::DomainEvent, "x", v))
                .await
                .expect("register");
        }
        let latest = reg
            .latest(SchemaKind::DomainEvent, "x")
            .await
            .expect("latest")
            .expect("present");
        assert_eq!(latest.version, SchemaVersion(3));
    }

    #[tokio::test]
    async fn get_missing_returns_none() {
        let reg = InMemorySchemaRegistry::new();
        let fetched = reg
            .get(SchemaKind::Command, "absent", SchemaVersion(1))
            .await
            .expect("get");
        assert!(fetched.is_none());
    }

    #[tokio::test]
    async fn list_filters_by_kind_and_sorts() {
        let reg = InMemorySchemaRegistry::new();
        reg.register(sample(SchemaKind::Command, "b", 1))
            .await
            .expect("register");
        reg.register(sample(SchemaKind::Command, "a", 2))
            .await
            .expect("register");
        reg.register(sample(SchemaKind::DomainEvent, "a", 1))
            .await
            .expect("register");
        let cmds = reg.list(SchemaKind::Command).await.expect("list");
        assert_eq!(cmds.len(), 2);
        assert_eq!(cmds[0].name, "a");
        assert_eq!(cmds[0].version, SchemaVersion(2));
        assert_eq!(cmds[1].name, "b");
    }
}
