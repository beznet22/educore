# smsengine-testkit

The testkit crate provides in-memory implementations of the engine's six ports (storage, auth, notify, payment, files, and event-bus) for use in unit and integration tests. It is a development-only dependency and must never be pulled into a production binary; consumers wire their real adapters at startup. See `docs/guides/test-strategy.md` for the full spec.
