# educore-storage-parity

The storage-parity crate is a cross-adapter test suite that runs the same schema-creation and CRUD scenarios against all three shipped storage adapters (PostgreSQL, MySQL, SQLite) and asserts that they produce equivalent DDL and equivalent observable behavior. It is a development-only crate; no consumer application depends on it. See `docs/schemas/data-migration/07-verification.md` for the full spec.
