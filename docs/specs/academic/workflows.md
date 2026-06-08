# Academic Domain — Workflows

Workflows orchestrate commands, queries, and policies to fulfill a
business goal. They are documented as ordered, conditional steps.

## Admission Workflow

```text
1. Receive admission inquiry (RegisterAdmissionQuery)
2. Schedule follow-up calls (FollowUpAdmissionQuery)
3. Convert inquiry into student (ConvertAdmissionQuery)
   └─ which internally calls AdmitStudent
4. Auto-create library membership (Library subscribes to StudentAdmitted)
5. Auto-create fees assignment (Finance subscribes to StudentAdmitted)
6. Send welcome SMS/email to guardians (Communication subscribes)
7. Issue ID card template (IdCard.Print on demand)
8. Print welcome certificate (Certificate.Issue on demand)
```

**Pre-conditions:**
- The student category (if any) exists.
- The class-section has free capacity (when capacity is configured).
- The current academic year is open for admission.

**Failure paths:**
- Duplicate admission number → `ValidationError::UniqueViolation`.
- Capacity exceeded → `ConflictError::CapacityExceeded`.
- Class-section does not exist → `NotFoundError`.

## Promotion Workflow

```text
1. SchoolAdmin marks current academic year for promotion closure.
2. System runs a `PreparePromotionPlan` report (read-only).
3. For each student:
   a. Result status is determined (Pass / Fail / Manual).
   b. Target class is selected based on result.
   c. Roll number is assigned in the new class-section.
4. PromoteStudent command is issued per student.
5. Finance subscribes and:
   a. Closes prior fees balance.
   b. Assigns new fees master for the new year.
6. Attendance resets its daily expectation.
7. Assessment archives old marks and prepares new exam schedule.
8. Library retains membership but updates class scope.
9. Transport reassigns route.
```

**Edge cases:**
- A student is held back manually → manual `ResultStatus` with no
  promotion to the next class.
- A student is transferred before promotion → no promotion record.
- A student has an outstanding fee balance → promotion is allowed
  (school policy permits it) but the carry-forward logic in finance
  applies.

## Withdrawal Workflow

```text
1. WithdrawStudent command is issued with reason.
2. The active StudentRecord is closed.
3. Library domain receives the event and flags outstanding books.
4. Finance domain receives the event and finalizes balances.
5. Transport domain receives the event and removes route assignment.
6. Communication sends a confirmation to the guardian.
7. Student appears in the Alumni register if eligible.
```

## Transfer Workflow (Cross-School)

```text
1. Source school issues TransferStudent command.
2. The system emits StudentTransferred with destination school id.
3. The destination school receives the transfer payload
   (admission number, name, DOB, class, etc.) through its inbound
   channel (a port-driven adapter).
4. Destination school issues AdmitStudent with the same admission
   number (or a remapped one).
5. Both schools reconcile financial ledgers.
```

**Note:** The transfer workflow requires a multi-tenant SaaS
infrastructure. Single-tenant deployments do not have a destination
school and the `Transfer` capability is unavailable.

## Routine Construction Workflow

```text
1. SchoolAdmin defines ClassTime periods.
2. SchoolAdmin defines ClassRooms.
3. SchoolAdmin creates ClassSections for the academic year.
4. SchoolAdmin assigns subject teachers to class-sections.
5. SchoolAdmin creates a ClassRoutine per (class-section, subject).
6. The system validates teacher/room conflicts.
7. Conflicts surface as ConflictError::TeacherOverlap or RoomOverlap.
8. SchoolAdmin swaps or updates periods to resolve.
```

## Homework Workflow

```text
1. Teacher creates homework for a class-section-subject.
2. Students receive a notification.
3. Students submit homework with description/file.
4. Teacher evaluates the homework and assigns marks.
5. The student receives a notification with marks and comments.
6. Late submissions are accepted but flagged.
7. Cancelled homework removes all pending submissions.
```

## Lesson Plan Workflow

```text
1. Teacher creates a Lesson.
2. Teacher adds LessonTopics.
3. Teacher creates a LessonPlan for a specific date.
4. Teacher marks sub-topics as completed.
5. SchoolAdmin can view a coverage report per class-section.
```

## Admission Query Workflow

```text
1. Front-office registers an admission query.
2. The query enters a "New" status.
3. Each follow-up is logged.
4. The query is closed by:
   a. Conversion (admit a student).
   b. Manual closure with reason.
5. Reports show conversion rates per source and per class.
```

## Class-Section Lifecycle

```text
1. SchoolAdmin creates a Class and one or more Sections.
2. SchoolAdmin creates ClassSections for the academic year.
3. SchoolAdmin assigns teachers and rooms.
4. Students are enrolled into ClassSections via AdmitStudent or
   AssignStudentToSection.
5. End of year:
   a. ClassSections are archived.
   b. New ClassSections are created in the next academic year
      (via AcademicYear.Copy).
6. SchoolAdmin may hard-delete a ClassSection only when no
   StudentRecords reference it.
```

## Capacity & Overflow Rules

- A class may have a `Capacity` (a domain configuration value, not a
  hard invariant). Admission may be configured to reject or warn when
  capacity is exceeded.
- Overflow handling is a school policy expressed as a domain service
  (see `services.md`).

## Idempotency

- `AdmitStudent` is idempotent on `(admission_no, school_id)`. A
  duplicate returns the existing student.
- `PromoteStudent` is idempotent on
  `(student_id, from_academic_year_id, to_academic_year_id)`. A duplicate
  returns the prior promotion.
- `WithdrawStudent` and `SuspendStudent` are idempotent on the same
  status. Re-issuing a withdraw for an already-withdrawn student is a
  no-op success.
