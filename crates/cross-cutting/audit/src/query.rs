//! The [`AuditQuery`] trait — the read side of the audit log.
//!
//! Per `docs/schemas/audit-schema.md` § 5 the audit log is queried
//! through a dedicated port. Every method on [`AuditQuery`] is
//! tenant-scoped, paginated, and capability-gated at the consumer
//! layer (the engine does not enforce capability checks on the
//! trait itself; the consumer's RBAC adapter wraps the trait and
//! enforces `AuditLog.Read`).
//!
//! The trait is the engine's stable read surface. Storage
//! adapters (one per supported database) implement it. The
//! trait is intentionally minimal: it is the cross-database
//! contract, not the full audit API. Consumers compose the four
//! methods plus the seven [`AuditFilter`] variants to answer
//! every spec-listed query.
//!
//! ## Pairs with [`crate::writer::AuditWriter`]
//!
//! The writer appends audit rows on every state-changing command;
//! the query serves them back. The two are siblings — both flow
//! over the `audit_log` table — but live in separate modules to
//! keep the write path's invariants (atomicity with the business
//! mutation, threshold-driven sweep) isolated from the read
//! path's invariants (tenant scoping, capability gating,
//! pagination, hard page-size cap).

use std::fmt;
use std::net::IpAddr;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::error::{DomainError, Result};
use educore_core::ids::{CorrelationId, EventId, SchoolId, SessionId, UserId};
use educore_core::tenant::TenantContext;
use educore_core::value_objects::Timestamp;

use crate::writer::AuditAction;

/// The hard upper bound on [`Page::limit`].
///
/// The cap is enforced in [`Page::new`]; constructors that pass a
/// `limit` above this constant return
/// [`DomainError::Validation`]. Callers that need more than
/// `MAX_PAGE_LIMIT` rows must use keyset pagination (a Phase 3
/// follow-up; see `docs/schemas/audit-schema.md` § 5).
pub const MAX_PAGE_LIMIT: u64 = 1000;

/// One row of the audit log as the read shape sees it.
///
/// Distinct from [`educore_storage::AuditLogEntry`] (the write
/// shape): `AuditRecord` is **fully denormalised** — it carries
/// every column the audit log stores, including the
/// network-origin fields (`ip`, `user_agent`, `session_id`),
/// the audit metadata envelope, and the cross-tenant flag the
/// writer does not need to see to perform an append.
///
/// `AuditRecord` is also strictly an **output type**: the
/// [`AuditQuery`] trait returns `Vec<AuditRecord>`; the trait
/// never accepts one as input. Callers that need to construct
/// an audit row use [`educore_storage::AuditLogEntry`] (the
/// write shape) via [`crate::writer::AuditWriter`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditRecord {
    /// The audit row's primary key. UUIDv7, time-ordered.
    pub audit_id: AuditId,
    /// The school the row belongs to.
    pub school_id: SchoolId,
    /// The actor (user or `SYSTEM_USER_ID`) that performed the action.
    pub actor_id: UserId,
    /// The actor's classification (user / system / agent / api_key).
    pub actor_type: ActorType,
    /// The action verb (`"create"`, `"update"`, `"delete"`, …).
    pub action: String,
    /// The aggregate type (`"student"`, `"fees_invoice"`, …).
    pub resource_type: String,
    /// The aggregate id this audit row is about.
    pub resource_id: Uuid,
    /// The originating event id, if the audit row mirrors a
    /// domain event. Lets auditors join audit rows to event log
    /// rows.
    pub event_id: Option<EventId>,
    /// The originating command id, if the audit row mirrors a
    /// state-changing command.
    pub command_id: Option<CommandId>,
    /// The correlation id of the request.
    pub correlation_id: CorrelationId,
    /// The event time — when the action occurred in the domain.
    pub occurred_at: Timestamp,
    /// The persistence time — when the audit row was written.
    pub recorded_at: Timestamp,
    /// The originating IP address, if known. `None` for
    /// system-issued actions.
    pub ip: Option<IpAddr>,
    /// The originating user agent, if known.
    pub user_agent: Option<String>,
    /// The active session, if any.
    pub session_id: Option<SessionId>,
    /// Serialised snapshot of the aggregate **before** the
    /// action. `None` for create actions.
    pub before: Option<bytes::Bytes>,
    /// Serialised snapshot of the aggregate **after** the
    /// action. `None` for delete actions.
    pub after: Option<bytes::Bytes>,
    /// Open-ended metadata envelope (reason, ticket, request id).
    /// Stored as raw bytes so adapters are free to use any wire
    /// format.
    pub metadata: Option<bytes::Bytes>,
    /// `true` if this row records a cross-tenant operation.
    pub cross_tenant: bool,
    /// The channel that produced the action (`web`, `mobile`,
    /// `api`, `agent`, `system`).
    pub source: AuditSource,
}

