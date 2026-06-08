# Aggregate Map

This document maps the primary aggregate roots in each domain
and the key relationships between them.

## 1. Academic — The Foundational Domain

```mermaid
classDiagram
    class Student {
        +StudentId id
        +SchoolId school_id
        +AdmissionNumber admission_no
        +PersonName first_name
        +PersonName last_name
        +DateOfBirth date_of_birth
        +Gender gender
        +StudentStatus status
    }
    class Guardian {
        +GuardianId id
        +PersonName full_name
        +PhoneNumber phone
        +EmailAddress email
    }
    class StudentRecord {
        +StudentRecordId id
        +StudentId student_id
        +AcademicYearId academic_year_id
        +ClassId class_id
        +SectionId section_id
        +RollNumber roll_no
    }
    class Class {
        +ClassId id
        +String name
        +i32 order
    }
    class Section {
        +SectionId id
        +String name
        +i32 capacity
    }
    class ClassSection {
        +ClassSectionId id
        +ClassId class_id
        +SectionId section_id
        +AcademicYearId academic_year_id
        +StaffId class_teacher_id
        +RoomId room_id
    }
    class Subject {
        +SubjectId id
        +String code
        +String name
        +SubjectType type
    }
    class AcademicYear {
        +AcademicYearId id
        +String title
        +NaiveDate start_date
        +NaiveDate end_date
        +bool is_current
    }
    class ClassRoutine {
        +ClassRoutineId id
        +ClassSectionId class_section_id
        +SubjectId subject_id
        +StaffId teacher_id
        +DayOfWeek day
        +TimeRange period
    }
    class Homework {
        +HomeworkId id
        +ClassSectionId class_section_id
        +SubjectId subject_id
        +StaffId assigned_by
        +NaiveDate due_date
    }
    class LessonPlan {
        +LessonPlanId id
        +ClassSectionId class_section_id
        +SubjectId subject_id
        +StaffId teacher_id
        +NaiveDate lesson_date
    }

    Student "1" --o "many" StudentRecord
    Student "many" --o "many" Guardian : linked
    StudentRecord "many" --> "1" Class : enrolled in
    StudentRecord "many" --> "1" Section : enrolled in
    StudentRecord "many" --> "1" AcademicYear : in year
    ClassSection "many" --> "1" Class
    ClassSection "many" --> "1" Section
    ClassSection "many" --> "1" AcademicYear
    ClassRoutine "many" --> "1" ClassSection
    ClassRoutine "many" --> "1" Subject
    Homework "many" --> "1" ClassSection
    Homework "many" --> "1" Subject
    LessonPlan "many" --> "1" ClassSection
    LessonPlan "many" --> "1" Subject
```

## 2. Finance — The Monetary Spine

```mermaid
classDiagram
    class FeesGroup {
        +FeesGroupId id
        +String name
    }
    class FeesType {
        +FeesTypeId id
        +FeesGroupId group_id
        +String name
        +Money amount
    }
    class FeesMaster {
        +FeesMasterId id
        +FeesTypeId type_id
        +ClassId class_id
        +AcademicYearId academic_year_id
        +Money amount
        +NaiveDate due_date
    }
    class FeesAssign {
        +FeesAssignId id
        +FeesMasterId master_id
        +StudentId student_id
        +StudentRecordId record_id
        +Money discount
        +Money paid
        +Money balance
    }
    class FeesInvoice {
        +FeesInvoiceId id
        +String prefix
        +i32 start_form
    }
    class FeesInstallment {
        +FeesInstallmentId id
        +FeesMasterId master_id
        +String title
        +f64 percentage
        +Money amount
        +NaiveDate due_date
    }
    class FeesPayment {
        +FeesPaymentId id
        +FeesAssignId assign_id
        +Money amount
        +NaiveDate payment_date
        +PaymentMode mode
        +BankAccountId bank_id
    }
    class BankAccount {
        +BankAccountId id
        +String name
        +String account_number
        +Money current_balance
    }
    class BankStatement {
        +BankStatementId id
        +BankAccountId account_id
        +Money debit
        +Money credit
        +NaiveDate statement_date
    }
    class Expense {
        +ExpenseId id
        +ExpenseHeadId head_id
        +Money amount
        +NaiveDate expense_date
    }
    class Income {
        +IncomeId id
        +IncomeHeadId head_id
        +Money amount
        +NaiveDate income_date
    }
    class Payroll {
        +PayrollId id
        +StaffId staff_id
        +PayrollPeriodId period_id
        +Money gross
        +Money deductions
        +Money net
        +PayrollStatus status
    }
    class Wallet {
        +WalletId id
        +UserId user_id
        +Money balance
    }

    FeesGroup "1" --o "many" FeesType
    FeesType "1" --o "many" FeesMaster
    FeesMaster "1" --o "many" FeesAssign
    FeesAssign "1" --o "many" FeesPayment
    FeesMaster "1" --o "many" FeesInstallment
    BankAccount "1" --o "many" BankStatement
    BankAccount "1" --o "many" FeesPayment : receives
    Payroll "many" --> "1" BankAccount : paid from
```

