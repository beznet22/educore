# Attendance Domain — Repositories

Repositories are ports (Rust traits). Adapters implement them. The
default adapter targets PostgreSQL; an SQLite adapter is provided
for embedded deployments.

## StudentAttendanceRepository

```rust
#[async_trait]
pub trait StudentAttendanceRepository: Send + Sync {
    async fn get(&self, id: StudentAttendanceId) -> Result<Option<StudentAttendance>>;
    async fn find(
        &self,
        school: SchoolId,
        student: StudentId,
        date: AttendanceDate,
    ) -> Result<Option<StudentAttendance>>;
    async fn list_for_student(
        &self,
        school: SchoolId,
        student: StudentId,
        from: AttendanceDate,
        to: AttendanceDate,
    ) -> Result<Vec<StudentAttendance>>;
    async fn list_for_section(
        &self,
        school: SchoolId,
        class: ClassId,
        section: SectionId,
        date: AttendanceDate,
    ) -> Result<Vec<StudentAttendance>>;
    async fn list_for_class_in_range(
        &self,
        school: SchoolId,
        class: ClassId,
        from: AttendanceDate,
        to: AttendanceDate,
    ) -> Result<Vec<StudentAttendance>>;
    async fn list_absent_for_day(
        &self,
        school: SchoolId,
        date: AttendanceDate,
    ) -> Result<Vec<StudentAttendance>>;
    async fn insert(&self, a: &StudentAttendance) -> Result<()>;
    async fn update(&self, a: &StudentAttendance) -> Result<()>;
}
```

## SubjectAttendanceRepository

```rust
#[async_trait]
pub trait SubjectAttendanceRepository: Send + Sync {
    async fn get(&self, id: SubjectAttendanceId) -> Result<Option<SubjectAttendance>>;
    async fn find(
        &self,
        school: SchoolId,
        student: StudentId,
        subject: SubjectId,
        date: AttendanceDate,
    ) -> Result<Option<SubjectAttendance>>;
    async fn list_for_section_date(
        &self,
        school: SchoolId,
        class: ClassId,
        section: SectionId,
        subject: SubjectId,
        date: AttendanceDate,
    ) -> Result<Vec<SubjectAttendance>>;
    async fn list_for_student(
        &self,
        school: SchoolId,
        student: StudentId,
        from: AttendanceDate,
        to: AttendanceDate,
    ) -> Result<Vec<SubjectAttendance>>;
    async fn insert(&self, a: &SubjectAttendance) -> Result<()>;
    async fn update(&self, a: &SubjectAttendance) -> Result<()>;
}
```

## StaffAttendanceRepository

```rust
#[async_trait]
pub trait StaffAttendanceRepository: Send + Sync {
    async fn get(&self, id: StaffAttendanceId) -> Result<Option<StaffAttendance>>;
    async fn find(
        &self,
        school: SchoolId,
        staff: StaffId,
        date: AttendanceDate,
    ) -> Result<Option<StaffAttendance>>;
    async fn list_for_staff(
        &self,
        school: SchoolId,
        staff: StaffId,
        from: AttendanceDate,
        to: AttendanceDate,
    ) -> Result<Vec<StaffAttendance>>;
    async fn list_for_school_in_range(
        &self,
        school: SchoolId,
        from: AttendanceDate,
        to: AttendanceDate,
    ) -> Result<Vec<StaffAttendance>>;
    async fn list_absent_for_day(
        &self,
        school: SchoolId,
        date: AttendanceDate,
    ) -> Result<Vec<StaffAttendance>>;
    async fn insert(&self, a: &StaffAttendance) -> Result<()>;
    async fn update(&self, a: &StaffAttendance) -> Result<()>;
}
```

## ExamAttendanceRepository (cross-reference)

The exam-day per-subject attendance is owned by the assessment
domain. The corresponding repository is documented in
`docs/specs/assessment/repositories.md`. The attendance domain
reads it for `ClassAttendance` recomputation; it does not own its
lifecycle.

## AttendanceImportRepository