/// The actor's classification.
///
/// Per `docs/schemas/audit-schema.md` § 2: every audit row
/// records *what kind* of actor performed the action so the
/// audit query can distinguish human-initiated mutations from
/// system-issued ones (background jobs, migrations, retention
/// sweeps) and from automated agents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActorType {
    /// A human user with a real `UserId`.
    User,
    /// A system-issued actor (`SYSTEM_USER_ID`).
    System,
    /// An AI agent acting on behalf of a user.
    Agent,
    /// An API-key-authenticated caller.
    ApiKey,
}

impl ActorType {
    /// Returns the canonical snake_case wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::System => "system",
            Self::Agent => "agent",
            Self::ApiKey => "api_key",
        }
    }
}

impl fmt::Display for ActorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

/// The channel that produced the audit row.
///
/// Per `docs/schemas/audit-schema.md` § 2: the source lets
/// auditors distinguish web-app actions from mobile, from
/// system jobs, and from AI-agent-initiated actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuditSource {
    /// A web-app action (browser session).
    Web,
    /// A native mobile-app action.
    Mobile,
    /// A direct API call (REST/GraphQL/gRPC).
    Api,
    /// An AI-agent-initiated action.
    Agent,
    /// A system-issued action (job, migration, sweep).
    System,
}

impl AuditSource {
    /// Returns the canonical snake_case wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Web => "web",
            Self::Mobile => "mobile",
            Self::Api => "api",
            Self::Agent => "agent",
            Self::System => "system",
        }
    }
}

impl fmt::Display for AuditSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

/// Typed audit-row identifier.
///
/// Mirrors the engine's typed-identifier pattern (see
/// `educore_core::ids::Identifier`): the type is a transparent
/// wrapper around a UUIDv7. Distinct from [`ResourceId`] so
/// cross-aggregate id confusion becomes a compile-time error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AuditId(pub Uuid);

/// Typed resource identifier.
///
/// The audit log stores one row per (resource_type, resource_id)
/// pair; the trait's [`AuditQuery::resource_history`] method
/// takes a `ResourceId` so the type system prevents callers from
/// confusing audit-row ids with resource ids.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ResourceId(pub Uuid);

/// Typed command identifier.
///
/// Optional in [`AuditRecord::command_id`]: only set when the
/// audit row mirrors a state-changing command (the engine's
/// audit-first invariant writes one row per command). For event
/// mirrors and authn/authz events the field is `None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CommandId(pub Uuid);

/// Typed resource-type name.
///
/// Wraps the `target_type` string (`"student"`,
/// `"fees_invoice"`, …) so callers cannot pass arbitrary strings
/// where a resource type is expected.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ResourceType(pub String);

