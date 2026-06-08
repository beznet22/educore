# HR Domain — Repositories

Repositories are ports (Rust traits). Adapters implement them. The
default adapter targets PostgreSQL; an SQLite adapter is provided
for embedded deployments.

All repositories include a `school_id` parameter on every read. The
storage adapter is responsible for tenant isolation.

## StaffRepository

```rust
#[async_trait]
pub trait StaffRepository: Send + Sync {
    async fn get(&self, id: StaffId) -> Result<Option<Staff>>;
    async fn get_by_email(&self, school: SchoolId, email: &EmailAddress) -> Result<Option<Staff>>;
    async fn get_by_mobile(&self, school: SchoolId, mobile: &PhoneNumber) -> Result<Option<Staff>>;
    async fn get_by_user(&self, school: SchoolId, user_id: UserId) -> Result<Option<Staff>>;
    async fn get_by_staff_no(&self, school: SchoolId, staff_no: StaffNo) -> Result<Option<Staff>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<Staff>>;
    async fn list_for_department(&self, school: SchoolId, department: DepartmentId) -> Result<Vec<Staff>>;
    async fn list_for_designation(&self, school: SchoolId, designation: DesignationId) -> Result<Vec<Staff>>;
    async fn list_for_role(&self, school: SchoolId, role: RoleId) -> Result<Vec<Staff>>;
    async fn search_by_name(&self, school: SchoolId, query: &str, limit: u32) -> Result<Vec<Staff>>;
    async fn insert(&self, s: &Staff) -> Result<()>;
    async fn update(&self, s: &Staff) -> Result<()>;
    async fn delete(&self, id: StaffId) -> Result<()>;
}
```

## DepartmentRepository

```rust
#[async_trait]
pub trait DepartmentRepository: Send + Sync {
    async fn get(&self, id: DepartmentId) -> Result<Option<Department>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<Department>>;
    async fn find_by_name(&self, school: SchoolId, name: &str) -> Result<Option<Department>>;
    async fn insert(&self, d: &Department) -> Result<()>;
    async fn update(&self, d: &Department) -> Result<()>;
    async fn delete(&self, id: DepartmentId) -> Result<()>;
}
```

## DesignationRepository

```rust
#[async_trait]
pub trait DesignationRepository: Send + Sync {
    async fn get(&self, id: DesignationId) -> Result<Option<Designation>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<Designation>>;
    async fn find_by_title(&self, school: SchoolId, title: &str) -> Result<Option<Designation>>;
    async fn insert(&self, d: &Designation) -> Result<()>;
    async fn update(&self, d: &Designation) -> Result<()>;
    async fn delete(&self, id: DesignationId) -> Result<()>;
}
```

## LeaveTypeRepository

```rust
#[async_trait]
pub trait LeaveTypeRepository: Send + Sync {
    async fn get(&self, id: LeaveTypeId) -> Result<Option<LeaveType>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<LeaveType>>;
    async fn find_by_name(&self, school: SchoolId, name: &str) -> Result<Option<LeaveType>>;
    async fn insert(&self, t: &LeaveType) -> Result<()>;
    async fn update(&self, t: &LeaveType) -> Result<()>;
    async fn delete(&self, id: LeaveTypeId) -> Result<()>;
}
```

## LeaveDefineRepository

```rust
#[async_trait]
pub trait LeaveDefineRepository: Send + Sync {
    async fn get(&self, id: LeaveDefineId) -> Result<Option<LeaveDefine>>;
    async fn list_for_school(&self, school: SchoolId, academic: AcademicYearId) -> Result<Vec<LeaveDefine>>;
    async fn find_for_role(&self, school: SchoolId, role: RoleId, type_id: LeaveTypeId, academic: AcademicYearId) -> Result<Option<LeaveDefine>>;
    async fn find_for_user(&self, school: SchoolId, user: UserId, type_id: LeaveTypeId, academic: AcademicYearId) -> Result<Option<LeaveDefine>>;
    async fn insert(&self, d: &LeaveDefine) -> Result<()>;
    async fn update(&self, d: &LeaveDefine) -> Result<()>;
    async fn delete(&self, id: LeaveDefineId) -> Result<()>;
}
```

## LeaveRequestRepository

