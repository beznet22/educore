//! # Conflict resolution port
//!
//! Per `ADR-018-SyncEngine.md` § 6 (and the related
//! `ADR-008-ConflictResolution.md`), the sync engine encounters
//! conflicts during snapshot apply and during live
//! out-of-order event replay. The port is a generic
//! `ConflictResolver` trait that consumers program against:
//!
//! - The **sync engine** holds an `Arc<dyn ConflictResolver>`
//!   and consults it whenever two events describe the same
//!   aggregate with different states.
//! - The **bus replay tool** uses the same trait to resolve
//!   conflicts that surface during event-log replay.
//!
//! This module declares:
//!
//! - [`ConflictKind`] — the typed reason a conflict arose.
//! - [`Conflict<T>`] — the local-vs-remote payload pair.
//! - [`ConflictResolution<T>`] — the resolver's verdict.
//! - [`ConflictResolver`] trait (object-safe) — the contract.
//! - [`LastWriteWinsResolver`] / [`FirstWriteWinsResolver`]
//!   reference impls.
//!
//! ## Object safety
//!
//! The trait is object-safe. The conflict payload `T` is
//! erased at the trait boundary (the trait operates on
//! `Conflict<T>` with `T: Clone`) so `Arc<dyn ConflictResolver>`
//! does not require a generic argument.

use std::marker::PhantomData;

use educore_core::error::{DomainError, Result};
use educore_core::value_objects::Timestamp;

/// The kind of conflict a [`Conflict<T>`] describes.
///
/// Resolvers may treat kinds differently — e.g. a
/// `ConcurrentWrite` might default to last-write-wins, while a
/// `SchemaIncompatible` always rejects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ConflictKind {
    /// Two writes raced; the aggregate version diverged.
    VersionMismatch,
    /// The aggregate was missing on the remote side (e.g.
    /// snapshot hadn't caught up before the live event).
    MissingAggregate,
    /// Two writers updated the aggregate concurrently.
    ConcurrentWrite,
    /// The schemas are incompatible (event has a newer
    /// `SCHEMA_VERSION` than the consumer understands).
    SchemaIncompatible,
}

/// The conflict payload: local vs. remote aggregate state at
/// the moment of divergence.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Conflict<T> {
    /// The local (consumer-side) view of the aggregate.
    pub local: T,
    /// The remote (producer-side) view of the aggregate.
    pub remote: T,
    /// Why the conflict arose.
    pub kind: ConflictKind,
    /// When the conflict was detected.
    pub detected_at: Timestamp,
}

impl<T> Conflict<T> {
    /// Constructs a new conflict payload with `detected_at =
    /// Timestamp::now()`.
    #[must_use]
    pub fn new(local: T, remote: T, kind: ConflictKind) -> Self {
        Self {
            local,
            remote,
            kind,
            detected_at: Timestamp::now(),
        }
    }

    /// Constructs a new conflict payload with an explicit
    /// `detected_at` timestamp (useful for replay tests).
    #[must_use]
    pub fn at(local: T, remote: T, kind: ConflictKind, detected_at: Timestamp) -> Self {
        Self {
            local,
            remote,
            kind,
            detected_at,
        }
    }
}

/// The resolver's verdict on a [`Conflict<T>`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ConflictResolution<T> {
    /// Keep the local view unchanged.
    UseLocal,
    /// Adopt the remote view as the new state.
    UseRemote(T),
    /// Apply a merged aggregate (the result of a custom merge
    /// function supplied by the consumer).
    Merge(T),
    /// Refuse to resolve; surface the conflict to the caller
    /// for manual intervention.
    Reject,
}

impl<T> ConflictResolution<T> {
    /// Returns `Some(&T)` for the verdict's payload (the
    /// winning aggregate), or `None` for `UseLocal` and
    /// `Reject`.
    #[must_use]
    pub fn into_winner(self) -> Option<T> {
        match self {
            Self::UseLocal | Self::Reject => None,
            Self::UseRemote(t) | Self::Merge(t) => Some(t),
        }
    }
}

/// The conflict resolver port.
///
/// Resolvers are pure functions: no I/O, no side effects.
/// They take a [`Conflict<T>`] and return a
/// [`ConflictResolution<T>`] (or an error if the conflict
/// cannot be resolved).
///
/// Resolvers are stateless; the sync engine holds one as
/// `Arc<dyn ConflictResolver>` and consults it on every
/// conflict.
///
/// ## Object safety
///
/// The trait is object-safe. Resolvers operate on the
/// `ConflictResolverContext` envelope (kind + a verdict
/// selector) rather than on a generic `T` so the trait can
/// be held as `Box<dyn ConflictResolver>` without generic
/// arguments. Callers wrap the local/remote aggregate pair
/// in their own struct and pass it via the `tag` field.
pub trait ConflictResolver: Send + Sync + std::fmt::Debug + 'static {
    /// Resolves the conflict described by `ctx` into a
    /// [`ConflictResolution<()>`]. The verdict carries no
    /// aggregate payload (the caller already has both local
    /// and remote); the resolver just decides which side to
    /// use, merge, or reject.
    ///
    /// # Errors
    ///
    /// Implementations MUST return
    /// `Err(DomainError::Conflict(_))` for conflicts they
    /// cannot resolve (the default impls never error — they
    /// always pick a side).
    fn resolve(&self, ctx: ConflictResolverContext) -> Result<ConflictResolution<()>>;
}

