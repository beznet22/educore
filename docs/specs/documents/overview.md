# Documents Domain Overview

## Purpose

The documents domain owns the school's two distinct paper-based
workflows that the digital system has to track: downloadable forms
that the school publishes for parents, students, or staff, and the
postal dispatch and receive log that the front office maintains.

The domain is intentionally **port-agnostic**. It models the records
and the lifecycle. Surface rendering to the website (for forms) and
the front-office portal (for postal) is performed by consumer
adapters. Actual file storage for form files and postal attachments is
held in the file storage port.

## Responsibilities

- Form publication: a downloadable form has a title, short description,
  publish date, optional URL, optional file, and a public-visibility
  flag.
- Form lifecycle: create, update, delete, and visibility toggle.
- Postal dispatch tracking: a dispatched letter has a `to_title`,
  `from_title`, reference number, address, date, note, and optional
  attachment.
- Postal receive tracking: a received letter has a `from_title`,
  `to_title`, reference number, address, date, note, and optional
  attachment.
- Postal dispatch and receive are non-overlapping aggregates; the
  lifecycle of each is independent.

## Boundaries

The documents domain does **not** own:

- File bytes. The file storage port holds the bytes; the domain holds
  only `FileReference`s.
- The physical transport of postal items. The domain records the fact
  of dispatch or receipt; it does not move a courier.
- Document signing or approval. The domain does not implement a
  signature workflow.
- Document categorization beyond the two aggregates it owns. Additional
  document types belong in other domains (e.g. certificate templates
  live in the academic domain; ID card templates live in the academic
  domain; notice attachments live in the communication domain).

The documents domain **does** provide identifier types and value
objects that other domains depend on: `FormDownloadId`,
`PostalDispatchId`, `PostalReceiveId`.

## Dependencies

- `smscore-core` — error types, result, identifier trait.
- `smscore-platform` — `SchoolId`, `UserId`, `TenantContext`.
- `smscore-rbac` — capability checks.
- `smscore-events` — domain event publishing.

## Domain Invariants

1. A `FormDownload` belongs to exactly one school.
2. A `FormDownload` has at least one of `link` or `file` set. A
   form with neither is not deliverable.
3. A `FormDownload` may be flagged `show_public = false`. Such forms
   are visible only to authenticated staff.
4. A `FormDownload` is never hard-deleted; it is soft-deleted via
   `active_status`.
5. A `PostalDispatch` belongs to exactly one school and one academic
   year.
6. A `PostalReceive` belongs to exactly one school and one academic
   year.
7. A `PostalDispatch` and a `PostalReceive` are independent; the
   engine does not pair them. Pairing is a reporting concern.
8. A `PostalDispatch` may be updated, but the `reference_no` is
   immutable once set; the dispatch is corrected by superseding
   records.
9. A `PostalReceive` may be updated, but the `reference_no` is
   immutable once set.

## Aggregate Roots

| Aggregate        | Root Type         | Purpose                                       |
| ---------------- | ----------------- | --------------------------------------------- |
| FormDownload     | `FormDownload`    | A downloadable form for parents, students, staff |
| PostalDispatch   | `PostalDispatch`  | A postal item dispatched by the school         |
| PostalReceive    | `PostalReceive`   | A postal item received by the school           |

Each aggregate is documented in detail under
`docs/specs/documents/aggregates.md`.

## Cross-Domain Impact

When a `FormDownload` is created, the documents domain emits
`FormUploaded`. The CMS domain may subscribe to surface the form on
the public site.

When a `PostalDispatch` is recorded, the documents domain emits
`PostalDispatched`. The communication domain may subscribe to send a
confirmation to the recipient. (This is a typical pattern; the
documents domain does not require it.)

When a `PostalReceive` is recorded, the documents domain emits
`PostalReceived`. The front office UI may subscribe to surface the
newly received item.

## Consumers

- Web admin UI (publish forms, log postal dispatch and receive).
- Web public site (download forms, when `show_public = true`).
- Mobile staff app (log postal items on the go).
- AI agent (publish forms, log postal items, update forms).

## Anti-Goals

- The documents domain does not present data to humans. It exposes
  commands, events, and queries.
- The documents domain does not implement a file storage backend.
  Files are held in the file storage port.
- The documents domain does not implement a postal tracking
  integration with a courier API. Tracking is a reporting concern.
- The documents domain does not own document approval or signature
  workflows.
