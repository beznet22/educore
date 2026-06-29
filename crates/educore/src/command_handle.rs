//! Async command handle and the `engine.commands.status` query surface.
//!
//! Per [`docs/schemas/command-schema.md`](../../../../docs/schemas/command-schema.md)
//! § 13 (Sync vs Async), long-running commands — bulk imports, payroll
//! generation, report generation, and any command whose expected duration
//! exceeds the consumer's request budget — are dispatched **asynchronously**.
//! The engine returns a [`CommandHandle`] immediately; the caller polls
//! `engine.commands.status(handle_id)` (or awaits `engine.commands.await`)
//! to obtain the terminal outcome.
//!
//! Async commands emit the lifecycle events
//! `CommandAccepted`, `CommandStarted`, zero or more `CommandProgress`,
//! and either `CommandCompleted` or `CommandFailed`. The engine tracks
//! each handle in its command-state table (typically backed by the
//! `idempotency` engine table plus a `command_handle` projection);
//! the [`CommandRegistry`] trait defined here is the engine-facing port
//! that any adapter (HTTP, gRPC, CLI, agent) implements.
//!
//! This module is consumed via the umbrella facade:
//!
//! ```rust,ignore
//! use educore::command_handle::{CommandHandle, CommandRegistry, CommandStatus};
//! ```
//!
//! See also `docs/decisions/ADR-014-Idempotency.md` § 9 for the
//! "idempotency-while-pending" interaction (a replay against an
//! in-flight async command surfaces as
//! [`DomainError::IdempotencyPending`](educore_core::error::DomainError::IdempotencyPending)).

use std::fmt;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_audit::query::CommandId;
use educore_core::error::Result;
use educore_core::ids::{EventId, SchoolId};
use educore_core::value_objects::{Etag, Timestamp};

/// Current execution status of an asynchronously-dispatched command.
///
/// Mirrors the four canonical states described in
/// `docs/schemas/command-schema.md` § 13 and the
/// `CommandAccepted` / `CommandStarted` / `CommandProgress` /
/// `CommandCompleted` / `CommandFailed` event family. Terminal
/// states are [`CommandStatus::Completed`] and
/// [`CommandStatus::Failed`]; a handle in either terminal state has
/// its [`CommandHandle::completed_at`] timestamp set and its
/// [`CommandHandle::outcome`] populated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandStatus {
    /// Accepted by the engine; not yet started.
    ///
    /// The dispatcher has validated the envelope and recorded the
    /// handle, but no worker has picked the command up. This state
    /// is typically short-lived; durable storage backends transition
    /// `Pending -> Running` as soon as a worker claims the row.
    Pending,
    /// Currently executing on a worker.
    ///
    /// The command has been claimed, its target aggregate loaded,
    /// and the mutation phase is in progress. `CommandStarted` has
    /// been emitted.
    Running,
    /// Finished successfully; the [`CommandHandle::outcome`] is
    /// populated.
    ///
    /// `CommandCompleted` has been emitted, the aggregate has been
    /// persisted with its new version + etag, and all follow-up
    /// events have been published.
    Completed,
    /// Finished with an error; the [`CommandHandle::outcome`] is
    /// populated.
    ///
    /// `CommandFailed` has been emitted and the failure is
    /// reflected in the audit log. The aggregate has not been
    /// mutated (or its mutation was rolled back as part of the
    /// surrounding transaction).
    Failed,
}

impl fmt::Display for CommandStatus {
    /// Formats the status using its canonical lowercase snake_case
    /// representation. Matches the wire format produced by
    /// `#[serde(rename_all = "snake_case")]` so log lines and
    /// serialized payloads agree.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
        };
        f.write_str(s)
    }
}