```rust
#[async_trait]
pub trait AttendanceImportRepository: Send + Sync {
    async fn get(&self, id: BulkAttendanceImportId) -> Result<Option<BulkAttendanceImport>>;
    async fn list_pending(&self, school: SchoolId) -> Result<Vec<BulkAttendanceImport>>;
    async fn list_for_source(
        &self,
        school: SchoolId,
        source: AttendanceSource,
    ) -> Result<Vec<BulkAttendanceImport>>;
    async fn insert(&self, j: &BulkAttendanceImport) -> Result<()>;
    async fn update(&self, j: &BulkAttendanceImport) -> Result<()>;
    async fn insert_row(&self, r: &StudentAttendanceImport) -> Result<()>;
    async fn list_rows(&self, job_id: BulkAttendanceImportId) -> Result<Vec<StudentAttendanceImport>>;
    async fn insert_staff_row(&self, r: &StaffAttendanceImport) -> Result<()>;
    async fn list_staff_rows(&self, job_id: BulkAttendanceImportId) -> Result<Vec<StaffAttendanceImport>>;
    async fn insert_bulk_row(&self, r: &AttendanceBulk) -> Result<()>;
    async fn list_bulk_rows(&self, job_id: BulkAttendanceImportId) -> Result<Vec<AttendanceBulk>>;
}
```

## ClassAttendanceRepository

```rust
#[async_trait]
pub trait ClassAttendanceRepository: Send + Sync {
    async fn get(
        &self,
        school: SchoolId,
        student: StudentId,
        exam_type: ExamTypeId,
        year: AcademicYearId,
    ) -> Result<Option<ClassAttendance>>;
    async fn list_for_student(
        &self,
        school: SchoolId,
        student: StudentId,
        year: AcademicYearId,
    ) -> Result<Vec<ClassAttendance>>;
    async fn list_for_exam_type(
        &self,
        school: SchoolId,
        exam_type: ExamTypeId,
        year: AcademicYearId,
    ) -> Result<Vec<ClassAttendance>>;
    async fn upsert(&self, c: &ClassAttendance) -> Result<()>;
}
```

`ClassAttendance` is a projection. The engine recomputes it from
the underlying events and rows; the `upsert` method is invoked by
the service.

## Indexes (recommended)

The default PostgreSQL adapter documents the following indexes;
consumers should declare them in their migrations:

```sql
CREATE INDEX ix_sm_student_attendances_school_id_class_section_date
    ON sm_student_attendances (school_id, academic_id, class_id, section_id, attendance_date);
CREATE UNIQUE INDEX ux_sm_student_attendances_school_id_student_date
    ON sm_student_attendances (school_id, academic_id, student_id, attendance_date);
CREATE INDEX ix_sm_student_attendances_school_id_date
    ON sm_student_attendances (school_id, attendance_date);
CREATE INDEX ix_sm_student_attendances_school_id_student_range
    ON sm_student_attendances (school_id, student_id, attendance_date);

CREATE INDEX ix_sm_subject_attendances_school_id_class_section_subject_date
    ON sm_subject_attendances (school_id, academic_id, class_id, section_id, subject_id, attendance_date);
CREATE UNIQUE INDEX ux_sm_subject_attendances_school_id_student_subject_date
    ON sm_subject_attendances (school_id, academic_id, student_id, subject_id, attendance_date);

CREATE INDEX ix_sm_staff_attendances_school_id_staff_date
    ON sm_staff_attendances (school_id, staff_id, attendance_date);
CREATE INDEX ix_sm_staff_attendances_school_id_date
    ON sm_staff_attendances (school_id, attendance_date);

CREATE INDEX ix_sm_student_attendance_imports_school_id_date
    ON sm_student_attendance_imports (school_id, attendance_date);
CREATE INDEX ix_sm_student_attendance_imports_school_id_student
    ON sm_student_attendance_imports (school_id, student_id);

CREATE UNIQUE INDEX ux_class_attendances_school_id_student_exam
    ON class_attendances (school_id, student_id, exam_type_id, academic_id);

CREATE INDEX ix_student_attendance_bulks_school_id_date
    ON student_attendance_bulks (school_id, attendance_date);
```

The `school_id` predicate is mandatory for tenant isolation.
