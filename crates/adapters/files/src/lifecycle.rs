//! # educore-files :: lifecycle
//!
//! Lifecycle-rule configuration port for the
//! [`FileStorage`](crate::port::FileStorage) port. Per
//! `docs/ports/file-storage.md` § "Lifecycle Rules":
//!
//! - Transition from `Hot` to `Cool` after `N` days.
//! - Transition from `Cool` to `Archive` after `M` days.
//! - Expire (delete) after `P` days.
//!
//! Rules are configured per bucket by the consumer; the engine
//! supplies a typed representation
//! ([`StorageTier`], [`LifecycleRule`], [`LifecyclePolicy`]) and
//! an evaluator ([`LifecycleEvaluator`] +
//! [`DefaultLifecycleEvaluator`]) so adapters can translate the
//! configuration to their provider-native lifecycle policy
//! (AWS S3 Lifecycle Configuration, GCS Object Lifecycle,
//! Azure Blob lifecycle management).
//!
//! This module is **port-only**: it defines the configuration
//! shape and the evaluation contract. It does not perform I/O.
//! The actual application of the transition is owned by each
//! adapter; the engine itself never mutates object storage
//! tiers directly.
//!
//! # Object safety
//!
//! [`LifecycleEvaluator`] is object-safe — every method takes
//! `&self`, has no generic parameters, and returns a concrete
//! type. Adapters may hold a `Box<dyn LifecycleEvaluator>` for
//! dynamic dispatch.
//!
//! # Deviations from `docs/ports/file-storage.md`
//!
//! - The spec defines the lifecycle rules in prose, not as Rust
//!   types. This module introduces a typed shape: a
//!   [`StorageTier`] enum distinct from the port's
//!   [`StorageClass`](crate::port::StorageClass) (the two answer
//!   different questions — `StorageClass` is recorded on a
//!   [`FileReference`](crate::port::FileReference) at upload
//!   time; `StorageTier` is the destination of a lifecycle
//!   transition). The two enums share the same three values
//!   today (`Hot` / `Cool` / `Archive`) but are kept separate
//!   so future divergence (e.g. a per-tier cost ceiling that
//!   only applies to transitions) does not break the
//!   upload-time contract.
//! - The `delete` field on [`LifecycleRule`] is honoured by a
//!   separate helper ([`LifecyclePolicy::is_expired_at`]) rather
//!   than by [`LifecycleEvaluator::next_tier`]. The two
//!   concerns (tier transition vs. expiration) are kept
//!   distinct so callers can answer "what tier should this
//!   object be in?" and "should this object be deleted?"
//!   independently — the way S3, GCS, and Azure model them.
//! - The evaluator is **stateless and pure**. The engine does
//!   not own a global lifecycle sweep; adapters run the sweep
//!   against their provider at the cadence the bucket config
//!   declares.

use std::fmt;

// ---------------------------------------------------------------------------
// StorageTier
// ---------------------------------------------------------------------------

/// The destination tier of a lifecycle transition.
///
/// Distinct from [`StorageClass`](crate::port::StorageClass),
/// which records the class on a
/// [`FileReference`](crate::port::FileReference) at upload time.
/// The two enums share the same three values today (`Hot` /
/// `Cool` / `Archive`) but are kept separate so future
/// divergence — e.g. a per-tier cost ceiling that only applies
/// to transitions — does not break the upload-time contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StorageTier {
    /// Frequent access. Default for newly uploaded files.
    Hot,

    /// Infrequent access. Lower cost per GB, slightly higher
    /// retrieval cost.
    Cool,

    /// Long-term, slower retrieval. Lowest cost per GB,
    /// minutes-to-hours retrieval latency on cold restores.
    Archive,
}