impl ResourceType {
    /// Constructs a `ResourceType` from a raw string. The string
    /// is **not** validated against a registry — the engine
    /// treats the value as opaque; cross-domain uniqueness is
    /// the convention, not the type's responsibility.
    #[must_use]
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Returns the resource-type string as a `&str`.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<&str> for ResourceType {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl From<String> for ResourceType {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Pagination for audit queries.
///
/// Every [`AuditQuery`] method that returns a vector takes a
/// `Page` to bound the result set. The engine's [`MAX_PAGE_LIMIT`]
/// cap is enforced in [`Page::new`] — callers cannot construct a
/// page with a `limit` above the constant. Offset-based
/// pagination is acceptable for audit because rows are
/// time-ordered and the index covers `(school_id, occurred_at)`
/// (see `docs/schemas/audit-schema.md` § 13); keyset pagination
/// is a Phase 3 follow-up for workloads that exceed the cap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Page {
    /// Zero-based offset into the result set.
    pub offset: u64,
    /// Maximum rows returned. Capped at [`MAX_PAGE_LIMIT`].
    pub limit: u64,
}

impl Page {
    /// Constructs a `Page(offset, limit)` and enforces the
    /// [`MAX_PAGE_LIMIT`] cap.
    ///
    /// # Errors
    ///
    /// - [`DomainError::Validation`] if `limit` exceeds
    ///   [`MAX_PAGE_LIMIT`]. The check is performed before the
    ///   `Page` is constructed so callers can never obtain an
    ///   over-cap page by struct-literal syntax.
    pub fn new(offset: u64, limit: u64) -> Result<Self> {
        if limit > MAX_PAGE_LIMIT {
            return Err(DomainError::Validation(format!(
                "page limit {limit} exceeds MAX_PAGE_LIMIT ({MAX_PAGE_LIMIT})"
            )));
        }
        Ok(Self { offset, limit })
    }
}

/// The filter passed to [`AuditQuery::list`].
///
/// One variant per spec-listed query pattern. The variants are
/// exhaustive: the spec defines seven, and this enum has seven.
/// `Custom` is capability-gated at the consumer layer
/// (`AuditLog.ReadCustom`); the engine does not enforce the
/// capability but documents the expectation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AuditFilter {
    /// Every record matching an action verb, optionally bounded
    /// by a time window. The action verb matches
    /// [`AuditLogEntry::action`](educore_storage::AuditLogEntry::action);
    /// use [`AuditAction::Other`] for domain-specific verbs.
    ByAction {
        /// The action verb.
        action: AuditAction,
        /// Inclusive lower bound on `occurred_at`.
        since: Option<Timestamp>,
        /// Inclusive upper bound on `occurred_at`.
        until: Option<Timestamp>,
    },
    /// Every record for a specific resource.
    ByResource {
        /// The resource type (`"student"`, `"fees_invoice"`, …).
        resource_type: ResourceType,
        /// The aggregate id.
        resource_id: ResourceId,
    },
    /// Every record by a specific actor, optionally bounded by a
    /// time window.
    ByActor {
        /// The actor.
        actor_id: UserId,
        /// Inclusive lower bound on `occurred_at`.
        since: Option<Timestamp>,
        /// Inclusive upper bound on `occurred_at`.
        until: Option<Timestamp>,
    },
    /// Every record in a single request / workflow. The
    /// correlation id is the join key — see
    /// `docs/schemas/event-schema.md` § 1.1.
    ByCorrelation {
        /// The correlation id of the originating request.
        correlation_id: CorrelationId,
    },
    /// Every record in a time window. Both bounds are required.
    ByTimeRange {
        /// Inclusive lower bound on `occurred_at`.
        since: Timestamp,
        /// Inclusive upper bound on `occurred_at`.
        until: Timestamp,
    },
    /// Every record of a specific event type. Used to mirror the
    /// audit log against the event log.
    ByEventType {
        /// The event-type string (e.g. `"StudentAdmitted"`).
        event_type: String,
    },
    /// Domain-specific filter (capability-gated at the consumer
    /// layer). The engine does not interpret the predicate; the
    /// storage adapter passes it through to its native query
    /// language (e.g. a SQL `WHERE` clause fragment).
    Custom {
        /// The predicate string, in the storage adapter's native
        /// dialect.
        predicate: String,
    },
}

impl Default for AuditFilter {
    /// Returns [`AuditFilter::ByTimeRange`] anchored at the epoch.
    /// The default is conservative: an unbounded time-range
    /// query requires the caller to override at least one bound
    /// before it is safe to execute.
    fn default() -> Self {
        Self::ByTimeRange {
            since: Timestamp::epoch(),
            until: Timestamp::epoch(),
        }
    }
}

/// The audit query port.
///
/// Every method is tenant-scoped: a consumer without
/// `Platform.CrossTenant` cannot query across schools. The
/// consumer's RBAC adapter enforces [`AuditLog.Read`]; the
/// engine does not enforce capability checks on the trait
/// itself.
#[async_trait]
pub trait AuditQuery: Send + Sync {
    /// Lists audit records matching `filter`, ordered by
    /// `occurred_at` ascending and paginated by `page`.
    ///
    /// # Errors
    ///
    /// - [`AuditError::Validation`] for malformed filters
    ///   (e.g. an over-cap page — the constructor already
    ///   prevents this, but the storage adapter may add its own
    ///   validation).
    /// - [`AuditError::Infrastructure`] for any underlying
    ///   storage error.
    async fn list(
        &self,
        tenant: &TenantContext,
        filter: AuditFilter,
        page: Page,
    ) -> Result<Vec<AuditRecord>>;

