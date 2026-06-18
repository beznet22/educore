# Phase 11 (Documents) — Prep Verification Report

**Verifier:** R1-reconcile-prep
**Timestamp:** 2026-06-15
**Mode:** Read-only verification (no source files modified, no commits made)

---

## Section A: PASS/FAIL Summary

| # | Check | Verdict |
|---|---|---|
| 1 | Documents crate file layout | **PASS** |
| 2 | Section markers in the 6 shared files | **PASS** (with note: markers include owner annotation) |
| 3 | Prelude re-exports the 9 modules | **PASS** (72 types re-exported across 9 modules) |
| 4 | Value objects match prelude names | **PASS** (22 types in vo, 0 missing, 0 extra) |
| 5 | 11 net-new Capabilities in rbac | **PASS** (rbac compiles cleanly) |
| 6 | 4 retained + 11 new = 15 Documents caps | **PASS** (15 documents-related caps in `all()`) |
| 7 | 2 new AuditTarget variants in audit | **PASS** (3 variants + arms present) |
| 8 | 3 coverage rows Pending | **PASS** (3 rows, all `status=Pending`, `phase=11`) |
| 9 | 6 prep commits in chronological order | **PARTIAL PASS** (5 Phase 11 commits; 6th "scaffold" already in place from prior workspace restructure) |
| 10 | `cargo check -p educore-documents` compiles | **FAIL** (13 `#![deny(missing_docs)]` errors in `errors.rs`) |

---

## Section B: Evidence Per Check

### Check 1: Documents crate file layout — PASS

```
$ ls -la crates/domains/documents/src/
total 80
-rw-rw-r-- 1 beznet beznet  1057 Jun 15 16:06 aggregate.rs
-rw-rw-r-- 1 beznet beznet   741 Jun 15 16:06 commands.rs
-rw-rw-r-- 1 beznet beznet  6527 Jun 15 16:03 entities.rs
-rw-rw-r-- 1 beznet beznet  1048 Jun 15 15:49 errors.rs
-rw-rw-r-- 1 beznet beznet  644 Jun 15 16:06 events.rs
-rw-rw-r-- 1 beznet beznet  3197 Jun 15 16:05 lib.rs
-rw-rw-r-- 1 beznet beznet   495 Jun 15 16:06 query.rs
-rw-rw-r-- 1 beznet beznet   548 Jun 15 16:06 repository.rs
-rw-rw-r-- 1 beznet beznet   840 Jun 15 16:06 services.rs
-rw-rw-r-- 1 beznet beznet 29968 Jun 15 16:03 value_objects.rs
```

All 10 required files present: `lib.rs`, `aggregate.rs`, `entities.rs`, `value_objects.rs`, `commands.rs`, `events.rs`, `services.rs`, `repository.rs`, `query.rs`, `errors.rs`.

### Check 2: Section markers in the 6 shared files — PASS

Files inspected: `aggregate.rs`, `events.rs`, `commands.rs`, `services.rs`, `repository.rs`, `query.rs`.

Each file has exactly 3 section marker pairs. Markers use the format:

```
// === FormDownload section begin (owner: 1A) ===
// === FormDownload section end ===
```

The owner annotation `(owner: 1A/1B/1C/2A/2B/2C/3A/3B/3C)` is in addition to the required name. The section-marker strings `FormDownload section begin/end`, `PostalDispatch section begin/end`, and `PostalReceive section begin/end` are all present.

Each section is non-empty (each contains at least one `pub struct`, `pub trait`, or `pub fn`). Examples:

- `aggregate.rs:20-22` — `FormDownload`, `FormDownloadFile`, `FormDownloadLink` structs
- `commands.rs:7-9` — `UploadFormCommand`, `UpdateFormCommand`, `DeleteFormCommand`
- `services.rs:21-24` — `receive_postal_service`, `update_postal_receive_service`, `delete_postal_receive_service`, `track_postal_service`

### Check 3: Prelude re-exports the 9 modules — PASS

`crates/domains/documents/src/lib.rs:28-63` defines `pub mod prelude { ... }` with 9 `pub use` blocks:

1. `crate::aggregate::{FormDownload, FormDownloadFile, FormDownloadLink, PostalDispatch, PostalDispatchAttachment, PostalReceive, PostalReceiveAttachment}` (7)
2. `crate::commands::{10 commands}` (10)
3. `crate::entities::{4 child entity ids}` (4)
4. `crate::errors::{DocumentsError, Result}` (2)
5. `crate::events::{9 events}` (9)
6. `crate::query::{3 query stubs}` (3)
7. `crate::repository::{3 repository traits}` (3)
8. `crate::services::{2 structs + 10 fns}` (12)
9. `crate::value_objects::{22 value types}` (22)

