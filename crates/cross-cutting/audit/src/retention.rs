//! Retention policy and threshold checker.
//!
//! Per `docs/schemas/audit-schema.md` § 9 the engine's audit log is
//! retained for a configurable period; rows older than the
//! configured cutoff may be archived or deleted by a consumer-side
//! job. The engine emits a [`RetentionSweepDue`] event when a sweep
//! is due; the consumer performs the actual deletion.
//!
//! This module owns:
//!
//! - [`RetentionPolicy`] — the configurable thresholds
//!   (`retention_days`, `sweep_check_interval`).
//! - [`RetentionSweeper`] — the threshold-check state machine used
//!   by [`crate::writer::AuditWriter::maybe_sweep`]. It is a
//!   standalone type so it can be unit-tested in isolation and so
//!   future callers (e.g. a cron-style consumer job) can reuse the
//!   same check.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use educore_core::value_objects::Timestamp;

/// The retention policy applied to the audit log.
///
/// Defaults per `docs/schemas/audit-schema.md` § 9: 90 days retention
/// with a 1-hour sweep check interval. Both fields are
/// configurable per deployment — the engine's default is the
/// education-sector baseline; regulated deployments (FERPA, GDPR
/// financial) override `retention_days` to 7 years.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Number of days a row is retained before it is eligible for
    /// archival / deletion. Default: **90 days** per
    /// `docs/schemas/audit-schema.md` § 9.
    pub retention_days: u32,

    /// Minimum wall-clock duration between two sweep checks. The
    /// check is opportunistic: the engine runs it after every
    /// `AuditWriter::write`, but the actual sweep_due event is
    /// emitted at most once per `sweep_check_interval`. Default:
    /// **1 hour** to bound the per-write overhead.
    pub sweep_check_interval: Duration,
}

impl Default for RetentionPolicy {
    /// Returns the engine default: 90-day retention, 1-hour sweep
    /// check interval.
    #[inline]
    fn default() -> Self {
        Self {
            retention_days: 90,
            sweep_check_interval: Duration::from_secs(3600),
        }
    }
}

impl RetentionPolicy {
    /// Constructs a custom `RetentionPolicy` from the two fields.
    /// Returns a [`Validation`](educore_core::error::DomainError::Validation)
    /// error if `retention_days` is zero (a zero-day retention
    /// would cause the engine to emit a `RetentionSweepDue` on
    /// every write, which is never the desired behaviour).
    pub fn new(
        retention_days: u32,
        sweep_check_interval: Duration,
    ) -> educore_core::error::Result<Self> {
        if retention_days == 0 {
            return Err(educore_core::error::DomainError::validation(
                "retention_days must be >= 1",
            ));
        }
        Ok(Self {
            retention_days,
            sweep_check_interval,
        })
    }

    /// Returns the `chrono::Duration` equivalent of
    /// `retention_days`. The conversion is infallible for any
    /// `u32` (the max representable `chrono::Duration` in days is
    /// bounded only by the year ~262000, well beyond any realistic
    /// retention setting).
    #[must_use]
    pub fn retention_chrono(&self) -> chrono::Duration {
        chrono::Duration::days(i64::from(self.retention_days))
    }

    /// Returns the `chrono::Duration` equivalent of
    /// `sweep_check_interval`. The `chrono::Duration::from_std`
    /// conversion can fail if the interval is larger than
    /// `chrono::Duration::MAX` (~9.2 trillion years in seconds),
    /// which is not a realistic configuration. On overflow the
    /// method falls back to the engine's maximum representable
    /// duration so the comparison still works in the correct
    /// direction (the check is `elapsed < threshold`).
    #[must_use]
    pub fn sweep_interval_chrono(&self) -> chrono::Duration {
        match chrono::Duration::from_std(self.sweep_check_interval) {
            Ok(d) => d,
            Err(_) => chrono::Duration::MAX,
        }
    }
}

/// The threshold-check state machine used by
/// [`crate::writer::AuditWriter::maybe_sweep`]. Tracks the last
/// sweep-check time and decides whether a new check is due.
///
/// This type is intentionally tiny so it can be unit-tested in
/// isolation (the audit integration tests in
/// `tests/audit_e2e.rs` exercise it without spinning up the full
/// `AuditWriter`).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RetentionSweeper {
    /// The `now` value of the most recent `should_sweep` call that
    /// returned `true` (or the seeded `now` of the first call,
    /// which always returns `false`). `None` means the sweeper has
    /// never run.
    last_sweep_at: Option<Timestamp>,
}

