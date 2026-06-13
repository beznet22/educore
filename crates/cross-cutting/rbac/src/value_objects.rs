//! # RBAC value objects
//!
//! The closed-enum [`Capability`] and its companion value objects.
//! The full set of capabilities recognised by the engine lives in
//! this module; new capabilities require a code change, a migration
//! to seed [`Permission`](crate::aggregate::Permission) rows, and a
//! new platform release (per `docs/specs/rbac/aggregates.md`).
//!
//! String form is `<Domain>.<Aggregate>.<Action>` (e.g.
//! `"Platform.School.Create"`, `"Rbac.Role.Manage"`). Parsing is
//! total: unknown strings return [`DomainError::Validation`].

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use educore_core::error::{DomainError, Result};

/// The atomic unit of authorization.
///
/// The full catalog is locked at compile time. Phase 2 covers the
/// RBAC and Platform domains; remaining domains are listed as
/// placeholders so the catalog compiles end-to-end while those
/// domains are still in the design phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Capability {
    // -- Platform --------------------------------------------------------
    /// Create a school.
    PlatformSchoolCreate,
    /// Read a school.
    PlatformSchoolRead,
    /// Update a school.
    PlatformSchoolUpdate,
    /// Delete / retire a school.
    PlatformSchoolDelete,
    /// Create a user.
    PlatformUserCreate,
    /// Read a user.
    PlatformUserRead,
    /// Update a user.
    PlatformUserUpdate,
    /// Delete / deactivate a user.
    PlatformUserDelete,

    // -- Rbac ------------------------------------------------------------
    /// Create a role.
    RbacRoleCreate,
    /// Read a role.
    RbacRoleRead,
    /// Update a role.
    RbacRoleUpdate,
    /// Delete a role.
    RbacRoleDelete,
    /// Manage system roles (rename, mutate sealed rows).
    RbacRoleManage,
    /// Clone a role.
    RbacRoleClone,
    /// Assign a capability to a role.
    RbacCapabilityAssign,
    /// Revoke a capability from a role.
    RbacCapabilityRevoke,
    /// Read the capability catalog.
    RbacCapabilityRead,
    /// Update the cosmetic metadata of a registered capability.
    RbacCapabilityUpdateMetadata,
    /// Bootstrap a fresh installation. Held only by `SuperAdmin` and
    /// is never revocable.
    RbacBootstrap,

    // -- Academic (Phase 3 placeholders) --------------------------------
    /// Create a student record. Placeholder for the academic domain.
    AcademicStudentCreate,
    /// Read a student record.
    AcademicStudentRead,
    /// Update a student record.
    AcademicStudentUpdate,
    /// Delete / retire a student record.
    AcademicStudentDelete,
    /// Create a class / section. Placeholder for the academic domain.
    AcademicClassCreate,
    /// Read a class / section.
    AcademicClassRead,
    /// Update a class / section.
    AcademicClassUpdate,
    /// Delete a class / section.
    AcademicClassDelete,

    // -- Assessment (Phase 4) -------------------------------------------
    /// Create an exam definition.
    AssessmentExamCreate,
    /// Read an exam definition.
    AssessmentExamRead,
    /// Update an exam definition.
    AssessmentExamUpdate,
    /// Delete / retire an exam definition.
    AssessmentExamDelete,
    /// Create an exam schedule (per-section per-subject slot).
    AssessmentExamScheduleCreate,
    /// Read an exam schedule.
    AssessmentExamScheduleRead,
    /// Update an exam schedule.
    AssessmentExamScheduleUpdate,
    /// Delete / cancel an exam schedule.
    AssessmentExamScheduleDelete,
    /// Initialise a marks register.
    AssessmentMarksRegisterCreate,
    /// Read a marks register.
    AssessmentMarksRegisterRead,
    /// Enter / update a marks register child row.
    AssessmentMarksRegisterUpdate,
    /// Submit / cancel a marks register.
    AssessmentMarksRegisterDelete,
    /// Create / save a result store row.
    AssessmentResultStoreCreate,
    /// Read a result store row.
    AssessmentResultStoreRead,
    /// Update a result store row's teacher remarks.
    AssessmentResultStoreUpdate,
    /// Publish / re-publish a result store row.
    AssessmentResultStoreDelete,
    /// Materialise a report card payload for a published result.
    AssessmentReportCardGenerate,
    /// Read a previously generated report card.
    AssessmentReportCardRead,
    /// Download a previously generated report card (PDF / HTML).
    AssessmentReportCardDownload,
    /// Create an online exam.
    AssessmentOnlineExamCreate,
    /// Read an online exam.
    AssessmentOnlineExamRead,
    /// Update an online exam.
    AssessmentOnlineExamUpdate,
    /// Delete / close an online exam.
    AssessmentOnlineExamDelete,
    /// Generate a seat plan.
    AssessmentSeatPlanCreate,
    /// Read a seat plan.
    AssessmentSeatPlanRead,
    /// Update a seat plan.
    AssessmentSeatPlanUpdate,
    /// Cancel a seat plan.
    AssessmentSeatPlanDelete,
    /// Generate an admit card.
    AssessmentAdmitCardCreate,
    /// Read an admit card.
    AssessmentAdmitCardRead,
    /// Update an admit card.
    AssessmentAdmitCardUpdate,
    /// Cancel an admit card.
    AssessmentAdmitCardDelete,

    // -- Attendance (Phase 5) -------------------------------------------
    /// Create (mark) a daily student attendance record.
    AttendanceStudentCreate,
    /// Read a daily student attendance record.
    AttendanceStudentRead,
    /// Update a daily student attendance record.
    AttendanceStudentUpdate,
    /// Soft-delete / cancel a daily student attendance record.
    AttendanceStudentDelete,
    /// Create (mark) a per-period subject attendance record.
    AttendanceSubjectCreate,
    /// Read a per-period subject attendance record.
    AttendanceSubjectRead,
    /// Update a per-period subject attendance record.
    AttendanceSubjectUpdate,
    /// Soft-delete / cancel a per-period subject attendance record.
    AttendanceSubjectDelete,
    /// Request a guardian notification for a subject-absence.
    AttendanceSubjectNotify,
    /// Create (mark) a daily staff attendance record.
    AttendanceStaffCreate,
    /// Read a daily staff attendance record.
    AttendanceStaffRead,
    /// Update a daily staff attendance record.
    AttendanceStaffUpdate,
    /// Soft-delete / cancel a daily staff attendance record.
    AttendanceStaffDelete,
    /// Create an exam-attendance record.
    AttendanceExamCreate,
    /// Read an exam-attendance record.
    AttendanceExamRead,
    /// Update an exam-attendance record.
    AttendanceExamUpdate,
    /// Soft-delete / cancel an exam-attendance record.
    AttendanceExamDelete,
    /// Create a bulk-attendance-import job (CSV / biometric).
    AttendanceImportCreate,
    /// Read a bulk-attendance-import job.
    AttendanceImportRead,
    /// Validate / commit a bulk-attendance-import job.
    AttendanceImportUpdate,
    /// Cancel a bulk-attendance-import job.
    AttendanceImportDelete,
    /// Bulk-mark attendance for a class-section.
    AttendanceBulkMark,
    /// Read an attendance report (daily, weekly, monthly, by-class, by-student, by-staff).
    AttendanceReportRead,
    /// Request a guardian absence notification.
    AttendanceNotify,

    // -- Finance (Phase 7) -----------------------------------------------
    // The 4 legacy `FinanceInvoice*` placeholders (Create/Read/Update/Delete)
    // are retained for backward compat with the Phase 2 catalog; the
    // newer `FinanceFeesInvoice*` variants below are the spec-aligned
    // surface (per `docs/specs/finance/permissions.md`).
    /// Create a finance invoice. Placeholder from Phase 2.
    FinanceInvoiceCreate,
    /// Read a finance invoice.
    FinanceInvoiceRead,
    /// Update a finance invoice.
    FinanceInvoiceUpdate,
    /// Delete / void a finance invoice.
    FinanceInvoiceDelete,
    // -- Fees catalog --
    /// Create a fees group.
    FinanceFeesGroupCreate,
    /// Read a fees group.
    FinanceFeesGroupRead,
    /// Update a fees group.
    FinanceFeesGroupUpdate,
    /// Delete a fees group.
    FinanceFeesGroupDelete,
    /// Create a fees type (within a fees group).
    FinanceFeesTypeCreate,
    /// Read a fees type.
    FinanceFeesTypeRead,
    /// Update a fees type.
    FinanceFeesTypeUpdate,
    /// Delete a fees type.
    FinanceFeesTypeDelete,
    /// Create a fees master (per-class per-fees-type).
    FinanceFeesMasterCreate,
    /// Read a fees master.
    FinanceFeesMasterRead,
    /// Update a fees master (amount / due date).
    FinanceFeesMasterUpdate,
    /// Delete a fees master.
    FinanceFeesMasterDelete,
    /// Create a fees discount.
    FinanceFeesDiscountCreate,
    /// Read a fees discount.
    FinanceFeesDiscountRead,
    /// Update a fees discount.
    FinanceFeesDiscountUpdate,
    /// Delete a fees discount.
    FinanceFeesDiscountDelete,
    // -- Assignment + installments --
    /// Create a per-student fees assignment.
    FinanceFeesAssignCreate,
    /// Read a per-student fees assignment.
    FinanceFeesAssignRead,
    /// Update a per-student fees assignment.
    FinanceFeesAssignUpdate,
    /// Close a per-student fees assignment.
    FinanceFeesAssignClose,
    /// Create a fees installment plan.
    FinanceFeesInstallmentCreate,
    /// Read a fees installment plan.
    FinanceFeesInstallmentRead,
    /// Update a fees installment plan.
    FinanceFeesInstallmentUpdate,
    /// Delete a fees installment plan.
    FinanceFeesInstallmentDelete,
    /// Assign a fees installment to a student.
    FinanceFeesInstallmentAssign,
    /// Create a direct (custom-percent) fees installment.
    FinanceDirectFeesInstallmentCreate,
    /// Read a direct fees installment.
    FinanceDirectFeesInstallmentRead,
    /// Update a direct fees installment.
    FinanceDirectFeesInstallmentUpdate,
    /// Delete a direct fees installment.
    FinanceDirectFeesInstallmentDelete,
    /// Assign a direct fees installment to a student.
    FinanceDirectFeesInstallmentAssign,
    /// Record payment against a direct fees installment.
    FinanceDirectFeesInstallmentPay,
    // -- Invoice + payment --
    /// Generate a fees invoice.
    FinanceFeesInvoiceGenerate,
    /// Read a fees invoice.
    FinanceFeesInvoiceRead,
    /// Update a fees invoice.
    FinanceFeesInvoiceUpdate,
    /// Cancel a fees invoice.
    FinanceFeesInvoiceCancel,
    /// Configure a fees invoice (numbering / layout).
    FinanceFeesInvoiceConfigure,
    /// Print a fees invoice.
    FinanceFeesInvoicePrint,
    /// Collect a fees payment (cashier / portal).
    FinancePaymentCollect,
    /// Read a fees payment.
    FinancePaymentRead,
    /// Reverse a fees payment.
    FinancePaymentReverse,
    /// Refund a fees payment (per the spec's `Refund` headline).
    FinancePaymentRefund,
    /// Create a payment method (cash / bank / gateway).
    FinancePaymentMethodCreate,
    /// Read a payment method.
    FinancePaymentMethodRead,
    /// Update a payment method.
    FinancePaymentMethodUpdate,
    /// Delete a payment method.
    FinancePaymentMethodDelete,
    /// Configure a payment gateway.
    FinancePaymentGatewayConfigure,
    /// Read a payment gateway configuration.
    FinancePaymentGatewayRead,
    /// Update a payment gateway configuration.
    FinancePaymentGatewayUpdate,
    /// Disable a payment gateway.
    FinancePaymentGatewayDisable,
    // -- Expense + Income --
    /// Create an expense.
    FinanceExpenseCreate,
    /// Read an expense.
    FinanceExpenseRead,
    /// Update an expense.
    FinanceExpenseUpdate,
    /// Delete an expense.
    FinanceExpenseDelete,
    /// Approve an expense.
    FinanceExpenseApprove,
    /// Create an expense head (category).
    FinanceExpenseHeadCreate,
    /// Read an expense head.
    FinanceExpenseHeadRead,
    /// Update an expense head.
    FinanceExpenseHeadUpdate,
    /// Delete an expense head.
    FinanceExpenseHeadDelete,
    /// Create an income.
    FinanceIncomeCreate,
    /// Read an income.
    FinanceIncomeRead,
    /// Update an income.
    FinanceIncomeUpdate,
    /// Delete an income.
    FinanceIncomeDelete,
    /// Approve an income.
    FinanceIncomeApprove,
    /// Create an income head (category).
    FinanceIncomeHeadCreate,
    /// Read an income head.
    FinanceIncomeHeadRead,
    /// Update an income head.
    FinanceIncomeHeadUpdate,
    /// Delete an income head.
    FinanceIncomeHeadDelete,
    // -- Banking --
    /// Open a bank / cash account.
    FinanceBankOpen,
    /// Read a bank / cash account.
    FinanceBankRead,
    /// Update a bank / cash account.
    FinanceBankUpdate,
    /// Close a bank / cash account.
    FinanceBankClose,
    /// Record a bank statement (debit / credit).
    FinanceBankStatementRecord,
    /// Reverse a bank statement.
    FinanceBankStatementReverse,
    /// Transfer funds between two bank accounts.
    FinanceBankTransfer,
    // -- Bank slip --
    /// Generate a bank payment slip.
    FinanceBankSlipGenerate,
    /// Read a bank payment slip.
    FinanceBankSlipRead,
    /// Approve a bank payment slip.
    FinanceBankSlipApprove,
    /// Reject a bank payment slip.
    FinanceBankSlipReject,
    // -- Payroll accounting (finance-side; distinct from HR's `Hr.Payroll.*`) --
    /// Read a payroll payment (cross-domain visibility for finance).
    FinancePayrollPaymentRead,
    /// Record a payroll payment (the HR→finance bridge entry point).
    FinancePayrollPaymentRecord,
    // -- Wallet --
    /// Credit a user's wallet (pending until approved).
    FinanceWalletCredit,
    /// Debit a user's wallet (pending until approved).
    FinanceWalletDebit,
    /// Read a wallet and its transactions.
    FinanceWalletRead,
    /// Approve a pending wallet transaction.
    FinanceWalletApprove,
    /// Reject a pending wallet transaction.
    FinanceWalletReject,
    // -- Carry forward --
    /// Execute a fees carry-forward (move balance to next academic year).
    FinanceFeesCarryForwardExecute,
    /// Read a fees carry-forward row.
    FinanceFeesCarryForwardRead,
    /// Configure a fees carry-forward rule.
    FinanceFeesCarryForwardConfigure,
    // -- Due fees login prevention --
    /// Block a user's login for overdue fees.
    FinanceDueFeesBlock,
    /// Unblock a user's login (balance cleared).
    FinanceDueFeesUnblock,
    /// Read the due-fees login-prevention list.
    FinanceDueFeesRead,
    // -- Fees reminder --
    /// Configure a fees reminder rule.
    FinanceFeesReminderConfigure,
    /// Read a fees reminder rule.
    FinanceFeesReminderRead,
    /// Update a fees reminder rule.
    FinanceFeesReminderUpdate,
    /// Delete a fees reminder rule.
    FinanceFeesReminderDelete,
    // -- Chart of accounts --
    /// Create a chart-of-account category.
    FinanceChartOfAccountCreate,
    /// Read a chart-of-account category.
    FinanceChartOfAccountRead,
    /// Update a chart-of-account category.
    FinanceChartOfAccountUpdate,
    /// Delete a chart-of-account category.
    FinanceChartOfAccountDelete,
    // -- Reports (read-only; finance) --
    /// Read the fees-collected report.
    FinanceReportFeesCollected,
    /// Read the fees-outstanding report.
    FinanceReportFeesOutstanding,
    /// Read the daily-collection report.
    FinanceReportDailyCollection,
    /// Read the expense report.
    FinanceReportExpense,
    /// Read the bank-reconciliation report.
    FinanceReportBankReconciliation,
    /// Read the wallet-ledger report.
    FinanceReportWalletLedger,
    /// Read the umbrella finance report (covers everything above).
    FinanceReportRead,

    // -- HR (Phase 6) -----------------------------------------------------
    /// Create a staff member.
    HrStaffCreate,
    /// Read a staff member.
    HrStaffRead,
    /// Update a staff member.
    HrStaffUpdate,
    /// Delete / deactivate a staff member.
    HrStaffDelete,
    /// Suspend a staff member.
    HrStaffSuspend,
    /// Reinstate a suspended staff member.
    HrStaffReinstate,
    /// Resign a staff member (lifecycle).
    HrStaffResign,
    /// Terminate a staff member (lifecycle).
    HrStaffTerminate,
    /// Retire a staff member (lifecycle).
    HrStaffRetire,
    /// Change a staff member's department.
    HrStaffChangeDepartment,
    /// Change a staff member's designation.
    HrStaffChangeDesignation,
    /// Change a staff member's role.
    HrStaffChangeRole,
    /// Assign a subject teacher to a class-section.
    HrStaffAssignSubjectTeacher,
    /// Create a class-teacher assignment.
    HrStaffAssignClassTeacherCreate,
    /// Update a class-teacher assignment.
    HrStaffAssignClassTeacherUpdate,
    /// Delete a class-teacher assignment.
    HrStaffAssignClassTeacherDelete,
    /// Initiate a bulk-staff-import.
    HrStaffImportBulk,
    /// Promote a staged bulk-import row to a `Staff` aggregate.
    HrStaffImportBulkPromote,
    /// Reject a staged bulk-import row.
    HrStaffImportBulkReject,
    /// Upload a staff document.
    HrStaffDocumentUpload,
    /// Download a staff document.
    HrStaffDocumentDownload,
    /// Create a department.
    HrDepartmentCreate,
    /// Read a department.
    HrDepartmentRead,
    /// Update a department.
    HrDepartmentUpdate,
    /// Delete a department.
    HrDepartmentDelete,
    /// Create a designation.
    HrDesignationCreate,
    /// Read a designation.
    HrDesignationRead,
    /// Update a designation.
    HrDesignationUpdate,
    /// Delete a designation.
    HrDesignationDelete,
    /// Create a leave type.
    HrLeaveTypeCreate,
    /// Read a leave type.
    HrLeaveTypeRead,
    /// Update a leave type.
    HrLeaveTypeUpdate,
    /// Delete a leave type.
    HrLeaveTypeDelete,
    /// Create a leave-entitlement policy (`LeaveDefine`).
    HrLeaveDefineCreate,
    /// Read a leave-entitlement policy.
    HrLeaveDefineRead,
    /// Update a leave-entitlement policy.
    HrLeaveDefineUpdate,
    /// Delete a leave-entitlement policy.
    HrLeaveDefineDelete,
    /// Request a leave period.
    HrLeaveRequest,
    /// Approve a leave request.
    HrLeaveApprove,
    /// Reject a leave request.
    HrLeaveReject,
    /// Cancel a leave request (pending or within grace window).
    HrLeaveCancel,
    /// Read leave requests and balances.
    HrLeaveRead,
    /// Mark daily staff attendance.
    HrAttendanceStaffMark,
    /// Update a staff-attendance record.
    HrAttendanceStaffUpdate,
    /// Delete a staff-attendance record.
    HrAttendanceStaffDelete,
    /// Read a staff-attendance record.
    HrAttendanceStaffRead,
    /// Initiate a bulk staff-attendance import.
    HrAttendanceStaffImport,
    /// Promote a staged staff-attendance import row.
    HrAttendanceStaffImportPromote,
    /// Reject a staged staff-attendance import row.
    HrAttendanceStaffImportReject,
    /// Generate a monthly payroll run.
    HrPayrollGenerate,
    /// Update a payroll run's amounts.
    HrPayrollUpdate,
    /// Approve a payroll run (segregation of duties).
    HrPayrollApprove,
    /// Mark a payroll run as paid (HR-side acknowledgement).
    HrPayrollMarkPaid,
    /// Read a payroll run.
    HrPayrollRead,
    /// Add a payroll earning line.
    HrPayrollEarningAdd,
    /// Update a payroll earning line.
    HrPayrollEarningUpdate,
    /// Delete a payroll earning line.
    HrPayrollEarningDelete,
    /// Add a payroll deduction line.
    HrPayrollDeductionAdd,
    /// Update a payroll deduction line.
    HrPayrollDeductionUpdate,
    /// Delete a payroll deduction line.
    HrPayrollDeductionDelete,
    /// Add a leave-deduction info row to a payroll run.
    HrPayrollLeaveDeductionAdd,
    /// Update a leave-deduction info row.
    HrPayrollLeaveDeductionUpdate,
    /// Delete a leave-deduction info row.
    HrPayrollLeaveDeductionDelete,
    /// Read a payroll-payment row (cross-domain visibility for HR).
    HrPayrollPaymentRead,
    /// Create a salary template (grade-based structure).
    HrSalaryTemplateCreate,
    /// Read a salary template.
    HrSalaryTemplateRead,
    /// Update a salary template.
    HrSalaryTemplateUpdate,
    /// Delete a salary template.
    HrSalaryTemplateDelete,
    /// Set a per-grade hourly rate.
    HrHourlyRateSet,
    /// Read an hourly-rate row.
    HrHourlyRateRead,
    /// Update an hourly-rate row.
    HrHourlyRateUpdate,
    /// Delete an hourly-rate row.
    HrHourlyRateDelete,
    /// Create a custom staff-registration field.
    HrStaffRegistrationFieldCreate,
    /// Read a custom staff-registration field.
    HrStaffRegistrationFieldRead,
    /// Update a custom staff-registration field.
    HrStaffRegistrationFieldUpdate,
    /// Delete a custom staff-registration field.
    HrStaffRegistrationFieldDelete,
    /// Read a staff roster report.
    HrReportStaffRoster,
    /// Read a staff-by-department report.
    HrReportStaffByDepartment,
    /// Read a staff-by-designation report.
    HrReportStaffByDesignation,
    /// Read a leave-usage report.
    HrReportLeaveUsage,
    /// Read a leave-balance report.
    HrReportLeaveBalance,
    /// Read a daily staff-attendance report.
    HrReportAttendanceDaily,
    /// Read a monthly staff-attendance report.
    HrReportAttendanceMonthly,
    /// Read a per-staff attendance report.
    HrReportAttendanceByStaff,
    /// Read a payroll register report.
    HrReportPayrollRegister,
    /// Read a per-staff payroll report.
    HrReportPayrollByStaff,
    /// Read a per-department payroll report.
    HrReportPayrollByDepartment,
    /// Read a payroll tax report.
    HrReportPayrollTax,
    /// Read a salary-structure report.
    HrReportSalaryStructure,
    /// Read an hourly-earnings report.
    HrReportHourlyEarnings,
    /// Read a leave-deduction report.
    HrReportLeaveDeduction,
    /// Read the umbrella HR report (covers everything above).
    HrReportRead,

    // -- Library, Communication, Documents, CMS, Facilities, Events ----
    /// Create a library book. Placeholder for the library domain.
    LibraryBookCreate,
    /// Read a library book.
    LibraryBookRead,
    /// Update a library book.
    LibraryBookUpdate,
    /// Delete a library book.
    LibraryBookDelete,
    /// Create a communication message.
    CommunicationMessageCreate,
    /// Read a communication message.
    CommunicationMessageRead,
    /// Update a communication message.
    CommunicationMessageUpdate,
    /// Delete a communication message.
    CommunicationMessageDelete,
    /// Create a documents folder.
    DocumentsFolderCreate,
    /// Read a documents folder.
    DocumentsFolderRead,
    /// Update a documents folder.
    DocumentsFolderUpdate,
    /// Delete a documents folder.
    DocumentsFolderDelete,
    /// Create a CMS page.
    CmsPageCreate,
    /// Read a CMS page.
    CmsPageRead,
    /// Update a CMS page.
    CmsPageUpdate,
    /// Delete a CMS page.
    CmsPageDelete,
    /// Create a facilities room.
    FacilitiesRoomCreate,
    /// Read a facilities room.
    FacilitiesRoomRead,
    /// Update a facilities room.
    FacilitiesRoomUpdate,
    /// Delete a facilities room.
    FacilitiesRoomDelete,
    /// Create an events-domain calendar entry.
    EventsCalendarCreate,
    /// Read an events-domain calendar entry.
    EventsCalendarRead,
    /// Update an events-domain calendar entry.
    EventsCalendarUpdate,
    /// Delete an events-domain calendar entry.
    EventsCalendarDelete,

    // -- Cross-cutting management ---------------------------------------
    /// Manage settings for the active school.
    SettingsManage,
    /// Manage operations (backups, jobs) for the active school.
    OperationsManage,
}

