# Attendance Domain — Services

Domain services encapsulate business logic that does not fit cleanly
in a single aggregate. They are stateless, sync, and pure (no I/O).

## AttendanceService

```rust
pub struct AttendanceService;

impl AttendanceService {
    pub fn mark(
        student: &Student,
        record: &StudentRecord,
        date: AttendanceDate,
        type_: AttendanceType,
        marked_by: UserId,
        marked_from: AttendanceSource,
    ) -> Result<StudentAttendance, ValidationError>;

    pub fn update(
        attendance: &mut StudentAttendance,
        type_: AttendanceType,
        notes: Option<String>,
    ) -> Result<(), ValidationError>;

    pub fn is_late(
        date: AttendanceDate,
        arrival: Time,
        threshold: Time,
    ) -> bool;

    pub fn is_half_day(
        school: &School,
        attendance: &StudentAttendance,
    ) -> bool;

    pub fn is_holiday(
        school: &School,
        date: AttendanceDate,
    ) -> bool;

    pub fn emit_absence_event(
        attendance: &StudentAttendance,
        school_id: SchoolId,
    ) -> Option<StudentAbsentForDay>;

    pub fn recompute_class_attendance(
        student: &Student,
        academic_year: AcademicYearId,
        events: &[StudentAttendanceEvent],
    ) -> ClassAttendance;
}
```

`AttendanceService::is_late` checks the school-defined late
threshold. `is_half_day` applies the school policy that converts
"late after threshold" or "left early" into a half-day
classification. `recompute_class_attendance` projects the daily
events into a per-exam-type summary.

## AbsenceDetectionService

```rust
pub struct AbsenceDetectionService;

impl AbsenceDetectionService {
    pub fn detect(
        school: &School,
        date: AttendanceDate,
        section: &ClassSection,
    ) -> Vec<StudentAbsentForDay>;

    pub fn should_notify(
        school: &School,
        attendance: &StudentAttendance,
    ) -> bool;

    pub fn dedup_within_day(
        events: Vec<StudentAbsentForDay>,
    ) -> Vec<StudentAbsentForDay>;
}
```

`AbsenceDetectionService::detect` returns the absent-student events
for a `(class, section, date)` triple. `dedup_within_day` collapses
multiple transitions into the same day into a single event.

## AttendanceReportService

```rust
pub struct AttendanceReportService;

impl AttendanceReportService {
    pub fn daily(
        school: SchoolId,
        date: AttendanceDate,
    ) -> DailyAttendanceReport;

    pub fn weekly(
        school: SchoolId,
        section: ClassSection,
        from: AttendanceDate,
        to: AttendanceDate,
    ) -> WeeklyAttendanceReport;

    pub fn monthly(
        school: SchoolId,
        section: ClassSection,
        month: YearMonth,
    ) -> MonthlyAttendanceReport;

    pub fn by_class(
        school: SchoolId,
        from: AttendanceDate,
        to: AttendanceDate,
    ) -> ByClassReport;

    pub fn by_student(
        school: SchoolId,
        student: StudentId,
        from: AttendanceDate,
        to: AttendanceDate,
    ) -> ByStudentReport;

    pub fn by_staff(
        school: SchoolId,
        from: AttendanceDate,
        to: AttendanceDate,
    ) -> ByStaffReport;
}
```

Each report is a typed struct that the consumer adapter renders
into PDF/HTML/CSV. The engine produces the structured payload; the
adapter handles formatting.

## AttendanceImportService

```rust
pub struct AttendanceImportService;

impl AttendanceImportService {
    pub fn stage(
        source: AttendanceSource,
        rows: Vec<ImportRow>,
    ) -> BulkAttendanceImport;

    pub fn validate(
        import: &BulkAttendanceImport,
        school: &School,
    ) -> Result<Vec<RowError>, ValidationError>;

    pub fn commit(
        import: &mut BulkAttendanceImport,
        school: &School,
        actor: UserId,
    ) -> Result<Vec<StudentAttendance>, ValidationError>;

    pub fn cancel(
        import: &mut BulkAttendanceImport,
        reason: String,
    ) -> Result<(), ValidationError>;
}
```

`AttendanceImportService::validate` runs per-row checks: the
student must be enrolled, the date must not be in the future, the
type must parse, and the row must not duplicate another row in the
import. `commit` is the only path that produces
`StudentAttendance` rows from a bulk import.

## Policy: AttendanceEligibility

```rust
pub struct AttendanceEligibility;

impl Policy<MarkStudentAttendanceCommand> for AttendanceEligibility {
    type Outcome = Eligible | NotEligible { reason: &'static str };
    fn check(&self, ctx: &Context, cmd: &MarkStudentAttendanceCommand) -> Outcome { ... }
}
```

A student is eligible for marking on a date when the date is not a
holiday, the student is enrolled in the section for the date, and
the date is not in the future.

## Policy: BulkMarkEligibility

A bulk mark is eligible when the actor is the assigned class
teacher (or higher privilege) and the date is within the school's
attendance window (typically 7 days back to today).

## Policy: NotificationEligibility

A `StudentAbsentForDay` is eligible for guardian notification when
the school's `notify_on_absence` flag is true, the student's
primary guardian has a reachable contact, and the student is not
suspended.

## Specification: ActiveOnDate

```rust
pub struct ActiveOnDate {
    pub date: AttendanceDate,
}

impl Specification<Student> for ActiveOnDate {
    fn is_satisfied_by(&self, s: &Student) -> bool { ... }
}
```

Used by queries to list students who are eligible for marking on a
date.

## Specification: HasOutstandingAbsence

```rust
pub struct HasOutstandingAbsence {
    pub threshold: f32, // e.g. 0.25 for 25%
}

impl Specification<Student> for HasOutstandingAbsence {
    fn is_satisfied_by(&self, s: &Student) -> bool { ... }
}
```

Used by the academic domain to flag students for promotion review
and by communication to trigger reminder messages.

## Specification: EligibleForExamAttendance

A student is eligible for exam attendance when they have a valid
admit card for the exam, the exam is scheduled, and the student
is not suspended.

## Cross-Domain Coordinator

A thin coordinator lives in the engine facade and orchestrates
multi-domain flows (e.g. attendance + notification). It is **not**
a service; it composes command calls:

```rust
pub struct AttendanceCoordinator<'a> {
    engine: &'a Engine,
}

impl<'a> AttendanceCoordinator<'a> {
    pub async fn mark_daily(
        &self,
        cmd: MarkStudentAttendanceCommand,
    ) -> Result<StudentAttendance, DomainError> {
        let attendance = self.engine.attendance().mark_student(cmd.clone()).await?;
        // Subscribers (communication, finance, academic) handle
        // their own side effects in response to StudentAbsentForDay.
        Ok(attendance)
    }
}
```

Domain services are pure. Cross-domain coordination happens
through events and command composition, never through
service-to-service calls.