impl RetentionSweeper {
    /// Constructs a new `RetentionSweeper` with no prior sweep
    /// recorded. The first [`should_sweep`](Self::should_sweep)
    /// call always returns `false` (the per-instance seed) and
    /// records the call time so the second call can be
    /// meaningfully compared.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            last_sweep_at: None,
        }
    }

    /// Returns `true` if the sweep check interval has elapsed
    /// since the last sweep (or since construction) AND the
    /// threshold check should run. Records `now` as the new
    /// `last_sweep_at` regardless of the return value so a
    /// subsequent call within the interval is a no-op.
    ///
    /// The semantics are:
    ///
    /// - **First call** (`last_sweep_at == None`): records `now`,
    ///   returns `false`. The very first call is a "seed" — the
    ///   engine does not want a sweep to fire the moment the
    ///   process starts.
    /// - **Subsequent call within interval**: returns `false`.
    /// - **Subsequent call at or after interval**: records `now`,
    ///   returns `true`.
    pub fn should_sweep(&mut self, now: Timestamp, policy: &RetentionPolicy) -> bool {
        match self.last_sweep_at {
            None => {
                self.last_sweep_at = Some(now);
                false
            }
            Some(prev) => {
                let elapsed = now.as_datetime().signed_duration_since(prev.as_datetime());
                if elapsed >= policy.sweep_interval_chrono() {
                    self.last_sweep_at = Some(now);
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Returns the cutoff timestamp for the sweep: rows with
    /// `occurred_at < cutoff` are eligible for archival /
    /// deletion. Computed as `now - retention_days`.
    ///
    /// `chrono::Duration::MAX` is reached only for retention
    /// values larger than the chrono upper bound (~year 262000);
    /// for any realistic `retention_days` (`u32` max ~4 billion
    /// days is still representable as a `chrono::Duration` in
    /// days) the conversion is exact.
    #[must_use]
    pub fn cutoff_for(now: Timestamp, policy: &RetentionPolicy) -> Timestamp {
        let delta = policy.retention_chrono();
        let raw = now.as_datetime() - delta;
        Timestamp::from_datetime(raw)
    }

    /// Returns the `last_sweep_at` recorded by the most recent
    /// [`should_sweep`](Self::should_sweep) call, or `None` if the
    /// sweeper has not run yet. Useful for tests and for
    /// observability.
    #[must_use]
    pub const fn last_sweep_at(&self) -> Option<Timestamp> {
        self.last_sweep_at
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
    use chrono::{Duration as ChronoDuration, TimeZone, Utc};

    fn ts(secs: i64) -> Timestamp {
        Timestamp::from_datetime(Utc.timestamp_opt(secs, 0).single().unwrap_or_else(Utc::now))
    }

    #[test]
    fn default_policy_is_90_days_one_hour() {
        let p = RetentionPolicy::default();
        assert_eq!(p.retention_days, 90);
        assert_eq!(p.sweep_check_interval, Duration::from_secs(3600));
    }

    #[test]
    fn new_rejects_zero_retention() {
        assert!(RetentionPolicy::new(0, Duration::from_secs(60)).is_err());
        assert!(RetentionPolicy::new(1, Duration::from_secs(60)).is_ok());
    }

    #[test]
    fn cutoff_for_subtracts_retention_days() {
        let policy = RetentionPolicy {
            retention_days: 7,
            sweep_check_interval: Duration::from_secs(60),
        };
        let now = ts(1_000_000);
        let cutoff = RetentionSweeper::cutoff_for(now, &policy);
        let expected = ts(1_000_000 - 7 * 86_400);
        assert_eq!(cutoff, expected);
    }

    #[test]
    fn cutoff_for_zero_retention_is_now_minus_zero() {
        // Even with retention_days = 1 the cutoff is exactly one
        // day behind now. We test the "1 day" case rather than
        // zero because the constructor rejects zero.
        let policy = RetentionPolicy {
            retention_days: 1,
            sweep_check_interval: Duration::from_secs(60),
        };
        let now = ts(86_400);
        let cutoff = RetentionSweeper::cutoff_for(now, &policy);
        assert_eq!(cutoff, ts(0));
    }

    #[test]
    fn sweeper_first_call_seeds_and_returns_false() {
        let mut s = RetentionSweeper::new();
        let now = ts(1_000);
        let policy = RetentionPolicy::default();
        assert!(!s.should_sweep(now, &policy));
        assert_eq!(s.last_sweep_at(), Some(now));
    }

    #[test]
    fn sweeper_within_interval_returns_false() {
        let mut s = RetentionSweeper::new();
        let policy = RetentionPolicy {
            retention_days: 90,
            sweep_check_interval: Duration::from_secs(60),
        };
        let t0 = ts(0);
        let t1 = ts(30); // 30 seconds after t0, within the 60s interval
        assert!(!s.should_sweep(t0, &policy));
        assert!(!s.should_sweep(t1, &policy));
    }

    #[test]
    fn sweeper_after_interval_returns_true() {
        let mut s = RetentionSweeper::new();
        let policy = RetentionPolicy {
            retention_days: 90,
            sweep_check_interval: Duration::from_secs(60),
        };
        let t0 = ts(0);
        let t1 = ts(120); // 120 seconds after t0, past the 60s interval
        assert!(!s.should_sweep(t0, &policy));
        assert!(s.should_sweep(t1, &policy));
    }

    #[test]
    fn sweeper_records_new_last_sweep_at() {
        let mut s = RetentionSweeper::new();
        let policy = RetentionPolicy {
            retention_days: 90,
            sweep_check_interval: Duration::from_secs(60),
        };
        let t0 = ts(0);
        let t1 = ts(120);
        let _ = s.should_sweep(t0, &policy);
        let _ = s.should_sweep(t1, &policy);
        assert_eq!(s.last_sweep_at(), Some(t1));
    }

    #[test]
    fn retention_chrono_matches_days() {
        let policy = RetentionPolicy {
            retention_days: 30,
            sweep_check_interval: Duration::from_secs(60),
        };
        assert_eq!(policy.retention_chrono(), ChronoDuration::days(30));
    }
}