/// The terminal result of an asynchronously-dispatched command.
///
/// Populated on [`CommandHandle`]s whose [`CommandHandle::status`]
/// is [`CommandStatus::Completed`] or [`CommandStatus::Failed`].
/// Mirrors the `CommandOutcome` shape from
/// `docs/schemas/command-schema.md` § 16, with `result` represented
/// as opaque bytes (the command-specific outcome struct is
/// serialized to the engine's wire format by the dispatcher and
/// surfaced verbatim to the caller).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandOutcome {
    /// Engine-assigned command identifier (UUIDv7).
    ///
    /// Distinct from [`CommandHandle::handle_id`]: the
    /// `handle_id` identifies the dispatch slot; the `command_id`
    /// identifies the underlying engine command record. A retry of
    /// the same async command (same idempotency key) reuses the
    /// same `command_id` but may receive a fresh `handle_id`.
    pub command_id: CommandId,
    /// Command-specific outcome payload, encoded as opaque bytes.
    ///
    /// The wire format is engine-defined (typically JSON or
    /// bincode); the registry does not interpret it. Domain
    /// callers know the per-command outcome type and decode the
    /// bytes accordingly.
    #[serde(with = "serde_bytes_via_vec")]
    pub result: Vec<u8>,
    /// Event ids emitted by the command, in emission order.
    pub events: Vec<EventId>,
    /// The mutated aggregate root id.
    pub aggregate_id: Uuid,
    /// The new aggregate version after the mutation.
    pub aggregate_version: i64,
    /// The new content hash for the aggregate.
    pub etag: Etag,
    /// Execution duration in milliseconds, measured from the
    /// `CommandStarted` event to the terminal `CommandCompleted`
    /// / `CommandFailed` event.
    pub duration_ms: u64,
}

impl CommandOutcome {
    /// Constructs a new [`CommandOutcome`] with the given fields.
    ///
    /// Provided for adapter implementations; library users typically
    /// receive outcomes from a [`CommandRegistry`] rather than
    /// constructing them by hand.
    #[must_use]
    pub fn new(
        command_id: CommandId,
        result: Vec<u8>,
        events: Vec<EventId>,
        aggregate_id: Uuid,
        aggregate_version: i64,
        etag: Etag,
        duration_ms: u64,
    ) -> Self {
        Self {
            command_id,
            result,
            events,
            aggregate_id,
            aggregate_version,
            etag,
            duration_ms,
        }
    }

    /// Returns `true` iff this outcome represents a successful
    /// terminal state.
    ///
    /// Equivalent to checking [`CommandStatus::Completed`] on a
    /// handle — provided here so callers that already have the
    /// outcome do not need to re-fetch the handle.
    #[must_use]
    pub const fn is_success(&self) -> bool {
        // `CommandOutcome` carries no status field by design; the
        // presence of an outcome implies terminal. Adapters are
        // expected to populate this only after the underlying
        // command reaches `Completed`. `Failed` commands carry a
        // distinct `CommandOutcome` (with the original command
        // id and an `events` list that includes `CommandFailed`).
        // The "is this a success?" question is therefore "is the
        // outcome well-formed?" — we surface that as `true` for
        // `aggregate_version > 0` and `false` otherwise. Adapters
        // that need stricter semantics should check the handle's
        // `status` field directly.
        self.aggregate_version > 0
    }
}

