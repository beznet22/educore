//! # Production CommandDispatcher
//!
//! Per `docs/architecture.md` § "Command Bus + Dispatcher",
//! every state-changing command flows through a single
//! [`CommandDispatcher::dispatch`] call. The dispatcher
//! enforces the engine's transactional guarantees at one
//! well-defined seam instead of being duplicated (and drifted)
//! at every service call site.
//!
//! ## Pipeline (all in a single transaction)
//!
//! Per `docs/architecture.md`, the dispatcher wraps every
//! service call with the following six steps inside one
//! transaction:
//!
//! 1. **RBAC check** — calls
//!    [`CapabilityCheck::has`] for every capability in the
//!    command's `required_capabilities` slice. The first
//!    failed capability returns [`DomainError::Forbidden`]
//!    before any storage I/O is performed.
//! 2. **Begin transaction** — opens a
//!    [`Transaction`](crate::Transaction) on the
//!    supplied [`StorageAdapter`]. All subsequent reads,
//!    writes, outbox appends, audit row appends, and
//!    idempotency record writes are staged against this
//!    transaction.
//! 3. **Idempotency lookup** — if the command carries an
//!    [`IdempotencyKey`], the dispatcher looks up
//!    `(school_id, command_type, idempotency_key)` in the
//!    idempotency store. A hit indicates a retry of an
//!    already-executed command; the dispatcher rolls the
//!    transaction back and returns
//!    [`DomainError::Conflict`] so the caller can replay the
//!    original outcome via a separate `lookup` call.
//! 4. **Service call** — invokes the caller-supplied closure,
//!    which performs the domain business logic and returns
//!    `(aggregate, event)`.
//! 5. **Outbox write** — serializes the event to a
//!    [`SerializedEnvelope`](crate::SerializedEnvelope)
//!    and stages it on `txn.outbox().append(...)`. The outbox
//!    write is part of the same transaction as the service
//!    call, so the event becomes durable atomically with the
//!    aggregate state (transactional outbox pattern per
//!    `docs/architecture.md` § "Event Architecture").
//! 6. **Audit row write** — stages an
//!    [`AuditLogEntry`](crate::AuditLogEntry) on
//!    `txn.audit_log().append(...)` referencing the event id
//!    and carrying the serialized event envelope as the
//!    `after` snapshot.
//! 7. **Idempotency record** (if the command carries a key)
//!    — stages an [`IdempotencyRecord`] on
//!    `txn.idempotency().record_outcome(...)` so the
//!    structured outcome is captured for replay. A conflict
//!    (same key, different outcome) is treated as a hard
//!    failure: the transaction is rolled back and
//!    [`DomainError::Conflict`] is returned.
//! 8. **Commit** — `txn.commit().await?`. All staged writes
//!    (aggregate mutations, outbox, audit row, idempotency
//!    record) become durable atomically.
//! 9. **Bus publish** — after a successful commit, publishes
//!    the [`EventEnvelope`](crate::envelope::EventEnvelope)
//!    to the [`EventBus`] so consumers see the event with
//!    at-least-once semantics. Publishing after commit (not
//!    before) guarantees no ghost events on rollback; the
//!    bus-side relay in `educore-event-bus` provides the
//!    consumer-side dedupe via `event_id`.
//!
//! ## Why a local `CapabilityCheck` trait?
//!
//! The dispatcher lives in a cross-cutting crate and cannot
//! depend on `educore-rbac` (the typed `Capability` enum and
//! `CapabilityCheck` port) without creating a workspace
//! cycle: `educore-rbac` already depends on `educore-events`
//! to emit `CapabilityAssigned` / `CapabilityRevoked`
//! events, and the dispatcher needs to sit above both
//! `educore-events` and `educore-storage`. The RBAC step is
//! therefore typed against a string-based port
//! ([`CapabilityCheck::has`] takes `&str`) and consumers
//! bridge the typed `educore_rbac::services::CapabilityCheck`
//! port (which takes the [`Capability`](educore_rbac::value_objects::Capability)
//! enum) to this port via a thin adapter that calls
//! `.as_str()` on the typed capability. The adapter lives in
//! the consumer crate; the canonical adapter pattern is
//! documented in `docs/ports/authentication.md`.
//!
//! Every capability string used here is the dotted
//! `domain.aggregate.action` form (e.g.
//! `"academic.student.create"`) — the same string form
//! [`Capability::as_str()`](educore_rbac::value_objects::Capability::as_str)
//! returns. This keeps the dispatcher's RBAC checks
//! string-stable across RBAC catalogue refactors (the
//! canonical authority for capabilities remains
//! `docs/specs/rbac/aggregates.md`).
//!
//! ## Errors
//!
//! Every fallible call returns
//! [`Result<_, DomainError>`](educore_core::error::Result).
//! The dispatcher translates:
//!
//! - `CapabilityCheck::has` returning `false` →
//!   [`DomainError::Forbidden`] with the missing capability
//!   stringified.
//! - Idempotency lookup hit → [`DomainError::Conflict`].
//! - Idempotency `record_outcome` returning
//!   [`IdempotencyOutcome::Conflict`](crate::IdempotencyOutcome::Conflict)
//!   → [`DomainError::Conflict`].
//! - All other errors propagate unchanged.

use std::future::Future;
use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument, warn};

use educore_core::clock::{Clock, IdGenerator};
use educore_core::error::{DomainError, Result};
use educore_core::ids::{IdempotencyKey, SchoolId};
use educore_core::tenant::TenantContext;
use educore_core::value_objects::ActiveStatus;
use educore_events::domain_event::DomainEvent;
use educore_events::envelope::EventEnvelope;
use educore_events::event_bus::EventBus;
use educore_storage::AuditLogEntry;
use educore_storage::idempotency::{
    IdempotencyCompositeKey, IdempotencyOutcome, IdempotencyRecord,
};
use educore_storage::outbox::SerializedEnvelope;
use educore_storage::StorageAdapter;