```rust
#[async_trait]
pub trait LeaveRequestRepository: Send + Sync {
    async fn get(&self, id: LeaveRequestId) -> Result<Option<LeaveRequest>>;
    async fn list_for_staff(&self, staff: StaffId) -> Result<Vec<LeaveRequest>>;
    async fn list_pending(&self, school: SchoolId) -> Result<Vec<LeaveRequest>>;
    async fn list_for_period(&self, school: SchoolId, from: NaiveDate, to: NaiveDate) -> Result<Vec<LeaveRequest>>;
    async fn list_for_type(&self, school: SchoolId, type_id: LeaveTypeId) -> Result<Vec<LeaveRequest>>;
    async fn insert(&self, r: &LeaveRequest) -> Result<()>;
    async fn update(&self, r: &LeaveRequest) -> Result<()>;
}
```

## StaffAttendanceRepository

```rust
#[async_trait]
pub trait StaffAttendanceRepository: Send + Sync {
    async fn get(&self, id: StaffAttendanceId) -> Result<Option<StaffAttendance>>;
    async fn list_for_staff(&self, staff: StaffId, from: NaiveDate, to: NaiveDate) -> Result<Vec<StaffAttendance>>;
    async fn list_for_school(&self, school: SchoolId, from: NaiveDate, to: NaiveDate) -> Result<Vec<StaffAttendance>>;
    async fn find_for_date(&self, staff: StaffId, date: NaiveDate) -> Result<Option<StaffAttendance>>;
    async fn insert(&self, a: &StaffAttendance) -> Result<()>;
    async fn update(&self, a: &StaffAttendance) -> Result<()>;
    async fn delete(&self, id: StaffAttendanceId) -> Result<()>;
}
```

## StaffAttendanceImportRepository

```rust
#[async_trait]
pub trait StaffAttendanceImportRepository: Send + Sync {
    async fn get(&self, id: StaffAttendanceImportId) -> Result<Option<StaffAttendanceImport>>;
    async fn list_pending(&self, school: SchoolId) -> Result<Vec<StaffAttendanceImport>>;
    async fn list_for_staff(&self, staff: StaffId) -> Result<Vec<StaffAttendanceImport>>;
    async fn list_for_batch(&self, batch: StaffAttendanceImportBatchId) -> Result<Vec<StaffAttendanceImport>>;
    async fn insert(&self, i: &StaffAttendanceImport) -> Result<()>;
    async fn update(&self, i: &StaffAttendanceImport) -> Result<()>;
    async fn delete(&self, id: StaffAttendanceImportId) -> Result<()>;
}
```

## AssignClassTeacherRepository

```rust
#[async_trait]
pub trait AssignClassTeacherRepository: Send + Sync {
    async fn get(&self, id: AssignClassTeacherId) -> Result<Option<AssignClassTeacher>>;
    async fn list_for_school(&self, school: SchoolId, academic: AcademicYearId) -> Result<Vec<AssignClassTeacher>>;
    async fn list_for_staff(&self, staff: StaffId, academic: AcademicYearId) -> Result<Vec<AssignClassTeacher>>;
    async fn find(&self, school: SchoolId, class: ClassId, section: SectionId, academic: AcademicYearId) -> Result<Option<AssignClassTeacher>>;
    async fn insert(&self, a: &AssignClassTeacher) -> Result<()>;
    async fn update(&self, a: &AssignClassTeacher) -> Result<()>;
    async fn delete(&self, id: AssignClassTeacherId) -> Result<()>;
}
```

## HourlyRateRepository

```rust
#[async_trait]
pub trait HourlyRateRepository: Send + Sync {
    async fn get(&self, id: HourlyRateId) -> Result<Option<HourlyRate>>;
    async fn list(&self, school: SchoolId, academic: AcademicYearId) -> Result<Vec<HourlyRate>>;
    async fn find_by_grade(&self, school: SchoolId, grade: &str, academic: AcademicYearId) -> Result<Option<HourlyRate>>;
    async fn insert(&self, r: &HourlyRate) -> Result<()>;
    async fn update(&self, r: &HourlyRate) -> Result<()>;
    async fn delete(&self, id: HourlyRateId) -> Result<()>;
}
```

## SalaryTemplateRepository

```rust
#[async_trait]
pub trait SalaryTemplateRepository: Send + Sync {
    async fn get(&self, id: SalaryTemplateId) -> Result<Option<SalaryTemplate>>;
    async fn list(&self, school: SchoolId, academic: AcademicYearId) -> Result<Vec<SalaryTemplate>>;
    async fn find_by_grade(&self, school: SchoolId, grade: &str, academic: AcademicYearId) -> Result<Option<SalaryTemplate>>;
    async fn insert(&self, t: &SalaryTemplate) -> Result<()>;
    async fn update(&self, t: &SalaryTemplate) -> Result<()>;
    async fn delete(&self, id: SalaryTemplateId) -> Result<()>;
}
```