impl StorageTier {
    /// Returns the canonical snake_case wire string for the
    /// tier. Matches [`StorageClass::as_str`](crate::port::StorageClass::as_str)
    /// for the shared values; kept as a separate method so the
    /// two enums can diverge later without a breaking change.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Hot => "hot",
            Self::Cool => "cool",
            Self::Archive => "archive",
        }
    }

    /// Returns the coldness rank: `0` (`Hot`), `1` (`Cool`),
    /// `2` (`Archive`).
    ///
    /// Used by [`DefaultLifecycleEvaluator`] to decide whether
    /// a rule's `target_tier` is a real transition (strictly
    /// colder than the current tier) or a no-op.
    #[must_use]
    pub const fn rank(self) -> u8 {
        match self {
            Self::Hot => 0,
            Self::Cool => 1,
            Self::Archive => 2,
        }
    }

    /// Returns `true` if `self` is strictly colder than `other`.
    ///
    /// A tier is "colder" than another when its [`rank`](Self::rank)
    /// is strictly greater: `Hot` < `Cool` < `Archive`. Equal
    /// ranks are NOT colder — staying at the same tier is not
    /// a transition.
    #[must_use]
    pub const fn is_colder_than(self, other: Self) -> bool {
        self.rank() > other.rank()
    }
}

impl Default for StorageTier {
    /// Defaults to `Hot`, matching the upload-time default of
    /// [`StorageClass`](crate::port::StorageClass).
    fn default() -> Self {
        Self::Hot
    }
}

impl fmt::Display for StorageTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

// ---------------------------------------------------------------------------
// LifecycleRule
// ---------------------------------------------------------------------------

/// A single lifecycle rule.
///
/// A rule either transitions an object to a colder
/// [`StorageTier`] after `after_days`, or expires (deletes) it.
/// The two modes are mutually exclusive per rule: setting
/// `delete: true` makes `target_tier` a no-op.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LifecycleRule {
    /// The age in days at which the rule fires. The age is
    /// measured from the object's `uploaded_at` timestamp on
    /// its [`FileReference`](crate::port::FileReference).
    pub after_days: u32,

    /// The destination tier when the rule fires. Ignored when
    /// `delete` is `true`.
    pub target_tier: StorageTier,

    /// When `true`, the rule expires (deletes) the object
    /// instead of transitioning it. Overrides `target_tier`.
    pub delete: bool,
}

impl LifecycleRule {
    /// Constructs a transition rule (e.g. `Hot` → `Cool` after
    /// `N` days). Equivalent to
    /// `Self { after_days, target_tier, delete: false }`.
    #[must_use]
    pub const fn transition(after_days: u32, target_tier: StorageTier) -> Self {
        Self {
            after_days,
            target_tier,
            delete: false,
        }
    }

    /// Constructs an expiration rule (delete after `N` days).
    /// Equivalent to `Self { after_days, target_tier: Hot,
    /// delete: true }` — the `target_tier` value is irrelevant
    /// when `delete` is `true`.
    #[must_use]
    pub const fn expire(after_days: u32) -> Self {
        Self {
            after_days,
            target_tier: StorageTier::Hot,
            delete: true,
        }
    }

    /// Returns `true` if the rule's `after_days` is `>= 1`.
    /// Rules with `after_days == 0` are nonsensical (they
    /// would fire at upload time) and adapters SHOULD reject
    /// them at configuration time.
    #[must_use]
    pub const fn is_well_formed(&self) -> bool {
        self.after_days >= 1
    }
}

// ---------------------------------------------------------------------------
// LifecyclePolicy
// ---------------------------------------------------------------------------

/// An ordered list of [`LifecycleRule`] values.
///
/// Adapters translate this into the provider's native lifecycle
/// configuration. The engine itself does not interpret the
/// rules at upload time — it just stores them on the per-bucket
/// config and exposes them via
/// [`LifecycleEvaluator::next_tier`] and
/// [`is_expired_at`](Self::is_expired_at).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LifecyclePolicy {
    /// The ordered list of rules. Order matters:
    /// [`DefaultLifecycleEvaluator`] iterates in `Vec` order
    /// and applies the first matching rule. Producers SHOULD
    /// emit rules in ascending `after_days` so that the first
    /// match is the most relevant transition.
    pub rules: Vec<LifecycleRule>,
}

impl LifecyclePolicy {
    /// Returns a policy with no rules. Objects governed by an
    /// empty policy never transition and never expire.
    #[must_use]
    pub const fn empty() -> Self {
        Self { rules: Vec::new() }
    }

    /// Returns the number of rules in the policy.
    #[must_use]
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// Returns `true` if the policy has no rules.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Appends a rule to the end of the policy. The order of
    /// [`push`](Self::push) calls determines iteration order
    /// for [`DefaultLifecycleEvaluator::next_tier`].
    pub fn push(&mut self, rule: LifecycleRule) {
        self.rules.push(rule);
    }

