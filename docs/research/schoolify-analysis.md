# Schoolify Analysis — High-Level Overview

## Purpose

This document is the bridge between the operational knowledge
of real schools and the design of the SMScore engine. It
describes what schools actually do, why they do it that way,
and how the engine encodes those behaviors.

The analysis is **behavioral**. It does not describe the
specific system that informed the engine, the file
layout, the technology stack, or the implementation
language. It describes the school.

## What a School Is

A school is a long-lived organization with a regular,
predictable rhythm: classes, attendance, exams, fees, payroll,
events, communication. It has a fixed catalog of people
(students, parents, staff) and a fixed catalog of structures
(classes, sections, subjects, academic years). It collects
money in (fees, donations) and pays money out (salaries,
expenses, vendor bills). It is regulated (by the government,
by an accreditation body, by its own board) and audited.

A school's most important quality is **trust**. Parents
trust the school with their children. Regulators trust the
school with student data. Courts trust the school's records
in disputes. Staff trust the school with their livelihoods.
The school's information system is the mechanism by which
these trusts are honored.

## The Domains of a School

A school is naturally partitioned into 15 bounded contexts:

1. **Academic** — students, classes, sections, subjects,
   academic years, routines, homework, lessons.
2. **Assessment** — exams, marks, results, report cards.
3. **Attendance** — daily attendance for students and staff.
4. **Finance** — fees, invoices, payments, banking, expenses,
   payroll, wallets.
5. **HR** — staff, leave, attendance, designations, payroll.
6. **Library** — books, members, issues, returns.
7. **Facilities** — transport, hostel, inventory.
8. **Communication** — notices, complaints, chat, notifications.
9. **Events** — calendar, holidays, incidents.
10. **Documents** — forms, postal dispatch / receive.
11. **CMS** — public website content.
12. **Platform** — multi-school tenancy, users, lookup data.
13. **RBAC** — roles, capabilities, two-factor authentication.
14. **Settings** — per-tenant configuration, theming, language.
15. **Operations** — backups, jobs, system versions, audit
    projection.

These domains are not arbitrary. They are the school's
**organizational** structure. A school has a principal, a
vice principal, a fee collector, a librarian, a transport
in-charge, a system administrator. Each of these roles owns
a bounded context. The school's information system maps to
its organization.

## Cross-Domain Dynamics

Domains do not live in isolation. A school's daily life is a
constant cross-domain flow:

- A student is **admitted** in academic; **fees are
  assigned** in finance; a **library membership** is
  created; a **welcome message** is sent to the parent;
  if transport was opted in, a **route assignment** is
  created.
- An exam is **scheduled** in assessment; **marks are
  recorded**; a **result is computed**; a **report card
  is generated**; a **notice** is sent to parents.
- A payroll is **generated** in finance from the month's
  attendance in attendance; **approved** by the principal;
  **paid** from the bank; a **payslip** is sent to the
  staff.
- A **notice** is sent; parents **read** it; the school
  has **evidence** of delivery.

The engine's event-driven design is not a technology
choice; it is a model of how schools actually work.

## Real-World Scenarios

A school's day, abstracted:

1. **Morning**: The school day begins. Staff clock in
   (HR attendance). The transport officer checks the
   bus routes (facilities).
2. **Mid-morning**: The teacher takes attendance for
   each period. Absent students generate notifications
   to parents. (attendance + communication).
3. **Mid-day**: The accountant collects a parent's fee
   payment (finance). The cashier logs an expense
   (finance). A staff member applies for leave
   (HR + finance).
4. **Afternoon**: The teacher enters marks for an exam
   (assessment). A parent files a complaint (communication).
5. **End of day**: The transport officer marks which
   students boarded which bus (facilities). The
   librarian issues a book to a student (library).
6. **End of month**: Payroll is generated (finance + HR).
   Bank reconciliation is performed (finance).
7. **End of term**: Exams are scheduled, marks are
   recorded, results are computed, report cards are
   generated (assessment). Promotions are decided
   (academic). Fees are carried forward (finance).
8. **End of year**: The current academic year is
   closed (academic). The next year is opened.
   Students are promoted. New admissions begin.

The engine's domains and commands model this rhythm. The
events are the school's heartbeat.

## Business Rules That Are Universal

Across schools, certain rules are universal:

- A student can be admitted only once per academic year.
- A fees payment cannot exceed the outstanding balance.
- A payroll cannot be paid twice for the same month.
- A book cannot be issued if all copies are out.
- A leave application requires approval before it affects
  payroll.
- An exam result cannot be published before all marks are
  entered.
- A bus route cannot be deleted while students are
  assigned to it.
- A user can log in only with the correct credentials and,
  if 2FA is enabled, the correct second factor.
- Every monetary transaction is recorded with a bank
  account, a date, a mode, and an actor.