/// The capability-check port used by [`CommandDispatcher`].
///
/// The dispatcher calls [`CapabilityCheck::has`] at the
/// command boundary with each capability string in
/// `required_capabilities`. This is the dispatcher-side
/// counterpart of `RbacPort::require` from
/// `docs/ports/authentication.md`; the dispatcher's
/// implementation is "call `has` and surface `Forbidden` on
/// `false`", which is exactly what `require` does.
///
/// The trait operates on **strings** rather than the typed
/// `educore_rbac::value_objects::Capability` enum to avoid a
/// workspace dependency cycle (see the
/// [module-level docs](self) for the rationale). Consumers
/// bridge the typed RBAC port to this string port with a
/// one-line adapter:
///
/// ```ignore
/// // In the consumer crate (or in educore-rbac once the
/// // cycle is broken at the workspace level):
/// struct RbacCapabilityAdapter(Arc<dyn educore_rbac::services::CapabilityCheck>);
/// #[async_trait]
/// impl educore_dispatcher::dispatcher::CapabilityCheck for RbacCapabilityAdapter {
///     async fn has(&self, ctx: &TenantContext, cap: &str) -> Result<bool> {
///         let typed = educore_rbac::value_objects::Capability::from_str(cap)
///             .map_err(|_| DomainError::validation(format!("unknown capability: {cap}")))?;
///         self.0.has(ctx, typed).await
///     }
/// }
/// ```
///
/// Every capability string is the dotted
/// `<Domain>.<Aggregate>.<Action>` form (e.g.
/// `"academic.student.create"`), matching
/// [`Capability::as_str()`](educore_rbac::value_objects::Capability::as_str)'s
/// output.
#[async_trait]
pub trait CapabilityCheck: Send + Sync {
    /// Returns `true` if the actor in `ctx` holds the
    /// capability identified by `capability`. The
    /// `capability` string is the dotted
    /// `<Domain>.<Aggregate>.<Action>` form documented in
    /// `docs/specs/rbac/aggregates.md`.
    async fn has(&self, ctx: &TenantContext, capability: &str) -> Result<bool>;
}

/// The trait every command type must satisfy to be dispatchable.
///
/// The dispatcher reads five pieces of metadata from each
/// command:
///
/// - [`tenant`](Self::tenant) — the [`TenantContext`] for the
///   active request. Used to drive the RBAC check, stamp the
///   outbox envelope, and anchor the audit row.
/// - [`command_type`](Self::command_type) — the dotted
///   `domain.aggregate.verb` string (e.g.
///   `"academic.student.admit"`). Used as the idempotency
///   store's `command_type` key.
/// - [`idempotency_key`](Self::idempotency_key) — the caller's
///   [`IdempotencyKey`], or `None` for commands that do not
///   participate in idempotency replay (e.g. system jobs that
///   are explicitly retried by the operator).
/// - [`action`](Self::action) — the audit row's `action` verb
///   (`"create"`, `"update"`, `"admit"`, …). Drives
///   observability dashboards and the auditor UI.
/// - [`target_type`](Self::target_type) — the audit row's
///   `target_type` (the aggregate name in lowercase singular,
///   e.g. `"student"`). Drives per-aggregate audit queries.
///
/// Implementations are typically a thin `impl` block alongside
/// the command struct. The trait has no methods that take
/// `&mut self`, so it is `dyn`-safe (callers can hold
/// `Box<dyn CommandBounds>` if they need to dispatch
/// heterogeneously, though the generic-shaped `dispatch` API
/// does not require it).
pub trait CommandBounds {
    /// Returns the active [`TenantContext`] for the command.
    fn tenant(&self) -> &TenantContext;

    /// Returns the dotted `domain.aggregate.verb` string
    /// identifying the command type.
    fn command_type(&self) -> &'static str;

    /// Returns the caller's [`IdempotencyKey`], or `None` if
    /// the command does not participate in idempotency
    /// replay.
    fn idempotency_key(&self) -> Option<IdempotencyKey>;

    /// Returns the audit row's `action` verb (e.g. `"create"`,
    /// `"update"`, `"admit"`).
    fn action(&self) -> &'static str;

    /// Returns the audit row's `target_type` (the aggregate
    /// name, e.g. `"student"`).
    fn target_type(&self) -> &'static str;
}

/// The production command dispatcher.
///
/// Holds `Arc`s to every port the pipeline touches. Built
/// once per engine process and shared via `Arc<CommandDispatcher>`
/// across request handlers, background jobs, and CLI entry
/// points.
///
/// # Object safety
///
/// The dispatcher itself is `Send + Sync` (its fields are all
/// `Arc<dyn Port + Send + Sync>`), so consumers can hold
/// `Arc<CommandDispatcher>` and dispatch from any async
/// runtime. The `dispatch` method is generic over the command
/// type, the closure shape, and the resulting `(Agg, Ev)`
/// tuple, so it cannot be put behind `dyn CommandDispatcher`
/// — the trait-object surface is reserved for the (rare) case
/// where a single dispatcher must handle heterogeneous command
/// types from a `Box<dyn CommandBounds>`.
///
/// `Debug` is **not** derived because the port trait objects
/// (`Arc<dyn CapabilityCheck>`, `Arc<dyn Clock>`, `Arc<dyn IdGenerator>`)
/// are not required to implement `Debug`. Consumers that need
/// `Debug` for logging can wrap the dispatcher in their own
/// newtype.
pub struct CommandDispatcher {
    /// The storage adapter. Every command runs inside a
    /// transaction opened on this adapter.
    storage: Arc<dyn StorageAdapter>,
    /// The capability check port. The dispatcher calls
    /// [`CapabilityCheck::has`] (the dispatcher-side
    /// equivalent of `RbacPort::require` from
    /// `docs/ports/authentication.md`) for each capability in
    /// the command's `required_capabilities` slice.
    rbac: Arc<dyn CapabilityCheck>,
    /// The bus adapter. The dispatcher publishes the
    /// post-commit [`EventEnvelope`] here so subscribers see
    /// the event with at-least-once semantics.
    bus: Arc<dyn EventBus>,
    /// The clock port. The dispatcher reads the wall-clock
    /// instant for `occurred_at` on the audit row and the
    /// `recorded_at` on the idempotency record.
    clock: Arc<dyn Clock>,
    /// The id generator port. Reserved for future use (e.g.
    /// minting a fresh `correlation_id` for retries). Held
    /// now so the dispatcher can be extended without a
    /// breaking constructor change.
    id_gen: Arc<dyn IdGenerator>,
}

