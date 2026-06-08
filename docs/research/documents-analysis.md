# Documents Domain — Business Analysis

## Purpose

The documents domain owns the school's formal
documentation: forms, certificates, and postal
correspondence. These are documents that the school
issues or receives as part of its operations.

This document describes how documents, forms, and
postal correspondence work in real schools, with the
edge cases that real schools hit.

## Key Concepts

- **Document** — a formal document issued or received
  by the school (transfer certificate, conduct
  certificate, bonafide certificate, etc.).
- **DocumentTemplate** — a template that defines the
  document's structure and content placeholders.
- **DocumentDispatch** — a postal dispatch record
  (outgoing mail).
- **DocumentReceive** — a postal receive record
  (incoming mail).
- **DocumentType** — a category of document
  (certificate, form, letter, circular).
- **DocumentStatus** — a lifecycle state
  (`Draft`, `Issued`, `Sent`, `Received`,
  `Archived`).

## Real-World Scenarios

### Transfer Certificate

A student is leaving the school. The admin issues
a Transfer Certificate (TC):

1. The admin opens the student's profile.
2. The admin clicks "Issue TC."
3. The system generates a TC with the student's
   details, the school details, and the reason
   for leaving.
4. The TC has a unique TC number, an issue date,
   and the principal's signature.
5. The TC is printed and handed to the parent.
6. The engine's `Document` aggregate records the
   TC; the document is archived in the student's
   record.

### Bonafide Certificate

A parent needs a Bonafide Certificate (e.g. for
a bank account opening). The parent applies
through the parent portal. The school's admin
reviews and issues. The engine's
`DocumentType::Bonafide` is the catalog entry.

### Conduct Certificate

A student is graduating. The school issues a
Conduct Certificate. The certificate states the
student's behavior and conduct during their time
at the school. The engine's `DocumentType::Conduct`
is the catalog entry.

### ID Card

The school issues ID cards to students and staff
at the start of the year. The engine's
`DocumentType::IdCard` is the catalog entry. The
ID card has:
- The student's photo.
- The student's name and class.
- The student's admission number.
- The school's name and logo.
- A QR code linking to the student's profile.

The engine's `IdCard` aggregate is a template;
the actual rendering is a port-driven PDF
generator.

### Postal Dispatch

The school sends letters to parents, vendors, and
government bodies. The admin records the dispatch:

1. The admin creates a `DocumentDispatch` with:
   - The recipient (name, address).
   - The subject.
   - The body (or a reference to a template).
   - The dispatch date.
   - The mode (post, courier, hand).
   - The tracking number (if any).
2. The dispatch is recorded.
3. The engine emits `DocumentDispatched`.

A dispatch may be linked to a parent complaint
resolution, a fee reminder, a leave approval
notification, etc.

### Postal Receive

The school receives letters from parents,
vendors, and government bodies. The admin records
the receive:

1. The admin creates a `DocumentReceive` with:
   - The sender (name, address).
   - The subject.
   - The receive date.
   - The mode (post, courier, hand).
   - An optional attachment scan.
2. The receive is recorded.
3. The engine emits `DocumentReceived`.

A receive may trigger an action (e.g. a parent's
letter requesting a TC triggers the TC issue
workflow).

### Bulk Document Generation

The school issues 200 TCs at the end of the
year (for graduating students). The engine's
`GenerateBulkDocuments` command is all-or-
nothing; a single validation failure aborts
the batch.

### Document Templates

A school may have custom templates for
certificates. The engine's `DocumentTemplate`
aggregate captures the template structure:
- A name.
- A category.
- A body template (with placeholders).
- An optional header / footer.
- An optional signature placeholder.

The actual rendering is a port-driven concern.

### Document Archive

A document is archived after the student
leaves or after a retention period. The
engine's `Document` aggregate's
`active_status` becomes `0`; the record is
retained for audit.

### Document Search

A school admin wants to find a specific TC.
The engine's document search supports
filtering by student, by document type, by
date range. The search is capability-gated.

## Business Rules

1. A `Document` requires a `DocumentType`, an
   issuer, a recipient, an issue date, and a
   status.
2. A `Document`'s status transitions are
   `Draft → Issued → Sent` (if dispatched)
   or `Draft → Issued → Archived`.
3. A `Document` cannot be deleted; it is
   archived with a reason.
4. A `DocumentDispatch` requires a recipient
   and a dispatch date.
5. A `DocumentReceive` requires a sender and
   a receive date.
6. A `DocumentTemplate` is unique by
   `(school_id, name)`.
7. A `Document`'s unique number (e.g. TC
   number) is auto-generated and unique
   within the school.
8. Documents are **storage-port driven**. The
   engine's `Document` aggregate is the
   metadata; the actual file is in the
   consumer's file storage.

## Edge Cases

### TC with Special Characters

A student's name contains a special character
(an apostrophe, an accent). The TC's PDF
renderer handles the character correctly. The
engine's `PersonName` value object validates
the name; the rendering is the consumer's
responsibility.

### TC for a Student Who Left Years Ago

A parent requests a TC for a student who left
the school 5 years ago. The admin retrieves
the archived `Document` and re-issues. The
audit log records the re-issue.

### Bulk TC Generation Failure

The school issues 200 TCs. The 50th TC
generation fails (a missing field). The
engine's bulk command aborts; the first 49
TCs are rolled back. The admin fixes the
issue and re-runs.

### Dispatch with No Tracking Number

The school sends a letter by regular post (no
tracking). The `DocumentDispatch` records the
mode as "post" and the tracking number as
`null`. The engine allows this.

### Receive with Attachment

The school receives a letter with a
supporting document (e.g. a court order). The
admin uploads the scan. The engine's
`DocumentReceive` records the attachment
reference.

### Document with Multiple Recipients

A circular is sent to all parents. The
engine's `Document` aggregate supports
multiple recipients. The dispatch is a single
record; the delivery is to each parent.

### Document Cross-Reference

A TC references a conduct certificate. The
engine's `Document` aggregate supports
cross-references; a document can list its
related documents.

### Document Retention

A school's retention policy is 7 years for
financial documents and indefinitely for
student records. The engine's retention is
configurable per document type.

### Document with Watermark

A school's TC has a watermark. The engine's
`DocumentTemplate` supports a watermark
configuration. The rendering is the
consumer's responsibility.

## Notes for SMScore Implementation

- The **documents** crate depends on
  `smscore-academic` for `StudentId` and
  `smscore-hr` for `StaffId`.
- The domain's documents are
  **storage-port driven**. The engine's
  `Document` aggregate is the metadata; the
  actual file is in the consumer's file
  storage.
- The domain's templates are
  **versioned**. A template may be updated;
  the engine retains the prior version for
  re-issuing old documents.
- The domain's bulk generation is
  **all-or-nothing**. A single validation
  failure aborts the batch.
- The domain's audit log captures every
  document issue, dispatch, and receive.
  The audit log is the canonical record
  for compliance.
- The domain's documents are linked to
  other aggregates (student, staff,
  complaint, dispatch). The links are
  enforced at the database level.
- The domain's retention is
  **per-document-type configuration**. The
  engine reads the retention from the
  settings domain.
- The domain's search is
  **capability-gated**. Only authorized
  users can search documents.
- The domain's dispatch and receive
  events feed the communication domain's
  notifications (e.g. a dispatched
  document to a parent triggers a
  "Document sent" notification).