## 3. Attendance

```mermaid
classDiagram
    class AttendanceSession {
        +AttendanceSessionId id
        +ClassSectionId class_section_id
        +NaiveDate session_date
        +StaffId taken_by
        +SessionStatus status
    }
    class AttendanceEntry {
        +AttendanceEntryId id
        +AttendanceSessionId session_id
        +StudentId student_id
        +AttendanceStatus status
        +Option~String~ note
    }
    class StaffAttendance {
        +StaffAttendanceId id
        +StaffId staff_id
        +NaiveDate attendance_date
        +TimeInOut time_in
        +TimeInOut time_out
    }
    class Holiday {
        +HolidayId id
        +NaiveDate date
        +String title
        +HolidayType type
    }

    AttendanceSession "1" --o "many" AttendanceEntry
    AttendanceSession "many" --> "1" ClassSection
```

## 4. Assessment

```mermaid
classDiagram
    class ExamType {
        +ExamTypeId id
        +String name
        +f32 weight
    }
    class Exam {
        +ExamId id
        +ExamTypeId type_id
        +ClassId class_id
        +AcademicYearId academic_year_id
        +String title
        +NaiveDate start_date
        +NaiveDate end_date
        +ExamStatus status
    }
    class ExamSchedule {
        +ExamScheduleId id
        +ExamId exam_id
        +SubjectId subject_id
        +ClassId class_id
        +SectionId section_id
        +NaiveDate exam_date
        +TimeRange time
        +RoomId room_id
    }
    class Mark {
        +MarkId id
        +ExamScheduleId schedule_id
        +StudentId student_id
        +StudentRecordId record_id
        +SubjectId subject_id
        +Option~f32~ marks_obtained
        +Option~String~ grade
    }
    class Result {
        +ResultId id
        +ExamId exam_id
        +StudentId student_id
        +StudentRecordId record_id
        +f32 total_marks
        +f32 percentage
        +String grade
        +ResultStatus status
    }
    class ReportCard {
        +ReportCardId id
        +ResultId result_id
        +StudentId student_id
        +StudentRecordId record_id
        +String template_id
        +ReportCardStatus status
    }

    Exam "1" --o "many" ExamSchedule
    Exam "1" --o "many" Result
    ExamSchedule "1" --o "many" Mark
    Result "1" --o "many" ReportCard
```

## 5. HR

```mermaid
classDiagram
    class Staff {
        +StaffId id
        +UserId user_id
        +StaffNo staff_no
        +PersonName full_name
        +NaiveDate joining_date
        +StaffStatus status
    }
    class Department {
        +DepartmentId id
        +String name
    }
    class Designation {
        +DesignationId id
        +String title
    }
    class LeaveType {
        +LeaveTypeId id
        +String name
        +i32 days_allowed
    }
    class LeaveApplication {
        +LeaveApplicationId id
        +StaffId staff_id
        +LeaveTypeId type_id
        +NaiveDate from_date
        +NaiveDate to_date
        +LeaveStatus status
    }
    class LeaveDeduction {
        +LeaveDeductionId id
        +LeaveTypeId type_id
        +String name
        +f32 amount
    }
    class SalaryTemplate {
        +SalaryTemplateId id
        +String grade
        +Money basic
        +Money overtime
        +Money house_rent
    }
    class PayrollPeriod {
        +PayrollPeriodId id
        +String month
        +i32 year
        +PayrollPeriodStatus status
    }

    Staff "many" --> "1" Department
    Staff "many" --> "1" Designation
    Staff "1" --o "many" LeaveApplication
    LeaveApplication "many" --> "1" LeaveType
```

## 6. Library

