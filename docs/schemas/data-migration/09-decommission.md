# 09 — Decommission (Phase 9)

## Goal

Archive `devdb` (the legacy Schoolify/InfixEdu database) 30 days
after cutover. The archive is a frozen, read-only snapshot kept for
compliance and forensics. The legacy database is no longer
queried by any application.

## T+30d — pre-decommission checklist

- [ ] Cutover is complete (Phase 8) and the app is on `devdb_v2`
      for at least 30 days.
- [ ] No production app is still configured to read from `devdb`.
- [ ] No scheduled job, cron, or background worker is still
      pointing at `devdb`.
- [ ] The school's audit retention policy has been met for the
      legacy data (typically 7 years for finance, 18 months for
      auth events, 3 years for system events). The legacy data is
      preserved in the archive to honor the retention.
- [ ] The school's data subject access requests (DSARs) for the
      legacy data are complete. The legacy database may still hold
      PII that is subject to erasure.
- [ ] A snapshot of `devdb` is taken immediately before the
      decommission. The snapshot is the archive.

## The archive

The archive is a single MySQL dump file:

```bash
mysqldump \
  --single-transaction \
  --quick \
  --routines \
  --triggers \
  --events \
  --hex-blob \
  --default-character-set=utf8mb4 \
  --result-file=devdb_archive_$(date +%Y%m%d).sql \
  devdb
```

The dump is gzipped:

```bash
gzip devdb_archive_*.sql
```

The archive is uploaded to cold storage (S3 Glacier, Azure Archive,
GCS Coldline) with object lock for the retention period. The
retention defaults:

| Data class | Retention |
| --- | --- |
| Finance, payroll | 7 years |
| Authentication, authorization | 18 months |
| Academic records | 7 years |
| All other | 3 years |

The archive's metadata:

```yaml
archive_id: devdb-archive-2026-08-08
source_database: devdb
decommission_reason: migration_to_educore_engine
migration_target_database: devdb_v2
migration_date: 2026-07-08
retention_until: 2033-08-08
encryption: aes-256-gcm
object_lock: governance-mode, retain-until-date
sha256: <digest-of-archive>
size: <bytes>
table_count: 310
row_count: ~52,000
```

## Decommission steps

1. **Take a final snapshot** of `devdb` (the dump above).
2. **Verify the dump is restorable** on a clone. Test restore, verify
   row counts, verify a sample row.
3. **Upload to cold storage** with object lock.
4. **Mark `devdb` as read-only** in MySQL:

   ```sql
   FLUSH TABLES WITH READ LOCK;
   -- All sessions block on writes. The DB is now read-only.
   ```

5. **Disconnect the legacy `DATABASE_URL`** from the consumer's
   app config. The app no longer has credentials to `devdb`.
6. **Disconnect from the consumer's secrets manager.** The legacy
   password is removed from rotation.
7. **Document the decommission** in the school's IT records. The
   archive's `archive_id` and `sha256` are recorded.

## What happens to `devdb` itself

The MySQL database `devdb` is **dropped** 90 days after the
decommission. The 90-day window is the safety margin in case a
late-discovered bug requires a rollback.

At T+90d, after the safety window:

```sql
DROP DATABASE devdb;
```

The MySQL `devuser` account is also revoked:

```sql
DROP USER 'devuser'@'127.0.0.1';
FLUSH PRIVILEGES;
```

## What happens to the `migrations/0001_*.sql` through `0015_*.sql` files

The 15 legacy migration files are **kept** in the repository. They
are:

- A research source for understanding the legacy data shapes.
- A reference for the ETL script.
- A historical record.

They are not applied to `devdb_v2` and will not be applied to any
new database. The engine's `migrations/README.md` documents this
explicitly.

## What happens to `schoolify/`

The `schoolify/` Laravel project source tree in the repository is
**kept** as a research source. It is read-only and is referenced
by the graphify plugin. The `docs/research/schoolify-analysis.md`
file is the only entry point; the engine docs do not link into the
Laravel source.

## The T+30d review

A formal review with the school confirms:

- All daily operations are stable on `devdb_v2`.
- No school staff are reporting data anomalies.
- The audit log is complete and verifiable.
- The school's data subject access requests (DSARs) for the
  legacy data are closed.

The review is the gate to T+30d decommission. If the school reports
issues, the decommission is delayed.

## What the school can still access after decommission

For 7 years (per the retention policy):

- **Audit records** are searchable in the engine's `audit_log`
  table on `devdb_v2`. The engine's storage adapter exposes a
  query API for this.
- **Archived legacy data** is in cold storage with object lock.
  Access requires a compliance team ticket.

For 90 days after decommission:

- **The legacy `devdb` itself** is read-only in MySQL. A DBA can
  run a one-off query against it for forensics.

After 90 days:

- The legacy `devdb` is dropped. The cold-storage archive is the
  only remaining record.

## Exit criteria

- `devdb` snapshot is in cold storage with object lock.
- `devdb` is read-only in MySQL.
- Consumer's `DATABASE_URL` is removed from production config.
- `devuser` is removed from secrets manager.
- T+30d review passed.
- T+90d `DROP DATABASE` is scheduled (cron entry).
- The school's IT records note the decommission with the
  `archive_id` and `sha256`.
