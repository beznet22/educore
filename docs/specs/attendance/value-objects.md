# Attendance Domain — Value Objects

Value objects are immutable, validated at construction, and have no
identity. They are compared by value.

## Identifiers

All identifiers are typed and tenant-scoped. The generic `Id<S, T>`
wrapper carries the `SchoolId` of the owning school and a `Uuid`.

| Identifier                       | Backing Type               | Notes                              |
| -------------------------------- | -------------------------- | ---------------------------------- |
| `StudentAttendanceId`            | `Id<StudentAttendance>`    | Daily student attendance           |
| `SubjectAttendanceId`            | `Id<SubjectAttendance>`    | Per-period student attendance      |
| `StaffAttendanceId`              | `Id<StaffAttendance>`      | Daily staff attendance             |
| `StudentAttendanceImportId`      | `Id<StudentAttendanceImport>` | Staging row for student import   |
| `StaffAttendanceImportId`        | `Id<StaffAttendanceImport>`   | Staging row for staff import     |
| `BulkAttendanceImportId`         | `Id<BulkAttendanceImport>` | A bulk import job                  |
| `ClassAttendanceId`              | `Id<ClassAttendance>`      | Per-(student, exam_type) summary   |
| `AttendanceBulkId`               | `Id<AttendanceBulk>`       | Denormalized staging row           |

## Attendance Enums

| Type                  | Values                                                            |
| --------------------- | ----------------------------------------------------------------- |
| `AttendanceStatus`    | `Present`, `Absent`, `Late`, `HalfDay`, `Holiday`, `OnLeave`      |
| `AttendanceType`      | `P` (Present), `A` (Absent), `L` (Late), `F` (HalfDay), `H` (Holiday) |
| `AttendanceSource`    | `Manual`, `Biometric`, `BulkImport`, `Api`                        |
| `ImportStatus`        | `Pending`, `Validated`, `Committed`, `Failed`, `Cancelled`        |

The `AttendanceStatus` enum is the canonical typed representation
used by the domain. The `AttendanceType` enum mirrors the legacy
single-character codes that may appear in imported data and is
mapped to `AttendanceStatus` on validation.

## Time and Period

| Type                  | Constraints                                                    |
| --------------------- | -------------------------------------------------------------- |
| `AttendanceDate`      | `NaiveDate`                                                    |
| `Period`              | `Time` start, `Time` end                                       |
| `InTime`              | Free text (e.g. "08:32") from the device                       |
| `OutTime`             | Free text (e.g. "15:05") from the device                       |
| `TimeWindow`          | `(start: Time, end: Time)`                                     |
| `LateThreshold`       | `Time` (the cutoff for "Late")                                 |
| `DayOfWeek`           | ISO `Mon..Sun` (1..7)                                          |

## Counts

| Type                  | Constraints                                                    |
| --------------------- | -------------------------------------------------------------- |
| `DaysOpened`          | `u32` non-negative                                             |
| `DaysPresent`         | `u32` non-negative                                             |
| `DaysAbsent`          | `u32` non-negative                                             |
| `DaysLate`            | `u32` non-negative                                             |
| `DaysHalfDay`         | `u32` non-negative                                             |
| `DaysOnLeave`         | `u32` non-negative                                             |

## Boolean Flags

| Type                       | Notes                                                    |
| -------------------------- | -------------------------------------------------------- |
| `Notify`                   | 0=do not notify, 1=request guardian notification         |
| `IsAbsent`                 | 0=present, 1=absent                                      |
| `ActiveStatus`             | 0=inactive, 1=active                                     |
| `IsHoliday`                | A holiday indicator (school calendar)                    |

## Marked By

| Type                       | Constraints                                              |
| -------------------------- | -------------------------------------------------------- |
| `MarkedBy`                 | `UserId` of the actor                                    |
| `MarkedAt`                 | `Timestamp` from the `Clock` port                        |
| `MarkedFrom`               | `AttendanceSource`                                       |

## Reports

| Type                  | Notes                                                       |
| --------------------- | ----------------------------------------------------------- |
| `AttendanceRange`     | `(from: NaiveDate, to: NaiveDate)`                          |
| `AttendanceReportKind`| `Daily`, `Weekly`, `Monthly`, `ByClass`, `ByStudent`, `ByStaff` |
| `AttendancePercentage`| `f32` in `[0, 100]`                                         |

## School Identity Bindings

| Type                  | Notes                                                       |
| --------------------- | ----------------------------------------------------------- |
| `SchoolId`            | From `educore-platform`                                     |
| `TenantContext`       | `(SchoolId, UserId, ...)` from `educore-platform`           |
| `UserId`              | From `educore-platform`                                     |
| `CorrelationId`       | From `educore-platform`                                     |

## Validation Rules

All value objects implement `Validate` and refuse construction when
validation fails:

```rust
pub trait Validate {
    fn validate(&self) -> Result<(), ValueError>;
}
```

Construction is the only entry point:

```rust
let status = AttendanceStatus::parse("L")?; // Late
let date = AttendanceDate::new(NaiveDate::from_ymd(2026, 6, 8))?;
```

Parsing returns `Result<T, ValueError>`. There are no setters that
bypass validation.

## Cross-Reference

- `StudentId`, `ClassId`, `SectionId`, `SubjectId`, `AcademicYearId`,
  `StaffId`, `StudentRecordId` — from `educore-academic`.
- `ExamTypeId` — from `educore-assessment`.
