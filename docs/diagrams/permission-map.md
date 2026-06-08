# Permission Map

The role-to-capability mapping for the engine's default role
catalog. Each role is a bundle of capabilities; a user inherits
all capabilities of every role they hold in the active school.

## 1. Role Hierarchy (Default)

```mermaid
graph TB
    SuperAdmin[SuperAdmin]
    SchoolAdmin[SchoolAdmin]
    Accountant[Accountant]
    Teacher[Teacher]
    Librarian[Librarian]
    Transport[Transport]
    Hostel[Hostel]
    Parent[Parent]
    Student[Student]

    SuperAdmin -->|sees all| SchoolAdmin
    SchoolAdmin -->|can create| Accountant
    SchoolAdmin -->|can create| Teacher
    SchoolAdmin -->|can create| Librarian
    SchoolAdmin -->|can create| Transport
    SchoolAdmin -->|can create| Hostel
    SchoolAdmin -->|implicit for own children| Parent
    SchoolAdmin -->|implicit for own children| Student

    classDef admin fill:#fff3e0,stroke:#e65100,stroke-width:3px
    classDef staff fill:#e1f5ff,stroke:#01579b,stroke-width:2px
    classDef guardian fill:#e8f5e9,stroke:#1b5e20,stroke-width:2px
    class SuperAdmin,SchoolAdmin admin
    class Accountant,Teacher,Librarian,Transport,Hostel staff
    class Parent,Student guardian
```

The arrows are illustrative (which roles a SchoolAdmin can
create) — they are NOT inheritance. Capabilities are not
inherited from role to role. A SchoolAdmin does not
automatically have Teacher capabilities.

## 2. SuperAdmin Capabilities

```mermaid
mindmap
  root((SuperAdmin))
    Platform
      Platform.CrossTenant
      School.Onboard
      School.Suspend
      School.Transfer
      School.Delete
      User.Create
      User.Suspend
    All Domains
      All Academic.* capabilities
      All Finance.* capabilities
      All HR.* capabilities
      All Assessment.* capabilities
      All Attendance.* capabilities
      All Library.* capabilities
      All Facilities.* capabilities
      All Communication.* capabilities
      All Documents.* capabilities
      All CMS.* capabilities
      All Settings.* capabilities
    RBAC
      Role.Create
      Role.Update
      Role.Delete
      Role.Assign
      Capability.Create
      Capability.Update
    Audit
      AuditLog.Read
      AuditLog.Export
```

The SuperAdmin role has every capability in the engine, in
every school, plus cross-tenant capabilities.

## 3. SchoolAdmin Capabilities

```mermaid
mindmap
  root((SchoolAdmin))
    Academic
      Student.* (full)
      Class.* (full)
      Section.* (full)
      Subject.* (full)
      AcademicYear.* (full)
      ClassRoutine.* (full)
      Homework.* (read)
    Assessment
      Exam.* (full)
      Mark.* (read)
      Result.* (read)
      ReportCard.* (read)
    Attendance
      Attendance.* (read)
    Finance
      FeesGroup.* (full)
      FeesType.* (full)
      FeesMaster.* (full)
      Invoice.* (full)
      Payment.* (read)
      Expense.* (full)
      Income.* (full)
      Bank.* (read)
      Payroll.* (read)
      Wallet.* (full)
    HR
      Staff.* (full)
      Leave.* (full)
      Department.* (full)
      Designation.* (full)
    Library
      Library.* (read)
    Facilities
      Facilities.Transport.* (read)
      Facilities.Hostel.* (read)
      Facilities.Inventory.* (read)
    Communication
      Notice.* (full)
      Complaint.* (full)
      Notification.* (full)
    RBAC
      Role.Create
      Role.Update
      Role.Assign
    Settings
      Settings.* (full)
    Users
      User.* (full)
    Audit
      AuditLog.Read
```

## 4. Teacher Capabilities

```mermaid
mindmap
  root((Teacher))
    Academic
      Student.Read
      Class.Read
      Section.Read
      Subject.Read
      ClassRoutine.Read
      ClassRoutine.Update (own)
      Homework.Create
      Homework.Update (own)
      Homework.Evaluate (own)
      LessonPlan.Create
      LessonPlan.Update (own)
    Assessment
      Mark.Create
      Mark.Update (own)
      Result.Read
      Exam.Read
    Attendance
      Attendance.Take (assigned classes)
      Attendance.Read (assigned classes)
    Communication
      Notice.Read
      Complaint.Create
    Reports
      Report.Attendance.Read (own classes)
      Report.Marks.Read (own classes)
```

## 5. Accountant Capabilities

```mermaid
mindmap
  root((Accountant))
    Finance
      FeesGroup.Read
      FeesType.Read
      FeesMaster.Read
      FeesAssign.Read
      FeesAssign.Create
      FeesAssign.Update
      Invoice.Generate
      Invoice.Read
      Invoice.Setting.Read
      Payment.Collect
      Payment.Reverse
      PaymentMethod.Read
      Bank.Read
      Bank.Reconcile
      Expense.Create
      Expense.Update
      Expense.Approve
      Expense.Read
      Income.Create
      Income.Read
      Donor.Read
      Wallet.Approve
      FeesReminder.Configure
      DirectFees.Configure
    Academic
      Student.Read
      Class.Read
    Reports
      Report.Finance.Read
      Report.Collection.Read
      Report.Expense.Read
      Report.Bank.Read
    Audit
      AuditLog.Read (finance scope)
```

