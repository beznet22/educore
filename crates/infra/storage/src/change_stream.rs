//! Types carried across the storage port boundary.
//!
//! Per `docs/ports/storage.md`, the storage port is a thin trait
//! that hands out handles to repository / sub-port abstractions.
//! The types in this module are the wire-format shapes for those
//! handles: change streams, snapshots, cursors, and the migration
//! report.

use std::fmt;
use std::pin::Pin;

use serde::{Deserialize, Serialize};

use educore_core::ids::{EventId, SchoolId};
use educore_core::value_objects::Timestamp;

/// Custom serde adapter for `bytes::Bytes` that round-trips
/// through `Vec<u8>` (see the same module in `outbox.rs` for
/// the rationale).
mod bytes_via_vec {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(value: &bytes::Bytes, ser: S) -> Result<S::Ok, S::Error> {
        value.as_ref().serialize(ser)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(de: D) -> Result<bytes::Bytes, D::Error> {
        let vec = Vec::<u8>::deserialize(de)?;
        Ok(bytes::Bytes::from(vec))
    }
}

/// A filter passed to `StorageAdapter::watch_changes` to scope
/// the change feed. Per `docs/ports/storage.md` § 3.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeFilter {
    /// The school to watch. Mandatory.
    pub school_id: SchoolId,
    /// Optional resume point; if `None`, the stream starts at the
    /// current cursor position for the school.
    pub since: Option<VersionCursor>,
    /// If non-empty, only events for these aggregate types are
    /// streamed. If empty, all aggregates are streamed.
    ///
    /// `String` (not `&'static str`) so the type can be
    /// deserialised from JSON / MessagePack; consumers that
    /// pass a literal can use `String::from("student")`.
    pub aggregate_types: Vec<AggregateTypeFilter>,
}

/// A filter clause for aggregate types. Either a static name or a
/// wildcard.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregateTypeFilter {
    /// Match this exact aggregate type name (e.g. `"student"`).
    Exact(String),
    /// Match any aggregate type. Storage adapters that don't
    /// support wildcards may treat this as "all types".
    Any,
}

impl ChangeFilter {
    /// Constructs a filter for the given school, watching all
    /// aggregates from the current cursor position.
    #[must_use]
    pub fn for_school(school_id: SchoolId) -> Self {
        Self {
            school_id,
            since: None,
            aggregate_types: Vec::new(),
        }
    }

    /// Resumes the filter from the given cursor.
    #[must_use]
    pub fn since(mut self, cursor: VersionCursor) -> Self {
        self.since = Some(cursor);
        self
    }

    /// Restricts the filter to the given aggregate type.
    #[must_use]
    pub fn only_aggregate(mut self, name: &str) -> Self {
        self.aggregate_types
            .push(AggregateTypeFilter::Exact(name.to_owned()));
        self
    }
}

/// A live change feed. Per `docs/ports/storage.md` § 3, the
/// stream is at-least-once; consumers must dedupe by `event_id`.
/// The stream closes when the underlying adapter shuts down or the
/// `ChangeStream` is dropped.
///
/// The inner stream is `Pin<Box<dyn Stream>>` so it can be
/// polled with `.next().await` from an async context.
pub struct ChangeStream {
    /// The inner stream of change events. Boxed and pinned to
    /// keep the type `dyn`-compatible and awaitable.
    pub inner: Pin<
        Box<
            dyn futures::Stream<Item = Result<ChangeEvent, educore_core::error::DomainError>>
                + Send
                + Sync,
        >,
    >,
}

impl ChangeStream {
    /// Polls the next event from the stream. Returns `Ok(None)`
    /// when the stream closes.
    pub async fn next(&mut self) -> Result<Option<ChangeEvent>, educore_core::error::DomainError> {
        use futures::StreamExt;
        self.inner.next().await.transpose()
    }

    /// Closes the stream. After `close`, `next` returns `Ok(None)`.
    pub async fn close(self) -> Result<(), educore_core::error::DomainError> {
        // The default implementation drops the inner stream, which
        // closes the underlying channel. Adapters that need an
        // explicit close handshake can override via a wrapper
        // stream type.
        drop(self);
        Ok(())
    }
}

impl fmt::Debug for ChangeStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChangeStream").finish_non_exhaustive()
    }
}

/// A single change event in the watch stream.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeEvent {
    /// The event id; consumers dedupe on this.
    pub event_id: EventId,
    /// The school the event belongs to.
    pub school_id: SchoolId,
    /// The aggregate type (e.g. `"student"`). `String` (not
    /// `&'static str`) so the type is `DeserializeOwned`.
    pub aggregate_type: String,
    /// The aggregate id.
    pub aggregate_id: uuid::Uuid,
    /// The serialized payload (JSON, MessagePack, etc.; the wire
    /// format is a storage-adapter concern). Uses the custom
    /// `bytes_via_vec` adapter so the parent type is
    /// `DeserializeOwned`.
    #[serde(with = "bytes_via_vec")]
    pub payload: bytes::Bytes,
    /// The event's monotonic position in the school's change log.
    pub cursor: VersionCursor,
}