impl Capability {
    /// Returns the domain prefix for this capability.
    #[must_use]
    pub const fn domain(self) -> CapabilityDomain {
        match self {
            Self::PlatformSchoolCreate
            | Self::PlatformSchoolRead
            | Self::PlatformSchoolUpdate
            | Self::PlatformSchoolDelete
            | Self::PlatformUserCreate
            | Self::PlatformUserRead
            | Self::PlatformUserUpdate
            | Self::PlatformUserDelete => CapabilityDomain::Platform,
            Self::RbacRoleCreate
            | Self::RbacRoleRead
            | Self::RbacRoleUpdate
            | Self::RbacRoleDelete
            | Self::RbacRoleManage
            | Self::RbacRoleClone
            | Self::RbacCapabilityAssign
            | Self::RbacCapabilityRevoke
            | Self::RbacCapabilityRead
            | Self::RbacCapabilityUpdateMetadata
            | Self::RbacBootstrap => CapabilityDomain::Rbac,
            Self::AcademicStudentCreate
            | Self::AcademicStudentRead
            | Self::AcademicStudentUpdate
            | Self::AcademicStudentDelete
            | Self::AcademicClassCreate
            | Self::AcademicClassRead
            | Self::AcademicClassUpdate
            | Self::AcademicClassDelete => CapabilityDomain::Academic,
            Self::AssessmentExamCreate
            | Self::AssessmentExamRead
            | Self::AssessmentExamUpdate
            | Self::AssessmentExamDelete
            | Self::AssessmentExamScheduleCreate
            | Self::AssessmentExamScheduleRead
            | Self::AssessmentExamScheduleUpdate
            | Self::AssessmentExamScheduleDelete
            | Self::AssessmentMarksRegisterCreate
            | Self::AssessmentMarksRegisterRead
            | Self::AssessmentMarksRegisterUpdate
            | Self::AssessmentMarksRegisterDelete
            | Self::AssessmentResultStoreCreate
            | Self::AssessmentResultStoreRead
            | Self::AssessmentResultStoreUpdate
            | Self::AssessmentResultStoreDelete
            | Self::AssessmentReportCardGenerate
            | Self::AssessmentReportCardRead
            | Self::AssessmentReportCardDownload
            | Self::AssessmentOnlineExamCreate
            | Self::AssessmentOnlineExamRead
            | Self::AssessmentOnlineExamUpdate
            | Self::AssessmentOnlineExamDelete
            | Self::AssessmentSeatPlanCreate
            | Self::AssessmentSeatPlanRead
            | Self::AssessmentSeatPlanUpdate
            | Self::AssessmentSeatPlanDelete
            | Self::AssessmentAdmitCardCreate
            | Self::AssessmentAdmitCardRead
            | Self::AssessmentAdmitCardUpdate
            | Self::AssessmentAdmitCardDelete => CapabilityDomain::Assessment,
            Self::AttendanceStudentCreate
            | Self::AttendanceStudentRead
            | Self::AttendanceStudentUpdate
            | Self::AttendanceStudentDelete
            | Self::AttendanceSubjectCreate
            | Self::AttendanceSubjectRead
            | Self::AttendanceSubjectUpdate
            | Self::AttendanceSubjectDelete
            | Self::AttendanceSubjectNotify
            | Self::AttendanceStaffCreate
            | Self::AttendanceStaffRead
            | Self::AttendanceStaffUpdate
            | Self::AttendanceStaffDelete
            | Self::AttendanceExamCreate
            | Self::AttendanceExamRead
            | Self::AttendanceExamUpdate
            | Self::AttendanceExamDelete
            | Self::AttendanceImportCreate
            | Self::AttendanceImportRead
            | Self::AttendanceImportUpdate
            | Self::AttendanceImportDelete
            | Self::AttendanceBulkMark
            | Self::AttendanceReportRead
            | Self::AttendanceNotify => CapabilityDomain::Attendance,
            Self::FinanceInvoiceCreate
            | Self::FinanceInvoiceRead
            | Self::FinanceInvoiceUpdate
            | Self::FinanceInvoiceDelete
            | Self::FinanceFeesGroupCreate
            | Self::FinanceFeesGroupRead
            | Self::FinanceFeesGroupUpdate
            | Self::FinanceFeesGroupDelete
            | Self::FinanceFeesTypeCreate
            | Self::FinanceFeesTypeRead
            | Self::FinanceFeesTypeUpdate
            | Self::FinanceFeesTypeDelete
            | Self::FinanceFeesMasterCreate
            | Self::FinanceFeesMasterRead
            | Self::FinanceFeesMasterUpdate
            | Self::FinanceFeesMasterDelete
            | Self::FinanceFeesDiscountCreate
            | Self::FinanceFeesDiscountRead
            | Self::FinanceFeesDiscountUpdate
            | Self::FinanceFeesDiscountDelete
            | Self::FinanceFeesAssignCreate
            | Self::FinanceFeesAssignRead
            | Self::FinanceFeesAssignUpdate
            | Self::FinanceFeesAssignClose
            | Self::FinanceFeesInstallmentCreate
            | Self::FinanceFeesInstallmentRead
            | Self::FinanceFeesInstallmentUpdate
            | Self::FinanceFeesInstallmentDelete
            | Self::FinanceFeesInstallmentAssign
            | Self::FinanceDirectFeesInstallmentCreate
            | Self::FinanceDirectFeesInstallmentRead
            | Self::FinanceDirectFeesInstallmentUpdate
            | Self::FinanceDirectFeesInstallmentDelete
            | Self::FinanceDirectFeesInstallmentAssign
            | Self::FinanceDirectFeesInstallmentPay
            | Self::FinanceFeesInvoiceGenerate
            | Self::FinanceFeesInvoiceRead
            | Self::FinanceFeesInvoiceUpdate
            | Self::FinanceFeesInvoiceCancel
            | Self::FinanceFeesInvoiceConfigure
            | Self::FinanceFeesInvoicePrint
            | Self::FinancePaymentCollect
            | Self::FinancePaymentRead
            | Self::FinancePaymentReverse
            | Self::FinancePaymentRefund
            | Self::FinancePaymentMethodCreate
            | Self::FinancePaymentMethodRead
            | Self::FinancePaymentMethodUpdate
            | Self::FinancePaymentMethodDelete
            | Self::FinancePaymentGatewayConfigure
            | Self::FinancePaymentGatewayRead
            | Self::FinancePaymentGatewayUpdate
            | Self::FinancePaymentGatewayDisable
            | Self::FinanceExpenseCreate
            | Self::FinanceExpenseRead
            | Self::FinanceExpenseUpdate
            | Self::FinanceExpenseDelete
            | Self::FinanceExpenseApprove
            | Self::FinanceExpenseHeadCreate
            | Self::FinanceExpenseHeadRead
            | Self::FinanceExpenseHeadUpdate
            | Self::FinanceExpenseHeadDelete
            | Self::FinanceIncomeCreate
            | Self::FinanceIncomeRead
            | Self::FinanceIncomeUpdate
            | Self::FinanceIncomeDelete
            | Self::FinanceIncomeApprove
            | Self::FinanceIncomeHeadCreate
            | Self::FinanceIncomeHeadRead
            | Self::FinanceIncomeHeadUpdate
            | Self::FinanceIncomeHeadDelete
            | Self::FinanceBankOpen
            | Self::FinanceBankRead
            | Self::FinanceBankUpdate
            | Self::FinanceBankClose
            | Self::FinanceBankStatementRecord
            | Self::FinanceBankStatementReverse
            | Self::FinanceBankTransfer
            | Self::FinanceBankSlipGenerate
            | Self::FinanceBankSlipRead
            | Self::FinanceBankSlipApprove
            | Self::FinanceBankSlipReject
            | Self::FinancePayrollPaymentRead
            | Self::FinancePayrollPaymentRecord
            | Self::FinanceWalletCredit
            | Self::FinanceWalletDebit
            | Self::FinanceWalletRead
            | Self::FinanceWalletApprove
            | Self::FinanceWalletReject
            | Self::FinanceFeesCarryForwardExecute
            | Self::FinanceFeesCarryForwardRead
            | Self::FinanceFeesCarryForwardConfigure
            | Self::FinanceDueFeesBlock
            | Self::FinanceDueFeesUnblock
            | Self::FinanceDueFeesRead
            | Self::FinanceFeesReminderConfigure
            | Self::FinanceFeesReminderRead
            | Self::FinanceFeesReminderUpdate
            | Self::FinanceFeesReminderDelete
            | Self::FinanceChartOfAccountCreate
            | Self::FinanceChartOfAccountRead
            | Self::FinanceChartOfAccountUpdate
            | Self::FinanceChartOfAccountDelete
            | Self::FinanceReportFeesCollected
            | Self::FinanceReportFeesOutstanding
            | Self::FinanceReportDailyCollection
            | Self::FinanceReportExpense
            | Self::FinanceReportBankReconciliation
            | Self::FinanceReportWalletLedger
            | Self::FinanceReportRead => CapabilityDomain::Finance,
            Self::HrStaffCreate
            | Self::HrStaffRead
            | Self::HrStaffUpdate
            | Self::HrStaffDelete
            | Self::HrStaffSuspend
            | Self::HrStaffReinstate
            | Self::HrStaffResign
            | Self::HrStaffTerminate
            | Self::HrStaffRetire
            | Self::HrStaffChangeDepartment
            | Self::HrStaffChangeDesignation
            | Self::HrStaffChangeRole
            | Self::HrStaffAssignSubjectTeacher
            | Self::HrStaffAssignClassTeacherCreate
            | Self::HrStaffAssignClassTeacherUpdate
            | Self::HrStaffAssignClassTeacherDelete
            | Self::HrStaffImportBulk
            | Self::HrStaffImportBulkPromote
            | Self::HrStaffImportBulkReject
            | Self::HrStaffDocumentUpload
            | Self::HrStaffDocumentDownload
            | Self::HrDepartmentCreate
            | Self::HrDepartmentRead
            | Self::HrDepartmentUpdate
            | Self::HrDepartmentDelete
            | Self::HrDesignationCreate
            | Self::HrDesignationRead
            | Self::HrDesignationUpdate
            | Self::HrDesignationDelete
            | Self::HrLeaveTypeCreate
            | Self::HrLeaveTypeRead
            | Self::HrLeaveTypeUpdate
            | Self::HrLeaveTypeDelete
            | Self::HrLeaveDefineCreate
            | Self::HrLeaveDefineRead
            | Self::HrLeaveDefineUpdate
            | Self::HrLeaveDefineDelete
            | Self::HrLeaveRequest
            | Self::HrLeaveApprove
            | Self::HrLeaveReject
            | Self::HrLeaveCancel
            | Self::HrLeaveRead
            | Self::HrAttendanceStaffMark
            | Self::HrAttendanceStaffUpdate
            | Self::HrAttendanceStaffDelete
            | Self::HrAttendanceStaffRead
            | Self::HrAttendanceStaffImport
            | Self::HrAttendanceStaffImportPromote
            | Self::HrAttendanceStaffImportReject
            | Self::HrPayrollGenerate
            | Self::HrPayrollUpdate
            | Self::HrPayrollApprove
            | Self::HrPayrollMarkPaid
            | Self::HrPayrollRead
            | Self::HrPayrollEarningAdd
            | Self::HrPayrollEarningUpdate
            | Self::HrPayrollEarningDelete
            | Self::HrPayrollDeductionAdd
            | Self::HrPayrollDeductionUpdate
            | Self::HrPayrollDeductionDelete
            | Self::HrPayrollLeaveDeductionAdd
            | Self::HrPayrollLeaveDeductionUpdate
            | Self::HrPayrollLeaveDeductionDelete
            | Self::HrPayrollPaymentRead
            | Self::HrSalaryTemplateCreate
            | Self::HrSalaryTemplateRead
            | Self::HrSalaryTemplateUpdate
            | Self::HrSalaryTemplateDelete
            | Self::HrHourlyRateSet
            | Self::HrHourlyRateRead
            | Self::HrHourlyRateUpdate
            | Self::HrHourlyRateDelete
            | Self::HrStaffRegistrationFieldCreate
            | Self::HrStaffRegistrationFieldRead
            | Self::HrStaffRegistrationFieldUpdate
            | Self::HrStaffRegistrationFieldDelete
            | Self::HrReportStaffRoster
            | Self::HrReportStaffByDepartment
            | Self::HrReportStaffByDesignation
            | Self::HrReportLeaveUsage
            | Self::HrReportLeaveBalance
            | Self::HrReportAttendanceDaily
            | Self::HrReportAttendanceMonthly
            | Self::HrReportAttendanceByStaff
            | Self::HrReportPayrollRegister
            | Self::HrReportPayrollByStaff
            | Self::HrReportPayrollByDepartment
            | Self::HrReportPayrollTax
            | Self::HrReportSalaryStructure
            | Self::HrReportHourlyEarnings
            | Self::HrReportLeaveDeduction
            | Self::HrReportRead => CapabilityDomain::Hr,
            Self::LibraryBookCreate
            | Self::LibraryBookRead
            | Self::LibraryBookUpdate
            | Self::LibraryBookDelete => CapabilityDomain::Library,
            Self::CommunicationMessageCreate
            | Self::CommunicationMessageRead
            | Self::CommunicationMessageUpdate
            | Self::CommunicationMessageDelete => CapabilityDomain::Communication,
            Self::DocumentsFolderCreate
            | Self::DocumentsFolderRead
            | Self::DocumentsFolderUpdate
            | Self::DocumentsFolderDelete => CapabilityDomain::Documents,
            Self::CmsPageCreate | Self::CmsPageRead | Self::CmsPageUpdate | Self::CmsPageDelete => {
                CapabilityDomain::Cms
            }
            Self::FacilitiesRoomCreate
            | Self::FacilitiesRoomRead
            | Self::FacilitiesRoomUpdate
            | Self::FacilitiesRoomDelete => CapabilityDomain::Facilities,
            Self::EventsCalendarCreate
            | Self::EventsCalendarRead
            | Self::EventsCalendarUpdate
            | Self::EventsCalendarDelete => CapabilityDomain::Events,
            Self::SettingsManage => CapabilityDomain::Settings,
            Self::OperationsManage => CapabilityDomain::Operations,
        }
    }