**Total: 72 re-exports** across 9 modules. All 9 modules and every required type from the verification checklist is present.

### Check 4: Value objects match prelude names — PASS

```python
prelude types: 22
vo types: 22
missing in vo: []
extra in vo (not re-exported): []
```

The 22 types defined in `value_objects.rs` are exactly the 22 re-exported via the prelude. Zero missing, zero extra. (The 50 other re-exports in the prelude come from `aggregate`, `commands`, `entities`, `errors`, `events`, `query`, `repository`, `services` modules — not `value_objects`.)

The 22 `value_objects` types:

```text
ActiveStatus, DispatchDate, DocumentType, DocumentVisibility, FileReference,
FormDescription, FormDownloadId, FormTitle, FromAddress, FromTitle,
PostalAddress, PostalDispatchId, PostalNote, PostalReceiveId, PostalReferenceNo,
PostalTitle, PublishDate, ReceiveDate, ShowPublic, ToAddress, ToTitle, Url
```

### Check 5: 11 net-new Capabilities in rbac — PASS

`crates/cross-cutting/rbac/src/value_objects.rs` declares 11 new `Capability` variants on lines 713-734:

```rust
FormDownloadUpload,       // 713
FormDownloadUpdate,       // 715
FormDownloadDelete,       // 717
FormDownloadRead,         // 719
PostalDispatchCreate,     // 721
PostalDispatchUpdate,     // 723
PostalDispatchDelete,     // 725
PostalReceiveCreate,      // 727
PostalReceiveUpdate,      // 729
PostalReceiveDelete,      // 731
PostalRead,               // 733
```

All 11 appear in `domain()` (lines 1214-1224, mapping to `CapabilityDomain::Documents`). All 11 appear in `all()` (see Check 6). `aggregate()` arms map `FormDownload*` to `"FormDownload"`, `PostalDispatch*` to `"PostalDispatch"`, `PostalReceive*` to `"PostalReceive"`, and `PostalRead` to `"Postal"` (lines 1617-1627).

`cargo check -p educore-rbac` finishes cleanly:

```
Checking educore-rbac v0.1.0 (/home/beznet/Workspace/smscore/crates/cross-cutting/rbac)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.17s
```

### Check 6: 4 retained + 11 new = 15 Documents caps — PASS

```python
Documents caps in all(): 15
['DocumentsFolderCreate', 'DocumentsFolderDelete', 'DocumentsFolderRead',
 'DocumentsFolderUpdate', 'FormDownloadDelete', 'FormDownloadRead',
 'FormDownloadUpdate', 'FormDownloadUpload', 'PostalDispatchCreate',
 'PostalDispatchDelete', 'PostalDispatchUpdate', 'PostalRead',
 'PostalReceiveCreate', 'PostalReceiveDelete', 'PostalReceiveUpdate']
```

Exactly 15: 4 retained Phase 2 `DocumentsFolder*` placeholders + 11 net-new Phase 11 variants.

### Check 7: 2 new AuditTarget variants in audit — PASS

`crates/cross-cutting/audit/src/writer.rs`:

- **Variants** (lines 298-304): `FormDownload(Uuid)`, `PostalDispatch(Uuid)`, `PostalReceive(Uuid)` — all 3 present (PostalDispatch was retained from prior phase).
- **`target_type()` arms** (lines 419-421):
  ```rust
  Self::FormDownload(_) => "form_download",
  Self::PostalDispatch(_) => "postal_dispatch",
  Self::PostalReceive(_) => "postal_receive",
  ```
- **`target_id()` arms** (lines 522-524):
  ```rust
  Self::FormDownload(id)
  | Self::PostalDispatch(id)
  | Self::PostalReceive(id)
  ```

A targeted test `documents_audit_target_round_trip_for_all_aggregates` exists (lines 956-972) that asserts all three wire forms and round-trip the inner UUIDs.

### Check 8: 3 coverage rows Pending — PASS

```
documents_postal_dispatches_aggregate   phase=11 status=Pending
documents_form_downloads_aggregate       phase=11 status=Pending
documents_postal_receives_aggregate      phase=11 status=Pending
```

All 3 rows present with `crate = "educore-documents"`, `phase = 11`, `status = "Pending"`.

### Check 9: 6 prep commits land in chronological order — PARTIAL PASS

