//! Helper functions for converting between engine value types
//! and `sqlx`-friendly wire types.
//!
//! These functions are crate-internal. They live in a separate
//! module so the `outbox`, `audit_log`, `event_log`, and
//! `idempotency` modules can share the JSON / UUID / timestamp
//! plumbing without circular dependencies.

use bytes::Bytes;
use serde_json::Value;
use sqlx::types::Json;

/// Convert `bytes::Bytes` (a wire-format payload) into a
/// `serde_json::Value` suitable for binding to a `JSONB` column.
/// If the bytes are not valid JSON, the value is wrapped in a
/// `Value::String` so the round-trip is lossless. This mirrors
/// the fallback in the SurrealDB adapter's outbox implementation.
#[inline]
#[must_use]
pub fn bytes_to_json_value(bytes: &Bytes) -> Value {
    serde_json::from_slice(bytes.as_ref())
        .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(bytes.as_ref()).into_owned()))
}

/// Convert an `Option<bytes::Bytes>` into an
/// `Option<Json<serde_json::Value>>` for binding to a `JSONB`
/// column that allows `NULL`.
#[inline]
#[must_use]
pub fn opt_bytes_to_json_value(bytes: &Option<Bytes>) -> Option<Json<Value>> {
    bytes.as_ref().map(|b| Json(bytes_to_json_value(b)))
}

/// Convert a `serde_json::Value` (read from a JSONB column) into
/// `bytes::Bytes`. `Value::to_string()` is infallible — every
/// `serde_json::Value` has a `Display` impl — so we use it
/// directly without a `Result`/unwrap path.
#[inline]
#[must_use]
pub fn json_value_to_bytes(v: &Value) -> Bytes {
    Bytes::from(v.to_string())
}

/// Convert `Option<Json<Value>>` (read from a nullable JSONB
/// column) into `Option<Bytes>`. `None` stays `None`.
#[inline]
#[must_use]
pub fn opt_json_to_opt_bytes(v: &Option<Json<Value>>) -> Option<Bytes> {
    v.as_ref().map(|j| json_value_to_bytes(&j.0))
}
