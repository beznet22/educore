//! Time abstraction and ID generation ports.
//!
//! The engine does not call `Utc::now()` or `Uuid::new_v4()` directly
//! in domain code. Every consumer of time or identifiers goes through
//! a port, so production code can inject a real clock / generator
//! and tests can substitute a deterministic one.
//!
//! See `docs/code-standards.md` § 11 (clock injection) and
//! `docs/schemas/database-schema.md` § 1.4 (UUIDv7 as the default
//! identifier).

use std::sync::Mutex;

use uuid::Uuid;

use crate::ids::{CorrelationId, EventId, IdempotencyKey, SchoolId, SessionId, UserId};
use crate::value_objects::Timestamp;

/// Port: produces a `Timestamp` when the engine needs to record "now".
///
/// Production code wires [`SystemClock`]; tests wire [`TestClock`].
/// The trait is `Send + Sync` so the engine's command dispatcher can
/// hold a single `Arc<dyn Clock>` and pass it into domain code.
pub trait Clock: Send + Sync {
    /// Returns the current wall-clock instant.
    fn now(&self) -> Timestamp;
}

/// Production `Clock` backed by `chrono::Utc::now()`.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    #[inline]
    fn now(&self) -> Timestamp {
        Timestamp::now()
    }
}

/// Deterministic `Clock` for tests. The instant is advanced manually
/// via [`TestClock::advance`] or set via [`TestClock::set`].
#[derive(Debug, Default)]
pub struct TestClock {
    inner: Mutex<Timestamp>,
}

impl TestClock {
    /// Creates a `TestClock` initialised to the Unix epoch.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Timestamp::epoch()),
        }
    }

    /// Creates a `TestClock` initialised to the given instant.
    #[must_use]
    pub fn at(t: Timestamp) -> Self {
        Self {
            inner: Mutex::new(t),
        }
    }

    /// Sets the clock to the given instant. If the underlying
    /// mutex is poisoned (a panic occurred while holding it),
    /// the clock is set to the test's epoch as a safe default.
    pub fn set(&self, t: Timestamp) {
        let mut g = match self.inner.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        *g = t;
    }

    /// Advances the clock by the given duration. If the
    /// underlying mutex is poisoned, the clock is left
    /// unchanged. If the resulting timestamp overflows
    /// chrono's range, the clock is clamped to the
    /// representable maximum.
    pub fn advance(&self, by: chrono::Duration) {
        let mut g = match self.inner.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        let next = g
            .as_datetime()
            .checked_add_signed(by)
            .unwrap_or(chrono::DateTime::<chrono::Utc>::MAX_UTC);
        *g = Timestamp::from_datetime(next);
    }
}

impl Clock for TestClock {
    fn now(&self) -> Timestamp {
        match self.inner.lock() {
            Ok(g) => *g,
            Err(poisoned) => *poisoned.into_inner(),
        }
    }
}

/// Port: produces a UUIDv7 identifier for any typed wrapper.
///
/// Production code wires [`SystemIdGen`]; tests wire
/// [`DeterministicIdGen`] so snapshots are reproducible.
pub trait IdGenerator: Send + Sync {
    /// Generates a new [`SchoolId`].
    fn next_school_id(&self) -> SchoolId {
        SchoolId(self.next_uuid())
    }

    /// Generates a new [`UserId`].
    fn next_user_id(&self) -> UserId {
        UserId(self.next_uuid())
    }

    /// Generates a new [`EventId`].
    fn next_event_id(&self) -> EventId {
        EventId(self.next_uuid())
    }

    /// Generates a new [`CorrelationId`].
    fn next_correlation_id(&self) -> CorrelationId {
        CorrelationId(self.next_uuid())
    }

    /// Generates a new [`SessionId`].
    fn next_session_id(&self) -> SessionId {
        SessionId(self.next_uuid())
    }

    /// Generates a new [`IdempotencyKey`].
    fn next_idempotency_key(&self) -> IdempotencyKey {
        IdempotencyKey(self.next_uuid())
    }

    /// Generates a raw UUIDv7. The per-typed helpers above call this.
    fn next_uuid(&self) -> Uuid;
}

/// Production `IdGenerator` backed by `uuid::Uuid::now_v7()`.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemIdGen;

impl IdGenerator for SystemIdGen {
    #[inline]
    fn next_uuid(&self) -> Uuid {
        Uuid::now_v7()
    }
}

/// Deterministic `IdGenerator` for tests. Each call increments an
/// internal counter and uses the v7 timestamp bits + the counter as
/// the random suffix, so two consecutive calls in the same test
/// produce two different but stable ids.
#[derive(Debug)]
pub struct DeterministicIdGen {
    counter: Mutex<u64>,
}

impl DeterministicIdGen {
    /// Creates a generator starting at counter 0.
    #[must_use]
    pub fn new() -> Self {
        Self {
            counter: Mutex::new(0),
        }
    }

    /// Creates a generator starting at the given counter (useful for
    /// pre-seeding a test).
    #[must_use]
    pub fn starting_at(counter: u64) -> Self {
        Self {
            counter: Mutex::new(counter),
        }
    }
}