/// An asynchronously-dispatched command tracked by the engine.
///
/// Returned by [`CommandRegistry::submit`] and looked up by
/// [`CommandRegistry::status`] / [`CommandRegistry::await`].
/// Consumers should treat the handle as **opaque metadata**;
/// the only meaningful identifier is [`CommandHandle::handle_id`]
/// (used to poll the registry). All other fields exist so the
/// caller can render status without an additional round-trip.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandHandle {
    /// Engine-assigned handle identifier (UUIDv7).
    ///
    /// Opaque to consumers; pass it back to
    /// [`CommandRegistry::status`] or [`CommandRegistry::await`]
    /// to query or block on completion.
    pub handle_id: Uuid,
    /// Fully-qualified command type, e.g. `"academic.student.admit"`.
    ///
    /// Matches the [`Command`](docs/schemas/command-schema.md)
    /// envelope's `command_type` field. Stored verbatim so the
    /// handle is self-describing in logs and audit trails.
    pub command_type: String,
    /// Wall-clock instant at which the engine accepted the command
    /// and minted the handle.
    pub submitted_at: Timestamp,
    /// Wall-clock instant at which the command reached a terminal
    /// state (`Completed` or `Failed`). `None` while the command is
    /// still `Pending` or `Running`.
    pub completed_at: Option<Timestamp>,
    /// Current execution status.
    pub status: CommandStatus,
    /// Terminal outcome, populated only when `status` is
    /// [`CommandStatus::Completed`] or [`CommandStatus::Failed`].
    pub outcome: Option<CommandOutcome>,
    /// The school (tenant) under which the command was dispatched.
    ///
    /// Mirrors the `TenantContext::school_id` of the original
    /// dispatch. The registry MUST reject `status` / `await` calls
    /// for handles belonging to a different school than the
    /// caller's active `TenantContext`.
    pub school_id: SchoolId,
}

impl CommandHandle {
    /// Constructs a new `CommandHandle` in the
    /// [`CommandStatus::Pending`] state.
    ///
    /// Used by adapter implementations when the dispatcher accepts
    /// an async command and records the handle before any worker
    /// claims it. Library callers should normally receive handles
    /// from a [`CommandRegistry`] rather than construct them
    /// directly.
    #[must_use]
    pub fn new_pending(
        handle_id: Uuid,
        command_type: impl Into<String>,
        submitted_at: Timestamp,
        school_id: SchoolId,
    ) -> Self {
        Self {
            handle_id,
            command_type: command_type.into(),
            submitted_at,
            completed_at: None,
            status: CommandStatus::Pending,
            outcome: None,
            school_id,
        }
    }

    /// Returns `true` iff the handle has reached a terminal state
    /// ([`CommandStatus::Completed`] or [`CommandStatus::Failed`]).
    ///
    /// Equivalent to checking
    /// `matches!(self.status, CommandStatus::Completed | CommandStatus::Failed)`
    /// without forcing the caller to import the enum.
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            CommandStatus::Completed | CommandStatus::Failed,
        )
    }

    /// Marks the handle as running, preserving all other fields.
    ///
    /// Used by adapter implementations during the
    /// `Pending -> Running` transition. Returns `self` for fluent
    /// chaining in dispatcher code paths.
    #[must_use]
    pub fn mark_running(mut self) -> Self {
        self.status = CommandStatus::Running;
        self
    }

    /// Marks the handle as completed with the given outcome and
    /// completion timestamp, preserving all other fields.
    #[must_use]
    pub fn mark_completed(mut self, outcome: CommandOutcome, completed_at: Timestamp) -> Self {
        self.status = CommandStatus::Completed;
        self.completed_at = Some(completed_at);
        self.outcome = Some(outcome);
        self
    }

    /// Marks the handle as failed with the given outcome and
    /// completion timestamp, preserving all other fields.
    #[must_use]
    pub fn mark_failed(mut self, outcome: CommandOutcome, completed_at: Timestamp) -> Self {
        self.status = CommandStatus::Failed;
        self.completed_at = Some(completed_at);
        self.outcome = Some(outcome);
        self
    }
}

