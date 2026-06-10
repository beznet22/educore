# Capability-Based RBAC Guide

## Goal

Permissions in Educore are **capabilities**, not role strings. A
capability is a typed enum value: `Capability::StudentAdmit`,
`Capability::FinanceInvoiceGenerate`. The engine refuses a command if
the actor's session does not contain the required capability.

## Why Capabilities

- **String-typed roles are error-prone**: `"admin"`, `"super_admin"`,
  `"Admin"`, `"admin "` are all distinct. Capabilities are typed
  enums; the compiler catches typos.
- **Roles bundle too much**: a "teacher" role grants all teacher
  permissions, even ones the teacher should not have. Capabilities
  allow fine-grained control.
- **Capabilities are auditable**: a permission change is the addition
  or removal of a capability, not the renaming of a role.

## Capability Type

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    StudentAdmit,
    StudentUpdate,
    StudentRead,
    StudentSuspend,
    StudentReinstate,
    StudentWithdraw,
    StudentTransfer,
    StudentPromote,
    StudentGraduate,

    AttendanceMark,
    AttendanceRead,
    AttendanceBulkMark,
    AttendanceReport,

    ExamCreate,
    ExamSchedule,
    ExamEnterMarks,
    ExamPublishResult,
    ExamReportCardGenerate,
    OnlineExamCreate,
    OnlineExamEvaluate,
    SeatPlanGenerate,
    AdmitCardGenerate,

    FeesCreate,
    FeesAssign,
    FeesInvoiceGenerate,
    FeesPaymentRecord,
    FeesRefund,
    FeesDiscount,
    FeesReport,

    PayrollGenerate,
    PayrollApprove,
    PayrollPay,

    StaffRegister,
    StaffUpdate,
    StaffRead,
    LeaveRequest,
    LeaveApprove,

    LibraryBookAdd,
    LibraryBookIssue,
    LibraryBookReturn,
    LibraryFineWaive,

    TransportAssign,
    TransportManage,

    DormitoryAssign,
    DormitoryManage,

    InventoryReceive,
    InventoryIssue,
    InventorySell,

    NoticeCreate,
    NoticePublish,
    ComplaintResolve,
    ChatModerate,
    NotificationSend,

    CalendarEventCreate,
    HolidayConfigure,
    IncidentResolve,

    FormUpload,
    PostalDispatch,
    PostalReceive,

    PageCreate,
    PagePublish,
    NewsCreate,
    NewsPublish,
    ContentCreate,
    ContentPublish,

    RbacRoleCreate,
    RbacRoleAssign,
    RbacTwoFactorConfigure,

    SettingsUpdate,
    LanguageAdd,
    ThemeConfigure,

    BackupCreate,
    BackupRestore,
    JobSchedule,

    AuditRead,
    ReportSchoolWide,
    ReportDistrictWide,
    ReportGenerate,
}
```

Each domain may extend the `Capability` enum. The engine provides a
`Capability::all()` for super-admin and a `Capability::all_school()`
for school-admin.

## Roles

Roles are named bundles of capabilities. The RBAC domain stores them.

```rust
pub struct Role {
    pub role_id: RoleId,
    pub name: RoleName,
    pub school_id: SchoolId,
    pub capabilities: BTreeSet<Capability>,
    pub is_system: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
```

The engine ships a default role catalog:

| Role          | Capabilities                                            |
| ------------- | ------------------------------------------------------- |
| `SuperAdmin`  | All                                                     |
| `SchoolAdmin` | All within the school                                   |
| `Teacher`     | `StudentRead`, `AttendanceMark`, `Exam*` (read), `Lesson*`, `Homework*` |
| `Accountant`  | `Fees*`, `Finance*`, `Bank*`                            |
| `Librarian`   | `Library*`                                              |
| `Transport`   | `Transport*`                                            |
| `Parent`      | `StudentRead` (linked), `HomeworkRead` (linked)         |
| `Student`     | `StudentRead` (self), `HomeworkSubmit` (self)           |
| `Auditor`     | `AuditRead`, `ReportGenerate`                           |

Consumers can extend or override.

## Command Flow

```text
Client request
    │
    ▼
Auth middleware → Session
    │
    ▼
Engine.dispatch(command)
    │
    ▼
Capability check
    │
    ├─ missing → DomainError::Forbidden
    │
    ▼
Tenant check
    │
    ├─ cross-tenant → DomainError::Forbidden
    │
    ▼
Command handler
```

```rust
impl Engine {
    pub async fn dispatch(&self, cmd: BoxedCommand) -> Result<BoxedOutcome> {
        let session = self.current_session().await?;
        let cap = cmd.required_capability();
        if !session.capabilities.contains(&cap) {
            return Err(DomainError::forbidden(format!(
                "missing capability: {:?}", cap
            )));
        }
        let tenant = TenantContext::from_session(&session);
        if !cmd.tenant_matches(&tenant) {
            return Err(DomainError::forbidden("tenant mismatch"));
        }
        self.dispatch_inner(cmd).await
    }
}
```

## Role-to-Capability Mapping

Roles are defined in the RBAC domain. The mapping is editable by
school admins (subject to `RbacRoleAssign` capability).

```rust
pub struct AssignCapabilityToRoleCommand {
    pub tenant: TenantContext,
    pub role_id: RoleId,
    pub capability: Capability,
    pub grant: bool,    // true = add, false = revoke
}
```

## User-to-Role Assignment

A user can hold multiple roles in a school. The session aggregates
their capabilities.

```rust
pub struct AssignRoleToUserCommand {
    pub tenant: TenantContext,
    pub user_id: UserId,
    pub role_id: RoleId,
}
```

When a user is assigned a role, the engine invalidates their existing
session(s) so that a new session with updated capabilities is issued
on next authentication.

## Two-Factor Authentication

The RBAC domain configures two-factor settings per role:

```rust
pub struct TwoFactorSetting {
    pub for_admin: bool,
    pub for_teacher: bool,
    pub for_staff: bool,
    pub for_student: bool,
    pub for_parent: bool,
    pub via_sms: bool,
    pub via_email: bool,
    pub via_totp: bool,
    pub expired_time: Duration,
}
```

A user with `mfa_satisfied = false` is restricted from sensitive
commands (e.g. `FinanceRefund`, `StudentWithdraw`).

## Resource Ownership

Some capabilities are paired with **resource ownership**: a student
can only submit their own homework, a teacher can only evaluate
homework for their assigned class-section. The engine performs the
ownership check after the capability check.

```rust
pub fn check_homework_ownership(actor: &Session, homework: &Homework) -> Result<()> {
    if actor.capabilities.contains(&Capability::HomeworkEvaluate) {
        return Ok(());
    }
    if actor.capabilities.contains(&Capability::HomeworkSubmit) {
        let student_id = actor.user_id.try_into_student()?;
        if homework.student_id == student_id {
            return Ok(());
        }
    }
    Err(DomainError::forbidden("not your homework"))
}
```

## Audit

Every capability check, grant, and revoke is recorded in the audit
log. A failed capability check is recorded with `outcome: Denied` and
the reason.

## Testing

- Unit tests of every capability being required by its command.
- Integration tests of role assignment changing session capabilities.
- A test of missing capability → Forbidden.
- A test of cross-tenant → Forbidden.
- A test of resource ownership enforcement.
- A test of session invalidation on role change.
- A test of MFA enforcement.
