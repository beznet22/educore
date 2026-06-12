# educore-platform

Multi-tenancy substrate for the Educore engine. Ships the
[`School`] and [`User`] aggregate roots, their typed events
(`SchoolCreated`, `UserRegistered`, etc.), commands, value
objects, and the per-aggregate repository port traits. The
remaining 30 secondary platform aggregates enumerated in
`docs/specs/platform/aggregates.md` (Course, OtpCode,
Module, Plugin, ...) are out of scope for Phase 2 and land in
later phases alongside their owning events.

This crate is a member of the `cross-cutting` tier. It depends
only on `educore-core` (the engine's foundation) and
`educore-events` (the bus port); the storage and bus
adapters live in the `adapters` tier and are wired in by the
consumer's binary.

See `docs/specs/platform/aggregates.md`, `commands.md`,
`events.md`, and `value-objects.md` for the design contract
and `docs/build-plan.md` § Phase 2 for the implementation
plan.