/// The input to a `ConflictResolver`. Carries the conflict
/// kind, detection timestamp, and an opaque caller-supplied
/// tag (typically a UUIDv7 envelope id) so resolvers can log
/// / correlate without needing the aggregate payload.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ConflictResolverContext {
    /// Why the conflict arose.
    pub kind: ConflictKind,
    /// When the conflict was detected.
    pub detected_at: Timestamp,
    /// An opaque caller-supplied tag. The default resolvers
    /// ignore this; custom resolvers may use it for logging
    /// or telemetry.
    pub tag: Option<String>,
}

impl ConflictResolverContext {
    /// Constructs a new context with `detected_at =
    /// Timestamp::now()`.
    #[must_use]
    pub fn new(kind: ConflictKind) -> Self {
        Self {
            kind,
            detected_at: Timestamp::now(),
            tag: None,
        }
    }

    /// Constructs a new context with an explicit timestamp.
    #[must_use]
    pub fn at(kind: ConflictKind, detected_at: Timestamp) -> Self {
        Self {
            kind,
            detected_at,
            tag: None,
        }
    }

    /// Builder-style method to attach an opaque tag.
    #[must_use]
    pub fn with_tag(mut self, tag: String) -> Self {
        self.tag = Some(tag);
        self
    }
}

/// Resolver that always returns `ConflictResolution::UseRemote`.
/// The "last write wins" semantics — the producer's view
/// overwrites the consumer's.
#[derive(Debug, Default, Clone, Copy)]
pub struct LastWriteWinsResolver;

impl ConflictResolver for LastWriteWinsResolver {
    fn resolve(&self, _ctx: ConflictResolverContext) -> Result<ConflictResolution<()>> {
        Ok(ConflictResolution::UseRemote(()))
    }
}

/// Resolver that always returns `ConflictResolution::UseLocal`.
/// The "first write wins" semantics — the consumer's view is
/// preserved and the producer's update is dropped.
#[derive(Debug, Default, Clone, Copy)]
pub struct FirstWriteWinsResolver;

impl ConflictResolver for FirstWriteWinsResolver {
    fn resolve(&self, _ctx: ConflictResolverContext) -> Result<ConflictResolution<()>> {
        Ok(ConflictResolution::UseLocal)
    }
}

/// Resolver that always returns `ConflictResolution::Reject`.
/// Every conflict is surfaced to the caller for manual
/// intervention; no automatic reconciliation.
#[derive(Debug, Default, Clone, Copy)]
pub struct RejectAllResolver;

impl ConflictResolver for RejectAllResolver {
    fn resolve(&self, _ctx: ConflictResolverContext) -> Result<ConflictResolution<()>> {
        Ok(ConflictResolution::Reject)
    }
}

/// Resolver that combines a per-`ConflictKind` policy: the
/// caller supplies a resolver per kind and `KindPolicyResolver`
/// dispatches. Useful for mixed policies (e.g. LWW for
/// `ConcurrentWrite`, Reject for `SchemaIncompatible`).
pub struct KindPolicyResolver {
    version_mismatch: Box<dyn ConflictResolver>,
    missing_aggregate: Box<dyn ConflictResolver>,
    concurrent_write: Box<dyn ConflictResolver>,
    schema_incompatible: Box<dyn ConflictResolver>,
}

impl std::fmt::Debug for KindPolicyResolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KindPolicyResolver").finish()
    }
}

impl Default for KindPolicyResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl KindPolicyResolver {
    /// Constructs a `KindPolicyResolver` with every kind
    /// defaulting to [`LastWriteWinsResolver`].
    #[must_use]
    pub fn new() -> Self {
        let lww: Box<dyn ConflictResolver> = Box::new(LastWriteWinsResolver);
        Self {
            version_mismatch: Box::new(LastWriteWinsResolver),
            missing_aggregate: Box::new(LastWriteWinsResolver),
            concurrent_write: Box::new(LastWriteWinsResolver),
            schema_incompatible: Box::new(RejectAllResolver),
        }
    }

    /// Overrides the resolver used for `VersionMismatch`
    /// conflicts.
    #[must_use]
    pub fn with_version_mismatch(mut self, resolver: Box<dyn ConflictResolver>) -> Self {
        self.version_mismatch = resolver;
        self
    }

    /// Overrides the resolver used for `MissingAggregate`
    /// conflicts.
    #[must_use]
    pub fn with_missing_aggregate(mut self, resolver: Box<dyn ConflictResolver>) -> Self {
        self.missing_aggregate = resolver;
        self
    }