    /// Returns the aggregate segment of the canonical string form
    /// (e.g. `"School"`, `"User"`, `"Role"`, `"Capability"`).
    #[must_use]
    pub const fn aggregate(self) -> &'static str {
        match self {
            Self::PlatformSchoolCreate
            | Self::PlatformSchoolRead
            | Self::PlatformSchoolUpdate
            | Self::PlatformSchoolDelete => "School",
            Self::PlatformUserCreate
            | Self::PlatformUserRead
            | Self::PlatformUserUpdate
            | Self::PlatformUserDelete => "User",
            Self::RbacRoleCreate
            | Self::RbacRoleRead
            | Self::RbacRoleUpdate
            | Self::RbacRoleDelete
            | Self::RbacRoleManage
            | Self::RbacRoleClone => "Role",
            Self::RbacCapabilityAssign
            | Self::RbacCapabilityRevoke
            | Self::RbacCapabilityRead
            | Self::RbacCapabilityUpdateMetadata
            | Self::RbacBootstrap => "Capability",
            Self::AcademicStudentCreate
            | Self::AcademicStudentRead
            | Self::AcademicStudentUpdate
            | Self::AcademicStudentDelete => "Student",
            Self::AcademicClassCreate
            | Self::AcademicClassRead
            | Self::AcademicClassUpdate
            | Self::AcademicClassDelete => "Class",
            Self::AssessmentExamCreate
            | Self::AssessmentExamRead
            | Self::AssessmentExamUpdate
            | Self::AssessmentExamDelete => "Exam",
            Self::AssessmentExamScheduleCreate
            | Self::AssessmentExamScheduleRead
            | Self::AssessmentExamScheduleUpdate
            | Self::AssessmentExamScheduleDelete => "ExamSchedule",
            Self::AssessmentMarksRegisterCreate
            | Self::AssessmentMarksRegisterRead
            | Self::AssessmentMarksRegisterUpdate
            | Self::AssessmentMarksRegisterDelete => "MarksRegister",
            Self::AssessmentResultStoreCreate
            | Self::AssessmentResultStoreRead
            | Self::AssessmentResultStoreUpdate
            | Self::AssessmentResultStoreDelete => "ResultStore",
            Self::AssessmentReportCardGenerate
            | Self::AssessmentReportCardRead
            | Self::AssessmentReportCardDownload => "ReportCard",
            Self::AssessmentOnlineExamCreate
            | Self::AssessmentOnlineExamRead
            | Self::AssessmentOnlineExamUpdate
            | Self::AssessmentOnlineExamDelete => "OnlineExam",
            Self::AssessmentSeatPlanCreate
            | Self::AssessmentSeatPlanRead
            | Self::AssessmentSeatPlanUpdate
            | Self::AssessmentSeatPlanDelete => "SeatPlan",
            Self::AssessmentAdmitCardCreate
            | Self::AssessmentAdmitCardRead
            | Self::AssessmentAdmitCardUpdate
            | Self::AssessmentAdmitCardDelete => "AdmitCard",
            Self::AttendanceStudentCreate
            | Self::AttendanceStudentRead
            | Self::AttendanceStudentUpdate
            | Self::AttendanceStudentDelete => "Student",
            Self::AttendanceSubjectCreate
            | Self::AttendanceSubjectRead
            | Self::AttendanceSubjectUpdate
            | Self::AttendanceSubjectDelete
            | Self::AttendanceSubjectNotify => "Subject",
            Self::AttendanceStaffCreate
            | Self::AttendanceStaffRead
            | Self::AttendanceStaffUpdate
            | Self::AttendanceStaffDelete => "Staff",
            Self::AttendanceExamCreate
            | Self::AttendanceExamRead
            | Self::AttendanceExamUpdate
            | Self::AttendanceExamDelete => "Exam",
            Self::AttendanceImportCreate
            | Self::AttendanceImportRead
            | Self::AttendanceImportUpdate
            | Self::AttendanceImportDelete => "Import",
            Self::AttendanceBulkMark => "BulkMark",
            Self::AttendanceReportRead => "Report",
            Self::AttendanceNotify => "Notify",
            Self::FinanceInvoiceCreate
            | Self::FinanceInvoiceRead
            | Self::FinanceInvoiceUpdate
            | Self::FinanceInvoiceDelete => "Invoice",
            Self::FinanceFeesGroupCreate
            | Self::FinanceFeesGroupRead
            | Self::FinanceFeesGroupUpdate
            | Self::FinanceFeesGroupDelete => "FeesGroup",
            Self::FinanceFeesTypeCreate
            | Self::FinanceFeesTypeRead
            | Self::FinanceFeesTypeUpdate
            | Self::FinanceFeesTypeDelete => "FeesType",
            Self::FinanceFeesMasterCreate
            | Self::FinanceFeesMasterRead
            | Self::FinanceFeesMasterUpdate
            | Self::FinanceFeesMasterDelete => "FeesMaster",
            Self::FinanceFeesDiscountCreate
            | Self::FinanceFeesDiscountRead
            | Self::FinanceFeesDiscountUpdate
            | Self::FinanceFeesDiscountDelete => "FeesDiscount",
            Self::FinanceFeesAssignCreate
            | Self::FinanceFeesAssignRead
            | Self::FinanceFeesAssignUpdate
            | Self::FinanceFeesAssignClose => "FeesAssign",
            Self::FinanceFeesInstallmentCreate
            | Self::FinanceFeesInstallmentRead
            | Self::FinanceFeesInstallmentUpdate
            | Self::FinanceFeesInstallmentDelete
            | Self::FinanceFeesInstallmentAssign => "FeesInstallment",
            Self::FinanceDirectFeesInstallmentCreate
            | Self::FinanceDirectFeesInstallmentRead
            | Self::FinanceDirectFeesInstallmentUpdate
            | Self::FinanceDirectFeesInstallmentDelete
            | Self::FinanceDirectFeesInstallmentAssign
            | Self::FinanceDirectFeesInstallmentPay => "DirectFeesInstallment",
            Self::FinanceFeesInvoiceGenerate
            | Self::FinanceFeesInvoiceRead
            | Self::FinanceFeesInvoiceUpdate
            | Self::FinanceFeesInvoiceCancel
            | Self::FinanceFeesInvoiceConfigure
            | Self::FinanceFeesInvoicePrint => "FeesInvoice",
            Self::FinancePaymentCollect
            | Self::FinancePaymentRead
            | Self::FinancePaymentReverse
            | Self::FinancePaymentRefund => "Payment",
            Self::FinancePaymentMethodCreate
            | Self::FinancePaymentMethodRead
            | Self::FinancePaymentMethodUpdate
            | Self::FinancePaymentMethodDelete => "PaymentMethod",
            Self::FinancePaymentGatewayConfigure
            | Self::FinancePaymentGatewayRead
            | Self::FinancePaymentGatewayUpdate
            | Self::FinancePaymentGatewayDisable => "PaymentGateway",
            Self::FinanceExpenseCreate
            | Self::FinanceExpenseRead
            | Self::FinanceExpenseUpdate
            | Self::FinanceExpenseDelete
            | Self::FinanceExpenseApprove => "Expense",
            Self::FinanceExpenseHeadCreate
            | Self::FinanceExpenseHeadRead
            | Self::FinanceExpenseHeadUpdate
            | Self::FinanceExpenseHeadDelete => "ExpenseHead",
            Self::FinanceIncomeCreate
            | Self::FinanceIncomeRead
            | Self::FinanceIncomeUpdate
            | Self::FinanceIncomeDelete
            | Self::FinanceIncomeApprove => "Income",
            Self::FinanceIncomeHeadCreate
            | Self::FinanceIncomeHeadRead
            | Self::FinanceIncomeHeadUpdate
            | Self::FinanceIncomeHeadDelete => "IncomeHead",
            Self::FinanceBankOpen
            | Self::FinanceBankRead
            | Self::FinanceBankUpdate
            | Self::FinanceBankClose
            | Self::FinanceBankStatementRecord
            | Self::FinanceBankStatementReverse
            | Self::FinanceBankTransfer => "Bank",
            Self::FinanceBankSlipGenerate
            | Self::FinanceBankSlipRead
            | Self::FinanceBankSlipApprove
            | Self::FinanceBankSlipReject => "BankSlip",
            Self::FinancePayrollPaymentRead | Self::FinancePayrollPaymentRecord => "PayrollPayment",
            Self::FinanceWalletCredit
            | Self::FinanceWalletDebit
            | Self::FinanceWalletRead
            | Self::FinanceWalletApprove
            | Self::FinanceWalletReject => "Wallet",
            Self::FinanceFeesCarryForwardExecute
            | Self::FinanceFeesCarryForwardRead
            | Self::FinanceFeesCarryForwardConfigure => "FeesCarryForward",
            Self::FinanceDueFeesBlock | Self::FinanceDueFeesUnblock | Self::FinanceDueFeesRead => {
                "DueFees"
            }
            Self::FinanceFeesReminderConfigure
            | Self::FinanceFeesReminderRead
            | Self::FinanceFeesReminderUpdate
            | Self::FinanceFeesReminderDelete => "FeesReminder",
            Self::FinanceChartOfAccountCreate
            | Self::FinanceChartOfAccountRead
            | Self::FinanceChartOfAccountUpdate
            | Self::FinanceChartOfAccountDelete => "ChartOfAccount",
            Self::FinanceReportFeesCollected
            | Self::FinanceReportFeesOutstanding
            | Self::FinanceReportDailyCollection
            | Self::FinanceReportExpense
            | Self::FinanceReportBankReconciliation
            | Self::FinanceReportWalletLedger
            | Self::FinanceReportRead => "Report",
            Self::HrStaffCreate
            | Self::HrStaffRead
            | Self::HrStaffUpdate
            | Self::HrStaffDelete
            | Self::HrStaffSuspend
            | Self::HrStaffReinstate
            | Self::HrStaffResign
            | Self::HrStaffTerminate
            | Self::HrStaffRetire
            | Self::HrStaffChangeDepartment
            | Self::HrStaffChangeDesignation
            | Self::HrStaffChangeRole
            | Self::HrStaffAssignSubjectTeacher
            | Self::HrStaffImportBulk
            | Self::HrStaffImportBulkPromote
            | Self::HrStaffImportBulkReject
            | Self::HrStaffDocumentUpload
            | Self::HrStaffDocumentDownload => "Staff",
            Self::HrStaffAssignClassTeacherCreate
            | Self::HrStaffAssignClassTeacherUpdate
            | Self::HrStaffAssignClassTeacherDelete => "AssignClassTeacher",
            Self::HrDepartmentCreate
            | Self::HrDepartmentRead
            | Self::HrDepartmentUpdate
            | Self::HrDepartmentDelete => "Department",
            Self::HrDesignationCreate
            | Self::HrDesignationRead
            | Self::HrDesignationUpdate
            | Self::HrDesignationDelete => "Designation",
            Self::HrLeaveTypeCreate
            | Self::HrLeaveTypeRead
            | Self::HrLeaveTypeUpdate
            | Self::HrLeaveTypeDelete => "LeaveType",
            Self::HrLeaveDefineCreate
            | Self::HrLeaveDefineRead
            | Self::HrLeaveDefineUpdate
            | Self::HrLeaveDefineDelete => "LeaveDefine",
            Self::HrLeaveRequest
            | Self::HrLeaveApprove
            | Self::HrLeaveReject
            | Self::HrLeaveCancel
            | Self::HrLeaveRead => "LeaveRequest",
            Self::HrAttendanceStaffMark
            | Self::HrAttendanceStaffUpdate
            | Self::HrAttendanceStaffDelete
            | Self::HrAttendanceStaffRead => "StaffAttendance",
            Self::HrAttendanceStaffImport
            | Self::HrAttendanceStaffImportPromote
            | Self::HrAttendanceStaffImportReject => "StaffAttendanceImport",
            Self::HrPayrollGenerate
            | Self::HrPayrollUpdate
            | Self::HrPayrollApprove
            | Self::HrPayrollMarkPaid
            | Self::HrPayrollRead
            | Self::HrPayrollPaymentRead => "Payroll",
            Self::HrPayrollEarningAdd
            | Self::HrPayrollEarningUpdate
            | Self::HrPayrollEarningDelete => "PayrollEarning",
            Self::HrPayrollDeductionAdd
            | Self::HrPayrollDeductionUpdate
            | Self::HrPayrollDeductionDelete => "PayrollDeduction",
            Self::HrPayrollLeaveDeductionAdd
            | Self::HrPayrollLeaveDeductionUpdate
            | Self::HrPayrollLeaveDeductionDelete => "LeaveDeduction",
            Self::HrSalaryTemplateCreate
            | Self::HrSalaryTemplateRead
            | Self::HrSalaryTemplateUpdate
            | Self::HrSalaryTemplateDelete => "SalaryTemplate",
            Self::HrHourlyRateSet
            | Self::HrHourlyRateRead
            | Self::HrHourlyRateUpdate
            | Self::HrHourlyRateDelete => "HourlyRate",
            Self::HrStaffRegistrationFieldCreate
            | Self::HrStaffRegistrationFieldRead
            | Self::HrStaffRegistrationFieldUpdate
            | Self::HrStaffRegistrationFieldDelete => "StaffRegistrationField",
            Self::HrReportStaffRoster
            | Self::HrReportStaffByDepartment
            | Self::HrReportStaffByDesignation
            | Self::HrReportLeaveUsage
            | Self::HrReportLeaveBalance
            | Self::HrReportAttendanceDaily
            | Self::HrReportAttendanceMonthly
            | Self::HrReportAttendanceByStaff
            | Self::HrReportPayrollRegister
            | Self::HrReportPayrollByStaff
            | Self::HrReportPayrollByDepartment
            | Self::HrReportPayrollTax
            | Self::HrReportSalaryStructure
            | Self::HrReportHourlyEarnings
            | Self::HrReportLeaveDeduction
            | Self::HrReportRead => "Report",
            Self::LibraryBookCreate
            | Self::LibraryBookRead
            | Self::LibraryBookUpdate
            | Self::LibraryBookDelete => "Book",
            Self::CommunicationMessageCreate
            | Self::CommunicationMessageRead
            | Self::CommunicationMessageUpdate
            | Self::CommunicationMessageDelete => "Message",
            Self::DocumentsFolderCreate
            | Self::DocumentsFolderRead
            | Self::DocumentsFolderUpdate
            | Self::DocumentsFolderDelete => "Folder",
            Self::CmsPageCreate | Self::CmsPageRead | Self::CmsPageUpdate | Self::CmsPageDelete => {
                "Page"
            }
            Self::FacilitiesRoomCreate
            | Self::FacilitiesRoomRead
            | Self::FacilitiesRoomUpdate
            | Self::FacilitiesRoomDelete => "Room",
            Self::EventsCalendarCreate
            | Self::EventsCalendarRead
            | Self::EventsCalendarUpdate
            | Self::EventsCalendarDelete => "Calendar",
            Self::SettingsManage => "Settings",
            Self::OperationsManage => "Operations",
        }
    }

    /// Returns the action segment of the canonical string form
    /// (e.g. `"Create"`, `"Read"`, `"Manage"`, `"Bootstrap"`).
    #[must_use]
    pub const fn action(self) -> &'static str {
        match self {
            Self::PlatformSchoolCreate
            | Self::PlatformUserCreate
            | Self::RbacRoleCreate
            | Self::AcademicStudentCreate
            | Self::AcademicClassCreate
            | Self::AssessmentExamCreate
            | Self::AssessmentExamScheduleCreate
            | Self::AssessmentMarksRegisterCreate
            | Self::AssessmentResultStoreCreate
            | Self::AssessmentOnlineExamCreate
            | Self::AssessmentSeatPlanCreate
            | Self::AssessmentAdmitCardCreate
            | Self::FinanceInvoiceCreate
            | Self::HrStaffCreate
            | Self::HrDepartmentCreate
            | Self::HrDesignationCreate
            | Self::HrLeaveTypeCreate
            | Self::HrLeaveDefineCreate
            | Self::HrStaffAssignClassTeacherCreate
            | Self::HrSalaryTemplateCreate
            | Self::HrStaffRegistrationFieldCreate
            | Self::LibraryBookCreate
            | Self::CommunicationMessageCreate
            | Self::DocumentsFolderCreate
            | Self::CmsPageCreate
            | Self::FacilitiesRoomCreate
            | Self::EventsCalendarCreate
            | Self::AttendanceStudentCreate
            | Self::AttendanceSubjectCreate
            | Self::AttendanceStaffCreate
            | Self::AttendanceExamCreate
            | Self::AttendanceImportCreate => "Create",
            Self::PlatformSchoolRead
            | Self::PlatformUserRead
            | Self::RbacRoleRead
            | Self::RbacCapabilityRead
            | Self::AcademicStudentRead
            | Self::AcademicClassRead
            | Self::AssessmentExamRead
            | Self::AssessmentExamScheduleRead
            | Self::AssessmentMarksRegisterRead
            | Self::AssessmentResultStoreRead
            | Self::AssessmentReportCardRead
            | Self::AssessmentOnlineExamRead
            | Self::AssessmentSeatPlanRead
            | Self::AssessmentAdmitCardRead
            | Self::FinanceInvoiceRead
            | Self::HrStaffRead
            | Self::HrDepartmentRead
            | Self::HrDesignationRead
            | Self::HrLeaveTypeRead
            | Self::HrLeaveDefineRead
            | Self::HrLeaveRead
            | Self::HrAttendanceStaffRead
            | Self::HrPayrollRead
            | Self::HrPayrollPaymentRead
            | Self::HrSalaryTemplateRead
            | Self::HrHourlyRateRead
            | Self::HrStaffRegistrationFieldRead
            | Self::HrReportStaffRoster
            | Self::HrReportStaffByDepartment
            | Self::HrReportStaffByDesignation
            | Self::HrReportLeaveUsage
            | Self::HrReportLeaveBalance
            | Self::HrReportAttendanceDaily
            | Self::HrReportAttendanceMonthly
            | Self::HrReportAttendanceByStaff
            | Self::HrReportPayrollRegister
            | Self::HrReportPayrollByStaff
            | Self::HrReportPayrollByDepartment
            | Self::HrReportPayrollTax
            | Self::HrReportSalaryStructure
            | Self::HrReportHourlyEarnings
            | Self::HrReportLeaveDeduction
            | Self::HrReportRead
            | Self::LibraryBookRead
            | Self::CommunicationMessageRead
            | Self::DocumentsFolderRead
            | Self::CmsPageRead
            | Self::FacilitiesRoomRead
            | Self::EventsCalendarRead
            | Self::AttendanceStudentRead
            | Self::AttendanceSubjectRead
            | Self::AttendanceStaffRead
            | Self::AttendanceExamRead
            | Self::AttendanceImportRead
            | Self::AttendanceReportRead => "Read",
            Self::PlatformSchoolUpdate
            | Self::PlatformUserUpdate
            | Self::RbacRoleUpdate
            | Self::RbacCapabilityUpdateMetadata
            | Self::AcademicStudentUpdate
            | Self::AcademicClassUpdate
            | Self::AssessmentExamUpdate
            | Self::AssessmentExamScheduleUpdate
            | Self::AssessmentMarksRegisterUpdate
            | Self::AssessmentResultStoreUpdate
            | Self::AssessmentOnlineExamUpdate
            | Self::AssessmentSeatPlanUpdate
            | Self::AssessmentAdmitCardUpdate
            | Self::FinanceInvoiceUpdate
            | Self::HrStaffUpdate
            | Self::HrDepartmentUpdate
            | Self::HrDesignationUpdate
            | Self::HrLeaveTypeUpdate
            | Self::HrLeaveDefineUpdate
            | Self::HrAttendanceStaffUpdate
            | Self::HrPayrollUpdate
            | Self::HrPayrollEarningUpdate
            | Self::HrPayrollDeductionUpdate
            | Self::HrPayrollLeaveDeductionUpdate
            | Self::HrSalaryTemplateUpdate
            | Self::HrHourlyRateUpdate
            | Self::HrStaffRegistrationFieldUpdate
            | Self::HrStaffAssignClassTeacherUpdate
            | Self::LibraryBookUpdate
            | Self::CommunicationMessageUpdate
            | Self::DocumentsFolderUpdate
            | Self::CmsPageUpdate
            | Self::FacilitiesRoomUpdate
            | Self::EventsCalendarUpdate
            | Self::AttendanceStudentUpdate
            | Self::AttendanceSubjectUpdate
            | Self::AttendanceStaffUpdate
            | Self::AttendanceExamUpdate
            | Self::AttendanceImportUpdate => "Update",
            Self::PlatformSchoolDelete
            | Self::PlatformUserDelete
            | Self::RbacRoleDelete
            | Self::AcademicStudentDelete
            | Self::AcademicClassDelete
            | Self::AssessmentExamDelete
            | Self::AssessmentExamScheduleDelete
            | Self::AssessmentMarksRegisterDelete
            | Self::AssessmentResultStoreDelete
            | Self::AssessmentOnlineExamDelete
            | Self::AssessmentSeatPlanDelete
            | Self::AssessmentAdmitCardDelete
            | Self::FinanceInvoiceDelete
            | Self::HrStaffDelete
            | Self::HrDepartmentDelete
            | Self::HrDesignationDelete
            | Self::HrLeaveTypeDelete
            | Self::HrLeaveDefineDelete
            | Self::HrAttendanceStaffDelete
            | Self::HrPayrollEarningDelete
            | Self::HrPayrollDeductionDelete
            | Self::HrPayrollLeaveDeductionDelete
            | Self::HrSalaryTemplateDelete
            | Self::HrHourlyRateDelete
            | Self::HrStaffRegistrationFieldDelete
            | Self::HrStaffAssignClassTeacherDelete
            | Self::LibraryBookDelete
            | Self::CommunicationMessageDelete
            | Self::DocumentsFolderDelete
            | Self::CmsPageDelete
            | Self::FacilitiesRoomDelete
            | Self::EventsCalendarDelete
            | Self::AttendanceStudentDelete
            | Self::AttendanceSubjectDelete
            | Self::AttendanceStaffDelete
            | Self::AttendanceExamDelete
            | Self::AttendanceImportDelete => "Delete",
            Self::RbacRoleManage => "Manage",
            Self::RbacRoleClone => "Clone",
            Self::RbacCapabilityAssign => "Assign",
            Self::RbacCapabilityRevoke => "Revoke",
            Self::RbacBootstrap => "Bootstrap",
            Self::AssessmentReportCardGenerate => "Generate",
            Self::AssessmentReportCardDownload => "Download",
            Self::AttendanceBulkMark => "BulkMark",
            Self::AttendanceSubjectNotify => "Notify",
            Self::AttendanceNotify => "Notify",
            Self::SettingsManage => "Manage",
            Self::OperationsManage => "Manage",
            Self::HrStaffSuspend => "Suspend",
            Self::HrStaffReinstate => "Reinstate",
            Self::HrStaffResign => "Resign",
            Self::HrStaffTerminate => "Terminate",
            Self::HrStaffRetire => "Retire",
            Self::HrStaffChangeDepartment => "ChangeDepartment",
            Self::HrStaffChangeDesignation => "ChangeDesignation",
            Self::HrStaffChangeRole => "ChangeRole",
            Self::HrStaffAssignSubjectTeacher => "AssignSubjectTeacher",
            Self::HrStaffImportBulk => "ImportBulk",
            Self::HrStaffImportBulkPromote => "Promote",
            Self::HrStaffImportBulkReject => "Reject",
            Self::HrStaffDocumentUpload => "Upload",
            Self::HrStaffDocumentDownload => "Download",
            Self::HrLeaveRequest => "Request",
            Self::HrLeaveApprove => "Approve",
            Self::HrLeaveReject => "Reject",
            Self::HrLeaveCancel => "Cancel",
            Self::HrAttendanceStaffMark => "Mark",
            Self::HrAttendanceStaffImport => "Import",
            Self::HrAttendanceStaffImportPromote => "Promote",
            Self::HrAttendanceStaffImportReject => "Reject",
            Self::HrPayrollGenerate => "Generate",
            Self::HrPayrollApprove => "Approve",
            Self::HrPayrollMarkPaid => "MarkPaid",
            Self::HrPayrollEarningAdd => "Add",
            Self::HrPayrollDeductionAdd => "Add",
            Self::HrPayrollLeaveDeductionAdd => "Add",
            Self::HrHourlyRateSet => "Set",
            Self::FinanceFeesGroupCreate
            | Self::FinanceFeesTypeCreate
            | Self::FinanceFeesMasterCreate
            | Self::FinanceFeesDiscountCreate
            | Self::FinanceFeesAssignCreate
            | Self::FinanceFeesInstallmentCreate
            | Self::FinanceDirectFeesInstallmentCreate
            | Self::FinancePaymentMethodCreate
            | Self::FinanceExpenseCreate
            | Self::FinanceExpenseHeadCreate
            | Self::FinanceIncomeCreate
            | Self::FinanceIncomeHeadCreate
            | Self::FinanceChartOfAccountCreate => "Create",
            Self::FinanceFeesGroupRead
            | Self::FinanceFeesTypeRead
            | Self::FinanceFeesMasterRead
            | Self::FinanceFeesDiscountRead
            | Self::FinanceFeesAssignRead
            | Self::FinanceFeesInstallmentRead
            | Self::FinanceDirectFeesInstallmentRead
            | Self::FinanceFeesInvoiceRead
            | Self::FinancePaymentRead
            | Self::FinancePaymentMethodRead
            | Self::FinancePaymentGatewayRead
            | Self::FinanceExpenseRead
            | Self::FinanceExpenseHeadRead
            | Self::FinanceIncomeRead
            | Self::FinanceIncomeHeadRead
            | Self::FinanceBankRead
            | Self::FinanceBankSlipRead
            | Self::FinancePayrollPaymentRead
            | Self::FinanceWalletRead
            | Self::FinanceFeesCarryForwardRead
            | Self::FinanceDueFeesRead
            | Self::FinanceFeesReminderRead
            | Self::FinanceChartOfAccountRead
            | Self::FinanceReportFeesCollected
            | Self::FinanceReportFeesOutstanding
            | Self::FinanceReportDailyCollection
            | Self::FinanceReportExpense
            | Self::FinanceReportBankReconciliation
            | Self::FinanceReportWalletLedger
            | Self::FinanceReportRead => "Read",
            Self::FinanceFeesGroupUpdate
            | Self::FinanceFeesTypeUpdate
            | Self::FinanceFeesMasterUpdate
            | Self::FinanceFeesDiscountUpdate
            | Self::FinanceFeesAssignUpdate
            | Self::FinanceFeesInstallmentUpdate
            | Self::FinanceDirectFeesInstallmentUpdate
            | Self::FinanceFeesInvoiceUpdate
            | Self::FinancePaymentMethodUpdate
            | Self::FinancePaymentGatewayUpdate
            | Self::FinanceExpenseUpdate
            | Self::FinanceExpenseHeadUpdate
            | Self::FinanceIncomeUpdate
            | Self::FinanceIncomeHeadUpdate
            | Self::FinanceBankUpdate
            | Self::FinanceFeesReminderUpdate
            | Self::FinanceChartOfAccountUpdate => "Update",
            Self::FinanceFeesGroupDelete
            | Self::FinanceFeesTypeDelete
            | Self::FinanceFeesMasterDelete
            | Self::FinanceFeesDiscountDelete
            | Self::FinanceFeesInstallmentDelete
            | Self::FinanceDirectFeesInstallmentDelete
            | Self::FinancePaymentMethodDelete
            | Self::FinanceExpenseDelete
            | Self::FinanceExpenseHeadDelete
            | Self::FinanceIncomeDelete
            | Self::FinanceIncomeHeadDelete
            | Self::FinanceFeesReminderDelete
            | Self::FinanceChartOfAccountDelete => "Delete",
            Self::FinanceFeesInvoiceGenerate => "Generate",
            Self::FinanceFeesInvoiceCancel => "Cancel",
            Self::FinanceFeesInvoiceConfigure => "Configure",
            Self::FinanceFeesInvoicePrint => "Print",
            Self::FinancePaymentCollect => "Collect",
            Self::FinancePaymentReverse => "Reverse",
            Self::FinancePaymentRefund => "Refund",
            Self::FinancePaymentGatewayConfigure => "Configure",
            Self::FinancePaymentGatewayDisable => "Disable",
            Self::FinanceExpenseApprove => "Approve",
            Self::FinanceIncomeApprove => "Approve",
            Self::FinanceBankOpen => "Open",
            Self::FinanceBankClose => "Close",
            Self::FinanceBankStatementRecord => "Record",
            Self::FinanceBankStatementReverse => "Reverse",
            Self::FinanceBankTransfer => "Transfer",
            Self::FinanceBankSlipGenerate => "Generate",
            Self::FinanceBankSlipApprove => "Approve",
            Self::FinanceBankSlipReject => "Reject",
            Self::FinancePayrollPaymentRecord => "Record",
            Self::FinanceWalletCredit => "Credit",
            Self::FinanceWalletDebit => "Debit",
            Self::FinanceWalletApprove => "Approve",
            Self::FinanceWalletReject => "Reject",
            Self::FinanceFeesCarryForwardExecute => "Execute",
            Self::FinanceFeesCarryForwardConfigure => "Configure",
            Self::FinanceDueFeesBlock => "Block",
            Self::FinanceDueFeesUnblock => "Unblock",
            Self::FinanceFeesReminderConfigure => "Configure",
            Self::FinanceFeesAssignClose => "Close",
            Self::FinanceDirectFeesInstallmentAssign => "Assign",
            Self::FinanceDirectFeesInstallmentPay => "Pay",
            Self::FinanceFeesInstallmentAssign => "Assign",
        }
    }

    /// Returns the canonical dotted string form (e.g.
    /// `"Platform.School.Create"`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PlatformSchoolCreate => "Platform.School.Create",
            Self::PlatformSchoolRead => "Platform.School.Read",
            Self::PlatformSchoolUpdate => "Platform.School.Update",
            Self::PlatformSchoolDelete => "Platform.School.Delete",
            Self::PlatformUserCreate => "Platform.User.Create",
            Self::PlatformUserRead => "Platform.User.Read",
            Self::PlatformUserUpdate => "Platform.User.Update",
            Self::PlatformUserDelete => "Platform.User.Delete",
            Self::RbacRoleCreate => "Rbac.Role.Create",
            Self::RbacRoleRead => "Rbac.Role.Read",
            Self::RbacRoleUpdate => "Rbac.Role.Update",
            Self::RbacRoleDelete => "Rbac.Role.Delete",
            Self::RbacRoleManage => "Rbac.Role.Manage",
            Self::RbacRoleClone => "Rbac.Role.Clone",
            Self::RbacCapabilityAssign => "Rbac.Capability.Assign",
            Self::RbacCapabilityRevoke => "Rbac.Capability.Revoke",
            Self::RbacCapabilityRead => "Rbac.Capability.Read",
            Self::RbacCapabilityUpdateMetadata => "Rbac.Capability.UpdateMetadata",
            Self::RbacBootstrap => "Rbac.Bootstrap",
            Self::AcademicStudentCreate => "Academic.Student.Create",
            Self::AcademicStudentRead => "Academic.Student.Read",
            Self::AcademicStudentUpdate => "Academic.Student.Update",
            Self::AcademicStudentDelete => "Academic.Student.Delete",
            Self::AcademicClassCreate => "Academic.Class.Create",
            Self::AcademicClassRead => "Academic.Class.Read",
            Self::AcademicClassUpdate => "Academic.Class.Update",
            Self::AcademicClassDelete => "Academic.Class.Delete",
            Self::AssessmentExamCreate => "Assessment.Exam.Create",
            Self::AssessmentExamRead => "Assessment.Exam.Read",
            Self::AssessmentExamUpdate => "Assessment.Exam.Update",
            Self::AssessmentExamDelete => "Assessment.Exam.Delete",
            Self::AssessmentExamScheduleCreate => "Assessment.ExamSchedule.Create",
            Self::AssessmentExamScheduleRead => "Assessment.ExamSchedule.Read",
            Self::AssessmentExamScheduleUpdate => "Assessment.ExamSchedule.Update",
            Self::AssessmentExamScheduleDelete => "Assessment.ExamSchedule.Delete",
            Self::AssessmentMarksRegisterCreate => "Assessment.MarksRegister.Create",
            Self::AssessmentMarksRegisterRead => "Assessment.MarksRegister.Read",
            Self::AssessmentMarksRegisterUpdate => "Assessment.MarksRegister.Update",
            Self::AssessmentMarksRegisterDelete => "Assessment.MarksRegister.Delete",
            Self::AssessmentResultStoreCreate => "Assessment.ResultStore.Create",
            Self::AssessmentResultStoreRead => "Assessment.ResultStore.Read",
            Self::AssessmentResultStoreUpdate => "Assessment.ResultStore.Update",
            Self::AssessmentResultStoreDelete => "Assessment.ResultStore.Delete",
            Self::AssessmentReportCardGenerate => "Assessment.ReportCard.Generate",
            Self::AssessmentReportCardRead => "Assessment.ReportCard.Read",
            Self::AssessmentReportCardDownload => "Assessment.ReportCard.Download",
            Self::AssessmentOnlineExamCreate => "Assessment.OnlineExam.Create",
            Self::AssessmentOnlineExamRead => "Assessment.OnlineExam.Read",
            Self::AssessmentOnlineExamUpdate => "Assessment.OnlineExam.Update",
            Self::AssessmentOnlineExamDelete => "Assessment.OnlineExam.Delete",
            Self::AssessmentSeatPlanCreate => "Assessment.SeatPlan.Create",
            Self::AssessmentSeatPlanRead => "Assessment.SeatPlan.Read",
            Self::AssessmentSeatPlanUpdate => "Assessment.SeatPlan.Update",
            Self::AssessmentSeatPlanDelete => "Assessment.SeatPlan.Delete",
            Self::AssessmentAdmitCardCreate => "Assessment.AdmitCard.Create",
            Self::AssessmentAdmitCardRead => "Assessment.AdmitCard.Read",
            Self::AssessmentAdmitCardUpdate => "Assessment.AdmitCard.Update",
            Self::AssessmentAdmitCardDelete => "Assessment.AdmitCard.Delete",
            Self::FinanceInvoiceCreate => "Finance.Invoice.Create",
            Self::FinanceInvoiceRead => "Finance.Invoice.Read",
            Self::FinanceInvoiceUpdate => "Finance.Invoice.Update",
            Self::FinanceInvoiceDelete => "Finance.Invoice.Delete",
            Self::HrStaffCreate => "Hr.Staff.Create",
            Self::HrStaffRead => "Hr.Staff.Read",
            Self::HrStaffUpdate => "Hr.Staff.Update",
            Self::HrStaffDelete => "Hr.Staff.Delete",
            Self::HrStaffSuspend => "Hr.Staff.Suspend",
            Self::HrStaffReinstate => "Hr.Staff.Reinstate",
            Self::HrStaffResign => "Hr.Staff.Resign",
            Self::HrStaffTerminate => "Hr.Staff.Terminate",
            Self::HrStaffRetire => "Hr.Staff.Retire",
            Self::HrStaffChangeDepartment => "Hr.Staff.ChangeDepartment",
            Self::HrStaffChangeDesignation => "Hr.Staff.ChangeDesignation",
            Self::HrStaffChangeRole => "Hr.Staff.ChangeRole",
            Self::HrStaffAssignSubjectTeacher => "Hr.Staff.AssignSubjectTeacher",
            Self::HrStaffAssignClassTeacherCreate => "Hr.AssignClassTeacher.Create",
            Self::HrStaffAssignClassTeacherUpdate => "Hr.AssignClassTeacher.Update",
            Self::HrStaffAssignClassTeacherDelete => "Hr.AssignClassTeacher.Delete",
            Self::HrStaffImportBulk => "Hr.Staff.ImportBulk",
            Self::HrStaffImportBulkPromote => "Hr.Staff.ImportBulk.Promote",
            Self::HrStaffImportBulkReject => "Hr.Staff.ImportBulk.Reject",
            Self::HrStaffDocumentUpload => "Hr.Staff.Document.Upload",
            Self::HrStaffDocumentDownload => "Hr.Staff.Document.Download",
            Self::HrDepartmentCreate => "Hr.Department.Create",
            Self::HrDepartmentRead => "Hr.Department.Read",
            Self::HrDepartmentUpdate => "Hr.Department.Update",
            Self::HrDepartmentDelete => "Hr.Department.Delete",
            Self::HrDesignationCreate => "Hr.Designation.Create",
            Self::HrDesignationRead => "Hr.Designation.Read",
            Self::HrDesignationUpdate => "Hr.Designation.Update",
            Self::HrDesignationDelete => "Hr.Designation.Delete",
            Self::HrLeaveTypeCreate => "Hr.LeaveType.Create",
            Self::HrLeaveTypeRead => "Hr.LeaveType.Read",
            Self::HrLeaveTypeUpdate => "Hr.LeaveType.Update",
            Self::HrLeaveTypeDelete => "Hr.LeaveType.Delete",
            Self::HrLeaveDefineCreate => "Hr.LeaveDefine.Create",
            Self::HrLeaveDefineRead => "Hr.LeaveDefine.Read",
            Self::HrLeaveDefineUpdate => "Hr.LeaveDefine.Update",
            Self::HrLeaveDefineDelete => "Hr.LeaveDefine.Delete",
            Self::HrLeaveRequest => "Hr.Leave.Request",
            Self::HrLeaveApprove => "Hr.Leave.Approve",
            Self::HrLeaveReject => "Hr.Leave.Reject",
            Self::HrLeaveCancel => "Hr.Leave.Cancel",
            Self::HrLeaveRead => "Hr.Leave.Read",
            Self::HrAttendanceStaffMark => "Hr.Attendance.Staff.Mark",
            Self::HrAttendanceStaffUpdate => "Hr.Attendance.Staff.Update",
            Self::HrAttendanceStaffDelete => "Hr.Attendance.Staff.Delete",
            Self::HrAttendanceStaffRead => "Hr.Attendance.Staff.Read",
            Self::HrAttendanceStaffImport => "Hr.Attendance.Staff.Import",
            Self::HrAttendanceStaffImportPromote => "Hr.Attendance.Staff.Import.Promote",
            Self::HrAttendanceStaffImportReject => "Hr.Attendance.Staff.Import.Reject",
            Self::HrPayrollGenerate => "Hr.Payroll.Generate",
            Self::HrPayrollUpdate => "Hr.Payroll.Update",
            Self::HrPayrollApprove => "Hr.Payroll.Approve",
            Self::HrPayrollMarkPaid => "Hr.Payroll.MarkPaid",
            Self::HrPayrollRead => "Hr.Payroll.Read",
            Self::HrPayrollEarningAdd => "Hr.Payroll.Earning.Add",
            Self::HrPayrollEarningUpdate => "Hr.Payroll.Earning.Update",
            Self::HrPayrollEarningDelete => "Hr.Payroll.Earning.Delete",
            Self::HrPayrollDeductionAdd => "Hr.Payroll.Deduction.Add",
            Self::HrPayrollDeductionUpdate => "Hr.Payroll.Deduction.Update",
            Self::HrPayrollDeductionDelete => "Hr.Payroll.Deduction.Delete",
            Self::HrPayrollLeaveDeductionAdd => "Hr.Payroll.LeaveDeduction.Add",
            Self::HrPayrollLeaveDeductionUpdate => "Hr.Payroll.LeaveDeduction.Update",
            Self::HrPayrollLeaveDeductionDelete => "Hr.Payroll.LeaveDeduction.Delete",
            Self::HrPayrollPaymentRead => "Hr.PayrollPayment.Read",
            Self::HrSalaryTemplateCreate => "Hr.SalaryTemplate.Create",
            Self::HrSalaryTemplateRead => "Hr.SalaryTemplate.Read",
            Self::HrSalaryTemplateUpdate => "Hr.SalaryTemplate.Update",
            Self::HrSalaryTemplateDelete => "Hr.SalaryTemplate.Delete",
            Self::HrHourlyRateSet => "Hr.HourlyRate.Set",
            Self::FinanceFeesGroupCreate => "Finance.FeesGroup.Create",
            Self::FinanceFeesGroupRead => "Finance.FeesGroup.Read",
            Self::FinanceFeesGroupUpdate => "Finance.FeesGroup.Update",
            Self::FinanceFeesGroupDelete => "Finance.FeesGroup.Delete",
            Self::FinanceFeesTypeCreate => "Finance.FeesType.Create",
            Self::FinanceFeesTypeRead => "Finance.FeesType.Read",
            Self::FinanceFeesTypeUpdate => "Finance.FeesType.Update",
            Self::FinanceFeesTypeDelete => "Finance.FeesType.Delete",
            Self::FinanceFeesMasterCreate => "Finance.FeesMaster.Create",
            Self::FinanceFeesMasterRead => "Finance.FeesMaster.Read",
            Self::FinanceFeesMasterUpdate => "Finance.FeesMaster.Update",
            Self::FinanceFeesMasterDelete => "Finance.FeesMaster.Delete",
            Self::FinanceFeesDiscountCreate => "Finance.FeesDiscount.Create",
            Self::FinanceFeesDiscountRead => "Finance.FeesDiscount.Read",
            Self::FinanceFeesDiscountUpdate => "Finance.FeesDiscount.Update",
            Self::FinanceFeesDiscountDelete => "Finance.FeesDiscount.Delete",
            Self::FinanceFeesAssignCreate => "Finance.FeesAssign.Create",
            Self::FinanceFeesAssignRead => "Finance.FeesAssign.Read",
            Self::FinanceFeesAssignUpdate => "Finance.FeesAssign.Update",
            Self::FinanceFeesAssignClose => "Finance.FeesAssign.Close",
            Self::FinanceFeesInstallmentCreate => "Finance.FeesInstallment.Create",
            Self::FinanceFeesInstallmentRead => "Finance.FeesInstallment.Read",
            Self::FinanceFeesInstallmentUpdate => "Finance.FeesInstallment.Update",
            Self::FinanceFeesInstallmentDelete => "Finance.FeesInstallment.Delete",
            Self::FinanceFeesInstallmentAssign => "Finance.FeesInstallment.Assign",
            Self::FinanceDirectFeesInstallmentCreate => "Finance.DirectFeesInstallment.Create",
            Self::FinanceDirectFeesInstallmentRead => "Finance.DirectFeesInstallment.Read",
            Self::FinanceDirectFeesInstallmentUpdate => "Finance.DirectFeesInstallment.Update",
            Self::FinanceDirectFeesInstallmentDelete => "Finance.DirectFeesInstallment.Delete",
            Self::FinanceDirectFeesInstallmentAssign => "Finance.DirectFeesInstallment.Assign",
            Self::FinanceDirectFeesInstallmentPay => "Finance.DirectFeesInstallment.Pay",
            Self::FinanceFeesInvoiceGenerate => "Finance.FeesInvoice.Generate",
            Self::FinanceFeesInvoiceRead => "Finance.FeesInvoice.Read",
            Self::FinanceFeesInvoiceUpdate => "Finance.FeesInvoice.Update",
            Self::FinanceFeesInvoiceCancel => "Finance.FeesInvoice.Cancel",
            Self::FinanceFeesInvoiceConfigure => "Finance.FeesInvoice.Configure",
            Self::FinanceFeesInvoicePrint => "Finance.FeesInvoice.Print",
            Self::FinancePaymentCollect => "Finance.Payment.Collect",
            Self::FinancePaymentRead => "Finance.Payment.Read",
            Self::FinancePaymentReverse => "Finance.Payment.Reverse",
            Self::FinancePaymentRefund => "Finance.Payment.Refund",
            Self::FinancePaymentMethodCreate => "Finance.PaymentMethod.Create",
            Self::FinancePaymentMethodRead => "Finance.PaymentMethod.Read",
            Self::FinancePaymentMethodUpdate => "Finance.PaymentMethod.Update",
            Self::FinancePaymentMethodDelete => "Finance.PaymentMethod.Delete",
            Self::FinancePaymentGatewayConfigure => "Finance.PaymentGateway.Configure",
            Self::FinancePaymentGatewayRead => "Finance.PaymentGateway.Read",
            Self::FinancePaymentGatewayUpdate => "Finance.PaymentGateway.Update",
            Self::FinancePaymentGatewayDisable => "Finance.PaymentGateway.Disable",
            Self::FinanceExpenseCreate => "Finance.Expense.Create",
            Self::FinanceExpenseRead => "Finance.Expense.Read",
            Self::FinanceExpenseUpdate => "Finance.Expense.Update",
            Self::FinanceExpenseDelete => "Finance.Expense.Delete",
            Self::FinanceExpenseApprove => "Finance.Expense.Approve",
            Self::FinanceExpenseHeadCreate => "Finance.ExpenseHead.Create",
            Self::FinanceExpenseHeadRead => "Finance.ExpenseHead.Read",
            Self::FinanceExpenseHeadUpdate => "Finance.ExpenseHead.Update",
            Self::FinanceExpenseHeadDelete => "Finance.ExpenseHead.Delete",
            Self::FinanceIncomeCreate => "Finance.Income.Create",
            Self::FinanceIncomeRead => "Finance.Income.Read",
            Self::FinanceIncomeUpdate => "Finance.Income.Update",
            Self::FinanceIncomeDelete => "Finance.Income.Delete",
            Self::FinanceIncomeApprove => "Finance.Income.Approve",
            Self::FinanceIncomeHeadCreate => "Finance.IncomeHead.Create",
            Self::FinanceIncomeHeadRead => "Finance.IncomeHead.Read",
            Self::FinanceIncomeHeadUpdate => "Finance.IncomeHead.Update",
            Self::FinanceIncomeHeadDelete => "Finance.IncomeHead.Delete",
            Self::FinanceBankOpen => "Finance.Bank.Open",
            Self::FinanceBankRead => "Finance.Bank.Read",
            Self::FinanceBankUpdate => "Finance.Bank.Update",
            Self::FinanceBankClose => "Finance.Bank.Close",
            Self::FinanceBankStatementRecord => "Finance.Bank.Statement.Record",
            Self::FinanceBankStatementReverse => "Finance.Bank.Statement.Reverse",
            Self::FinanceBankTransfer => "Finance.Bank.Transfer",
            Self::FinanceBankSlipGenerate => "Finance.BankSlip.Generate",
            Self::FinanceBankSlipRead => "Finance.BankSlip.Read",
            Self::FinanceBankSlipApprove => "Finance.BankSlip.Approve",
            Self::FinanceBankSlipReject => "Finance.BankSlip.Reject",
            Self::FinancePayrollPaymentRead => "Finance.PayrollPayment.Read",
            Self::FinancePayrollPaymentRecord => "Finance.PayrollPayment.Record",
            Self::FinanceWalletCredit => "Finance.Wallet.Credit",
            Self::FinanceWalletDebit => "Finance.Wallet.Debit",
            Self::FinanceWalletRead => "Finance.Wallet.Read",
            Self::FinanceWalletApprove => "Finance.Wallet.Approve",
            Self::FinanceWalletReject => "Finance.Wallet.Reject",
            Self::FinanceFeesCarryForwardExecute => "Finance.FeesCarryForward.Execute",
            Self::FinanceFeesCarryForwardRead => "Finance.FeesCarryForward.Read",
            Self::FinanceFeesCarryForwardConfigure => "Finance.FeesCarryForward.Configure",
            Self::FinanceDueFeesBlock => "Finance.DueFees.Block",
            Self::FinanceDueFeesUnblock => "Finance.DueFees.Unblock",
            Self::FinanceDueFeesRead => "Finance.DueFees.Read",
            Self::FinanceFeesReminderConfigure => "Finance.FeesReminder.Configure",
            Self::FinanceFeesReminderRead => "Finance.FeesReminder.Read",
            Self::FinanceFeesReminderUpdate => "Finance.FeesReminder.Update",
            Self::FinanceFeesReminderDelete => "Finance.FeesReminder.Delete",
            Self::FinanceChartOfAccountCreate => "Finance.ChartOfAccount.Create",
            Self::FinanceChartOfAccountRead => "Finance.ChartOfAccount.Read",
            Self::FinanceChartOfAccountUpdate => "Finance.ChartOfAccount.Update",
            Self::FinanceChartOfAccountDelete => "Finance.ChartOfAccount.Delete",
            Self::FinanceReportFeesCollected => "Finance.Report.Read.FeesCollected",
            Self::FinanceReportFeesOutstanding => "Finance.Report.Read.FeesOutstanding",
            Self::FinanceReportDailyCollection => "Finance.Report.Read.DailyCollection",
            Self::FinanceReportExpense => "Finance.Report.Read.Expense",
            Self::FinanceReportBankReconciliation => "Finance.Report.Read.BankReconciliation",
            Self::FinanceReportWalletLedger => "Finance.Report.Read.WalletLedger",
            Self::FinanceReportRead => "Finance.Report.Read.Finance",
            Self::HrHourlyRateRead => "Hr.HourlyRate.Read",
            Self::HrHourlyRateUpdate => "Hr.HourlyRate.Update",
            Self::HrHourlyRateDelete => "Hr.HourlyRate.Delete",
            Self::HrStaffRegistrationFieldCreate => "Hr.StaffRegistrationField.Create",
            Self::HrStaffRegistrationFieldRead => "Hr.StaffRegistrationField.Read",
            Self::HrStaffRegistrationFieldUpdate => "Hr.StaffRegistrationField.Update",
            Self::HrStaffRegistrationFieldDelete => "Hr.StaffRegistrationField.Delete",
            Self::HrReportStaffRoster => "Hr.Report.Read.StaffRoster",
            Self::HrReportStaffByDepartment => "Hr.Report.Read.StaffByDepartment",
            Self::HrReportStaffByDesignation => "Hr.Report.Read.StaffByDesignation",
            Self::HrReportLeaveUsage => "Hr.Report.Read.LeaveUsage",
            Self::HrReportLeaveBalance => "Hr.Report.Read.LeaveBalance",
            Self::HrReportAttendanceDaily => "Hr.Report.Read.AttendanceDaily",
            Self::HrReportAttendanceMonthly => "Hr.Report.Read.AttendanceMonthly",
            Self::HrReportAttendanceByStaff => "Hr.Report.Read.AttendanceByStaff",
            Self::HrReportPayrollRegister => "Hr.Report.Read.PayrollRegister",
            Self::HrReportPayrollByStaff => "Hr.Report.Read.PayrollByStaff",
            Self::HrReportPayrollByDepartment => "Hr.Report.Read.PayrollByDepartment",
            Self::HrReportPayrollTax => "Hr.Report.Read.PayrollTax",
            Self::HrReportSalaryStructure => "Hr.Report.Read.SalaryStructure",
            Self::HrReportHourlyEarnings => "Hr.Report.Read.HourlyEarnings",
            Self::HrReportLeaveDeduction => "Hr.Report.Read.LeaveDeduction",
            Self::HrReportRead => "Hr.Report.Read.HR",
            Self::LibraryBookCreate => "Library.Book.Create",
            Self::LibraryBookRead => "Library.Book.Read",
            Self::LibraryBookUpdate => "Library.Book.Update",
            Self::LibraryBookDelete => "Library.Book.Delete",
            Self::CommunicationMessageCreate => "Communication.Message.Create",
            Self::CommunicationMessageRead => "Communication.Message.Read",
            Self::CommunicationMessageUpdate => "Communication.Message.Update",
            Self::CommunicationMessageDelete => "Communication.Message.Delete",
            Self::DocumentsFolderCreate => "Documents.Folder.Create",
            Self::DocumentsFolderRead => "Documents.Folder.Read",
            Self::DocumentsFolderUpdate => "Documents.Folder.Update",
            Self::DocumentsFolderDelete => "Documents.Folder.Delete",
            Self::CmsPageCreate => "Cms.Page.Create",
            Self::CmsPageRead => "Cms.Page.Read",
            Self::CmsPageUpdate => "Cms.Page.Update",
            Self::CmsPageDelete => "Cms.Page.Delete",
            Self::FacilitiesRoomCreate => "Facilities.Room.Create",
            Self::FacilitiesRoomRead => "Facilities.Room.Read",
            Self::FacilitiesRoomUpdate => "Facilities.Room.Update",
            Self::FacilitiesRoomDelete => "Facilities.Room.Delete",
            Self::EventsCalendarCreate => "Events.Calendar.Create",
            Self::EventsCalendarRead => "Events.Calendar.Read",
            Self::EventsCalendarUpdate => "Events.Calendar.Update",
            Self::EventsCalendarDelete => "Events.Calendar.Delete",
            Self::AttendanceStudentCreate => "Attendance.Student.Create",
            Self::AttendanceStudentRead => "Attendance.Student.Read",
            Self::AttendanceStudentUpdate => "Attendance.Student.Update",
            Self::AttendanceStudentDelete => "Attendance.Student.Delete",
            Self::AttendanceSubjectCreate => "Attendance.Subject.Create",
            Self::AttendanceSubjectRead => "Attendance.Subject.Read",
            Self::AttendanceSubjectUpdate => "Attendance.Subject.Update",
            Self::AttendanceSubjectDelete => "Attendance.Subject.Delete",
            Self::AttendanceSubjectNotify => "Attendance.Subject.Notify",
            Self::AttendanceStaffCreate => "Attendance.Staff.Create",
            Self::AttendanceStaffRead => "Attendance.Staff.Read",
            Self::AttendanceStaffUpdate => "Attendance.Staff.Update",
            Self::AttendanceStaffDelete => "Attendance.Staff.Delete",
            Self::AttendanceExamCreate => "Attendance.Exam.Create",
            Self::AttendanceExamRead => "Attendance.Exam.Read",
            Self::AttendanceExamUpdate => "Attendance.Exam.Update",
            Self::AttendanceExamDelete => "Attendance.Exam.Delete",
            Self::AttendanceImportCreate => "Attendance.Import.Create",
            Self::AttendanceImportRead => "Attendance.Import.Read",
            Self::AttendanceImportUpdate => "Attendance.Import.Update",
            Self::AttendanceImportDelete => "Attendance.Import.Delete",
            Self::AttendanceBulkMark => "Attendance.BulkMark.BulkMark",
            Self::AttendanceReportRead => "Attendance.Report.Read",
            Self::AttendanceNotify => "Attendance.Notify.Notify",
            Self::SettingsManage => "Settings.Manage",
            Self::OperationsManage => "Operations.Manage",
        }
    }

    /// Returns the full set of variants defined by the catalog. Used
    /// by the [`DefaultRoleCatalog`](crate::services::DefaultRoleCatalog)
    /// and by tests that iterate the full enum.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::PlatformSchoolCreate,
            Self::PlatformSchoolRead,
            Self::PlatformSchoolUpdate,
            Self::PlatformSchoolDelete,
            Self::PlatformUserCreate,
            Self::PlatformUserRead,
            Self::PlatformUserUpdate,
            Self::PlatformUserDelete,
            Self::RbacRoleCreate,
            Self::RbacRoleRead,
            Self::RbacRoleUpdate,
            Self::RbacRoleDelete,
            Self::RbacRoleManage,
            Self::RbacRoleClone,
            Self::RbacCapabilityAssign,
            Self::RbacCapabilityRevoke,
            Self::RbacCapabilityRead,
            Self::RbacCapabilityUpdateMetadata,
            Self::RbacBootstrap,
            Self::AcademicStudentCreate,
            Self::AcademicStudentRead,
            Self::AcademicStudentUpdate,
            Self::AcademicStudentDelete,
            Self::AcademicClassCreate,
            Self::AcademicClassRead,
            Self::AcademicClassUpdate,
            Self::AcademicClassDelete,
            Self::AssessmentExamCreate,
            Self::AssessmentExamRead,
            Self::AssessmentExamUpdate,
            Self::AssessmentExamDelete,
            Self::AssessmentExamScheduleCreate,
            Self::AssessmentExamScheduleRead,
            Self::AssessmentExamScheduleUpdate,
            Self::AssessmentExamScheduleDelete,
            Self::AssessmentMarksRegisterCreate,
            Self::AssessmentMarksRegisterRead,
            Self::AssessmentMarksRegisterUpdate,
            Self::AssessmentMarksRegisterDelete,
            Self::AssessmentResultStoreCreate,
            Self::AssessmentResultStoreRead,
            Self::AssessmentResultStoreUpdate,
            Self::AssessmentResultStoreDelete,
            Self::AssessmentReportCardGenerate,
            Self::AssessmentReportCardRead,
            Self::AssessmentReportCardDownload,
            Self::AssessmentOnlineExamCreate,
            Self::AssessmentOnlineExamRead,
            Self::AssessmentOnlineExamUpdate,
            Self::AssessmentOnlineExamDelete,
            Self::AssessmentSeatPlanCreate,
            Self::AssessmentSeatPlanRead,
            Self::AssessmentSeatPlanUpdate,
            Self::AssessmentSeatPlanDelete,
            Self::AssessmentAdmitCardCreate,
            Self::AssessmentAdmitCardRead,
            Self::AssessmentAdmitCardUpdate,
            Self::AssessmentAdmitCardDelete,
            Self::FinanceInvoiceCreate,
            Self::FinanceInvoiceRead,
            Self::FinanceInvoiceUpdate,
            Self::FinanceInvoiceDelete,
            Self::HrStaffCreate,
            Self::HrStaffRead,
            Self::HrStaffUpdate,
            Self::HrStaffDelete,
            Self::HrStaffSuspend,
            Self::HrStaffReinstate,
            Self::HrStaffResign,
            Self::HrStaffTerminate,
            Self::HrStaffRetire,
            Self::HrStaffChangeDepartment,
            Self::HrStaffChangeDesignation,
            Self::HrStaffChangeRole,
            Self::HrStaffAssignSubjectTeacher,
            Self::HrStaffAssignClassTeacherCreate,
            Self::HrStaffAssignClassTeacherUpdate,
            Self::HrStaffAssignClassTeacherDelete,
            Self::HrStaffImportBulk,
            Self::HrStaffImportBulkPromote,
            Self::HrStaffImportBulkReject,
            Self::HrStaffDocumentUpload,
            Self::HrStaffDocumentDownload,
            Self::HrDepartmentCreate,
            Self::HrDepartmentRead,
            Self::HrDepartmentUpdate,
            Self::HrDepartmentDelete,
            Self::HrDesignationCreate,
            Self::HrDesignationRead,
            Self::HrDesignationUpdate,
            Self::HrDesignationDelete,
            Self::HrLeaveTypeCreate,
            Self::HrLeaveTypeRead,
            Self::HrLeaveTypeUpdate,
            Self::HrLeaveTypeDelete,
            Self::HrLeaveDefineCreate,
            Self::HrLeaveDefineRead,
            Self::HrLeaveDefineUpdate,
            Self::HrLeaveDefineDelete,
            Self::HrLeaveRequest,
            Self::HrLeaveApprove,
            Self::HrLeaveReject,
            Self::HrLeaveCancel,
            Self::HrLeaveRead,
            Self::HrAttendanceStaffMark,
            Self::HrAttendanceStaffUpdate,
            Self::HrAttendanceStaffDelete,
            Self::HrAttendanceStaffRead,
            Self::HrAttendanceStaffImport,
            Self::HrAttendanceStaffImportPromote,
            Self::HrAttendanceStaffImportReject,
            Self::HrPayrollGenerate,
            Self::HrPayrollUpdate,
            Self::HrPayrollApprove,
            Self::HrPayrollMarkPaid,
            Self::HrPayrollRead,
            Self::HrPayrollEarningAdd,
            Self::HrPayrollEarningUpdate,
            Self::HrPayrollEarningDelete,
            Self::HrPayrollDeductionAdd,
            Self::HrPayrollDeductionUpdate,
            Self::HrPayrollDeductionDelete,
            Self::HrPayrollLeaveDeductionAdd,
            Self::HrPayrollLeaveDeductionUpdate,
            Self::HrPayrollLeaveDeductionDelete,
            Self::HrPayrollPaymentRead,
            Self::HrSalaryTemplateCreate,
            Self::HrSalaryTemplateRead,
            Self::HrSalaryTemplateUpdate,
            Self::HrSalaryTemplateDelete,
            Self::HrHourlyRateSet,
            Self::HrHourlyRateRead,
            Self::HrHourlyRateUpdate,
            Self::HrHourlyRateDelete,
            Self::FinanceInvoiceCreate,
            Self::FinanceInvoiceRead,
            Self::FinanceInvoiceUpdate,
            Self::FinanceInvoiceDelete,
            Self::FinanceFeesGroupCreate,
            Self::FinanceFeesGroupRead,
            Self::FinanceFeesGroupUpdate,
            Self::FinanceFeesGroupDelete,
            Self::FinanceFeesTypeCreate,
            Self::FinanceFeesTypeRead,
            Self::FinanceFeesTypeUpdate,
            Self::FinanceFeesTypeDelete,
            Self::FinanceFeesMasterCreate,
            Self::FinanceFeesMasterRead,
            Self::FinanceFeesMasterUpdate,
            Self::FinanceFeesMasterDelete,
            Self::FinanceFeesDiscountCreate,
            Self::FinanceFeesDiscountRead,
            Self::FinanceFeesDiscountUpdate,
            Self::FinanceFeesDiscountDelete,
            Self::FinanceFeesAssignCreate,
            Self::FinanceFeesAssignRead,
            Self::FinanceFeesAssignUpdate,
            Self::FinanceFeesAssignClose,
            Self::FinanceFeesInstallmentCreate,
            Self::FinanceFeesInstallmentRead,
            Self::FinanceFeesInstallmentUpdate,
            Self::FinanceFeesInstallmentDelete,
            Self::FinanceFeesInstallmentAssign,
            Self::FinanceDirectFeesInstallmentCreate,
            Self::FinanceDirectFeesInstallmentRead,
            Self::FinanceDirectFeesInstallmentUpdate,
            Self::FinanceDirectFeesInstallmentDelete,
            Self::FinanceDirectFeesInstallmentAssign,
            Self::FinanceDirectFeesInstallmentPay,
            Self::FinanceFeesInvoiceGenerate,
            Self::FinanceFeesInvoiceRead,
            Self::FinanceFeesInvoiceUpdate,
            Self::FinanceFeesInvoiceCancel,
            Self::FinanceFeesInvoiceConfigure,
            Self::FinanceFeesInvoicePrint,
            Self::FinancePaymentCollect,
            Self::FinancePaymentRead,
            Self::FinancePaymentReverse,
            Self::FinancePaymentRefund,
            Self::FinancePaymentMethodCreate,
            Self::FinancePaymentMethodRead,
            Self::FinancePaymentMethodUpdate,
            Self::FinancePaymentMethodDelete,
            Self::FinancePaymentGatewayConfigure,
            Self::FinancePaymentGatewayRead,
            Self::FinancePaymentGatewayUpdate,
            Self::FinancePaymentGatewayDisable,
            Self::FinanceExpenseCreate,
            Self::FinanceExpenseRead,
            Self::FinanceExpenseUpdate,
            Self::FinanceExpenseDelete,
            Self::FinanceExpenseApprove,
            Self::FinanceExpenseHeadCreate,
            Self::FinanceExpenseHeadRead,
            Self::FinanceExpenseHeadUpdate,
            Self::FinanceExpenseHeadDelete,
            Self::FinanceIncomeCreate,
            Self::FinanceIncomeRead,
            Self::FinanceIncomeUpdate,
            Self::FinanceIncomeDelete,
            Self::FinanceIncomeApprove,
            Self::FinanceIncomeHeadCreate,
            Self::FinanceIncomeHeadRead,
            Self::FinanceIncomeHeadUpdate,
            Self::FinanceIncomeHeadDelete,
            Self::FinanceBankOpen,
            Self::FinanceBankRead,
            Self::FinanceBankUpdate,
            Self::FinanceBankClose,
            Self::FinanceBankStatementRecord,
            Self::FinanceBankStatementReverse,
            Self::FinanceBankTransfer,
            Self::FinanceBankSlipGenerate,
            Self::FinanceBankSlipRead,
            Self::FinanceBankSlipApprove,
            Self::FinanceBankSlipReject,
            Self::FinancePayrollPaymentRead,
            Self::FinancePayrollPaymentRecord,
            Self::FinanceWalletCredit,
            Self::FinanceWalletDebit,
            Self::FinanceWalletRead,
            Self::FinanceWalletApprove,
            Self::FinanceWalletReject,
            Self::FinanceFeesCarryForwardExecute,
            Self::FinanceFeesCarryForwardRead,
            Self::FinanceFeesCarryForwardConfigure,
            Self::FinanceDueFeesBlock,
            Self::FinanceDueFeesUnblock,
            Self::FinanceDueFeesRead,
            Self::FinanceFeesReminderConfigure,
            Self::FinanceFeesReminderRead,
            Self::FinanceFeesReminderUpdate,
            Self::FinanceFeesReminderDelete,
            Self::FinanceChartOfAccountCreate,
            Self::FinanceChartOfAccountRead,
            Self::FinanceChartOfAccountUpdate,
            Self::FinanceChartOfAccountDelete,
            Self::FinanceReportFeesCollected,
            Self::FinanceReportFeesOutstanding,
            Self::FinanceReportDailyCollection,
            Self::FinanceReportExpense,
            Self::FinanceReportBankReconciliation,
            Self::FinanceReportWalletLedger,
            Self::FinanceReportRead,
            Self::HrStaffRegistrationFieldCreate,
            Self::HrStaffRegistrationFieldRead,
            Self::HrStaffRegistrationFieldUpdate,
            Self::HrStaffRegistrationFieldDelete,
            Self::HrReportStaffRoster,
            Self::HrReportStaffByDepartment,
            Self::HrReportStaffByDesignation,
            Self::HrReportLeaveUsage,
            Self::HrReportLeaveBalance,
            Self::HrReportAttendanceDaily,
            Self::HrReportAttendanceMonthly,
            Self::HrReportAttendanceByStaff,
            Self::HrReportPayrollRegister,
            Self::HrReportPayrollByStaff,
            Self::HrReportPayrollByDepartment,
            Self::HrReportPayrollTax,
            Self::HrReportSalaryStructure,
            Self::HrReportHourlyEarnings,
            Self::HrReportLeaveDeduction,
            Self::HrReportRead,
            Self::LibraryBookCreate,
            Self::LibraryBookRead,
            Self::LibraryBookUpdate,
            Self::LibraryBookDelete,
            Self::CommunicationMessageCreate,
            Self::CommunicationMessageRead,
            Self::CommunicationMessageUpdate,
            Self::CommunicationMessageDelete,
            Self::DocumentsFolderCreate,
            Self::DocumentsFolderRead,
            Self::DocumentsFolderUpdate,
            Self::DocumentsFolderDelete,
            Self::CmsPageCreate,
            Self::CmsPageRead,
            Self::CmsPageUpdate,
            Self::CmsPageDelete,
            Self::FacilitiesRoomCreate,
            Self::FacilitiesRoomRead,
            Self::FacilitiesRoomUpdate,
            Self::FacilitiesRoomDelete,
            Self::EventsCalendarCreate,
            Self::EventsCalendarRead,
            Self::EventsCalendarUpdate,
            Self::EventsCalendarDelete,
            Self::AttendanceStudentCreate,
            Self::AttendanceStudentRead,
            Self::AttendanceStudentUpdate,
            Self::AttendanceStudentDelete,
            Self::AttendanceSubjectCreate,
            Self::AttendanceSubjectRead,
            Self::AttendanceSubjectUpdate,
            Self::AttendanceSubjectDelete,
            Self::AttendanceSubjectNotify,
            Self::AttendanceStaffCreate,
            Self::AttendanceStaffRead,
            Self::AttendanceStaffUpdate,
            Self::AttendanceStaffDelete,
            Self::AttendanceExamCreate,
            Self::AttendanceExamRead,
            Self::AttendanceExamUpdate,
            Self::AttendanceExamDelete,
            Self::AttendanceImportCreate,
            Self::AttendanceImportRead,
            Self::AttendanceImportUpdate,
            Self::AttendanceImportDelete,
            Self::AttendanceBulkMark,
            Self::AttendanceReportRead,
            Self::AttendanceNotify,
            Self::SettingsManage,
            Self::OperationsManage,
        ]
    }

    /// Parses a canonical string form (e.g. `"Platform.School.Create"`)
    /// into a `Capability`. Returns `None` on unknown strings.
    #[must_use]
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "Platform.School.Create" => Some(Self::PlatformSchoolCreate),
            "Platform.School.Read" => Some(Self::PlatformSchoolRead),
            "Platform.School.Update" => Some(Self::PlatformSchoolUpdate),
            "Platform.School.Delete" => Some(Self::PlatformSchoolDelete),
            "Platform.User.Create" => Some(Self::PlatformUserCreate),
            "Platform.User.Read" => Some(Self::PlatformUserRead),
            "Platform.User.Update" => Some(Self::PlatformUserUpdate),
            "Platform.User.Delete" => Some(Self::PlatformUserDelete),
            "Rbac.Role.Create" => Some(Self::RbacRoleCreate),
            "Rbac.Role.Read" => Some(Self::RbacRoleRead),
            "Rbac.Role.Update" => Some(Self::RbacRoleUpdate),
            "Rbac.Role.Delete" => Some(Self::RbacRoleDelete),
            "Rbac.Role.Manage" => Some(Self::RbacRoleManage),
            "Rbac.Role.Clone" => Some(Self::RbacRoleClone),
            "Rbac.Capability.Assign" => Some(Self::RbacCapabilityAssign),
            "Rbac.Capability.Revoke" => Some(Self::RbacCapabilityRevoke),
            "Rbac.Capability.Read" => Some(Self::RbacCapabilityRead),
            "Rbac.Capability.UpdateMetadata" => Some(Self::RbacCapabilityUpdateMetadata),
            "Rbac.Bootstrap" => Some(Self::RbacBootstrap),
            "Academic.Student.Create" => Some(Self::AcademicStudentCreate),
            "Academic.Student.Read" => Some(Self::AcademicStudentRead),
            "Academic.Student.Update" => Some(Self::AcademicStudentUpdate),
            "Academic.Student.Delete" => Some(Self::AcademicStudentDelete),
            "Academic.Class.Create" => Some(Self::AcademicClassCreate),
            "Academic.Class.Read" => Some(Self::AcademicClassRead),
            "Academic.Class.Update" => Some(Self::AcademicClassUpdate),
            "Academic.Class.Delete" => Some(Self::AcademicClassDelete),
            "Assessment.Exam.Create" => Some(Self::AssessmentExamCreate),
            "Assessment.Exam.Read" => Some(Self::AssessmentExamRead),
            "Assessment.Exam.Update" => Some(Self::AssessmentExamUpdate),
            "Assessment.Exam.Delete" => Some(Self::AssessmentExamDelete),
            "Assessment.ExamSchedule.Create" => Some(Self::AssessmentExamScheduleCreate),
            "Assessment.ExamSchedule.Read" => Some(Self::AssessmentExamScheduleRead),
            "Assessment.ExamSchedule.Update" => Some(Self::AssessmentExamScheduleUpdate),
            "Assessment.ExamSchedule.Delete" => Some(Self::AssessmentExamScheduleDelete),
            "Assessment.MarksRegister.Create" => Some(Self::AssessmentMarksRegisterCreate),
            "Assessment.MarksRegister.Read" => Some(Self::AssessmentMarksRegisterRead),
            "Assessment.MarksRegister.Update" => Some(Self::AssessmentMarksRegisterUpdate),
            "Assessment.MarksRegister.Delete" => Some(Self::AssessmentMarksRegisterDelete),
            "Assessment.ResultStore.Create" => Some(Self::AssessmentResultStoreCreate),
            "Assessment.ResultStore.Read" => Some(Self::AssessmentResultStoreRead),
            "Assessment.ResultStore.Update" => Some(Self::AssessmentResultStoreUpdate),
            "Assessment.ResultStore.Delete" => Some(Self::AssessmentResultStoreDelete),
            "Assessment.ReportCard.Generate" => Some(Self::AssessmentReportCardGenerate),
            "Assessment.ReportCard.Read" => Some(Self::AssessmentReportCardRead),
            "Assessment.ReportCard.Download" => Some(Self::AssessmentReportCardDownload),
            "Assessment.OnlineExam.Create" => Some(Self::AssessmentOnlineExamCreate),
            "Assessment.OnlineExam.Read" => Some(Self::AssessmentOnlineExamRead),
            "Assessment.OnlineExam.Update" => Some(Self::AssessmentOnlineExamUpdate),
            "Assessment.OnlineExam.Delete" => Some(Self::AssessmentOnlineExamDelete),
            "Assessment.SeatPlan.Create" => Some(Self::AssessmentSeatPlanCreate),
            "Assessment.SeatPlan.Read" => Some(Self::AssessmentSeatPlanRead),
            "Assessment.SeatPlan.Update" => Some(Self::AssessmentSeatPlanUpdate),
            "Assessment.SeatPlan.Delete" => Some(Self::AssessmentSeatPlanDelete),
            "Assessment.AdmitCard.Create" => Some(Self::AssessmentAdmitCardCreate),
            "Assessment.AdmitCard.Read" => Some(Self::AssessmentAdmitCardRead),
            "Assessment.AdmitCard.Update" => Some(Self::AssessmentAdmitCardUpdate),
            "Assessment.AdmitCard.Delete" => Some(Self::AssessmentAdmitCardDelete),
            "Finance.Invoice.Create" => Some(Self::FinanceInvoiceCreate),
            "Finance.Invoice.Read" => Some(Self::FinanceInvoiceRead),
            "Finance.Invoice.Update" => Some(Self::FinanceInvoiceUpdate),
            "Finance.Invoice.Delete" => Some(Self::FinanceInvoiceDelete),
            "Hr.Staff.Create" => Some(Self::HrStaffCreate),
            "Hr.Staff.Read" => Some(Self::HrStaffRead),
            "Hr.Staff.Update" => Some(Self::HrStaffUpdate),
            "Hr.Staff.Delete" => Some(Self::HrStaffDelete),
            "Hr.Staff.Suspend" => Some(Self::HrStaffSuspend),
            "Hr.Staff.Reinstate" => Some(Self::HrStaffReinstate),
            "Hr.Staff.Resign" => Some(Self::HrStaffResign),
            "Hr.Staff.Terminate" => Some(Self::HrStaffTerminate),
            "Hr.Staff.Retire" => Some(Self::HrStaffRetire),
            "Hr.Staff.ChangeDepartment" => Some(Self::HrStaffChangeDepartment),
            "Hr.Staff.ChangeDesignation" => Some(Self::HrStaffChangeDesignation),
            "Hr.Staff.ChangeRole" => Some(Self::HrStaffChangeRole),
            "Hr.Staff.AssignSubjectTeacher" => Some(Self::HrStaffAssignSubjectTeacher),
            "Hr.AssignClassTeacher.Create" => Some(Self::HrStaffAssignClassTeacherCreate),
            "Hr.AssignClassTeacher.Update" => Some(Self::HrStaffAssignClassTeacherUpdate),
            "Hr.AssignClassTeacher.Delete" => Some(Self::HrStaffAssignClassTeacherDelete),
            "Hr.Staff.ImportBulk" => Some(Self::HrStaffImportBulk),
            "Hr.Staff.ImportBulk.Promote" => Some(Self::HrStaffImportBulkPromote),
            "Hr.Staff.ImportBulk.Reject" => Some(Self::HrStaffImportBulkReject),
            "Hr.Staff.Document.Upload" => Some(Self::HrStaffDocumentUpload),
            "Hr.Staff.Document.Download" => Some(Self::HrStaffDocumentDownload),
            "Hr.Department.Create" => Some(Self::HrDepartmentCreate),
            "Hr.Department.Read" => Some(Self::HrDepartmentRead),
            "Hr.Department.Update" => Some(Self::HrDepartmentUpdate),
            "Hr.Department.Delete" => Some(Self::HrDepartmentDelete),
            "Hr.Designation.Create" => Some(Self::HrDesignationCreate),
            "Hr.Designation.Read" => Some(Self::HrDesignationRead),
            "Hr.Designation.Update" => Some(Self::HrDesignationUpdate),
            "Hr.Designation.Delete" => Some(Self::HrDesignationDelete),
            "Hr.LeaveType.Create" => Some(Self::HrLeaveTypeCreate),
            "Hr.LeaveType.Read" => Some(Self::HrLeaveTypeRead),
            "Hr.LeaveType.Update" => Some(Self::HrLeaveTypeUpdate),
            "Hr.LeaveType.Delete" => Some(Self::HrLeaveTypeDelete),
            "Hr.LeaveDefine.Create" => Some(Self::HrLeaveDefineCreate),
            "Hr.LeaveDefine.Read" => Some(Self::HrLeaveDefineRead),
            "Hr.LeaveDefine.Update" => Some(Self::HrLeaveDefineUpdate),
            "Hr.LeaveDefine.Delete" => Some(Self::HrLeaveDefineDelete),
            "Hr.Leave.Request" => Some(Self::HrLeaveRequest),
            "Hr.Leave.Approve" => Some(Self::HrLeaveApprove),
            "Hr.Leave.Reject" => Some(Self::HrLeaveReject),
            "Hr.Leave.Cancel" => Some(Self::HrLeaveCancel),
            "Hr.Leave.Read" => Some(Self::HrLeaveRead),
            "Hr.Attendance.Staff.Mark" => Some(Self::HrAttendanceStaffMark),
            "Hr.Attendance.Staff.Update" => Some(Self::HrAttendanceStaffUpdate),
            "Hr.Attendance.Staff.Delete" => Some(Self::HrAttendanceStaffDelete),
            "Hr.Attendance.Staff.Read" => Some(Self::HrAttendanceStaffRead),
            "Hr.Attendance.Staff.Import" => Some(Self::HrAttendanceStaffImport),
            "Hr.Attendance.Staff.Import.Promote" => Some(Self::HrAttendanceStaffImportPromote),
            "Hr.Attendance.Staff.Import.Reject" => Some(Self::HrAttendanceStaffImportReject),
            "Hr.Payroll.Generate" => Some(Self::HrPayrollGenerate),
            "Hr.Payroll.Update" => Some(Self::HrPayrollUpdate),
            "Hr.Payroll.Approve" => Some(Self::HrPayrollApprove),
            "Hr.Payroll.MarkPaid" => Some(Self::HrPayrollMarkPaid),
            "Hr.Payroll.Read" => Some(Self::HrPayrollRead),
            "Hr.Payroll.Earning.Add" => Some(Self::HrPayrollEarningAdd),
            "Hr.Payroll.Earning.Update" => Some(Self::HrPayrollEarningUpdate),
            "Hr.Payroll.Earning.Delete" => Some(Self::HrPayrollEarningDelete),
            "Hr.Payroll.Deduction.Add" => Some(Self::HrPayrollDeductionAdd),
            "Hr.Payroll.Deduction.Update" => Some(Self::HrPayrollDeductionUpdate),
            "Hr.Payroll.Deduction.Delete" => Some(Self::HrPayrollDeductionDelete),
            "Hr.Payroll.LeaveDeduction.Add" => Some(Self::HrPayrollLeaveDeductionAdd),
            "Hr.Payroll.LeaveDeduction.Update" => Some(Self::HrPayrollLeaveDeductionUpdate),
            "Hr.Payroll.LeaveDeduction.Delete" => Some(Self::HrPayrollLeaveDeductionDelete),
            "Hr.PayrollPayment.Read" => Some(Self::HrPayrollPaymentRead),
            "Hr.SalaryTemplate.Create" => Some(Self::HrSalaryTemplateCreate),
            "Hr.SalaryTemplate.Read" => Some(Self::HrSalaryTemplateRead),
            "Hr.SalaryTemplate.Update" => Some(Self::HrSalaryTemplateUpdate),
            "Hr.SalaryTemplate.Delete" => Some(Self::HrSalaryTemplateDelete),
            "Hr.HourlyRate.Set" => Some(Self::HrHourlyRateSet),
            "Finance.FeesGroup.Create" => Some(Self::FinanceFeesGroupCreate),
            "Finance.FeesGroup.Read" => Some(Self::FinanceFeesGroupRead),
            "Finance.FeesGroup.Update" => Some(Self::FinanceFeesGroupUpdate),
            "Finance.FeesGroup.Delete" => Some(Self::FinanceFeesGroupDelete),
            "Finance.FeesType.Create" => Some(Self::FinanceFeesTypeCreate),
            "Finance.FeesType.Read" => Some(Self::FinanceFeesTypeRead),
            "Finance.FeesType.Update" => Some(Self::FinanceFeesTypeUpdate),
            "Finance.FeesType.Delete" => Some(Self::FinanceFeesTypeDelete),
            "Finance.FeesMaster.Create" => Some(Self::FinanceFeesMasterCreate),
            "Finance.FeesMaster.Read" => Some(Self::FinanceFeesMasterRead),
            "Finance.FeesMaster.Update" => Some(Self::FinanceFeesMasterUpdate),
            "Finance.FeesMaster.Delete" => Some(Self::FinanceFeesMasterDelete),
            "Finance.FeesDiscount.Create" => Some(Self::FinanceFeesDiscountCreate),
            "Finance.FeesDiscount.Read" => Some(Self::FinanceFeesDiscountRead),
            "Finance.FeesDiscount.Update" => Some(Self::FinanceFeesDiscountUpdate),
            "Finance.FeesDiscount.Delete" => Some(Self::FinanceFeesDiscountDelete),
            "Finance.FeesAssign.Create" => Some(Self::FinanceFeesAssignCreate),
            "Finance.FeesAssign.Read" => Some(Self::FinanceFeesAssignRead),
            "Finance.FeesAssign.Update" => Some(Self::FinanceFeesAssignUpdate),
            "Finance.FeesAssign.Close" => Some(Self::FinanceFeesAssignClose),
            "Finance.FeesInstallment.Create" => Some(Self::FinanceFeesInstallmentCreate),
            "Finance.FeesInstallment.Read" => Some(Self::FinanceFeesInstallmentRead),
            "Finance.FeesInstallment.Update" => Some(Self::FinanceFeesInstallmentUpdate),
            "Finance.FeesInstallment.Delete" => Some(Self::FinanceFeesInstallmentDelete),
            "Finance.FeesInstallment.Assign" => Some(Self::FinanceFeesInstallmentAssign),
            "Finance.DirectFeesInstallment.Create" => {
                Some(Self::FinanceDirectFeesInstallmentCreate)
            }
            "Finance.DirectFeesInstallment.Read" => Some(Self::FinanceDirectFeesInstallmentRead),
            "Finance.DirectFeesInstallment.Update" => {
                Some(Self::FinanceDirectFeesInstallmentUpdate)
            }
            "Finance.DirectFeesInstallment.Delete" => {
                Some(Self::FinanceDirectFeesInstallmentDelete)
            }
            "Finance.DirectFeesInstallment.Assign" => {
                Some(Self::FinanceDirectFeesInstallmentAssign)
            }
            "Finance.DirectFeesInstallment.Pay" => Some(Self::FinanceDirectFeesInstallmentPay),
            "Finance.FeesInvoice.Generate" => Some(Self::FinanceFeesInvoiceGenerate),
            "Finance.FeesInvoice.Read" => Some(Self::FinanceFeesInvoiceRead),
            "Finance.FeesInvoice.Update" => Some(Self::FinanceFeesInvoiceUpdate),
            "Finance.FeesInvoice.Cancel" => Some(Self::FinanceFeesInvoiceCancel),
            "Finance.FeesInvoice.Configure" => Some(Self::FinanceFeesInvoiceConfigure),
            "Finance.FeesInvoice.Print" => Some(Self::FinanceFeesInvoicePrint),
            "Finance.Payment.Collect" => Some(Self::FinancePaymentCollect),
            "Finance.Payment.Read" => Some(Self::FinancePaymentRead),
            "Finance.Payment.Reverse" => Some(Self::FinancePaymentReverse),
            "Finance.Payment.Refund" => Some(Self::FinancePaymentRefund),
            "Finance.PaymentMethod.Create" => Some(Self::FinancePaymentMethodCreate),
            "Finance.PaymentMethod.Read" => Some(Self::FinancePaymentMethodRead),
            "Finance.PaymentMethod.Update" => Some(Self::FinancePaymentMethodUpdate),
            "Finance.PaymentMethod.Delete" => Some(Self::FinancePaymentMethodDelete),
            "Finance.PaymentGateway.Configure" => Some(Self::FinancePaymentGatewayConfigure),
            "Finance.PaymentGateway.Read" => Some(Self::FinancePaymentGatewayRead),
            "Finance.PaymentGateway.Update" => Some(Self::FinancePaymentGatewayUpdate),
            "Finance.PaymentGateway.Disable" => Some(Self::FinancePaymentGatewayDisable),
            "Finance.Expense.Create" => Some(Self::FinanceExpenseCreate),
            "Finance.Expense.Read" => Some(Self::FinanceExpenseRead),
            "Finance.Expense.Update" => Some(Self::FinanceExpenseUpdate),
            "Finance.Expense.Delete" => Some(Self::FinanceExpenseDelete),
            "Finance.Expense.Approve" => Some(Self::FinanceExpenseApprove),
            "Finance.ExpenseHead.Create" => Some(Self::FinanceExpenseHeadCreate),
            "Finance.ExpenseHead.Read" => Some(Self::FinanceExpenseHeadRead),
            "Finance.ExpenseHead.Update" => Some(Self::FinanceExpenseHeadUpdate),
            "Finance.ExpenseHead.Delete" => Some(Self::FinanceExpenseHeadDelete),
            "Finance.Income.Create" => Some(Self::FinanceIncomeCreate),
            "Finance.Income.Read" => Some(Self::FinanceIncomeRead),
            "Finance.Income.Update" => Some(Self::FinanceIncomeUpdate),
            "Finance.Income.Delete" => Some(Self::FinanceIncomeDelete),
            "Finance.Income.Approve" => Some(Self::FinanceIncomeApprove),
            "Finance.IncomeHead.Create" => Some(Self::FinanceIncomeHeadCreate),
            "Finance.IncomeHead.Read" => Some(Self::FinanceIncomeHeadRead),
            "Finance.IncomeHead.Update" => Some(Self::FinanceIncomeHeadUpdate),
            "Finance.IncomeHead.Delete" => Some(Self::FinanceIncomeHeadDelete),
            "Finance.Bank.Open" => Some(Self::FinanceBankOpen),
            "Finance.Bank.Read" => Some(Self::FinanceBankRead),
            "Finance.Bank.Update" => Some(Self::FinanceBankUpdate),
            "Finance.Bank.Close" => Some(Self::FinanceBankClose),
            "Finance.Bank.Statement.Record" => Some(Self::FinanceBankStatementRecord),
            "Finance.Bank.Statement.Reverse" => Some(Self::FinanceBankStatementReverse),
            "Finance.Bank.Transfer" => Some(Self::FinanceBankTransfer),
            "Finance.BankSlip.Generate" => Some(Self::FinanceBankSlipGenerate),
            "Finance.BankSlip.Read" => Some(Self::FinanceBankSlipRead),
            "Finance.BankSlip.Approve" => Some(Self::FinanceBankSlipApprove),
            "Finance.BankSlip.Reject" => Some(Self::FinanceBankSlipReject),
            "Finance.PayrollPayment.Read" => Some(Self::FinancePayrollPaymentRead),
            "Finance.PayrollPayment.Record" => Some(Self::FinancePayrollPaymentRecord),
            "Finance.Wallet.Credit" => Some(Self::FinanceWalletCredit),
            "Finance.Wallet.Debit" => Some(Self::FinanceWalletDebit),
            "Finance.Wallet.Read" => Some(Self::FinanceWalletRead),
            "Finance.Wallet.Approve" => Some(Self::FinanceWalletApprove),
            "Finance.Wallet.Reject" => Some(Self::FinanceWalletReject),
            "Finance.FeesCarryForward.Execute" => Some(Self::FinanceFeesCarryForwardExecute),
            "Finance.FeesCarryForward.Read" => Some(Self::FinanceFeesCarryForwardRead),
            "Finance.FeesCarryForward.Configure" => Some(Self::FinanceFeesCarryForwardConfigure),
            "Finance.DueFees.Block" => Some(Self::FinanceDueFeesBlock),
            "Finance.DueFees.Unblock" => Some(Self::FinanceDueFeesUnblock),
            "Finance.DueFees.Read" => Some(Self::FinanceDueFeesRead),
            "Finance.FeesReminder.Configure" => Some(Self::FinanceFeesReminderConfigure),
            "Finance.FeesReminder.Read" => Some(Self::FinanceFeesReminderRead),
            "Finance.FeesReminder.Update" => Some(Self::FinanceFeesReminderUpdate),
            "Finance.FeesReminder.Delete" => Some(Self::FinanceFeesReminderDelete),
            "Finance.ChartOfAccount.Create" => Some(Self::FinanceChartOfAccountCreate),
            "Finance.ChartOfAccount.Read" => Some(Self::FinanceChartOfAccountRead),
            "Finance.ChartOfAccount.Update" => Some(Self::FinanceChartOfAccountUpdate),
            "Finance.ChartOfAccount.Delete" => Some(Self::FinanceChartOfAccountDelete),
            "Finance.Report.Read.FeesCollected" => Some(Self::FinanceReportFeesCollected),
            "Finance.Report.Read.FeesOutstanding" => Some(Self::FinanceReportFeesOutstanding),
            "Finance.Report.Read.DailyCollection" => Some(Self::FinanceReportDailyCollection),
            "Finance.Report.Read.Expense" => Some(Self::FinanceReportExpense),
            "Finance.Report.Read.BankReconciliation" => Some(Self::FinanceReportBankReconciliation),
            "Finance.Report.Read.WalletLedger" => Some(Self::FinanceReportWalletLedger),
            "Finance.Report.Read.Finance" => Some(Self::FinanceReportRead),
            "Hr.HourlyRate.Read" => Some(Self::HrHourlyRateRead),
            "Hr.HourlyRate.Update" => Some(Self::HrHourlyRateUpdate),
            "Hr.HourlyRate.Delete" => Some(Self::HrHourlyRateDelete),
            "Hr.StaffRegistrationField.Create" => Some(Self::HrStaffRegistrationFieldCreate),
            "Hr.StaffRegistrationField.Read" => Some(Self::HrStaffRegistrationFieldRead),
            "Hr.StaffRegistrationField.Update" => Some(Self::HrStaffRegistrationFieldUpdate),
            "Hr.StaffRegistrationField.Delete" => Some(Self::HrStaffRegistrationFieldDelete),
            "Hr.Report.Read.StaffRoster" => Some(Self::HrReportStaffRoster),
            "Hr.Report.Read.StaffByDepartment" => Some(Self::HrReportStaffByDepartment),
            "Hr.Report.Read.StaffByDesignation" => Some(Self::HrReportStaffByDesignation),
            "Hr.Report.Read.LeaveUsage" => Some(Self::HrReportLeaveUsage),
            "Hr.Report.Read.LeaveBalance" => Some(Self::HrReportLeaveBalance),
            "Hr.Report.Read.AttendanceDaily" => Some(Self::HrReportAttendanceDaily),
            "Hr.Report.Read.AttendanceMonthly" => Some(Self::HrReportAttendanceMonthly),
            "Hr.Report.Read.AttendanceByStaff" => Some(Self::HrReportAttendanceByStaff),
            "Hr.Report.Read.PayrollRegister" => Some(Self::HrReportPayrollRegister),
            "Hr.Report.Read.PayrollByStaff" => Some(Self::HrReportPayrollByStaff),
            "Hr.Report.Read.PayrollByDepartment" => Some(Self::HrReportPayrollByDepartment),
            "Hr.Report.Read.PayrollTax" => Some(Self::HrReportPayrollTax),
            "Hr.Report.Read.SalaryStructure" => Some(Self::HrReportSalaryStructure),
            "Hr.Report.Read.HourlyEarnings" => Some(Self::HrReportHourlyEarnings),
            "Hr.Report.Read.LeaveDeduction" => Some(Self::HrReportLeaveDeduction),
            "Hr.Report.Read.HR" => Some(Self::HrReportRead),
            "Library.Book.Create" => Some(Self::LibraryBookCreate),
            "Library.Book.Read" => Some(Self::LibraryBookRead),
            "Library.Book.Update" => Some(Self::LibraryBookUpdate),
            "Library.Book.Delete" => Some(Self::LibraryBookDelete),
            "Communication.Message.Create" => Some(Self::CommunicationMessageCreate),
            "Communication.Message.Read" => Some(Self::CommunicationMessageRead),
            "Communication.Message.Update" => Some(Self::CommunicationMessageUpdate),
            "Communication.Message.Delete" => Some(Self::CommunicationMessageDelete),
            "Documents.Folder.Create" => Some(Self::DocumentsFolderCreate),
            "Documents.Folder.Read" => Some(Self::DocumentsFolderRead),
            "Documents.Folder.Update" => Some(Self::DocumentsFolderUpdate),
            "Documents.Folder.Delete" => Some(Self::DocumentsFolderDelete),
            "Cms.Page.Create" => Some(Self::CmsPageCreate),
            "Cms.Page.Read" => Some(Self::CmsPageRead),
            "Cms.Page.Update" => Some(Self::CmsPageUpdate),
            "Cms.Page.Delete" => Some(Self::CmsPageDelete),
            "Facilities.Room.Create" => Some(Self::FacilitiesRoomCreate),
            "Facilities.Room.Read" => Some(Self::FacilitiesRoomRead),
            "Facilities.Room.Update" => Some(Self::FacilitiesRoomUpdate),
            "Facilities.Room.Delete" => Some(Self::FacilitiesRoomDelete),
            "Events.Calendar.Create" => Some(Self::EventsCalendarCreate),
            "Events.Calendar.Read" => Some(Self::EventsCalendarRead),
            "Events.Calendar.Update" => Some(Self::EventsCalendarUpdate),
            "Events.Calendar.Delete" => Some(Self::EventsCalendarDelete),
            "Attendance.Student.Create" => Some(Self::AttendanceStudentCreate),
            "Attendance.Student.Read" => Some(Self::AttendanceStudentRead),
            "Attendance.Student.Update" => Some(Self::AttendanceStudentUpdate),
            "Attendance.Student.Delete" => Some(Self::AttendanceStudentDelete),
            "Attendance.Subject.Create" => Some(Self::AttendanceSubjectCreate),
            "Attendance.Subject.Read" => Some(Self::AttendanceSubjectRead),
            "Attendance.Subject.Update" => Some(Self::AttendanceSubjectUpdate),
            "Attendance.Subject.Delete" => Some(Self::AttendanceSubjectDelete),
            "Attendance.Subject.Notify" => Some(Self::AttendanceSubjectNotify),
            "Attendance.Staff.Create" => Some(Self::AttendanceStaffCreate),
            "Attendance.Staff.Read" => Some(Self::AttendanceStaffRead),
            "Attendance.Staff.Update" => Some(Self::AttendanceStaffUpdate),
            "Attendance.Staff.Delete" => Some(Self::AttendanceStaffDelete),
            "Attendance.Exam.Create" => Some(Self::AttendanceExamCreate),
            "Attendance.Exam.Read" => Some(Self::AttendanceExamRead),
            "Attendance.Exam.Update" => Some(Self::AttendanceExamUpdate),
            "Attendance.Exam.Delete" => Some(Self::AttendanceExamDelete),
            "Attendance.Import.Create" => Some(Self::AttendanceImportCreate),
            "Attendance.Import.Read" => Some(Self::AttendanceImportRead),
            "Attendance.Import.Update" => Some(Self::AttendanceImportUpdate),
            "Attendance.Import.Delete" => Some(Self::AttendanceImportDelete),
            "Attendance.BulkMark.BulkMark" => Some(Self::AttendanceBulkMark),
            "Attendance.Report.Read" => Some(Self::AttendanceReportRead),
            "Attendance.Notify.Notify" => Some(Self::AttendanceNotify),
            "Settings.Manage" => Some(Self::SettingsManage),
            "Operations.Manage" => Some(Self::OperationsManage),
            _ => None,
        }
    }
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Capability {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self> {
        Self::from_str_opt(s)
            .ok_or_else(|| DomainError::validation(format!("unknown capability: {s:?}")))
    }
}