impl CommandDispatcher {
    /// Constructs a new `CommandDispatcher`.
    ///
    /// All five ports are required. Pass concrete adapter
    /// types from the engine's `crates/adapters/` tree in
    /// production; pass testkit mocks (or inline test
    /// doubles) in unit tests.
    #[must_use]
    pub fn new(
        storage: Arc<dyn StorageAdapter>,
        rbac: Arc<dyn CapabilityCheck>,
        bus: Arc<dyn EventBus>,
        clock: Arc<dyn Clock>,
        id_gen: Arc<dyn IdGenerator>,
    ) -> Self {
        Self {
            storage,
            rbac,
            bus,
            clock,
            id_gen,
        }
    }

    /// Returns the storage adapter. Exposed for callers that
    /// need to begin a transaction outside the dispatcher's
    /// pipeline (e.g. read-only queries that still need to
    /// share the dispatcher's clock + id generator). Mutating
    /// writes should always go through `dispatch`.
    #[must_use]
    pub fn storage(&self) -> &Arc<dyn StorageAdapter> {
        &self.storage
    }

    /// Returns the bus adapter. Exposed so callers can
    /// publish application-level events (e.g. health-check
    /// pings, operator notifications) without going through
    /// the command pipeline.
    #[must_use]
    pub fn bus(&self) -> &Arc<dyn EventBus> {
        &self.bus
    }