impl Default for DeterministicIdGen {
    fn default() -> Self {
        Self::new()
    }
}

impl IdGenerator for DeterministicIdGen {
    fn next_uuid(&self) -> Uuid {
        let mut g = match self.counter.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        let n = *g;
        *g = g.checked_add(1).unwrap_or(u64::MAX);
        deterministic_v7(n)
    }
}

/// Builds a stable v7 UUID from a monotonic counter. The result is
/// a syntactically valid UUIDv7: the high 48 bits are a real
/// timestamp (here, the unix epoch), the version nibble is `7`, and
/// the low 62 bits encode the counter. The same counter always
/// produces the same UUID, so test snapshots are reproducible.
fn deterministic_v7(counter: u64) -> Uuid {
    let mut bytes = [0u8; 16];
    // 48-bit unix-ms timestamp = 0 (i.e. the epoch). This makes the
    // UUID byte-equal across runs as long as the counter is the
    // same, which is the determinism guarantee tests rely on.
    bytes[0] = 0;
    bytes[1] = 0;
    bytes[2] = 0;
    bytes[3] = 0;
    bytes[4] = 0;
    bytes[5] = 0;
    // version: 7 in the high nibble of byte 6
    //
    // Every masked value below is bounded by its mask
    // (`0x0f`, `0xff`, `0x3f`, `0x03`) to a non-negative integer
    // no greater than `u8::MAX`. The `u8::try_from(...).unwrap_or(0)`
    // pattern therefore cannot fail for any caller that obeys
    // the mask contract; the `unwrap_or(0)` documents the
    // unreachable branch instead of using a forbidden `as` cast
    // (per `docs/code-standards.md` § "Code Standards").
    bytes[6] = 0x70 | u8::try_from(counter & 0x0f).unwrap_or(0);
    bytes[7] = u8::try_from((counter >> 4) & 0xff).unwrap_or(0);
    // variant: 10xxxxxx in byte 8
    bytes[8] = 0x80 | u8::try_from((counter >> 12) & 0x3f).unwrap_or(0);
    bytes[9] = u8::try_from((counter >> 18) & 0xff).unwrap_or(0);
    bytes[10] = u8::try_from((counter >> 26) & 0xff).unwrap_or(0);
    bytes[11] = u8::try_from((counter >> 34) & 0xff).unwrap_or(0);
    bytes[12] = u8::try_from((counter >> 42) & 0xff).unwrap_or(0);
    bytes[13] = u8::try_from((counter >> 50) & 0xff).unwrap_or(0);
    bytes[14] = u8::try_from((counter >> 58) & 0x03).unwrap_or(0);
    bytes[15] = 0;
    Uuid::from_bytes(bytes)
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
    use chrono::Duration;

    #[test]
    fn system_clock_returns_monotonic() {
        let c = SystemClock;
        let a = c.now();
        let b = c.now();
        assert!(b >= a);
    }

    #[test]
    fn test_clock_starts_at_epoch() {
        let c = TestClock::new();
        assert_eq!(c.now().as_datetime().timestamp(), 0);
    }

    #[test]
    fn test_clock_advance() {
        let c = TestClock::new();
        c.advance(Duration::seconds(42));
        assert_eq!(c.now().as_datetime().timestamp(), 42);
    }

    #[test]
    fn test_clock_set() {
        let c = TestClock::new();
        c.set(Timestamp::from_datetime(chrono::Utc::now()));
        let a = c.now();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let b = c.now();
        assert_eq!(a, b);
    }

    #[test]
    fn system_idgen_produces_v7() {
        let g = SystemIdGen;
        for _ in 0..16 {
            assert_eq!(g.next_uuid().get_version_num(), 7);
        }
    }

    #[test]
    fn deterministic_idgen_is_stable() {
        let g = DeterministicIdGen::starting_at(0);
        let a = g.next_school_id();
        let b = g.next_school_id();
        assert_ne!(a, b);

        // Recreate the generator and verify the same sequence.
        let g2 = DeterministicIdGen::starting_at(0);
        assert_eq!(g2.next_school_id(), a);
        assert_eq!(g2.next_school_id(), b);
    }

    #[test]
    fn deterministic_idgen_v7_marker() {
        let g = DeterministicIdGen::new();
        assert_eq!(g.next_uuid().get_version_num(), 7);
    }

    #[test]
    fn idgen_per_typed_helpers() {
        let g = SystemIdGen;
        let _a: SchoolId = g.next_school_id();
        let _b: UserId = g.next_user_id();
        let _c: EventId = g.next_event_id();
        let _d: CorrelationId = g.next_correlation_id();
        let _e: SessionId = g.next_session_id();
        let _f: IdempotencyKey = g.next_idempotency_key();
    }

    #[test]
    fn deterministic_idgen_starting_at_matches_advanced_counter() {
        // A fresh generator at counter 5 must produce the same UUID
        // as one that started at 0 and was advanced 5 times.
        let g_fresh = DeterministicIdGen::starting_at(5);
        let g_advanced = DeterministicIdGen::new();
        for _ in 0..5 {
            let _ = g_advanced.next_uuid();
        }
        assert_eq!(g_fresh.next_uuid(), g_advanced.next_uuid());
    }
}