/// The domain prefix of a [`Capability`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CapabilityDomain {
    /// Platform / multi-tenancy substrate.
    Platform,
    /// RBAC (roles, capabilities, sections, overrides, 2FA).
    Rbac,
    /// Academic aggregates (students, classes, sections).
    Academic,
    /// Assessment aggregates (exams, marks).
    Assessment,
    /// Attendance aggregates.
    Attendance,
    /// Finance aggregates (invoices, payments, refunds).
    Finance,
    /// HR aggregates (staff, payroll).
    Hr,
    /// Library aggregates (books, members, loans).
    Library,
    /// Communication aggregates (messages, announcements).
    Communication,
    /// Documents aggregates (folders, files).
    Documents,
    /// CMS aggregates (pages, posts).
    Cms,
    /// Facilities aggregates (rooms, assets).
    Facilities,
    /// Events domain (calendar, holidays, incidents).
    Events,
    /// Settings domain.
    Settings,
    /// Operations domain.
    Operations,
}

impl CapabilityDomain {
    /// Returns the canonical PascalCase wire string for the domain
    /// (e.g. `"Rbac"`, `"Finance"`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Platform => "Platform",
            Self::Rbac => "Rbac",
            Self::Academic => "Academic",
            Self::Assessment => "Assessment",
            Self::Attendance => "Attendance",
            Self::Finance => "Finance",
            Self::Hr => "Hr",
            Self::Library => "Library",
            Self::Communication => "Communication",
            Self::Documents => "Documents",
            Self::Cms => "Cms",
            Self::Facilities => "Facilities",
            Self::Events => "Events",
            Self::Settings => "Settings",
            Self::Operations => "Operations",
        }
    }
}

