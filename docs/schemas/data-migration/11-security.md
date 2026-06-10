# 11 — Security

## Goal

Address the **real database credential that is in git history** and
any other security items that the migration touches. This work is
**independent of the migration phases** and must be done before any
public push of the repository.

## The credential in git history

Commit `5fa148c` ("docs: establish comprehensive project
documentation...") added a `.env` file with this line:

```text
DATABASE_URL="mysql://devuser:paxxw0rd@2791@127.0.0.1:3306/devdb"
```

The password `paxxw0rd@2791` is the live credential for the
`devuser` MySQL account on the `devdb` database. It is in the
repository's git history and is publicly visible in the commit.

**This is a real security issue.** Even if the repository is
private, any developer with clone access has the credential, and
the credential may be reused on other systems.

## Required actions (T-7d)

### 1. Rotate the `devuser` password

On the live MySQL server:

```sql
ALTER USER 'devuser'@'127.0.0.1' IDENTIFIED BY '<new-strong-password>';
FLUSH PRIVILEGES;
```

The new password should be:

- At least 32 random bytes (use `openssl rand -base64 48`).
- Stored only in the consumer's secrets manager (1Password,
  Vault, AWS Secrets Manager, etc.).
- Never committed to any file in the repository.

### 2. Update the consumer's secrets manager

Replace the old password with the new one. Every consumer app that
uses the `devuser` account is reconfigured.

### 3. Verify the rotation

From a separate machine, log in to MySQL with the new password:

```bash
mysql -u devuser -p'<new-password>' -h 127.0.0.1 -P 3306 devdb -e "SELECT 1;"
```

If the login succeeds, the rotation is complete. The old password
no longer works.

### 4. Purge the credential from git history

The credential is in commit `5fa148c`'s `.env` file. Even after the
rotation, the credential is **still in git history** and is
visible via `git log -p -- .env | grep paxxw0rd`. Any future
publish of the repository will leak the old password.

Use `git filter-repo` (recommended) or BFG Repo-Cleaner to remove
the file from history:

```bash
# Install git filter-repo (https://github.com/newren/git-filter-repo)
pip install git-filter-repo

# Remove .env from all history
git filter-repo --path .env --invert-paths

# Force-push to the remote (coordinate with the team)
git push origin --force --all
git push origin --force --tags
```

After the force-push, every developer must re-clone the repository
or run `git fetch origin && git reset --hard origin/master` (after
backing up any local work).

### 5. Verify the purge

```bash
# Should return nothing
git log -p --all -- .env | grep -i paxxw0rd
git log -p --all -- .env | grep -i 2791
```

Both greps should return empty. If anything is returned, the
purge is incomplete; re-run the filter-repo command.

### 6. Rotate again if the repository has been public

If the repository has been public at any point, the old credential
may be cached in:

- GitHub's public commit history (use `git filter-repo` and force-push
  to remove it from the upstream).
- Search engines, web archives, and security scanners.
- Public Git mirrors.

In that case, the password is **already known to the world**. The
rotation at step 1 is the only effective mitigation; the
`filter-repo` purge is cosmetic. A public-incident disclosure may
be required depending on the school's data protection policy.

## Other security items

### `.env` is gitignored

The `.gitignore` already has:

```text
.env
.env.*
!.env.example
```

The `.env` is gitignored. The `.env.example` is tracked. The
`.env` is on every developer's local disk but is never committed.

The current state of `.env.example` is the safe template:

- `DATABASE_URL` is a placeholder (`mysql://educore:educore@...`).
- All credentials are placeholder values (`replace-me-*`).
- The real credential is not in the file.

### No secrets in the docs

A grep of the `docs/` tree for common credential patterns:

```bash
grep -rE 'password.*=.*[a-zA-Z0-9]{8,}' docs/
grep -rE 'secret.*=.*[a-zA-Z0-9]{16,}' docs/
grep -rE 'token.*=.*[a-zA-Z0-9]{16,}' docs/
```

These should return only placeholders. The `.env.example` has
explicit `replace-me` markers, not real credentials.

### No secrets in the migrations

The 15 legacy migration files (`migrations/0001_academic.sql` through
`migrations/0015_settings.sql`) do not contain credentials. They
are SQL DDL only. The MySQL connection is via the consumer's
`DATABASE_URL`, not embedded in the migration files.

### The `system_user` row id is not a secret