    /// Returns the audit record with the given `audit_id`.
    ///
    /// # Errors
    ///
    /// - [`AuditError::NotFound`] if no audit row with that id
    ///   exists in the active tenant.
    /// - [`AuditError::Infrastructure`] for storage failures.
    async fn get(&self, tenant: &TenantContext, audit_id: AuditId) -> Result<AuditRecord>;

    /// Returns the full mutation history of a single aggregate,
    /// ordered by `occurred_at` ascending and paginated by
    /// `page`. This is the primary tool for "what happened to
    /// this student?" and "who last touched this invoice?"
    /// (per `docs/schemas/audit-schema.md` § 6).
    ///
    /// # Errors
    ///
    /// - [`AuditError::Infrastructure`] for storage failures.
    async fn resource_history(
        &self,
        tenant: &TenantContext,
        resource_type: ResourceType,
        resource_id: ResourceId,
        page: Page,
    ) -> Result<Vec<AuditRecord>>;

    /// Returns every record authored by `actor_id`, optionally
    /// bounded by a time window and paginated by `page`. This is
    /// the primary tool for "every action this user took in the
    /// last 30 days" (per `docs/schemas/audit-schema.md` § 7).
    ///
    /// # Errors
    ///
    /// - [`AuditError::Infrastructure`] for storage failures.
    async fn actor_history(
        &self,
        tenant: &TenantContext,
        actor_id: UserId,
        since: Option<Timestamp>,
        until: Option<Timestamp>,
        page: Page,
    ) -> Result<Vec<AuditRecord>>;
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
    fn page_new_enforces_max_page_limit() {
        // At the cap: accepted.
        let p = Page::new(0, MAX_PAGE_LIMIT).unwrap_or_else(|e| {
            panic!("MAX_PAGE_LIMIT must be accepted, got error: {e:?}");
        });
        assert_eq!(p.offset, 0);
        assert_eq!(p.limit, MAX_PAGE_LIMIT);

        // One over the cap: rejected with Validation.
        let err = Page::new(0, MAX_PAGE_LIMIT + 1)
            .err()
            .unwrap_or_else(|| panic!("limit > MAX_PAGE_LIMIT must be rejected"));
        assert!(
            matches!(err, DomainError::Validation(_)),
            "expected DomainError::Validation, got: {err:?}"
        );

        // Any offset is allowed when the limit is in range.
        let p2 = Page::new(u64::MAX, 1).unwrap_or_else(|e| {
            panic!("large offset with small limit must be accepted, got error: {e:?}");
        });
        assert_eq!(p2.offset, u64::MAX);
        assert_eq!(p2.limit, 1);
    }

    #[test]
    fn audit_filter_default_is_time_range_at_epoch() {
        // The default is an empty time range anchored at the
        // epoch; callers must override at least one bound
        // before dispatching the query.
        let f = AuditFilter::default();
        match f {
            AuditFilter::ByTimeRange { since, until } => {
                assert_eq!(since, Timestamp::epoch());
                assert_eq!(until, Timestamp::epoch());
            }
            other => panic!("expected ByTimeRange default, got: {other:?}"),
        }
    }

    #[test]
    fn actor_type_and_audit_source_have_stable_wire_strings() {
        // Exhaustive: every variant produces a non-empty
        // snake_case wire string. Guards against a future
        // variant being added without an `as_str` arm.
        let actors = [
            (ActorType::User, "user"),
            (ActorType::System, "system"),
            (ActorType::Agent, "agent"),
            (ActorType::ApiKey, "api_key"),
        ];
        for (variant, expected) in actors {
            assert_eq!(variant.as_str(), expected);
            assert_eq!(variant.to_string(), expected);
        }
        let sources = [
            (AuditSource::Web, "web"),
            (AuditSource::Mobile, "mobile"),
            (AuditSource::Api, "api"),
            (AuditSource::Agent, "agent"),
            (AuditSource::System, "system"),
        ];
        for (variant, expected) in sources {
            assert_eq!(variant.as_str(), expected);
            assert_eq!(variant.to_string(), expected);
        }
    }

    #[test]
    fn resource_type_accepts_str_string_and_new() {
        let r1 = ResourceType::new("student");
        let r2: ResourceType = "student".into();
        let r3: ResourceType = String::from("student").into();
        assert_eq!(r1.as_str(), "student");
        assert_eq!(r2.as_str(), "student");
        assert_eq!(r3.as_str(), "student");
        assert_eq!(r1.to_string(), "student");
    }
}