impl fmt::Display for CapabilityDomain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Whether an [`AssignPermission`](crate::entities::AssignPermission)
/// row grants or explicitly revokes a capability.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssignmentStatus {
    /// The capability is granted.
    #[default]
    Granted = 1,
    /// The capability is explicitly denied (a deliberate override).
    Revoked = 0,
}

impl AssignmentStatus {
    /// Returns the wire byte (1 = Granted, 0 = Revoked).
    #[must_use]
    pub const fn to_byte(self) -> u8 {
        match self {
            Self::Granted => 1,
            Self::Revoked => 0,
        }
    }

    /// Constructs a status from a wire byte.
    pub fn from_byte(b: u8) -> Result<Self> {
        match b {
            1 => Ok(Self::Granted),
            0 => Ok(Self::Revoked),
            other => Err(DomainError::validation(format!(
                "assignment_status must be 0 or 1, got {other}"
            ))),
        }
    }

    /// Returns `true` for [`AssignmentStatus::Granted`].
    #[must_use]
    pub const fn is_granted(self) -> bool {
        matches!(self, Self::Granted)
    }
}

impl fmt::Display for AssignmentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Granted => f.write_str("granted"),
            Self::Revoked => f.write_str("revoked"),
        }
    }
}

/// Whether an assignment row contributes to the role's menu
/// rendering. Does not affect authorization.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MenuStatus {
    /// The menu item is rendered for the role.
    #[default]
    Visible = 1,
    /// The menu item is hidden for the role.
    Hidden = 0,
}