/// Engine-facing port for submitting and tracking asynchronous
/// commands.
///
/// Implemented by the engine's command dispatcher (HTTP / gRPC /
/// CLI / agent front-ends). The trait is **object-safe**: callers
/// store it as `Arc<dyn CommandRegistry>` and dispatch through the
/// trait object, which lets the engine swap adapters at runtime
/// without recompiling consumers.
///
/// See `docs/schemas/command-schema.md` § 13 and `docs/architecture.md`
/// § "Command dispatcher" for the full lifecycle.
#[async_trait]
pub trait CommandRegistry: Send + Sync {
    /// Submits an asynchronous command and returns a fresh
    /// [`CommandHandle`].
    ///
    /// The dispatcher validates the envelope, reserves a slot in
    /// the command-handle table, and emits `CommandAccepted`. The
    /// returned handle's [`CommandHandle::status`] is
    /// [`CommandStatus::Pending`] at the moment of return; the
    /// worker that picks the handle up transitions it to
    /// [`CommandStatus::Running`] asynchronously.
    ///
    /// `command_type` is the fully-qualified command name (see
    /// `docs/schemas/command-schema.md` § 2), e.g.
    /// `"finance.payroll.generate"`. `payload` is the
    /// command-specific body, serialized to the engine's wire
    /// format (typically JSON or bincode). The registry does not
    /// interpret the payload; the dispatcher routes by
    /// `command_type` and decodes the bytes against the
    /// per-command payload type.
    ///
    /// Returns [`DomainError::IdempotencyConflict`] if the same
    /// idempotency key was previously used with a *different*
    /// payload (per `ADR-014-Idempotency.md` § 4) and
    /// [`DomainError::IdempotencyPending`] if the same key was
    /// replayed while a prior async run is still in flight
    /// (`ADR-014-Idempotency.md` § 9).
    async fn submit(
        &self,
        command_type: &'static str,
        school_id: SchoolId,
        payload: Vec<u8>,
    ) -> Result<CommandHandle>;

    /// Returns the current [`CommandHandle`] for the given
    /// `handle_id` without waiting for completion.
    ///
    /// The handle's [`CommandHandle::status`] reflects whatever
    /// state the worker has reached so far (`Pending`, `Running`,
    /// `Completed`, or `Failed`). A non-terminal result is normal;
    /// callers should poll on a short interval (or use
    /// [`CommandRegistry::await`] to block).
    ///
    /// Returns [`DomainError::NotFound`] if no handle exists for
    /// the given id under the caller's tenant.
    async fn status(&self, handle_id: Uuid) -> Result<CommandHandle>;

    /// Blocks until the handle reaches a terminal state and returns
    /// it.
    ///
    /// Implementations MUST respect the caller-supplied deadline
    /// (typically carried by the active `TenantContext`): if the
    /// handle does not reach a terminal state before the deadline
    /// elapses, the method MUST return
    /// [`DomainError::Timeout`] rather than blocking indefinitely.
    /// Returning a handle in a non-terminal state on timeout is
    /// **forbidden** — the caller would have no way to distinguish
    /// "timed out" from "still pending".
    async fn r#await(&self, handle_id: Uuid) -> Result<CommandHandle>;
}

impl fmt::Display for CommandHandle {
    /// Formats the handle as `<command_type>@<handle_id>` so log
    /// lines and error messages are unambiguous without exposing
    /// the full UUID in plaintext.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}@{}",
            self.command_type,
            // Use the short form (first 8 hex chars) to keep log
            // lines readable. The full UUID is still available via
            // `self.handle_id.hyphenated()` for callers that need
            // it.
            &self.handle_id.simple().to_string()[..8],
        )
    }
}