/// A monotonic, opaque per-school cursor. Implementations may use
/// an LSN, a logical clock, or an event_log id; consumers treat
/// it as an opaque value to pass back to `cursor_for` or
/// `advance_cursor`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct VersionCursor(pub u64);

impl VersionCursor {
    /// The zero cursor. The first event in a school's log is at
    /// `VersionCursor(1)`; the "before the first event" cursor is
    /// `VersionCursor(0)`.
    pub const ZERO: Self = Self(0);

    /// Returns the raw counter.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Returns the next cursor.
    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

impl fmt::Display for VersionCursor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A bulk snapshot of a school used for first-time client
/// hydration. Per `docs/ports/storage.md` § 3 and `docs/ports/sync.md`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchoolSnapshot {
    /// The school the snapshot is for.
    pub school_id: SchoolId,
    /// The cursor position this snapshot was taken at.
    pub cursor: VersionCursor,
    /// The aggregates included in the snapshot, in insertion order.
    pub aggregates: Vec<SnapshotAggregate>,
    /// The events emitted after the snapshot, in order. Consumers
    /// apply the snapshot first, then tail-apply these events to
    /// reach the latest state.
    pub tail_events: Vec<SerializedChangeEvent>,
}

/// A single aggregate row inside a `SchoolSnapshot`. The wire
/// format for `payload` is a storage-adapter concern.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotAggregate {
    /// The aggregate type (e.g. `"student"`). `String` so the
    /// type is `DeserializeOwned`.
    pub aggregate_type: String,
    /// The aggregate id.
    pub aggregate_id: uuid::Uuid,
    /// The serialized payload (JSON, MessagePack, etc.). Uses
    /// the custom `bytes_via_vec` adapter so the parent type
    /// is `DeserializeOwned`.
    #[serde(with = "bytes_via_vec")]
    pub payload: bytes::Bytes,
}

/// A serialized change event inside a `SchoolSnapshot::tail_events`.
/// Mirrors `ChangeEvent` but with a stable wire form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializedChangeEvent {
    /// The event id.
    pub event_id: EventId,
    /// The event's cursor position.
    pub cursor: VersionCursor,
    /// The school the event belongs to.
    pub school_id: SchoolId,
    /// The aggregate type. `String` so the type is
    /// `DeserializeOwned`.
    pub aggregate_type: String,
    /// The aggregate id.
    pub aggregate_id: uuid::Uuid,
    /// The serialized payload. Uses the custom `bytes_via_vec`
    /// adapter so the parent type is `DeserializeOwned`.
    #[serde(with = "bytes_via_vec")]
    pub payload: bytes::Bytes,
    /// The clock time of the event.
    pub occurred_at: Timestamp,
}

/// The result of a `StorageAdapter::migrate` call. Per
/// `docs/ports/storage.md` § 2.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationReport {
    /// The schema version the adapter migrated to.
    pub version: u32,
    /// The number of statements executed (DDL or DML).
    pub statements_executed: u32,
    /// The wall-clock duration of the migration.
    pub duration: std::time::Duration,
    /// Whether the migration was a no-op (already at `version`).
    pub already_at_version: bool,
}

impl fmt::Display for MigrationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "migrated to v{} ({} statements, {:?}{})",
            self.version,
            self.statements_executed,
            self.duration,
            if self.already_at_version {
                ", no-op"
            } else {
                ""
            }
        )
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
    use educore_core::clock::IdGenerator;

    #[test]
    fn version_cursor_zero() {
        assert_eq!(VersionCursor::ZERO.get(), 0);
        assert_eq!(VersionCursor::ZERO.next().get(), 1);
    }

    #[test]
    fn change_filter_for_school() {
        let g = educore_core::clock::SystemIdGen;
        let school = g.next_school_id();
        let f = ChangeFilter::for_school(school);
        assert_eq!(f.school_id, school);
        assert!(f.since.is_none());
        assert!(f.aggregate_types.is_empty());
    }

    #[test]
    fn change_filter_since_and_aggregate() {
        let g = educore_core::clock::SystemIdGen;
        let school = g.next_school_id();
        let f = ChangeFilter::for_school(school)
            .since(VersionCursor(42))
            .only_aggregate("student");
        assert_eq!(f.since, Some(VersionCursor(42)));
        assert_eq!(
            f.aggregate_types,
            vec![AggregateTypeFilter::Exact("student".to_owned())]
        );
    }

    #[test]
    fn migration_report_display() {
        let r = MigrationReport {
            version: 3,
            statements_executed: 12,
            duration: std::time::Duration::from_millis(150),
            already_at_version: false,
        };
        let s = r.to_string();
        assert!(s.contains("v3"));
        assert!(s.contains("12 statements"));
        assert!(!s.contains("no-op"));
    }
}