impl MenuStatus {
    /// Returns the wire byte (1 = Visible, 0 = Hidden).
    #[must_use]
    pub const fn to_byte(self) -> u8 {
        match self {
            Self::Visible => 1,
            Self::Hidden => 0,
        }
    }

    /// Constructs a status from a wire byte.
    pub fn from_byte(b: u8) -> Result<Self> {
        match b {
            1 => Ok(Self::Visible),
            0 => Ok(Self::Hidden),
            other => Err(DomainError::validation(format!(
                "menu_status must be 0 or 1, got {other}"
            ))),
        }
    }
}

impl fmt::Display for MenuStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Visible => f.write_str("visible"),
            Self::Hidden => f.write_str("hidden"),
        }
    }
}

/// The kind of permission row.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermissionType {
    /// Top-level menu item.
    Menu = 1,
    /// Sub-menu item rendered under a [`PermissionType::Menu`].
    SubMenu = 2,
    /// Action button / link rendered in a page.
    #[default]
    Action = 3,
}

impl PermissionType {
    /// Returns the wire byte (1 = Menu, 2 = SubMenu, 3 = Action).
    #[must_use]
    pub const fn to_byte(self) -> u8 {
        match self {
            Self::Menu => 1,
            Self::SubMenu => 2,
            Self::Action => 3,
        }
    }

