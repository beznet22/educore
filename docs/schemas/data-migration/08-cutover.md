# 08 — Application Cutover (Phase 8)

## Goal

Switch the consumer's application (the Laravel Schoolify app being
replaced, or the new Educore consumer app) to read from
`devdb_v2` instead of `devdb`. The switch is a config change and a
process restart. Downtime is seconds.

## Pre-cutover checklist (T-1h)

- [ ] Phase 1 (engine tables) applied and verified.
- [ ] Phases 2-6 (rename, ID conversion, column adds, brand
      removal, field data flow) applied and verified.
- [ ] Phase 7 (verification) passed end-to-end.
- [ ] Rollback script printed and taped to the wall.
- [ ] On-call contact list current.
- [ ] Read-only backup of `devdb` taken and confirmed restorable.
- [ ] Consumer's `.env` updated to point at `devdb_v2`.
- [ ] Consumer's HTTP / API process stopped (or in maintenance
      mode).
- [ ] Outbox relay stopped.
- [ ] Audit sink consumer stopped.

## Cutover steps (T+0)

1. **Stop the consumer's HTTP process** (or put it in maintenance
   mode). The app returns 503 for all requests.

2. **Stop the outbox relay** (if running). This is a consumer-side
   process; it reads from `outbox` and publishes to the event bus.

3. **Stop the audit sink consumer** (if running). It writes to
   `audit_log`; not strictly necessary to stop, but cleaner.

4. **Update the consumer's `DATABASE_URL`** to point at `devdb_v2`.
   The format is unchanged:

   ```bash
   # devdb (legacy)
   DATABASE_URL="mysql://devuser:...@127.0.0.1:3306/devdb"

   # devdb_v2 (engine)
   DATABASE_URL="mysql://educore:...@127.0.0.1:3306/devdb_v2"
   ```

   The new password is the one issued at T-7d after the rotation
   (see `11-security.md`).

5. **Restart the consumer's HTTP process.** The new connection
   pool comes up against `devdb_v2`.

6. **Restart the outbox relay.** It reads from `devdb_v2.outbox`
   and publishes to the bus.

7. **Restart the audit sink consumer.**

8. **Smoke test the consumer's app:**
   - Login as a school admin.
   - Admit a test student (creates `academic_students` row + emits
     `academic.student.admitted` event + writes `audit_log` row).
   - Verify the new row appears in `devdb_v2.academic_students`.
   - Verify the new event appears in `devdb_v2.outbox` and is
     marked `published_at` after the relay runs.
   - Verify the audit record appears in `devdb_v2.audit_log`.

9. **Take the app out of maintenance mode.**

## Post-cutover (T+0 to T+1h)

- **Watch the error rate** for the first hour. Any spike is a
  sign of a missed column transform or a FK mismatch.
- **Watch the outbox lag** (rows in `outbox` with `published_at
  IS NULL`). A growing lag means the relay can't process new
  events; the likely cause is a missing event type registration
  in `schema_registry`.
- **Watch the audit volume** (rows in `audit_log` per hour). A
  sudden drop means the engine's audit sink is failing silently.

## Failure modes

### Engine rejects a command because the schema is missing a column

The error: `StorageError::ColumnNotFound` from
`educore-storage-mysql`. The engine emits a `DomainError::Infrastructure`
to the caller. The HTTP layer returns 500.

**Recovery**: roll back per `10-rollback.md`. The rollback script
restores `devdb` (the read-only backup) and the app re-points at it.
The migration is re-attempted after the bug is fixed.

### Outbox is growing, not draining

The relay worker is reading from `outbox` but failing to publish.
Check the `last_error` column on the rows in question:

```sql
SELECT event_id, event_type, attempts, last_error
FROM devdb_v2.outbox
WHERE published_at IS NULL
ORDER BY enqueued_at ASC
LIMIT 10;
```

**Recovery**: if the failure is a transient (e.g. event bus is
down), the relay retries with backoff. If the failure is a bug
(e.g. a malformed event), the row is quarantined in the
`outbox_quarantine` table (consumer-side) and the rest of the
queue proceeds.

### Audit volume drops to zero

The audit sink is failing. Check the audit consumer's logs.

**Recovery**: roll back if the failure is not recoverable in 15
minutes.

## What the consumer should monitor

A monitoring dashboard for the first 7 days post-cutover:

- HTTP request rate and error rate
- Outbox lag (rows with `published_at IS NULL`)
- Audit volume (rows per hour)
- Event bus lag (consumer-side metric)
- Storage connection pool exhaustion
- Idempotency cache hit rate (rows in `idempotency` that are within
  their 7-day window)

## T+7d review

A formal review of the cutover with the school:

- Are all school operations working as expected?
- Are any school staff reporting issues?
- Is the data consistent with the legacy DB (where the school has
  the patience to spot-check)?
- Are any audit log entries missing or duplicated?

## Exit criteria

- App is in maintenance mode.
- `devdb_v2` is populated and verified (Phase 7).
- Consumer's `DATABASE_URL` points at `devdb_v2`.
- App is restarted; smoke test passes.
- App is out of maintenance mode.
- Monitoring dashboard is in place.
- On-call contact is established for the next 7 days.

## Rollback at cutover

If anything goes wrong in the first 15 minutes after cutover, run
the rollback script in `10-rollback.md`:

1. Stop the consumer's HTTP process.
2. Re-point `DATABASE_URL` at `devdb`.
3. Restart the consumer's HTTP process.
4. Verify the app works against `devdb`.
5. Investigate the bug.
6. Re-attempt the cutover when the bug is fixed.

The rollback is fully reversible because `devdb` is unchanged
during the migration (Option B is side-by-side + cutover).
