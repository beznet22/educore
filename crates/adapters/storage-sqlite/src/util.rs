//! Shared helpers for the SQLite adapter.

use bytes::Bytes;

/// Parses `bytes` as a `serde_json::Value`. If the bytes are
/// not valid JSON, the value is wrapped as a JSON string
/// (lossy from UTF-8) so the round-trip is total. Mirrors the
/// `unwrap_or_else` default pattern used by the SurrealDB
/// adapter's outbox payload path.
pub(crate) fn bytes_to_json(bytes: &Bytes) -> serde_json::Value {
    serde_json::from_slice(bytes)
        .unwrap_or_else(|_| serde_json::Value::String(String::from_utf8_lossy(bytes).into_owned()))
}

/// Inverse of [`bytes_to_json`]: renders a `serde_json::Value`
/// as `Bytes`. Object/array values are re-serialised to JSON
/// text; string values are passed through unchanged.
pub(crate) fn json_to_bytes(v: &serde_json::Value) -> Bytes {
    let s = match v {
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    };
    Bytes::from(s.into_bytes())
}