```mermaid
classDiagram
    class Book {
        +BookId id
        +String isbn
        +String title
        +String author
        +i32 total_copies
    }
    class BookCopy {
        +BookCopyId id
        +BookId book_id
        +String accession_number
        +BookCopyStatus status
    }
    class Member {
        +MemberId id
        +UserId user_id
        +MembershipType member_type
        +NaiveDate membership_date
    }
    class BookIssue {
        +BookIssueId id
        +BookCopyId copy_id
        +MemberId member_id
        +NaiveDate issue_date
        +NaiveDate due_date
        +Option~NaiveDate~ return_date
    }

    Book "1" --o "many" BookCopy
    Member "1" --o "many" BookIssue
    BookCopy "1" --o "many" BookIssue
```

## 7. Facilities (Transport / Hostel / Inventory)

```mermaid
classDiagram
    class TransportRoute {
        +TransportRouteId id
        +String name
        +f32 fare
    }
    class TransportVehicle {
        +TransportVehicleId id
        +String vehicle_number
        +i32 capacity
    }
    class TransportAssignment {
        +TransportAssignmentId id
        +TransportRouteId route_id
        +TransportVehicleId vehicle_id
        +StudentId student_id
        +PickupPoint pickup_point
    }
    class Hostel {
        +HostelId id
        +String name
        +HostelType type
    }
    class HostelRoom {
        +HostelRoomId id
        +HostelId hostel_id
        +String room_number
        +i32 capacity
    }
    class RoomAssignment {
        +RoomAssignmentId id
        +HostelRoomId room_id
        +StudentId student_id
        +NaiveDate assigned_date
    }
    class ItemCategory {
        +ItemCategoryId id
        +String name
    }
    class Item {
        +ItemId id
        +ItemCategoryId category_id
        +String name
        +i32 quantity
        +Money unit_price
    }
    class ItemIssue {
        +ItemIssueId id
        +ItemId item_id
        +UserId issued_to
        +i32 quantity
        +NaiveDate issue_date
    }
```

## 8. Communication

```mermaid
classDiagram
    class Notice {
        +NoticeId id
        +String title
        +String body
        +NoticeAudience audience
        +NaiveDate publish_date
        +NaiveDate expire_date
    }
    class Complaint {
        +ComplaintId id
        +UserId complainant_id
        +String subject
        +String description
        +ComplaintStatus status
    }
    class ChatThread {
        +ChatThreadId id
        +String subject
        +ChatThreadType type
    }
    class ChatMessage {
        +ChatMessageId id
        +ChatThreadId thread_id
        +UserId sender_id
        +String body
        +NaiveTime sent_at
    }
    class Notification {
        +NotificationId id
        +UserId user_id
        +NotificationType type
        +String subject
        +String body
        +NotificationStatus status
    }
```

## 9. Cross-Aggregate Relationships

```mermaid
graph TB
    subgraph academic [Academic]
        Student
        Class
        Section
        AcademicYear
    end
    subgraph finance [Finance]
        FeesAssign
        Invoice
        Payment
    end
    subgraph attendance [Attendance]
        AttendanceSession
    end
    subgraph assessment [Assessment]
        Exam
        Result
    end
    subgraph library [Library]
        Member
    end
    subgraph facilities [Facilities]
        TransportAssignment
    end
    subgraph hr [HR]
        Staff
    end

    Student -->|fees assigned| FeesAssign
    FeesAssign -->|invoiced| Invoice
    FeesAssign -->|paid by| Payment
    Student -->|attendance in| AttendanceSession
    Student -->|result for| Result
    Result -->|of exam| Exam
    Student -->|member of| Member
    Student -->|assigned to| TransportAssignment
    Staff -->|salary| Payment
```

## 10. Aggregate Cardinality Quick Reference

| Aggregate             | Per School (typical) | Per Academic Year      |
| --------------------- | -------------------- | ---------------------- |
| `Student`             | 500 - 5,000          | active subset          |
| `Staff`               | 50 - 500             | active subset          |
| `Class`               | 10 - 20              | same                   |
| `Section`             | 30 - 100             | same                   |
| `Subject`             | 20 - 50              | same                   |
| `AttendanceSession`   | n/a                  | 200 / class / year     |
| `Exam`                | n/a                  | 4 - 8                  |
| `Mark`                | n/a                  | 5,000 - 50,000         |
| `Result`              | n/a                  | 500 - 5,000            |
| `FeesPayment`         | n/a                  | 10,000 - 100,000       |
| `Payroll`             | n/a                  | 600 - 6,000            |
| `Book`                | 500 - 10,000         | n/a                    |
| `BookIssue`           | n/a                  | 1,000 - 20,000         |
| `Notice`              | 50 - 500 / year      | n/a                    |
| `Notification`        | 10,000 - 500,000 / yr| n/a                    |
| `AuditRecord`         | n/a                  | 1M - 50M               |