## PayrollGenerateRepository

```rust
#[async_trait]
pub trait PayrollGenerateRepository: Send + Sync {
    async fn get(&self, id: PayrollGenerateId) -> Result<Option<PayrollGenerate>>;
    async fn list_for_staff(&self, staff: StaffId) -> Result<Vec<PayrollGenerate>>;
    async fn list_for_period(&self, school: SchoolId, period: PayPeriod) -> Result<Vec<PayrollGenerate>>;
    async fn list_pending_approval(&self, school: SchoolId) -> Result<Vec<PayrollGenerate>>;
    async fn find_for_period(&self, staff: StaffId, period: PayPeriod) -> Result<Option<PayrollGenerate>>;
    async fn insert(&self, p: &PayrollGenerate) -> Result<()>;
    async fn update(&self, p: &PayrollGenerate) -> Result<()>;
}
```

## PayrollEarnDeducRepository

```rust
#[async_trait]
pub trait PayrollEarnDeducRepository: Send + Sync {
    async fn get(&self, id: PayrollEarnDeducId) -> Result<Option<PayrollEarnDeduc>>;
    async fn list_for_payroll(&self, payroll: PayrollGenerateId) -> Result<Vec<PayrollEarnDeduc>>;
    async fn list_earnings(&self, payroll: PayrollGenerateId) -> Result<Vec<PayrollEarnDeduc>>;
    async fn list_deductions(&self, payroll: PayrollGenerateId) -> Result<Vec<PayrollEarnDeduc>>;
    async fn insert(&self, e: &PayrollEarnDeduc) -> Result<()>;
    async fn update(&self, e: &PayrollEarnDeduc) -> Result<()>;
    async fn delete(&self, id: PayrollEarnDeducId) -> Result<()>;
}
```

## LeaveDeductionInfoRepository

```rust
#[async_trait]
pub trait LeaveDeductionInfoRepository: Send + Sync {
    async fn get(&self, id: LeaveDeductionInfoId) -> Result<Option<LeaveDeductionInfo>>;
    async fn list_for_payroll(&self, payroll: PayrollGenerateId) -> Result<Vec<LeaveDeductionInfo>>;
    async fn list_for_staff(&self, staff: StaffId) -> Result<Vec<LeaveDeductionInfo>>;
    async fn insert(&self, l: &LeaveDeductionInfo) -> Result<()>;
    async fn update(&self, l: &LeaveDeductionInfo) -> Result<()>;
    async fn delete(&self, id: LeaveDeductionInfoId) -> Result<()>;
}
```

## StaffRegistrationFieldRepository

```rust
#[async_trait]
pub trait StaffRegistrationFieldRepository: Send + Sync {
    async fn get(&self, id: StaffRegistrationFieldId) -> Result<Option<StaffRegistrationField>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<StaffRegistrationField>>;
    async fn insert(&self, f: &StaffRegistrationField) -> Result<()>;
    async fn update(&self, f: &StaffRegistrationField) -> Result<()>;
    async fn delete(&self, id: StaffRegistrationFieldId) -> Result<()>;
}
```

## StaffImportBulkTemporaryRepository

```rust
#[async_trait]
pub trait StaffImportBulkTemporaryRepository: Send + Sync {
    async fn get(&self, id: StaffImportBulkTemporaryId) -> Result<Option<StaffImportBulkTemporary>>;
    async fn list_for_job(&self, job: BulkImportJobId) -> Result<Vec<StaffImportBulkTemporary>>;
    async fn list_active(&self, school: SchoolId) -> Result<Vec<StaffImportBulkTemporary>>;
    async fn insert(&self, t: &StaffImportBulkTemporary) -> Result<()>;
    async fn update(&self, t: &StaffImportBulkTemporary) -> Result<()>;
    async fn delete(&self, id: StaffImportBulkTemporaryId) -> Result<()>;
}
```

## Indexes (recommended)

The default PostgreSQL adapter documents the following indexes;
consumers should declare them in their migrations:

```sql
-- Staff
CREATE UNIQUE INDEX ux_sm_staffs_school_id_staff_no ON sm_staffs (school_id, staff_no) WHERE staff_no IS NOT NULL;
CREATE UNIQUE INDEX ux_sm_staffs_school_id_email ON sm_staffs (school_id, email) WHERE email IS NOT NULL;
CREATE UNIQUE INDEX ux_sm_staffs_school_id_mobile ON sm_staffs (school_id, mobile) WHERE mobile IS NOT NULL;
CREATE UNIQUE INDEX ux_sm_staffs_school_id_user_id ON sm_staffs (school_id, user_id);
CREATE INDEX ix_sm_staffs_school_id_department ON sm_staffs (school_id, department_id);
CREATE INDEX ix_sm_staffs_school_id_designation ON sm_staffs (school_id, designation_id);
CREATE INDEX ix_sm_staffs_school_id_role ON sm_staffs (school_id, role_id);
-- Department / Designation
CREATE UNIQUE INDEX ux_sm_human_departments_school_id_name ON sm_human_departments (school_id, name);
CREATE UNIQUE INDEX ux_sm_designations_school_id_title ON sm_designations (school_id, title);
-- Leave
CREATE INDEX ix_sm_leave_types_school_id ON sm_leave_types (school_id);
CREATE INDEX ix_sm_leave_defines_school_id_role_type ON sm_leave_defines (school_id, role_id, type_id, academic_id);
CREATE INDEX ix_sm_leave_defines_school_id_user_type ON sm_leave_defines (school_id, user_id, type_id, academic_id);
CREATE INDEX ix_sm_leave_requests_school_id_staff ON sm_leave_requests (school_id, staff_id);
CREATE INDEX ix_sm_leave_requests_school_id_status ON sm_leave_requests (school_id, approve_status);
CREATE INDEX ix_sm_leave_requests_school_id_type ON sm_leave_requests (school_id, type_id);
-- Attendance
CREATE UNIQUE INDEX ux_sm_staff_attendences_school_id_staff_date ON sm_staff_attendences (school_id, staff_id, attendance_date);
CREATE INDEX ix_sm_staff_attendences_school_id_date ON sm_staff_attendences (school_id, attendance_date);
CREATE INDEX ix_sm_staff_attendance_imports_school_id_staff ON sm_staff_attendance_imports (school_id, staff_id);
CREATE INDEX ix_sm_staff_attendance_imports_school_id_date ON sm_staff_attendance_imports (school_id, attendence_date);
-- Class teacher
CREATE UNIQUE INDEX ux_sm_assign_class_teachers_school_id_class_section_year
  ON sm_assign_class_teachers (school_id, class_id, section_id, academic_id);
CREATE INDEX ix_sm_assign_class_teachers_school_id_school_year ON sm_assign_class_teachers (school_id, academic_id);
-- Salary / Rate
CREATE UNIQUE INDEX ux_sm_hr_salary_templates_school_id_grade ON sm_hr_salary_templates (school_id, salary_grades, academic_id);
CREATE UNIQUE INDEX ux_sm_hourly_rates_school_id_grade ON sm_hourly_rates (school_id, grade, academic_id);
-- Payroll
CREATE INDEX ix_sm_hr_payroll_generates_school_id_staff ON sm_hr_payroll_generates (school_id, staff_id);
CREATE UNIQUE INDEX ux_sm_hr_payroll_generates_school_id_staff_period
  ON sm_hr_payroll_generates (school_id, staff_id, payroll_month, payroll_year);
CREATE INDEX ix_sm_hr_payroll_generates_school_id_status ON sm_hr_payroll_generates (school_id, payroll_status);
CREATE INDEX ix_sm_hr_payroll_earn_deducs_school_id_payroll ON sm_hr_payroll_earn_deducs (school_id, payroll_generate_id);
-- Leave deduction
CREATE INDEX ix_sm_leave_deduction_infos_school_id_staff ON sm_leave_deduction_infos (school_id, staff_id);
CREATE INDEX ix_sm_leave_deduction_infos_school_id_payroll ON sm_leave_deduction_infos (school_id, payroll_id);
-- Registration field
CREATE INDEX ix_sm_staff_registration_fields_school_id ON sm_staff_registration_fields (school_id, position);
-- Bulk import
CREATE INDEX ix_staff_import_bulk_temporaries_user_id ON staff_import_bulk_temporaries (user_id);
CREATE INDEX ix_staff_import_bulk_temporaries_email ON staff_import_bulk_temporaries (email);
CREATE INDEX ix_staff_import_bulk_temporaries_staff_no ON staff_import_bulk_temporaries (staff_no);
```

The `school_id` predicate is mandatory for tenant isolation.