    /// Returns `true` if the policy says to delete an object at
    /// the given age. Iterates the rules and returns `true` if
    /// any rule with `delete: true` has `after_days <=
    /// age_days`.
    ///
    /// Use this after [`LifecycleEvaluator::next_tier`] returns
    /// `None` to disambiguate "no transition applies" from
    /// "object is expired". The two checks together answer the
    /// engine's full "what action should the sweep take for
    /// this object?" question.
    #[must_use]
    pub fn is_expired_at(&self, age_days: u32) -> bool {
        self.rules
            .iter()
            .any(|rule| rule.delete && rule.is_well_formed() && age_days >= rule.after_days)
    }
}

// ---------------------------------------------------------------------------
// LifecycleEvaluator trait
// ---------------------------------------------------------------------------

/// Evaluates lifecycle rules to determine whether an object
/// should transition to a colder tier.
///
/// The evaluator is **stateless and pure**: it does not perform
/// I/O and does not mutate the policy. Adapters and the
/// engine's lifecycle sweep call it once per object per sweep
/// to decide what action to take.
///
/// # Object safety
///
/// The trait is object-safe — every method takes `&self`, has
/// no generic parameters, and returns a concrete type. Adapters
/// may hold a `Box<dyn LifecycleEvaluator>` for dynamic
/// dispatch. A compile-time assertion in this module verifies
/// the object-safety contract.
pub trait LifecycleEvaluator {
    /// Returns the next [`StorageTier`] for an object at the
    /// given age, given the policy.
    ///
    /// Returns:
    ///
    /// - `Some(tier)` if a transition rule in `policy` has
    ///   `after_days <= age_days` AND the rule's `target_tier`
    ///   is strictly colder than `current`.
    /// - `None` if no transition rule matches. The caller should
    ///   follow up with [`LifecyclePolicy::is_expired_at`] to
    ///   distinguish "no transition applies" from "the policy
    ///   says to delete this object".
    fn next_tier(
        &self,
        current: StorageTier,
        age_days: u32,
        policy: &LifecyclePolicy,
    ) -> Option<StorageTier>;
}

// ---------------------------------------------------------------------------
// DefaultLifecycleEvaluator
// ---------------------------------------------------------------------------

/// The default [`LifecycleEvaluator`] implementation.
///
/// `DefaultLifecycleEvaluator::next_tier` walks the policy's
/// rules in `Vec` order and returns the first matching rule's
/// `target_tier`. A rule matches when:
///
/// 1. The rule's action is a transition (`delete: false`).
/// 2. The rule's `after_days <= age_days`.
/// 3. The rule's `target_tier` is strictly colder than
///    `current`.
///
/// Rules with `delete: true` are **not** considered for tier
/// transitions — the engine answers "should this be deleted?"
/// via [`LifecyclePolicy::is_expired_at`] instead.
///
/// The struct is a zero-sized marker; the trait implementation
/// is pure and stateless.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct DefaultLifecycleEvaluator;

impl LifecycleEvaluator for DefaultLifecycleEvaluator {
    fn next_tier(
        &self,
        current: StorageTier,
        age_days: u32,
        policy: &LifecyclePolicy,
    ) -> Option<StorageTier> {
        for rule in &policy.rules {
            // Skip expiration rules — they do not produce a tier
            // transition. Callers query LifecyclePolicy::is_expired_at
            // for the delete decision.
            if rule.delete {
                continue;
            }
            // Skip rules that have not yet fired.
            if age_days < rule.after_days {
                continue;
            }
            // Skip no-op transitions (target == current, or
            // moving to a hotter tier).
            if !rule.target_tier.is_colder_than(current) {
                continue;
            }
            return Some(rule.target_tier);
        }
        None
    }
}

