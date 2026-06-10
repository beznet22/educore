# educore-cli

The cli crate is a sample binary that demonstrates how a consumer wires the Educore for daily operations — starting the runtime, applying migrations, running scheduled jobs, and draining the outbox. The engine itself is library-only; this CLI is a reference implementation and is not re-exported from the umbrella crate. See `docs/guides/saas-backend.md` for the full spec.