```
$ git log --pretty=format:"%h %ai %s" -10
fb4e7e0 2026-06-15 17:32:36 Phase 11: add 11 Documents* capabilities + extend DefaultRoleCatalog (prep)
cf58c8f 2026-06-15 16:10:58 Phase 11: documents value objects + child entities (prep)
15c8c02 2026-06-15 15:50:04 Phase 11: add 2 Documents AuditTarget variants + 3 coverage rows (prep)
f886295 2026-06-15 15:47:16 Phase 11: add educore-documents dep to storage-parity (prep)
350a2e2 2026-06-15 15:45:12 Phase 11: add deps to educore-documents Cargo.toml (prep)
fc0bfda 2026-06-15 15:07:50 feat(communication): ship 6-scenario integration test + docs + coverage flips
...
```

5 Phase 11 prep commits are present (oldest first):

1. `350a2e2` P0a — add deps to educore-documents Cargo.toml
2. `f886295` P0b — add educore-documents dep to storage-parity
3. `15c8c02` P0f — add 2 Documents AuditTarget variants + 3 coverage rows
4. `cf58c8f` P0d — documents value objects + child entities
5. `fb4e7e0` P0e — add 11 Documents* capabilities + extend DefaultRoleCatalog

**Missing 6th commit:** `Phase 11: documents crate scaffold + prelude + shared types (prep)` (P0c).

**Explanation:** The scaffold files (`lib.rs`, `aggregate.rs`, `events.rs`, `commands.rs`, `services.rs`, `repository.rs`, `query.rs`, `errors.rs`) were created on 2026-06-09/10 in prior commits `c280e91` and `7f447f6` (the workspace restructuring: "migrate core and engine modules to new infra crate structure" and "restructure workspace into domain, adapter, and cross-cutting layers"). When the P0c agent ran, the scaffold was already in place — the section markers and empty structs in the 6 shared files were the only P0c additions, and these were rolled into the P0d value-objects commit (the value-objects commit is dated 16:10:58 and adds 1220 lines to `value_objects.rs` and `entities.rs`).

The intent of P0c (a 9-file module layout with section markers) is satisfied — the 6 shared files all exist with section markers, and `lib.rs` has the prelude. The work landed, just not as a separate commit.

**Chronological order:** The 5 visible commits are in correct chronological order (a, b, f, d, e). Note that the conventional subagent order would be a → b → c → d → e → f, but the order observed (a → b → f → d → e) reflects that f landed before d/e because f needed only the audit writer and coverage.toml, both of which were ready first.

### Check 10: `cargo check -p educore-documents` compiles — FAIL

```
$ cargo check -p educore-documents
    Checking educore-documents v0.1.0 (/home/beznet/Workspace/smscore/crates/domains/documents)
error: missing documentation for a type alias
 --> crates/domains/documents/src/errors.rs:5:1
  |
5 | pub type Result<T> = core::result::Result<T, DocumentsError>;
  | ^^^^^^^^^^^^^^^^^^

error: missing documentation for an enum
 --> crates/domains/documents/src/errors.rs:8:1
  |
8 | pub enum DocumentsError {

error: missing documentation for a variant
  --> crates/domains/documents/src/errors.rs:10:5
   |
10 |     Validation(String),
...

error: could not compile `educore-documents` (lib) due to 13 previous errors
```

**All 13 errors are in `crates/domains/documents/src/errors.rs`**, triggered by `#![deny(missing_docs)]` in `lib.rs:8`. The lint requires rustdoc on:

- The `Result` type alias (line 5)
- The `DocumentsError` enum (line 8)
- 11 enum variants (`Validation`, `FormHasNoContent`, `FormNotFound`, `PostalDispatchNotFound`, `PostalReceiveNotFound`, `DuplicateReferenceNo`, `ReferenceNoImmutable`, `Forbidden`, `Conflict`, `Infrastructure`, `Other`)

The other 6 shared files (`aggregate.rs`, `events.rs`, `commands.rs`, `services.rs`, `repository.rs`, `query.rs`) all start with `#![allow(missing_docs)]` to suppress this lint, which is why they don't fail. `errors.rs` was not given the same allow attribute and lacks doc comments.

This is a 1-file, ~13-line fix. The crate is otherwise complete and correct.

---

## Section C: Recommendations

### 1. Fix the `errors.rs` `#![deny(missing_docs)]` failures (blocks 1A)

The 1A subagent (or a 0.x prep subagent) must add rustdoc to all 11 variants of `DocumentsError`, the `DocumentsError` enum itself, and the `Result` type alias. The pattern is straightforward — one `///` line per item. Example for the validation variant:

```rust
/// Validation failed: a value-object invariant was violated
/// (empty string, length out of range, malformed URL, etc.).
#[error("validation: {0}")]
Validation(String),
```

This pattern matches every other domain crate (e.g. `crates/domains/finance/src/errors.rs`, `crates/domains/library/src/errors.rs`) — copy the style from there. After the fix, `cargo check -p educore-documents` should pass with zero errors.

**Alternative (less recommended):** Add `#![allow(missing_docs)]` at the top of `errors.rs`, matching the 6 shared files. The project standard is doc comments (`#![deny(missing_docs)]` is set in `lib.rs`), so the doc-comment fix is preferred.

### 2. Optional: add a 6th P0c commit to the Phase 11 commit chain

Not blocking, but for consistency with the prompt's expectation of 6 prep commits, a 0.x agent could amend the prep history to add a `Phase 11: documents crate scaffold + prelude + shared types (prep)` commit between P0b and P0d, capturing the section-marker additions in the 6 shared files. This is purely cosmetic — the work is already on `main`.

### 3. Notes for subagent 1A (FormDownload owner)

- The `FormDownload`, `FormDownloadFile`, `FormDownloadLink` placeholder structs in `aggregate.rs:20-22` need to be replaced with real implementations. The struct names and module paths are already exported via the prelude, so 1A only needs to fill in the bodies.
- The 3 `Form*` event structs and 3 `Form*` command structs are placeholders in `events.rs:7-9` and `commands.rs:7-9`. They have the right names; 1A/2A must replace them with real types.
- The 3 service fns in `services.rs:8-10` and the `FormDownloadRepository` trait in `repository.rs:7` are empty stubs. 3A must wire them.
- The `FormDownloadQuery` stub in `query.rs:7` is a unit struct; 3A must populate it.

### 4. Notes for subagent 1B (PostalDispatch owner)

Same pattern as 1A, applied to `aggregate.rs:26-27` (`PostalDispatch`, `PostalDispatchAttachment`), `events.rs:13-15` (3 events), `commands.rs:13-15` (3 commands), `services.rs:15-17` (3 service fns), `repository.rs:11` (`PostalDispatchRepository`), `query.rs:11` (`PostalDispatchQuery`).

### 5. Notes for subagent 1C (PostalReceive owner)

Same pattern as 1A, applied to `aggregate.rs:31-32` (`PostalReceive`, `PostalReceiveAttachment`), `events.rs:19-21` (3 events), `commands.rs:19-22` (4 commands — `ReceivePostalCommand`, `UpdatePostalReceiveCommand`, `DeletePostalReceiveCommand`, `TrackPostalCommand`), `services.rs:21-24` (4 service fns), `repository.rs:15` (`PostalReceiveRepository`), `query.rs:15` (`PostalReceiveQuery`).

### 6. Note on the `educore_platform::FileReference` re-export

The task brief mentions "`educore_platform::FileReference` re-export". The current `value_objects.rs` defines a **local** `FileReference` (lines 685-716) and exports it from the prelude. The comment at the top of `value_objects.rs:15-18` explains the rationale:

> The `FileReference` type is a local copy of the engine-wide file-reference pattern; the engine-wide type lives in `educore_files` (Phase 15) but the documents crate needs a local copy today to keep the dependency surface minimal.

This is a deliberate design decision. The prelude re-export resolves to the local type, not to `educore_platform::FileReference`. 1A/1B/1C should be aware that the public type name is `FileReference` (local), and that the platform adapter can be added later in Phase 15.

---

## Section D: GO/NO-GO

**NO-GO** for wave 1 (1A/1B/1C).

**Blocking failure (1):**

- **Check 10:** `cargo check -p educore-documents` fails with 13 `#![deny(missing_docs)]` errors in `crates/domains/documents/src/errors.rs`. The crate does not compile, so the 9-file module layout is not yet "shippable" as required by the validation checklist. Fix is small (~13 doc-comment lines) and well-localized.

**Non-blocking (no action required, but documented):**

- **Check 9:** 5 Phase 11 prep commits instead of 6. The 6th ("scaffold + prelude + shared types") was rolled into the value-objects commit because the scaffold files were already in place from the earlier workspace restructure (`c280e91` / `7f447f6` on 2026-06-09/10). The intent of the missing P0c commit is satisfied; the commit-history nit is cosmetic only.

Once the `errors.rs` doc-comment fix lands, all 10 checks pass and wave 1 (1A/1B/1C) is GO.