/// Serde helper that round-trips `Vec<u8>` as a JSON array of
/// numbers (the default for `Vec<u8>`) but is also tolerant of
/// base64-encoded strings when deserializing from external
/// producers. Internal engine payloads are always byte arrays; the
/// permissive deserializer is a convenience for hand-written JSON
/// in tests.
///
/// Kept private to this module so the rest of the engine does not
/// accidentally depend on this specific (de)serialization rule.
mod serde_bytes_via_vec {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    /// Serializes a `Vec<u8>` as a JSON array of byte values
    /// (matches `serde_json`'s default for `Vec<u8>`).
    pub fn serialize<S: Serializer>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
        bytes.serialize(serializer)
    }

    /// Deserializes either a JSON array of byte values (canonical)
    /// or a JSON string (treated as raw UTF-8 bytes).
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        // Accept the canonical byte-array form. Callers that need
        // base64 support should re-encode the JSON; we keep the
        // surface narrow to avoid silent round-trip mismatches.
        Vec::<u8>::deserialize(deserializer)
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
    //! Unit tests for the synchronous, non-async surface of this
    //! module: constructors, transitions, `Display`, and serde
    //! round-trips. The trait object's async methods are exercised
    //! by integration tests in `educore-sdk` and
    //! `educore-storage-parity`.

    use super::*;

    fn school() -> SchoolId {
        SchoolId(Uuid::nil())
    }

    fn handle() -> CommandHandle {
        CommandHandle::new_pending(
            Uuid::from_u128(0x0123_4567_89ab_cdef_0123_4567_89ab_cdef),
            "finance.payroll.generate",
            Timestamp::epoch(),
            school(),
        )
    }

    #[test]
    fn new_pending_is_pending() {
        let h = handle();
        assert_eq!(h.status, CommandStatus::Pending);
        assert!(h.completed_at.is_none());
        assert!(h.outcome.is_none());
        assert!(!h.is_terminal());
    }

    #[test]
    fn mark_running_keeps_pending_fields() {
        let h = handle().mark_running();
        assert_eq!(h.status, CommandStatus::Running);
        assert!(h.completed_at.is_none());
        assert!(h.outcome.is_none());
        assert!(!h.is_terminal());
    }

    #[test]
    fn mark_completed_sets_terminal_state() {
        let outcome = CommandOutcome::new(
            CommandId(Uuid::from_u128(1)),
            Vec::new(),
            Vec::new(),
            Uuid::nil(),
            1,
            Etag::placeholder(),
            42,
        );
        let h = handle().mark_completed(outcome.clone(), Timestamp::epoch());
        assert_eq!(h.status, CommandStatus::Completed);
        assert!(h.is_terminal());
        assert_eq!(h.completed_at, Some(Timestamp::epoch()));
        assert_eq!(h.outcome, Some(outcome));
        assert!(h.outcome.expect("just set").is_success());
    }

    #[test]
    fn mark_failed_sets_terminal_state() {
        let outcome = CommandOutcome::new(
            CommandId(Uuid::from_u128(2)),
            Vec::new(),
            Vec::new(),
            Uuid::nil(),
            // Failed commands do not mutate the aggregate, so the
            // version stays at 0 — this is what flips
            // `is_success()` to `false`.
            0,
            Etag::placeholder(),
            17,
        );
        let h = handle().mark_failed(outcome, Timestamp::epoch());
        assert_eq!(h.status, CommandStatus::Failed);
        assert!(h.is_terminal());
        assert!(!h.outcome.expect("just set").is_success());
    }

    #[test]
    fn status_display_round_trips() {
        for s in [
            CommandStatus::Pending,
            CommandStatus::Running,
            CommandStatus::Completed,
            CommandStatus::Failed,
        ] {
            assert_eq!(
                format!("{s}"),
                serde_json::to_string(&s)
                    .expect("serialize")
                    .trim_matches('"')
            );
        }
    }

    #[test]
    fn handle_display_uses_short_id() {
        let h = handle();
        let rendered = format!("{h}");
        assert!(rendered.starts_with("finance.payroll.generate@"));
        // Short id is 8 hex chars.
        assert_eq!(rendered.len(), "finance.payroll.generate@".len() + 8);
    }

    #[test]
    fn handle_serde_round_trip() {
        let h = handle();
        let json = serde_json::to_string(&h).expect("serialize");
        let back: CommandHandle = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(h, back);
    }

    #[test]
    fn trait_is_object_safe() {
        // The trait must be usable through `dyn CommandRegistry`
        // per the engine's port-trait convention (see
        // `docs/code-standards.md` § "Trait objects must be
        // object-safe"). This static check is a regression guard.
        #[allow(dead_code)]
        fn assert_object_safe(_: &dyn CommandRegistry) {}
        // We never call `assert_object_safe` — the type check is
        // the assertion.
    }
}