// Compile-time assertion: LifecycleEvaluator is object-safe.
const _: fn() = || {
    let _ = || -> Box<dyn LifecycleEvaluator> { Box::new(DefaultLifecycleEvaluator) };
};

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

    // ----- StorageTier -----

    #[test]
    fn storage_tier_as_str_matches_storage_class() {
        assert_eq!(StorageTier::Hot.as_str(), "hot");
        assert_eq!(StorageTier::Cool.as_str(), "cool");
        assert_eq!(StorageTier::Archive.as_str(), "archive");
    }

    #[test]
    fn storage_tier_rank_is_ordered_hot_to_archive() {
        assert!(StorageTier::Hot.rank() < StorageTier::Cool.rank());
        assert!(StorageTier::Cool.rank() < StorageTier::Archive.rank());
    }

    #[test]
    fn storage_tier_default_is_hot() {
        assert_eq!(StorageTier::default(), StorageTier::Hot);
    }

    #[test]
    fn storage_tier_display() {
        assert_eq!(StorageTier::Hot.to_string(), "hot");
        assert_eq!(StorageTier::Cool.to_string(), "cool");
        assert_eq!(StorageTier::Archive.to_string(), "archive");
    }

    #[test]
    fn storage_tier_is_colder_than() {
        assert!(StorageTier::Cool.is_colder_than(StorageTier::Hot));
        assert!(StorageTier::Archive.is_colder_than(StorageTier::Cool));
        assert!(StorageTier::Archive.is_colder_than(StorageTier::Hot));
        // Equal and hotter are NOT colder.
        assert!(!StorageTier::Hot.is_colder_than(StorageTier::Hot));
        assert!(!StorageTier::Cool.is_colder_than(StorageTier::Cool));
        assert!(!StorageTier::Hot.is_colder_than(StorageTier::Cool));
    }

    // ----- LifecycleRule -----

    #[test]
    fn transition_rule_constructor() {
        let rule = LifecycleRule::transition(30, StorageTier::Cool);
        assert_eq!(rule.after_days, 30);
        assert_eq!(rule.target_tier, StorageTier::Cool);
        assert!(!rule.delete);
        assert!(rule.is_well_formed());
    }

    #[test]
    fn expire_rule_constructor() {
        let rule = LifecycleRule::expire(365);
        assert_eq!(rule.after_days, 365);
        assert!(rule.delete);
        assert!(rule.is_well_formed());
    }

    #[test]
    fn rule_with_zero_after_days_is_not_well_formed() {
        let rule = LifecycleRule::transition(0, StorageTier::Cool);
        assert!(!rule.is_well_formed());
    }

    // ----- LifecyclePolicy -----

    #[test]
    fn policy_empty_has_no_rules() {
        let policy = LifecyclePolicy::empty();
        assert!(policy.is_empty());
        assert_eq!(policy.len(), 0);
        assert!(!policy.is_expired_at(0));
        assert!(!policy.is_expired_at(u32::MAX));
    }

    #[test]
    fn policy_push_appends_rule() {
        let mut policy = LifecyclePolicy::empty();
        policy.push(LifecycleRule::transition(30, StorageTier::Cool));
        policy.push(LifecycleRule::transition(90, StorageTier::Archive));
        policy.push(LifecycleRule::expire(365));
        assert_eq!(policy.len(), 3);
        assert!(!policy.is_empty());
    }

    #[test]
    fn policy_default_is_empty() {
        let policy = LifecyclePolicy::default();
        assert!(policy.is_empty());
    }

    #[test]
    fn policy_is_expired_at_respects_age_threshold() {
        let policy = LifecyclePolicy {
            rules: vec![LifecycleRule::expire(90)],
        };
        assert!(!policy.is_expired_at(0));
        assert!(!policy.is_expired_at(89));
        assert!(policy.is_expired_at(90));
        assert!(policy.is_expired_at(365));
    }

    #[test]
    fn policy_is_expired_at_ignores_transition_rules() {
        let policy = LifecyclePolicy {
            rules: vec![
                LifecycleRule::transition(30, StorageTier::Cool),
                LifecycleRule::transition(90, StorageTier::Archive),
            ],
        };
        // Transition rules never expire the object.
        assert!(!policy.is_expired_at(0));
        assert!(!policy.is_expired_at(90));
        assert!(!policy.is_expired_at(u32::MAX));
    }

    #[test]
    fn policy_is_expired_at_ignores_zero_after_days() {
        let rule = LifecycleRule {
            after_days: 0,
            target_tier: StorageTier::Hot,
            delete: true,
        };
        let policy = LifecyclePolicy { rules: vec![rule] };
        // A zero-day delete rule is malformed; the helper must
        // not fire on it.
        assert!(!policy.is_expired_at(0));
        assert!(!policy.is_expired_at(u32::MAX));
    }

    // ----- DefaultLifecycleEvaluator -----

    #[test]
    fn default_evaluator_returns_first_matching_transition() {
        let policy = LifecyclePolicy {
            rules: vec![
                LifecycleRule::transition(30, StorageTier::Cool),
                LifecycleRule::transition(90, StorageTier::Archive),
            ],
        };
        let ev = DefaultLifecycleEvaluator;
        assert_eq!(
            ev.next_tier(StorageTier::Hot, 10, &policy),
            None,
            "younger than first rule -> no transition",
        );
        assert_eq!(
            ev.next_tier(StorageTier::Hot, 30, &policy),
            Some(StorageTier::Cool),
        );
        assert_eq!(
            ev.next_tier(StorageTier::Hot, 90, &policy),
            Some(StorageTier::Cool),
            "returns the first matching rule, not the latest",
        );
    }

    #[test]
    fn default_evaluator_skips_no_op_transitions() {
        let policy = LifecyclePolicy {
            rules: vec![LifecycleRule::transition(30, StorageTier::Hot)],
        };
        let ev = DefaultLifecycleEvaluator;
        assert_eq!(
            ev.next_tier(StorageTier::Hot, 30, &policy),
            None,
            "target == current is not a transition",
        );
        assert_eq!(
            ev.next_tier(StorageTier::Cool, 30, &policy),
            None,
            "moving to a hotter tier is not a transition",
        );
    }

    #[test]
    fn default_evaluator_ignores_delete_rules() {
        let policy = LifecyclePolicy {
            rules: vec![LifecycleRule::expire(30), LifecycleRule::expire(365)],
        };
        let ev = DefaultLifecycleEvaluator;
        // Delete rules never produce a tier transition.
        assert_eq!(ev.next_tier(StorageTier::Hot, 30, &policy), None);
        assert_eq!(ev.next_tier(StorageTier::Hot, 365, &policy), None);
        assert_eq!(ev.next_tier(StorageTier::Hot, u32::MAX, &policy), None);
        // But is_expired_at does pick them up.
        assert!(policy.is_expired_at(30));
        assert!(policy.is_expired_at(365));
    }

    #[test]
    fn default_evaluator_returns_none_when_no_rule_matches() {
        let policy = LifecyclePolicy::empty();
        let ev = DefaultLifecycleEvaluator;
        assert_eq!(ev.next_tier(StorageTier::Hot, 0, &policy), None);
        assert_eq!(ev.next_tier(StorageTier::Hot, u32::MAX, &policy), None);
    }

    #[test]
    fn default_evaluator_walks_archive_through_full_chain() {
        // A complete Hot -> Cool -> Archive chain.
        let policy = LifecyclePolicy {
            rules: vec![
                LifecycleRule::transition(30, StorageTier::Cool),
                LifecycleRule::transition(90, StorageTier::Archive),
            ],
        };
        let ev = DefaultLifecycleEvaluator;
        assert_eq!(
            ev.next_tier(StorageTier::Hot, 30, &policy),
            Some(StorageTier::Cool),
        );
        // After transitioning to Cool, the next rule fires next.
        assert_eq!(
            ev.next_tier(StorageTier::Cool, 90, &policy),
            Some(StorageTier::Archive),
        );
        // Once at Archive, no further transitions exist.
        assert_eq!(ev.next_tier(StorageTier::Archive, 365, &policy), None,);
    }

    // ----- Object safety -----

    #[test]
    fn lifecycle_evaluator_is_object_safe() {
        let boxed: Box<dyn LifecycleEvaluator> = Box::new(DefaultLifecycleEvaluator);
        let policy = LifecyclePolicy {
            rules: vec![LifecycleRule::transition(30, StorageTier::Cool)],
        };
        // Call through the trait object to confirm dispatch works.
        assert_eq!(
            boxed.next_tier(StorageTier::Hot, 30, &policy),
            Some(StorageTier::Cool),
        );
    }
}