    /// Constructs a permission type from a wire byte.
    pub fn from_byte(b: u8) -> Result<Self> {
        match b {
            1 => Ok(Self::Menu),
            2 => Ok(Self::SubMenu),
            3 => Ok(Self::Action),
            other => Err(DomainError::validation(format!(
                "permission_type must be 1, 2, or 3, got {other}"
            ))),
        }
    }
}

impl fmt::Display for PermissionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Menu => f.write_str("menu"),
            Self::SubMenu => f.write_str("submenu"),
            Self::Action => f.write_str("action"),
        }
    }
}

/// The lifecycle type of a role.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RoleType {
    /// A role seeded by the engine at school activation. Cannot be
    /// deleted; rename is gated on `RbacRoleManage`.
    System,
    /// A user-defined role. Full lifecycle (create, update, delete).
    #[default]
    Custom,
}

impl RoleType {
    /// Returns the canonical snake_case wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Custom => "custom",
        }
    }

    /// Returns `true` for [`RoleType::System`].
    #[must_use]
    pub const fn is_system(self) -> bool {
        matches!(self, Self::System)
    }
}

impl fmt::Display for RoleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A validated, non-empty role name. Per `docs/specs/rbac/value-objects.md`
/// `RoleName` is 1..=100 chars and unique within `(school_id, lower(name))`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RoleName(String);

