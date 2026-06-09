# smsengine-audit

The audit crate is the engine's writer for the immutable, append-only `audit_log` table. Every state change in the engine produces exactly one audit row inside the same transaction as the state change itself, and the audit crate owns the retention and redaction policies that govern how those rows age. See `docs/schemas/audit-schema.md` for the full spec.