The `system_user` row id is the well-known constant
`00000000-0000-7000-8000-000000000001`. This is a constant, not a
secret; it is the only "well-known" UUIDv7 in the engine. Every
aggregate's `created_by` and `updated_by` reference this id when
the system is the actor.

If the constant is changed in the future, the migration's
backfill (`UPDATE` statements that set `created_by` to the constant)
must be re-run.

### The IdGenerator's clock and the UUIDv7 timestamp

The `IdGenerator` uses the system clock to generate UUIDv7
timestamps. If the system clock is wrong (e.g. NTP drift), the
generated ids carry the wrong timestamp. The engine's `Clock`
port allows injecting a `FrozenClock` for tests; the production
deployment must use `SystemClock` and ensure NTP is configured.

The `id_v7_legacy BIGINT UNSIGNED NULL` column is **not** a secret
(it carries the original BIGINT for 90-day rollback support), but
it does carry the legacy `users` table's id for every user. If
the school's data subject access requests (DSARs) for the legacy
data are open, the `id_v7_legacy` column carries the same PII
exposure as the legacy `users` table. The column is dropped at
T+90d.

### The audit log's PII snapshots

The `audit_log` table has `before_snapshot` and `after_snapshot`
JSON columns that may contain PII (the engine's snapshot policy is
`Diff` for most aggregates and `Full` for high-sensitivity
aggregates). The retention period for these records is per the
audit-schema.md § 9:

| Record type | Retention |
| --- | --- |
| Finance mutations | 7 years |
| Payroll mutations | 7 years |
| Academic mutations | 7 years |
| Authentication events | 18 months |
| Authorization denials | 36 months |
| AI agent actions | 36 months |
| Settings changes | 3 years |
| Backup events | 3 years |
| Library / facilities | 3 years |

The consumer's deployment configures these via `AuditRetention`
config. PII is redacted from the `before` / `after` snapshots
based on the consumer's `AuditRedactor` config (default: redact
`password`, `secret`, `api_key`, `token`, `otp`).

### TLS for everything

All communication with the database, the event bus, the
notification provider, the payment provider, and the file
storage adapter is over TLS (`rustls`, per `code-standards.md`).
The `Cargo.toml` of every adapter crate has
`default-features = false, features = ["rustls-tls"]` or
equivalent.

The MySQL connection string must be `mysql://...?ssl-mode=REQUIRED`
or similar. The PostgreSQL connection string must be
`postgres://...?sslmode=require`. The SQLite engine does not
support TLS (it's a local file), so the consumer is responsible
for the file's filesystem encryption.

### Audit log access

The `audit_log` table is INSERT-only at the database level. The
consumer's MySQL user for the engine's writes has `INSERT` on
`audit_log` and no other privileges. A separate read-only user
exists for the audit query port.

```sql
-- Writes (engine's user)
GRANT INSERT ON devdb_v2.audit_log TO 'educore_writer'@'%';

-- Reads (audit query port's user)
GRANT SELECT ON devdb_v2.audit_log TO 'educore_audit_reader'@'%';
```

## Pre-push checklist

Before any public push of the repository:

- [ ] Live `devuser` password rotated.
- [ ] `.env` removed from git history (`git filter-repo`).
- [ ] Force-push to all remotes.
- [ ] Every developer has re-cloned or rebased.
- [ ] Grep verification passes.
- [ ] No real credentials in any file in the repository.
- [ ] `.env.example` is a clean template.
- [ ] `.gitignore` has `.env` and `.env.*`.
- [ ] MySQL `devuser` and `educore_writer` have least-privilege
      roles.

## Post-cutover (T+0)

- [ ] The consumer's app config has the new password.
- [ ] The legacy `devuser` is not used anywhere in production.
- [ ] The consumer's secrets manager rotation is complete.
- [ ] The school's data subject access requests (DSARs) for the
      legacy data are tracked and disclosed in the school's
      privacy records if required.

## Incident disclosure

If the credential has been public at any point (e.g. the repository
has been on a public Git host), the school may need to disclose
the incident under GDPR, FERPA, or local data protection
regulations. The disclosure requirements are country-specific;
the school's compliance team leads this.

The disclosure includes:

- The type of credential (database password).
- The data potentially exposed (real school data in `devdb`).
- The time window of exposure.
- The remediation steps (rotation, purge, MFA on the new password).
- The contact for follow-up.

## Exit criteria

- The live password is rotated.
- Git history is purged of the old password.
- The repository is safe to push publicly.
- The school's compliance team has signed off.