    /// Dispatches a single command.
    ///
    /// See the [module-level docs](self) for the full
    /// pipeline. The method is generic over:
    ///
    /// - `C: CommandBounds` — the command type.
    /// - `F: FnOnce() -> Fut` — the service-call closure. The
    ///   closure captures whatever domain-specific state it
    ///   needs (a repository handle, a policy engine, an
    ///   in-process cache) and returns a future that yields
    ///   `(Agg, Ev)` on success.
    /// - `Agg` — the resulting aggregate. Returned to the
    ///   caller verbatim; the dispatcher does not introspect
    ///   it.
    /// - `Ev` — the domain event. Must implement
    ///   [`DomainEvent`] + [`Serialize`] so the dispatcher can
    ///   build the [`EventEnvelope`], stage it on the outbox,
    ///   and reference its `event_id` from the audit row.
    ///
    /// `required_capabilities` is a slice of dotted
    /// `<Domain>.<Aggregate>.<Action>` strings (the form
    /// [`Capability::as_str()`](educore_rbac::value_objects::Capability::as_str)
    /// returns). The dispatcher forwards each string verbatim
    /// to [`CapabilityCheck::has`].
    ///
    /// # Errors
    ///
    /// - [`DomainError::Forbidden`] if any capability in
    ///   `required_capabilities` is denied.
    /// - [`DomainError::Conflict`] if the idempotency store
    ///   already holds a record for the same
    ///   `(school_id, command_type, idempotency_key)`
    ///   composite key, or if a concurrent duplicate-key
    ///   write is detected at `record_outcome` time.
    /// - [`DomainError::Infrastructure`] for any underlying
    ///   storage, RBAC, or bus failure.
    /// - Any error returned by `service_call` propagates
    ///   unchanged; the transaction is rolled back via the
    ///   [`Transaction`](crate::Transaction)'s drop
    ///   contract (PORT-STORE-014).
    #[instrument(
        name = "command_dispatcher.dispatch",
        skip_all,
        fields(
            command_type = command.command_type(),
            action = command.action(),
            target_type = command.target_type(),
            school_id = %command.tenant().school_id,
            actor_id = %command.tenant().actor_id,
            correlation_id = %command.tenant().correlation_id,
            has_idempotency_key = command.idempotency_key().is_some(),
        )
    )]
    pub async fn dispatch<C, F, Fut, Agg, Ev>(
        &self,
        command: &C,
        required_capabilities: &[&str],
        service_call: F,
    ) -> Result<(Agg, Ev)>
    where
        C: CommandBounds,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<(Agg, Ev)>>,
        Ev: DomainEvent + Serialize + serde::de::DeserializeOwned,
    {
        let tenant = command.tenant();
        let command_type = command.command_type();
        let idempotency_key = command.idempotency_key();

        // -----------------------------------------------------------------
        // Step 1: RBAC check.
        //
        // The first failed capability short-circuits before any
        // storage I/O. We log the failure (with the actor and
        // missing capability) at `warn` level so the
        // security-monitor pipeline can surface unauthorised
        // attempts without flooding `info`.
        // -----------------------------------------------------------------
        for cap in required_capabilities {
            let granted = self.rbac.has(tenant, cap).await?;
            if !granted {
                warn!(
                    capability = %cap,
                    actor_id = %tenant.actor_id,
                    school_id = %tenant.school_id,
                    "rbac: command rejected — missing capability",
                );
                return Err(DomainError::Forbidden(format!(
                    "missing capability {cap}",
                )));
            }
        }
        debug!(
            capabilities = required_capabilities.len(),
            "rbac: all required capabilities granted",
        );

        // -----------------------------------------------------------------
        // Step 2: Begin transaction.
        //
        // All subsequent reads/writes are staged against this
        // transaction. If anything fails before `commit`, the
        // transaction's `Drop` impl rolls back the staged
        // writes (PORT-STORE-014).
        // -----------------------------------------------------------------
        let mut txn = self.storage.begin().await?;

        // -----------------------------------------------------------------
        // Step 3: Idempotency lookup.
        //
        // A hit means a previous run already executed this
        // command; the caller's contract is "do not re-execute
        // side effects on retry". We roll back the empty
        // transaction and surface a structured Conflict so the
        // caller can recover the original outcome via a
        // separate `lookup` call.
        //
        // Commands without an idempotency key (system jobs,
        // operator retried flows) skip this step.
        // -----------------------------------------------------------------
        if let Some(key) = idempotency_key {
            let composite = IdempotencyCompositeKey {
                school_id: tenant.school_id,
                command_type,
                idempotency_key: key,
            };
            if let Some(existing) = txn.idempotency().lookup(composite).await? {
                warn!(
                    command_type,
                    idempotency_key = %key,
                    "idempotency: command already executed — replay required",
                );
                txn.rollback().await?;
                return Err(DomainError::Conflict(format!(
                    "command {command_type} with idempotency_key {key} \
                     already executed; existing record outcome_version={}, \
                     recorded_at={}",
                    existing.outcome_version, existing.recorded_at,
                )));
            }
        }

        // -----------------------------------------------------------------
        // Step 4: Service call.
        //
        // The closure performs the domain business logic and
        // returns `(Agg, Ev)`. Errors propagate unchanged; the
        // transaction's drop contract rolls back the empty
        // staged writes.
        // -----------------------------------------------------------------
        let (aggregate, event) = service_call().await?;
        let envelope = event.into_envelope(tenant);
        debug!(
            event_id = %envelope.event_id,
            event_type = %envelope.event_type,
            aggregate_id = %envelope.aggregate_id,
            "service call returned event",
        );

        // -----------------------------------------------------------------
        // Step 5: Outbox write (transactional outbox pattern).
        //
        // `SerializedEnvelope::from_event_envelope` JSON-encodes
        // the typed event's payload and clones the bus-port
        // envelope fields into the storage-port shape. The
        // outbox row is staged against the current transaction
        // and becomes durable on commit.
        // -----------------------------------------------------------------
        let serialized = SerializedEnvelope::from_event_envelope(&envelope);
        txn.outbox().append(tenant.school_id, serialized).await?;

        // -----------------------------------------------------------------
        // Step 6: Audit row write.
        //
        // The audit row references the event id (for
        // correlation with the event log) and carries the
        // serialized event envelope as the `after` snapshot.
        // `before` is `None`: the dispatcher does not load the
        // pre-image of the aggregate, so it cannot supply a
        // diff. Domains that need a `before` snapshot should
        // stage the row inside the service call closure (which
        // has the loaded aggregate in scope) and the
        // dispatcher's audit row is then written in addition.
        // -----------------------------------------------------------------
        let audit = AuditLogEntry {
            school_id: tenant.school_id,
            actor_id: tenant.actor_id,
            action: command.action().to_owned(),
            target_type: command.target_type().to_owned(),
            target_id: envelope.aggregate_id,
            before: None,
            after: Some(Bytes::from(
                serde_json::to_vec(&envelope).unwrap_or_default(),
            )),
            event_id: Some(envelope.event_id),
            correlation_id: tenant.correlation_id,
            occurred_at: self.clock.now(),
            active_status: ActiveStatus::Active,
            metadata: serde_json::Value::Null,
        };
        txn.audit_log().append(audit).await?;

        // -----------------------------------------------------------------
        // Step 7: Idempotency record (structured outcome).
        //
        // `record_outcome` (preferred over `record`) returns
        // `IdempotencyOutcome::Recorded` for a new write and
        // `IdempotencyOutcome::Conflict { existing }` for a
        // duplicate-key write with a different outcome. The
        // conflict case indicates a concurrent duplicate
        // command (same key, two writers) — a hard failure:
        // rollback and surface Conflict.
        //
        // The `Recorded` case on a duplicate-key write with an
        // *identical* outcome is a successful retry; the
        // adapter is responsible for collapsing it into
        // `Recorded` (the four shipped adapters and the
        // testkit do this). The engine relies on this for
        // at-least-once delivery.
        // -----------------------------------------------------------------
        if let Some(key) = idempotency_key {
            let outcome_bytes =
                Bytes::from(serde_json::to_vec(&envelope).unwrap_or_default());
            let record = IdempotencyRecord {
                school_id: tenant.school_id,
                command_type,
                idempotency_key: key,
                outcome: outcome_bytes,
                outcome_version: envelope.schema_version,
                recorded_at: self.clock.now(),
                affected_aggregate_ids: vec![envelope.aggregate_id],
                ..IdempotencyRecord::default()
            };
            match txn.idempotency().record_outcome(record).await? {
                IdempotencyOutcome::Recorded => {
                    debug!(
                        command_type,
                        idempotency_key = %key,
                        "idempotency: outcome recorded",
                    );
                }
                IdempotencyOutcome::Conflict { existing } => {
                    warn!(
                        command_type,
                        idempotency_key = %key,
                        existing_outcome_version = existing.outcome_version,
                        "idempotency: duplicate key with different outcome — rolling back",
                    );
                    txn.rollback().await?;
                    return Err(DomainError::Conflict(format!(
                        "command {command_type} with idempotency_key {key} \
                         collided with an existing record (outcome_version={}); \
                         refusing to overwrite",
                        existing.outcome_version,
                    )));
                }
            }
        }

        // -----------------------------------------------------------------
        // Step 8: Commit.
        //
        // All staged writes (aggregate mutations in the
        // service call, outbox append, audit row, idempotency
        // record) become durable atomically.
        // -----------------------------------------------------------------
        txn.commit().await?;
        debug!("transaction committed");

        // -----------------------------------------------------------------
        // Step 9: Bus publish (post-commit).
        //
        // Publish AFTER commit so a rollback never produces a
        // ghost event. The bus-port contract provides
        // at-least-once delivery; the relay in
        // `educore-event-bus` consumer-side dedupes by
        // `event_id`.
        // -----------------------------------------------------------------
        let receipt = self.bus.publish(envelope.clone()).await?;
        debug!(
            event_id = %envelope.event_id,
            receipt_id = %receipt.event_id,
            "event published to bus",
        );

        // Reconstruct the typed event from the envelope's JSON
        // payload. `into_envelope` (step 4) consumed the typed
        // event to populate `envelope.payload`; the spec
        // requires `dispatch` to return `(Agg, Ev)` so the
        // caller can chain on the typed value. The
        // reconstruction is a JSON round-trip through
        // `envelope.payload` (already `serde_json::Value`), so
        // it is O(payload size) and incurs no extra adapter
        // round-trip. Consumers that need the envelope itself
        // can hold the return tuple and call
        // `DomainEvent::into_envelope` themselves.
        let event: Ev = serde_json::from_value(envelope.payload.clone())
            .map_err(|e| DomainError::validation(format!(
                "command_dispatcher: cannot reconstruct typed event \
                 from envelope payload ({event_type}): {e}",
                event_type = envelope.event_type,
            )))?;
        Ok((aggregate, event))
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
    //! Unit tests for [`CommandDispatcher`].
    //!
    //! The tests use inline in-memory mocks for the five ports
    //! the dispatcher touches: storage, RBAC, bus, clock,
    //! id generator. The mocks are intentionally minimal —
    //! they record calls and let each test assert what the
    //! dispatcher wrote.
    //!
    //! We do NOT depend on `educore-testkit` from this crate
    //! because the testkit is in the `tools` tier and the
    //! dispatcher is in the `cross-cutting` tier (the
    //! `educore-core::lint` boundary enforcement forbids the
    //! import). The inline mocks are also a faster, hermetic
    //! alternative to wiring the testkit.

    use super::*;
    use std::sync::Mutex;

    use async_trait::async_trait;
    use chrono::{TimeZone, Utc};

    use educore_core::clock::SystemIdGen;
    use educore_core::ids::{CorrelationId, EventId, UserId};
    use educore_core::tenant::UserType;
    use educore_storage::idempotency::{Idempotency, IdempotencyCompositeKey};
    use educore_storage::outbox::{Outbox, SerializedEnvelope};
    use educore_storage::transaction::{TenantTransaction, Transaction};

    use educore_events::envelope::EventEnvelope;
    use educore_events::event_bus::{
        BatchReceipt, EventBus, EventSubscription, PublishReceipt, SubscribeOptions,
    };

    /// Capability strings used in the unit tests. Mirrors the
    /// string form `educore_rbac::value_objects::Capability::as_str()`
    /// returns.
    const CAP_ACADEMIC_STUDENT_CREATE: &str = "academic.student.create";

    // =================================================================
    // Test command + event
    // =================================================================

    /// A minimal command for the unit tests.
    #[derive(Debug)]
    struct TestCommand {
        tenant: TenantContext,
        command_type: &'static str,
        idempotency_key: Option<IdempotencyKey>,
        action: &'static str,
        target_type: &'static str,
    }

    impl CommandBounds for TestCommand {
        fn tenant(&self) -> &TenantContext {
            &self.tenant
        }
        fn command_type(&self) -> &'static str {
            self.command_type
        }
        fn idempotency_key(&self) -> Option<IdempotencyKey> {
            self.idempotency_key
        }
        fn action(&self) -> &'static str {
            self.action
        }
        fn target_type(&self) -> &'static str {
            self.target_type
        }
    }

    /// A minimal aggregate the test command produces.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    struct TestAggregate {
        /// Stable aggregate id.
        id: uuid::Uuid,
        /// Display name.
        name: String,
    }

    /// A minimal domain event the test command emits.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestEvent {
        event_id: EventId,
        aggregate_id: uuid::Uuid,
        school_id: SchoolId,
        occurred_at: educore_core::value_objects::Timestamp,
        name: String,
    }

    impl DomainEvent for TestEvent {
        const EVENT_TYPE: &'static str = "test.aggregate.created";
        const SCHEMA_VERSION: u32 = 1;
        const AGGREGATE_TYPE: &'static str = "test_aggregate";

        fn event_id(&self) -> EventId {
            self.event_id
        }
        fn aggregate_id(&self) -> uuid::Uuid {
            self.aggregate_id
        }
        fn school_id(&self) -> SchoolId {
            self.school_id
        }
        fn occurred_at(&self) -> educore_core::value_objects::Timestamp {
            self.occurred_at
        }
    }

    // =================================================================
    // Inline mocks for the five ports
    // =================================================================

    /// In-memory bus. Records every published envelope so the
    /// test can assert that the dispatcher published exactly
    /// one event after commit.
    #[derive(Debug, Default)]
    struct InMemoryBus {
        published: Mutex<Vec<EventEnvelope>>,
    }

    impl InMemoryBus {
        fn new() -> Self {
            Self::default()
        }
        fn snapshot(&self) -> Vec<EventEnvelope> {
            lock_unpoisoned(&self.published).clone()
        }
    }

    #[async_trait]
    impl EventBus for InMemoryBus {
        async fn publish(&self, envelope: EventEnvelope) -> Result<PublishReceipt> {
            let receipt = PublishReceipt {
                event_id: envelope.event_id,
                topic: "default".to_owned(),
                accepted_at: envelope.occurred_at,
            };
            lock_unpoisoned(&self.published).push(envelope);
            Ok(receipt)
        }

        async fn publish_batch(
            &self,
            _envelopes: Vec<EventEnvelope>,
        ) -> Result<BatchReceipt> {
            Err(DomainError::not_supported(
                "InMemoryBus::publish_batch is not used by the dispatcher tests",
            ))
        }

        async fn subscribe(
            &self,
            _options: SubscribeOptions,
        ) -> Result<Box<dyn EventSubscription>> {
            Err(DomainError::not_supported(
                "InMemoryBus::subscribe is not used by the dispatcher tests",
            ))
        }
    }

    /// In-memory outbox. Records every `append` call so the
    /// test can assert that the dispatcher staged exactly one
    /// outbox row.
    #[derive(Debug, Default)]
    struct InMemoryOutbox {
        rows: Mutex<Vec<SerializedEnvelope>>,
    }

    impl InMemoryOutbox {
        fn new() -> Self {
            Self::default()
        }
        fn snapshot(&self) -> Vec<SerializedEnvelope> {
            lock_unpoisoned(&self.rows).clone()
        }
    }

    #[async_trait]
    impl Outbox for InMemoryOutbox {
        async fn append(
            &self,
            school_id: SchoolId,
            envelope: SerializedEnvelope,
        ) -> Result<()> {
            assert_eq!(
                envelope.school_id, school_id,
                "Outbox::append: school_id argument must match envelope.school_id",
            );
            lock_unpoisoned(&self.rows).push(envelope);
            Ok(())
        }

        async fn pending(
            &self,
            _school_id: SchoolId,
            _limit: u32,
        ) -> Result<Vec<SerializedEnvelope>> {
            Ok(Vec::new())
        }

        async fn mark_published(
            &self,
            _school_id: SchoolId,
            _ids: &[EventId],
        ) -> Result<()> {
            Ok(())
        }
    }

    /// In-memory audit log. Records every `append` call.
    #[derive(Debug, Default)]
    struct InMemoryAuditLog {
        rows: Mutex<Vec<AuditLogEntry>>,
    }

    impl InMemoryAuditLog {
        fn new() -> Self {
            Self::default()
        }
        fn snapshot(&self) -> Vec<AuditLogEntry> {
            lock_unpoisoned(&self.rows).clone()
        }
    }

    #[async_trait]
    impl educore_storage::audit::AuditLog for InMemoryAuditLog {
        async fn append(&self, entry: AuditLogEntry) -> Result<()> {
            lock_unpoisoned(&self.rows).push(entry);
            Ok(())
        }

        async fn read_for_target(
            &self,
            _school_id: SchoolId,
            _target_id: uuid::Uuid,
            _limit: u32,
        ) -> Result<Vec<AuditLogEntry>> {
            Ok(Vec::new())
        }
    }

    /// In-memory idempotency store. Detects duplicate-key
    /// collisions via `record_outcome` (mirrors the
    /// `MockIdempotency` in `educore-storage::idempotency`'s
    /// own test suite).
    #[derive(Debug, Default)]
    struct InMemoryIdempotency {
        records: Mutex<std::collections::HashMap<IdempotencyCompositeKey, IdempotencyRecord>>,
    }

    impl InMemoryIdempotency {
        fn new() -> Self {
            Self::default()
        }
        fn len(&self) -> usize {
            lock_unpoisoned(&self.records).len()
        }
    }

    #[async_trait]
    impl educore_storage::idempotency::Idempotency for InMemoryIdempotency {
        async fn lookup(
            &self,
            key: IdempotencyCompositeKey,
        ) -> Result<Option<IdempotencyRecord>> {
            Ok(lock_unpoisoned(&self.records).get(&key).cloned())
        }

        async fn record(&self, record: IdempotencyRecord) -> Result<()> {
            let key = IdempotencyRecord::composite_key(
                record.school_id,
                record.command_type,
                record.idempotency_key,
            );
            lock_unpoisoned(&self.records).insert(key, record);
            Ok(())
        }

        async fn record_outcome(
            &self,
            record: IdempotencyRecord,
        ) -> Result<IdempotencyOutcome> {
            let key = IdempotencyRecord::composite_key(
                record.school_id,
                record.command_type,
                record.idempotency_key,
            );
            let mut store = lock_unpoisoned(&self.records);
            if let Some(existing) = store.get(&key) {
                if existing.outcome == record.outcome {
                    return Ok(IdempotencyOutcome::Recorded);
                }
                return Ok(IdempotencyOutcome::Conflict {
                    existing: existing.clone(),
                });
            }
            store.insert(key, record);
            Ok(IdempotencyOutcome::Recorded)
        }
    }

    /// In-memory transaction. Forwards every sub-port to the
    /// shared mocks. `commit` and `rollback` are no-ops (the
    /// sub-ports write directly to the shared state).
    #[derive(Debug)]
    struct InMemoryTransaction {
        school_id: SchoolId,
        outbox: Arc<InMemoryOutbox>,
        audit_log: Arc<InMemoryAuditLog>,
        idempotency: Arc<InMemoryIdempotency>,
        event_log: StubEventLog,
    }

    #[async_trait]
    impl Transaction for InMemoryTransaction {
        async fn commit(self: Box<Self>) -> Result<()> {
            Ok(())
        }
        async fn rollback(self: Box<Self>) -> Result<()> {
            Ok(())
        }
        fn outbox(&self) -> &dyn Outbox {
            &*self.outbox
        }
        fn audit_log(&self) -> &dyn educore_storage::audit::AuditLog {
            &*self.audit_log
        }
        fn idempotency(&self) -> &dyn educore_storage::idempotency::Idempotency {
            &*self.idempotency
        }
        fn event_log(&self) -> &dyn educore_storage::event_log::EventLog {
            &self.event_log
        }
    }

    impl TenantTransaction for InMemoryTransaction {
        fn school_id(&self) -> SchoolId {
            self.school_id
        }
    }

    /// In-memory storage adapter. Hands out a fresh
    /// `InMemoryTransaction` per `begin` call.
    #[derive(Debug)]
    struct InMemoryStorage {
        outbox: Arc<InMemoryOutbox>,
        audit_log: Arc<InMemoryAuditLog>,
        idempotency: Arc<InMemoryIdempotency>,
    }

    impl InMemoryStorage {
        fn new(
            outbox: Arc<InMemoryOutbox>,
            audit_log: Arc<InMemoryAuditLog>,
            idempotency: Arc<InMemoryIdempotency>,
        ) -> Self {
            Self {
                outbox,
                audit_log,
                idempotency,
            }
        }
    }

    #[async_trait]
    impl StorageAdapter for InMemoryStorage {
        async fn begin(&self) -> Result<Box<dyn Transaction>> {
            Ok(Box::new(InMemoryTransaction {
                school_id: SchoolId(uuid::Uuid::nil()),
                outbox: self.outbox.clone(),
                audit_log: self.audit_log.clone(),
                idempotency: self.idempotency.clone(),
                event_log: StubEventLog,
            }))
        }
        async fn migrate(&self) -> Result<educore_storage::change_stream::MigrationReport> {
            Ok(educore_storage::change_stream::MigrationReport { version: 0, statements_executed: 0, duration: std::time::Duration::from_secs(0), already_at_version: true })
        }
        async fn ping(&self) -> Result<()> {
            Ok(())
        }
        async fn close(self: Box<Self>) -> Result<()> {
            Ok(())
        }
    }

    /// Stub `EventLog`. The dispatcher does not call
    /// `event_log`; we still need an impl so the transaction
    /// can return `&dyn EventLog`.
    #[derive(Debug, Default)]
    struct StubEventLog;

    #[async_trait]
    impl educore_storage::event_log::EventLog for StubEventLog {
        async fn append(&self, _entry: educore_storage::event_log::EventLogEntry) -> Result<()> {
            Ok(())
        }
        async fn read(
            &self,
            _filter: educore_storage::event_log::EventLogFilter,
        ) -> Result<Vec<educore_storage::event_log::EventLogEntry>> {
            Ok(Vec::new())
        }
        async fn count(
            &self,
            _filter: educore_storage::event_log::EventLogFilter,
        ) -> Result<u64> {
            Ok(0)
        }
    }

    /// RBAC mock. Records whether each capability string is
    /// granted (configured per-test via `allow_caps`).
    #[derive(Debug)]
    struct StubRbac {
        allow_caps: Mutex<std::collections::BTreeSet<String>>,
    }

    impl StubRbac {
        fn new(allow_caps: std::collections::BTreeSet<String>) -> Self {
            Self {
                allow_caps: Mutex::new(allow_caps),
            }
        }
    }

    #[async_trait]
    impl super::CapabilityCheck for StubRbac {
        async fn has(&self, _ctx: &TenantContext, capability: &str) -> Result<bool> {
            Ok(lock_unpoisoned(&self.allow_caps).contains(capability))
        }
    }

    // =================================================================
    // Helpers
    // =================================================================

    /// Lock a mutex, recovering from poisoning by returning
    /// the inner value. Mirrors the helper used elsewhere in
    /// the codebase.
    fn lock_unpoisoned<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
        match m.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    fn ts(secs: i64) -> educore_core::value_objects::Timestamp {
        educore_core::value_objects::Timestamp::from_datetime(
            Utc.timestamp_opt(secs, 0).single().unwrap_or_else(Utc::now),
        )
    }

    fn make_ctx(school: SchoolId, actor: UserId) -> TenantContext {
        TenantContext::for_user(
            school,
            actor,
            CorrelationId(uuid::Uuid::now_v7()),
            UserType::Teacher,
        )
    }

    fn make_dispatcher(
        allow_caps: std::collections::BTreeSet<String>,
    ) -> (
        CommandDispatcher,
        Arc<InMemoryBus>,
        Arc<InMemoryOutbox>,
        Arc<InMemoryAuditLog>,
        Arc<InMemoryIdempotency>,
    ) {
        let outbox = Arc::new(InMemoryOutbox::new());
        let audit_log = Arc::new(InMemoryAuditLog::new());
        let idempotency = Arc::new(InMemoryIdempotency::new());
        let bus = Arc::new(InMemoryBus::new());
        let storage: Arc<dyn StorageAdapter> = Arc::new(InMemoryStorage::new(
            outbox.clone(),
            audit_log.clone(),
            idempotency.clone(),
        ));
        let rbac: Arc<dyn super::CapabilityCheck> = Arc::new(StubRbac::new(allow_caps));
        let clock: Arc<dyn Clock> = Arc::new(educore_core::clock::TestClock::at(ts(1_700_000_000)));
        let id_gen: Arc<dyn IdGenerator> = Arc::new(SystemIdGen);

        let dispatcher = CommandDispatcher::new(storage, rbac, bus.clone(), clock, id_gen);
        (dispatcher, bus, outbox, audit_log, idempotency)
    }

    fn make_command(
        school: SchoolId,
        actor: UserId,
        command_type: &'static str,
        idempotency_key: Option<IdempotencyKey>,
        action: &'static str,
        target_type: &'static str,
    ) -> TestCommand {
        TestCommand {
            tenant: make_ctx(school, actor),
            command_type,
            idempotency_key,
            action,
            target_type,
        }
    }

    fn sample_service_call(
        school: SchoolId,
    ) -> impl FnOnce() -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(TestAggregate, TestEvent)>> + Send>,
    > {
        let g = SystemIdGen;
        let aggregate_id = g.next_uuid();
        let event_id = g.next_event_id();
        let school_inner = school;
        move || {
            Box::pin(async move {
                Ok((
                    TestAggregate {
                        id: aggregate_id,
                        name: "test".to_owned(),
                    },
                    TestEvent {
                        event_id,
                        aggregate_id,
                        school_id: school_inner,
                        occurred_at: ts(1_700_000_001),
                        name: "test".to_owned(),
                    },
                ))
            })
        }
    }

    // =================================================================
    // Tests
    // =================================================================

    /// Happy path: RBAC granted, no idempotency key, service
    /// call succeeds. The dispatcher must:
    /// - Stage one outbox row
    /// - Stage one audit row
    /// - NOT stage an idempotency record (no key)
    /// - Commit
    /// - Publish exactly one event
    #[tokio::test]
    async fn dispatch_happy_path_writes_outbox_audit_and_publishes() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let actor = g.next_user_id();

        let mut allow = std::collections::BTreeSet::new();
        allow.insert(CAP_ACADEMIC_STUDENT_CREATE.to_owned());
        let (dispatcher, bus, outbox, audit_log, idempotency) = make_dispatcher(allow);

        let cmd = make_command(
            school,
            actor,
            "academic.student.admit",
            None,
            "create",
            "student",
        );

        let (agg, _event) = dispatcher
            .dispatch(&cmd, &[CAP_ACADEMIC_STUDENT_CREATE], || {
                sample_service_call(school)()
            })
            .await
            .expect("happy-path dispatch must succeed");

        assert_eq!(agg.name, "test");
        assert_eq!(outbox.snapshot().len(), 1, "one outbox row must be staged");
        assert_eq!(audit_log.snapshot().len(), 1, "one audit row must be staged");
        assert_eq!(idempotency.len(), 0, "no idempotency record must be staged when key is absent");
        assert_eq!(bus.snapshot().len(), 1, "exactly one event must be published post-commit");

        let audit = &audit_log.snapshot()[0];
        assert_eq!(audit.action, "create");
        assert_eq!(audit.target_type, "student");
        assert_eq!(audit.school_id, school);
        assert_eq!(audit.actor_id, actor);
    }

    /// RBAC denial: the dispatcher must reject the command
    /// before opening a transaction or invoking the service
    /// call.
    #[tokio::test]
    async fn dispatch_rbac_denied_returns_forbidden_without_io() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let actor = g.next_user_id();

        // Empty allow set: every capability is denied.
        let (dispatcher, bus, outbox, audit_log, idempotency) =
            make_dispatcher(std::collections::BTreeSet::new());

        let cmd = make_command(
            school,
            actor,
            "academic.student.admit",
            None,
            "create",
            "student",
        );

        let result = dispatcher
            .dispatch(&cmd, &[CAP_ACADEMIC_STUDENT_CREATE], || {
                sample_service_call(school)()
            })
            .await;

        assert!(matches!(result, Err(DomainError::Forbidden(_))));
        assert_eq!(outbox.snapshot().len(), 0, "no outbox writes on RBAC denial");
        assert_eq!(audit_log.snapshot().len(), 0, "no audit rows on RBAC denial");
        assert_eq!(idempotency.len(), 0);
        assert_eq!(bus.snapshot().len(), 0, "no bus publishes on RBAC denial");
    }

    /// Idempotency replay: a previous run with the same
    /// composite key must cause the second dispatch to fail
    /// with `Conflict` BEFORE the service call runs.
    #[tokio::test]
    async fn dispatch_idempotency_hit_returns_conflict_without_service_call() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let actor = g.next_user_id();
        let key = g.next_idempotency_key();

        let mut allow = std::collections::BTreeSet::new();
        allow.insert(CAP_ACADEMIC_STUDENT_CREATE.to_owned());
        let (dispatcher, bus, outbox, audit_log, idempotency) = make_dispatcher(allow);

        // Pre-seed an idempotency record so the dispatcher's
        // lookup hits.
        let pre_existing = IdempotencyRecord {
            school_id: school,
            command_type: "academic.student.admit",
            idempotency_key: key,
            outcome: Bytes::from_static(b"previously-stored-envelope"),
            outcome_version: 1,
            recorded_at: ts(1_700_000_000),
            affected_aggregate_ids: vec![g.next_uuid()],
            aggregate_version: 1,
            duration_ms: 100,
            emitted_event_ids: vec![],
            etag: Some("test-etag".to_owned()),
        };
        idempotency
            .record(pre_existing)
            .await
            .expect("pre-seeding idempotency must succeed");

        let cmd = make_command(
            school,
            actor,
            "academic.student.admit",
            Some(key),
            "create",
            "student",
        );

        let result = dispatcher
            .dispatch(&cmd, &[CAP_ACADEMIC_STUDENT_CREATE], || {
                sample_service_call(school)()
            })
            .await;

        assert!(
            matches!(result, Err(DomainError::Conflict(_))),
            "idempotency hit must surface Conflict, got {result:?}",
        );
        assert_eq!(outbox.snapshot().len(), 0);
        assert_eq!(audit_log.snapshot().len(), 0);
        assert_eq!(bus.snapshot().len(), 0);
        assert_eq!(
            idempotency.len(),
            1,
            "the pre-existing record must remain; no overwrite on Conflict",
        );
    }

    /// Idempotency record write: a successful first dispatch
    /// with a key must stage exactly one idempotency record
    /// alongside the outbox row and audit row.
    #[tokio::test]
    async fn dispatch_records_idempotency_outcome_on_success() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let actor = g.next_user_id();
        let key = g.next_idempotency_key();

        let mut allow = std::collections::BTreeSet::new();
        allow.insert(CAP_ACADEMIC_STUDENT_CREATE.to_owned());
        let (dispatcher, _bus, _outbox, _audit_log, idempotency) = make_dispatcher(allow);

        let cmd = make_command(
            school,
            actor,
            "academic.student.admit",
            Some(key),
            "create",
            "student",
        );

        let (_agg, _event) = dispatcher
            .dispatch(&cmd, &[CAP_ACADEMIC_STUDENT_CREATE], || {
                sample_service_call(school)()
            })
            .await
            .expect("first dispatch must succeed");

        assert_eq!(
            idempotency.len(),
            1,
            "one idempotency record must be staged after a successful dispatch",
        );
    }

    /// Service-call error: the closure returns Err. The
    /// dispatcher must NOT commit, NOT publish, NOT stage an
    /// outbox row, NOT stage an audit row. The transaction's
    /// drop contract handles the rollback in production; the
    /// in-memory transaction's `commit` is a no-op anyway, so
    /// the test asserts "no side effects" instead.
    #[tokio::test]
    async fn dispatch_service_call_error_skips_side_effects() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let actor = g.next_user_id();

        let mut allow = std::collections::BTreeSet::new();
        allow.insert(CAP_ACADEMIC_STUDENT_CREATE.to_owned());
        let (dispatcher, bus, outbox, audit_log, idempotency) = make_dispatcher(allow);

        let cmd = make_command(
            school,
            actor,
            "academic.student.admit",
            None,
            "create",
            "student",
        );

        let result: Result<(TestAggregate, TestEvent)> = dispatcher
            .dispatch(&cmd, &[CAP_ACADEMIC_STUDENT_CREATE], || async {
                Err(DomainError::validation("service-call forced failure"))
            })
            .await;

        assert!(matches!(result, Err(DomainError::Validation(_))));
        assert_eq!(outbox.snapshot().len(), 0);
        assert_eq!(audit_log.snapshot().len(), 0);
        assert_eq!(idempotency.len(), 0);
        assert_eq!(bus.snapshot().len(), 0);
    }
}
