# smsengine-operations

The operations domain is the engine's port to operational state — backups, scheduled jobs, system version tracking, and runtime maintenance windows. It is the place where consumers record the engine's own lifecycle (what ran, when, on whose behalf, and how it completed) and the only domain that is allowed to mutate engine-internal tables outside of an aggregate transaction. See `docs/specs/operations/` for the full spec.