    /// Overrides the resolver used for `ConcurrentWrite`
    /// conflicts.
    #[must_use]
    pub fn with_concurrent_write(mut self, resolver: Box<dyn ConflictResolver>) -> Self {
        self.concurrent_write = resolver;
        self
    }

    /// Overrides the resolver used for `SchemaIncompatible`
    /// conflicts.
    #[must_use]
    pub fn with_schema_incompatible(mut self, resolver: Box<dyn ConflictResolver>) -> Self {
        self.schema_incompatible = resolver;
        self
    }
}

impl ConflictResolver for KindPolicyResolver {
    fn resolve(&self, ctx: ConflictResolverContext) -> Result<ConflictResolution<()>> {
        let resolver: &dyn ConflictResolver = match ctx.kind {
            ConflictKind::VersionMismatch => &*self.version_mismatch,
            ConflictKind::MissingAggregate => &*self.missing_aggregate,
            ConflictKind::ConcurrentWrite => &*self.concurrent_write,
            ConflictKind::SchemaIncompatible => &*self.schema_incompatible,
        };
        resolver.resolve(ctx)
    }
}

/// Helper trait: keeps the trait-object story tidy. The
/// `PhantomData<T>` lets callers name a `Box<dyn ConflictResolver>`
/// against a specific `T` without requiring the trait to
/// declare `T`.
pub trait ConflictResolverExt: ConflictResolver + Sized {
    /// Boxes the resolver into a `Box<dyn ConflictResolver>`.
    fn boxed(self) -> Box<dyn ConflictResolver> {
        Box::new(self)
    }
}

impl<T: ConflictResolver> ConflictResolverExt for T {}

/// Helper newtype: a `ConflictResolver` parameterised by a
/// default `T`. The sync engine uses this to hold a
/// `ConflictResolver<T = AggregateSnapshot>`.
pub struct TypedResolver<R, T> {
    inner: R,
    _marker: PhantomData<T>,
}

impl<R: ConflictResolver, T> TypedResolver<R, T> {
    /// Wraps an untyped resolver as a typed one.
    #[must_use]
    pub const fn new(inner: R) -> Self {
        Self {
            inner,
            _marker: PhantomData,
        }
    }
}

impl<R: ConflictResolver, T> std::fmt::Debug for TypedResolver<R, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypedResolver")
            .field("inner", &self.inner)
            .finish()
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

    fn ctx(kind: ConflictKind) -> ConflictResolverContext {
        ConflictResolverContext::new(kind)
    }

    #[test]
    fn last_write_wins_picks_remote() {
        let r = LastWriteWinsResolver;
        let verdict = r.resolve(ctx(ConflictKind::ConcurrentWrite)).expect("ok");
        assert!(matches!(verdict, ConflictResolution::UseRemote(())));
    }

    #[test]
    fn first_write_wins_picks_local() {
        let r = FirstWriteWinsResolver;
        let verdict = r.resolve(ctx(ConflictKind::ConcurrentWrite)).expect("ok");
        assert!(matches!(verdict, ConflictResolution::UseLocal));
    }

    #[test]
    fn reject_all_rejects() {
        let r = RejectAllResolver;
        let verdict = r.resolve(ctx(ConflictKind::ConcurrentWrite)).expect("ok");
        assert!(matches!(verdict, ConflictResolution::Reject));
    }

    #[test]
    fn kind_policy_dispatches_per_kind() {
        let r = KindPolicyResolver::new()
            .with_concurrent_write(FirstWriteWinsResolver.boxed())
            .with_schema_incompatible(RejectAllResolver.boxed());

        let v1 = r.resolve(ctx(ConflictKind::VersionMismatch)).expect("ok");
        assert!(matches!(v1, ConflictResolution::UseRemote(())));

        let v2 = r.resolve(ctx(ConflictKind::ConcurrentWrite)).expect("ok");
        assert!(matches!(v2, ConflictResolution::UseLocal));

        let v3 = r
            .resolve(ctx(ConflictKind::SchemaIncompatible))
            .expect("ok");
        assert!(matches!(v3, ConflictResolution::Reject));
    }

    #[test]
    fn conflict_resolution_into_winner() {
        assert_eq!(ConflictResolution::<i32>::UseLocal.into_winner(), None);
        assert_eq!(ConflictResolution::UseRemote(42).into_winner(), Some(42));
        assert_eq!(ConflictResolution::Merge(99).into_winner(), Some(99));
        assert_eq!(ConflictResolution::<i32>::Reject.into_winner(), None);
    }

    #[test]
    fn trait_object_safe() {
        // Compile-time check: a Box<dyn ConflictResolver> is
        // valid (the trait is object-safe).
        let _: Box<dyn ConflictResolver> = Box::new(LastWriteWinsResolver);
        let _: Box<dyn ConflictResolver> = Box::new(RejectAllResolver);
    }
}