The engine encodes these rules in its domain logic. The
rules are not configurable per school; they are the
engine's invariants.

## Business Rules That Vary

Other rules vary by school:

- The pass mark for an exam (50% in some schools, 33% in
  others).
- The fees amount per class.
- The number of installment plans offered.
- The grading scale (A/B/C/D/F or numerical).
- The school year start date.
- The currency, language, and time zone.
- The list of optional subjects.
- The list of student categories (scholarship, staff
  child, etc.).
- The notification channels (SMS, email, push).

The engine treats these as **per-tenant configuration**,
not as domain rules. The consumer's school admin configures
them at onboarding; the engine reads them at command
dispatch time.

## Edge Cases That Real Schools Hit

Real schools hit edge cases the engine must handle:

- **A student whose family is in arrears but who has a
  scholarship.** The fees waiver applies; the parent's
  portal shows the waived amount.
- **A staff member who is also a parent in the same
  school.** Their user account has both staff and parent
  roles. The engine resolves the active role from the
  active `TenantContext`.
- **A class that has more students than its capacity.**
  The school may choose to exceed the capacity for a
  one-time intake. The engine does not enforce the
  capacity; the consumer configures whether to enforce.
- **An exam scheduled on a holiday.** The school
  reschedules. The engine emits a `ExamRescheduled`
  event; downstream systems (notifications, room
  bookings) react.
- **A bank transfer that fails.** The engine emits
  `PaymentFailed`; the cashier retries; the engine
  produces a `PaymentReversed` and a new `PaymentCollected`.
- **A student who transfers mid-term.** The engine
  withdraws the student in the source school and
  admits them in the destination school. The
  `TransferStudent` command is cross-tenant and
  capability-gated.
- **A parent who is also a guardian of two siblings.**
  The parent has two `StudentRecord` children. Fees
  are per-student; the parent sees both. The
  communication routes to the parent's user account.

## How the Knowledge Shaped the Engine

The behavioral knowledge of schools shaped the engine's
design in concrete ways:

- **Multi-tenancy by default.** Every school is a tenant;
  isolation is structural.
- **Capability-based permissions.** A school admin can do
  everything; a teacher can do their work; a parent can
  see their children; a student can see their own
  progress. The roles are domain-native, not generic.
- **Event-driven by default.** Schools run on cross-
  domain flows. The engine's event bus is the
  nervous system.
- **Audit-first.** Schools are audited. Every change is
  recorded with the actor, the time, the IP, the
  before/after.
- **Offline-capable.** Schools operate in low-connectivity
  environments. The engine's command envelope supports
  idempotent retries and event-log-based sync.
- **Compile-time safety.** A school's invariants are too
  important to leave to runtime checks. The Rust type
  system encodes the school's rules.
- **One crate per domain.** The school's organizational
  structure maps to the engine's crate layout.

## Notes for SMScore Implementation

- The engine's **academic** domain is foundational. Every
  other domain depends on it. The `Student`, `Class`,
  `Section`, `Subject`, `AcademicYear` aggregates are
  the primitives.
- The engine's **finance** domain is the most complex.
  It has the most aggregates, the most business rules,
  the most integrations, and the most regulatory
  attention.
- The engine's **HR** domain is tightly coupled to
  finance (payroll) and academic (staff who teach).
- The engine's **attendance** domain is high-frequency.
  A school with 1,000 students marks attendance five
  times a day. The engine's command pipeline must be
  fast.
- The engine's **communication** domain is reactive. It
  subscribes to every other domain's events and produces
  notifications.
- The engine's **assessment** domain is read-heavy.
  Marks are written once, read many times (for report
  cards, transcripts, parent portals).
- The engine's **library** domain is straightforward but
  has unique edge cases (overdue, lost, damaged,
  reserved).
- The engine's **facilities** domain is the most
  heterogeneous: transport (a moving operation), hostel
  (a residential operation), inventory (a stock
  operation).
- The engine's **platform** domain is the substrate.
  School, User, custom fields, lookup data.
- The engine's **rbac** domain is the gatekeeper. Every
  other domain consumes its capabilities.
- The engine's **settings** domain is per-tenant
  configuration. Read by every domain.
- The engine's **operations** domain is the system's
  hygiene: backups, jobs, versions, audit projections.

## Mapping to SMScore Files

- Per-domain behavior: `docs/research/<domain>-analysis.md`.
- Per-domain aggregates: `docs/specs/<domain>/aggregates.md`.
- Per-domain commands: `docs/specs/<domain>/commands.md`.
- Per-domain events: `docs/specs/<domain>/events.md`.
- Per-domain permissions: `docs/specs/<domain>/permissions.md`.
- Cross-cutting schemas: `docs/schemas/`.
- Architectural decisions: `docs/decisions/`.

The research layer is the **why**; the specs are the
**what**; the diagrams are the **how it fits together**.