impl RoleName {
    /// Maximum length of a role name, per the spec.
    pub const MAX_LEN: usize = 100;

    /// Constructs a `RoleName`, rejecting empty or overlong input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_role_name(&s)?;
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RoleName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for RoleName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

fn validate_role_name(s: &str) -> Result<()> {
    if s.is_empty() {
        return Err(DomainError::validation("role name must not be empty"));
    }
    if s.chars().count() > RoleName::MAX_LEN {
        return Err(DomainError::validation(format!(
            "role name must be at most {} chars, got {}",
            RoleName::MAX_LEN,
            s.chars().count()
        )));
    }
    Ok(())
}

/// Two-factor mode (placeholder for the Phase 2 follow-up workstream
/// that will land the `TwoFactorSetting` aggregate). Encoded as
/// `1 = Required`, `2 = Optional`, `3 = Disabled` per
/// `docs/specs/rbac/aggregates.md` § `TwoFactorSetting`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TwoFactorMode {
    /// 2FA is required for the role.
    Required = 1,
    /// 2FA is offered but optional.
    Optional = 2,
    /// 2FA is disabled.
    #[default]
    Disabled = 3,
}

impl TwoFactorMode {
    /// Returns the wire byte (1..=3).
    #[must_use]
    pub const fn to_byte(self) -> u8 {
        match self {
            Self::Required => 1,
            Self::Optional => 2,
            Self::Disabled => 3,
        }
    }

    /// Constructs a `TwoFactorMode` from a wire byte.
    pub fn from_byte(b: u8) -> Result<Self> {
        match b {
            1 => Ok(Self::Required),
            2 => Ok(Self::Optional),
            3 => Ok(Self::Disabled),
            other => Err(DomainError::validation(format!(
                "two_factor_mode must be 1, 2, or 3, got {other}"
            ))),
        }
    }
}

impl fmt::Display for TwoFactorMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Required => f.write_str("required"),
            Self::Optional => f.write_str("optional"),
            Self::Disabled => f.write_str("disabled"),
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    clippy::manual_str_repeat
)]
mod tests {
    use super::*;

    #[test]
    fn capability_round_trip_via_display_and_from_str() {
        for c in Capability::all() {
            let s = c.to_string();
            let parsed = Capability::from_str(&s).unwrap();
            assert_eq!(parsed, *c);
        }
    }

    #[test]
    fn assessment_capabilities_round_trip_and_resolve_to_assessment_domain() {
        let mut count = 0u32;
        for c in Capability::all() {
            let s = c.as_str();
            if s.starts_with("Assessment.") {
                let parsed = Capability::from_str(s).unwrap();
                assert_eq!(parsed, *c);
                assert_eq!(c.domain(), CapabilityDomain::Assessment);
                count += 1;
            }
        }
        assert_eq!(count, 31, "expected 31 Assessment.* capabilities");
    }

    #[test]
    fn attendance_capabilities_round_trip_and_resolve_to_attendance_domain() {
        let mut count = 0u32;
        for c in Capability::all() {
            let s = c.as_str();
            if s.starts_with("Attendance.") {
                let parsed = Capability::from_str(s).unwrap();
                assert_eq!(parsed, *c, "round-trip failed for {s}");
                assert_eq!(
                    c.domain(),
                    CapabilityDomain::Attendance,
                    "domain mismatch for {s}"
                );
                count += 1;
            }
        }
        assert_eq!(
            count, 24,
            "expected 24 Attendance.* capabilities (got {count})"
        );
    }

    #[test]
    fn hr_capabilities_round_trip_and_resolve_to_hr_domain() {
        let mut count = 0u32;
        for c in Capability::all() {
            let s = c.as_str();
            if s.starts_with("Hr.") {
                let parsed = Capability::from_str(s).unwrap();
                assert_eq!(parsed, *c, "round-trip failed for {s}");
                assert_eq!(c.domain(), CapabilityDomain::Hr, "domain mismatch for {s}");
                count += 1;
            }
        }
        assert_eq!(count, 92, "expected 92 Hr.* capabilities (got {count})");
    }

    #[test]
    fn finance_capabilities_round_trip_and_resolve_to_finance_domain() {
        let mut count = 0u32;
        for c in Capability::all() {
            let s = c.as_str();
            if s.starts_with("Finance.") {
                let parsed = Capability::from_str(s).unwrap();
                assert_eq!(parsed, *c, "round-trip failed for {s}");
                assert_eq!(
                    c.domain(),
                    CapabilityDomain::Finance,
                    "domain mismatch for {s}"
                );
                count += 1;
            }
        }
        assert_eq!(
            count, 114,
            "expected 114 Finance.* capabilities (4 legacy placeholders + 110 Phase 7; got {count})"
        );
    }

    #[test]
    fn capability_from_str_unknown_returns_err() {
        let err = Capability::from_str("Foo.Bar.Baz").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn capability_from_str_opt_unknown_returns_none() {
        assert!(Capability::from_str_opt("nope").is_none());
    }

    #[test]
    fn capability_domain_matches_aggregate_prefix() {
        for c in Capability::all() {
            let s = c.as_str();
            let domain = s.split_once('.').map(|(d, _)| d).unwrap_or("");
            assert_eq!(domain, c.domain().as_str(), "domain mismatch for {c:?}");
        }
    }

    #[test]
    fn capability_action_matches_third_segment() {
        for c in Capability::all() {
            let s = c.as_str();
            let parts: Vec<&str> = s.split('.').collect();
            // Two-segment exceptions: `Rbac.Bootstrap`,
            // `Settings.Manage`, `Operations.Manage`. Every other
            // capability uses the three-segment form
            // `<Domain>.<Aggregate>.<Action>` (or the four-segment
            // `<Domain>.<Aggregate>.<Action>.<Subject>` for
            // read-only report capabilities, e.g.
            // `Hr.Report.Read.StaffRoster`, and for compound
            // sub-aggregate capabilities like
            // `Hr.Staff.AssignClassTeacher.Create` or
            // `Hr.Staff.Document.Upload`).
            assert!(
                parts.len() >= 2,
                "expected at least 2 segments for {c:?}, got {s:?}"
            );
            let action = c.action();
            let last_two = if parts.len() >= 2 {
                format!("{}.{}", parts[parts.len() - 2], parts[parts.len() - 1])
            } else {
                String::new()
            };
            let last = parts[parts.len() - 1];
            assert!(
                last == action
                    || last.starts_with(action)
                    || last_two.starts_with(action)
                    || s == "Rbac.Bootstrap"
                    || s == "Settings.Manage"
                    || s == "Operations.Manage",
                "action mismatch for {c:?}: wire={s:?} action={action:?}"
            );
        }
    }

    #[test]
    fn assignment_status_byte_round_trip() {
        assert_eq!(
            AssignmentStatus::from_byte(1).unwrap(),
            AssignmentStatus::Granted
        );
        assert_eq!(
            AssignmentStatus::from_byte(0).unwrap(),
            AssignmentStatus::Revoked
        );
        assert!(AssignmentStatus::from_byte(7).is_err());
    }

    #[test]
    fn menu_status_byte_round_trip() {
        assert_eq!(MenuStatus::from_byte(1).unwrap(), MenuStatus::Visible);
        assert_eq!(MenuStatus::from_byte(0).unwrap(), MenuStatus::Hidden);
        assert!(MenuStatus::from_byte(3).is_err());
    }

    #[test]
    fn permission_type_byte_round_trip() {
        assert_eq!(PermissionType::from_byte(1).unwrap(), PermissionType::Menu);
        assert_eq!(
            PermissionType::from_byte(2).unwrap(),
            PermissionType::SubMenu
        );
        assert_eq!(
            PermissionType::from_byte(3).unwrap(),
            PermissionType::Action
        );
        assert!(PermissionType::from_byte(0).is_err());
    }

    #[test]
    fn two_factor_mode_byte_round_trip() {
        assert_eq!(
            TwoFactorMode::from_byte(1).unwrap(),
            TwoFactorMode::Required
        );
        assert_eq!(
            TwoFactorMode::from_byte(2).unwrap(),
            TwoFactorMode::Optional
        );
        assert_eq!(
            TwoFactorMode::from_byte(3).unwrap(),
            TwoFactorMode::Disabled
        );
        assert!(TwoFactorMode::from_byte(0).is_err());
    }

    #[test]
    fn role_name_rejects_empty() {
        let err = RoleName::new("").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn role_name_rejects_too_long() {
        let s: String = std::iter::repeat('a').take(RoleName::MAX_LEN + 1).collect();
        let err = RoleName::new(s).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn role_name_accepts_max_len() {
        let s: String = std::iter::repeat('a').take(RoleName::MAX_LEN).collect();
        let name = RoleName::new(s).unwrap();
        assert_eq!(name.as_str().chars().count(), RoleName::MAX_LEN);
    }

    #[test]
    fn role_type_default_is_custom() {
        assert_eq!(RoleType::default(), RoleType::Custom);
        assert!(!RoleType::Custom.is_system());
        assert!(RoleType::System.is_system());
    }
}