## 6. Student Capabilities

```mermaid
mindmap
  root((Student))
    Own profile
      Student.Read (own)
      Student.Update (limited fields, own)
    Academic
      Class.Read
      Section.Read
      Subject.Read
      ClassRoutine.Read
      Homework.Read
      Homework.Submit (own)
      Lesson.Read
      LessonTopic.Read
    Assessment
      Exam.Read
      Mark.Read (own)
      Result.Read (own)
      ReportCard.Read (own)
    Attendance
      Attendance.Read (own)
    Library
      Library.Book.Read
      Library.Book.Issue (own)
      Library.Book.Return (own)
    Facilities
      Facilities.Transport.Read (own)
    Communication
      Notice.Read
      Complaint.Create
      Notification.Read
    Reports
      Report.Attendance.Read (own)
      Report.Marks.Read (own)
```

## 7. Parent Capabilities

```mermaid
mindmap
  root((Parent))
    Linked students
      Student.Read (linked)
      Student.Update (limited)
    Academic
      Class.Read (linked)
      Section.Read (linked)
      ClassRoutine.Read (linked)
      Homework.Read (linked)
    Assessment
      Mark.Read (linked)
      Result.Read (linked)
      ReportCard.Read (linked)
    Attendance
      Attendance.Read (linked)
    Finance
      FeesAssign.Read (linked)
      Invoice.Read (linked)
      Payment.Read (linked)
    Library
      Library.Book.Read (linked)
    Facilities
      Facilities.Transport.Read (linked)
    Communication
      Notice.Read
      Complaint.Create
      Notification.Read
      Chat.Participate
```

## 8. Librarian Capabilities

```mermaid
mindmap
  root((Librarian))
    Library
      Book.Create
      Book.Update
      Book.Delete
      Book.Read
      BookCategory.*
      Member.Create
      Member.Update
      Member.Read
      Book.Issue
      Book.Return
      Book.Renew
      Book.Reserve
      Library.Setting.Configure
    Academic
      Student.Read (for issue context)
    Reports
      Report.Library.Read
```

## 9. Transport / Hostel Capabilities

```mermaid
mindmap
  root((Transport))
    Transport
      TransportRoute.Create
      TransportRoute.Update
      TransportRoute.Delete
      TransportRoute.Read
      TransportVehicle.*
      TransportAssignment.Create
      TransportAssignment.Update
      TransportAssignment.Delete
      TransportAssignment.Read
      Transport.Fee.Configure
    Academic
      Student.Read (for route context)
    Reports
      Report.Transport.Read

    Hostel
      Hostel.*
      HostelRoom.*
      RoomAssignment.*
      Hostel.Fee.Configure
      Hostel.Setting.Configure
```

## 10. Permission Section Map (UI)

```mermaid
graph TB
    UI[UI Sidebar]
    UI --> Sec1[Student Information]
    UI --> Sec2[Academic Setup]
    UI --> Sec3[Attendance]
    UI --> Sec4[Examination]
    UI --> Sec5[Fees Collection]
    UI --> Sec6[Accounts]
    UI --> Sec7[Payroll]
    UI --> Sec8[Human Resource]
    UI --> Sec9[Library]
    UI --> Sec10[Transport]
    UI --> Sec11[Hostel]
    UI --> Sec12[Inventory]
    UI --> Sec13[Communication]
    UI --> Sec14[Reports]
    UI --> Sec15[Settings]
    UI --> Sec16[Roles & Permissions]
    UI --> Sec17[Users]
    UI --> Sec18[School Management]
    UI --> Sec19[Audit]

    Sec1 --> C1[Student.*]
    Sec2 --> C2[Class.*, Section.*, Subject.*, AcademicYear.*]
    Sec3 --> C3[Attendance.*]
    Sec4 --> C4[Exam.*, Mark.*, Result.*, ReportCard.*]
    Sec5 --> C5[FeesGroup.*, FeesType.*, FeesMaster.*, Invoice.*, Payment.*, Bank.*]
    Sec6 --> C6[Expense.*, Income.*, Wallet.*, Donor.*]
    Sec7 --> C7[Payroll.*, SalaryTemplate.*]
    Sec8 --> C8[Staff.*, Leave.*, Department.*, Designation.*]
    Sec9 --> C9[Library.*]
    Sec10 --> C10[Facilities.Transport.*]
    Sec11 --> C11[Facilities.Hostel.*]
    Sec12 --> C12[Facilities.Inventory.*]
    Sec13 --> C13[Notice.*, Complaint.*, Chat.*, Notification.*]
    Sec14 --> C14[Report.*]
    Sec15 --> C15[Settings.*, Theme.*]
    Sec16 --> C16[Role.*, Capability.*, PermissionSection.*, TwoFactor.*]
    Sec17 --> C17[User.*]
    Sec18 --> C18[School.*]
    Sec19 --> C19[AuditLog.*]
```

The `PermissionSection` catalog groups capabilities for UI
rendering. The same grouping helps AI agents reason over
the capability space.
