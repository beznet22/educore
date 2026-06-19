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

    // -- Library (Phase 9) -----------------------------------------------
    // The 4 Phase 2 placeholders (`LibraryBook{Create,Read,Update,Delete}`)
    // were deduplicated during implementation; the canonical
    // `Book{Add,Read,Update,Delete}` variants below use the same wire
    // forms (`Library.Book.{Add,Read,Update,Delete}`) as the Phase 2
    // placeholders. Consumers that referenced the placeholders by name
    // (only `DefaultRoleCatalog::librarian()` in this workspace) were
    // updated to the new names. This matches the Phase 8
    // `FacilitiesRoom*` dedup pattern.
    /// Read library-wide aggregate reports (overdue, stock, fine roll-up).
    LibraryRead,
    /// Configure the per-school `LibrarySettings` in the settings domain.
    LibraryConfigure,
    /// Run aggregate reports across the library domain.
    LibraryReport,
    /// Create a book category.
    BookCategoryCreate,
    /// Read a book category.
    BookCategoryRead,
    /// Update a book category.
    BookCategoryUpdate,
    /// Delete a book category.
    BookCategoryDelete,
    /// Add a book to the catalog.
    BookAdd,
    /// Read a book.
    BookRead,
    /// Update a book's bibliographic metadata.
    BookUpdate,
    /// Delete a book from the catalog.
    BookDelete,
    /// Adjust a book's stock count (acquisition or write-off).
    BookAdjustQuantity,
    /// Search the book catalog.
    BookSearch,
    /// Register a library member.
    MemberRegister,
    /// Read a library member.
    MemberRead,
    /// Update a library member.
    MemberUpdate,
    /// Delete a library member.
    MemberDelete,
    /// Deactivate a library member.
    MemberDeactivate,
    /// Reactivate a library member.
    MemberReactivate,
    /// Issue a book to a library member.
    BookIssueIssue,
    /// Read a book issue.
    BookIssueRead,
    /// Return a book that was issued.
    BookIssueReturn,
    /// Renew a book issue (extend due date).
    BookIssueRenew,
    /// Mark a book issue as lost.
    BookIssueMarkLost,
    /// Calculate the late-fine for a book issue.
    BookIssueCalculateFine,
    /// Waive a fine for a book issue.
    BookIssueWaiveFine,
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
    // -- Documents domain (Phase 11 net-new) -----------------------------
    // The 4 Phase 2 `DocumentsFolder*` placeholders above were
    // retained during the Phase 11 implementation (per
    // `docs/handoff/PHASE-11-HANDOFF.md`); they keep the same wire
    // forms (`Documents.Folder.*`) and are part of the
    // `DefaultRoleCatalog::school_admin` set. The 11 net-new variants
    // below extend the Documents catalog with FormDownload and
    // Postal* aggregates.
    /// Upload a form download.
    FormDownloadUpload,
    /// Update a form download.
    FormDownloadUpdate,
    /// Delete (soft-delete) a form download.
    FormDownloadDelete,
    /// Read a form download (staff scope).
    FormDownloadRead,
    /// Create a postal dispatch.
    PostalDispatchCreate,
    /// Update a postal dispatch (reference_no is immutable).
    PostalDispatchUpdate,
    /// Delete (soft-delete) a postal dispatch.
    PostalDispatchDelete,
    /// Create a postal receive.
    PostalReceiveCreate,
    /// Update a postal receive (reference_no is immutable).
    PostalReceiveUpdate,
    /// Delete (soft-delete) a postal receive.
    PostalReceiveDelete,
    /// Read postal dispatch/receive (used by TrackPostal).
    PostalRead,
    /// Create a CMS page.
    CmsPageCreate,
    /// Read a CMS page.
    CmsPageRead,
    /// Update a CMS page.
    CmsPageUpdate,
    /// Delete a CMS page.
    CmsPageDelete,
    /// Publish a CMS page (state transition to `PageStatus::Published`).
    CmsPagePublish,
    /// Archive a CMS page (state transition back to `PageStatus::Draft`).
    CmsPageArchive,
    // -- CMS domain (Phase 12 net-new) --------------------------------------
    // The 6 Phase 2 `CmsPage{Create,Read,Update,Delete,Publish,Archive}`
    // placeholders above are the start point for the CMS catalog.
    // The 95+ net-new variants below extend the CMS catalog with
    // News, NewsCategory, NewsComment, NewsPage, NoticeBoard,
    // Testimonial, HomeSlider, SpeechSlider, Content, ContentType,
    // ContentShareList, TeacherUploadContent, UploadContent,
    // AboutPage, ContactPage, CoursePage, HomePageSetting, and
    // FrontendPage. Wire form per `docs/specs/cms/permissions.md`:
    // `<Domain>.<Aggregate>.<Action>` (e.g. `Cms.News.Create`).
    // The CMS-side `SpeechSlider` is distinct from the
    // Communication-domain `SpeechSlider`; both are surfaced under
    // the `Cms.*` namespace per the spec.
    /// Create a news entry.
    CmsNewsCreate,
    /// Read a news entry.
    CmsNewsRead,
    /// Update a news entry.
    CmsNewsUpdate,
    /// Delete (soft-delete) a news entry.
    CmsNewsDelete,
    /// Publish a news entry.
    CmsNewsPublish,
    /// Unpublish a news entry.
    CmsNewsUnpublish,
    /// Increment a news entry's view count.
    CmsNewsIncrementView,
    /// Create a news category.
    CmsNewsCategoryCreate,
    /// Read a news category.
    CmsNewsCategoryRead,
    /// Update a news category.
    CmsNewsCategoryUpdate,
    /// Delete a news category.
    CmsNewsCategoryDelete,
    /// Comment on a news entry.
    CmsNewsCommentCreate,
    /// Moderate a news comment (approve / hide).
    CmsNewsCommentModerate,
    /// Delete a news comment.
    CmsNewsCommentDelete,
    /// Read news comments.
    CmsNewsCommentRead,
    /// Create a news landing-page configuration.
    CmsNewsPageCreate,
    /// Read a news landing-page configuration.
    CmsNewsPageRead,
    /// Update a news landing-page configuration.
    CmsNewsPageUpdate,
    /// Delete a news landing-page configuration.
    CmsNewsPageDelete,
    /// Create a public-site notice board.
    CmsNoticeBoardCreate,
    /// Read a public-site notice board.
    CmsNoticeBoardRead,
    /// Update a public-site notice board.
    CmsNoticeBoardUpdate,
    /// Delete a public-site notice board.
    CmsNoticeBoardDelete,
    /// Publish a public-site notice board.
    CmsNoticeBoardPublish,
    /// Unpublish a public-site notice board.
    CmsNoticeBoardUnpublish,
    /// Create a testimonial.
    CmsTestimonialCreate,
    /// Read a testimonial.
    CmsTestimonialRead,
    /// Update a testimonial.
    CmsTestimonialUpdate,
    /// Delete a testimonial.
    CmsTestimonialDelete,
    /// Create a home-slider entry.
    CmsHomeSliderCreate,
    /// Read a home-slider entry.
    CmsHomeSliderRead,
    /// Update a home-slider entry.
    CmsHomeSliderUpdate,
    /// Delete a home-slider entry.
    CmsHomeSliderDelete,
    /// Create a speech-slider entry (CMS-side).
    CmsSpeechSliderCreate,
    /// Read a speech-slider entry (CMS-side).
    CmsSpeechSliderRead,
    /// Update a speech-slider entry (CMS-side).
    CmsSpeechSliderUpdate,
    /// Delete a speech-slider entry (CMS-side).
    CmsSpeechSliderDelete,
    /// Create a content item.
    CmsContentCreate,
    /// Read a content item.
    CmsContentRead,
    /// Update a content item.
    CmsContentUpdate,
    /// Delete a content item.
    CmsContentDelete,
    /// Create a content type (taxonomy).
    CmsContentTypeCreate,
    /// Read a content type (taxonomy).
    CmsContentTypeRead,
    /// Update a content type (taxonomy).
    CmsContentTypeUpdate,
    /// Delete a content type (taxonomy).
    CmsContentTypeDelete,
    /// Create a content share list.
    CmsContentShareListCreate,
    /// Read a content share list.
    CmsContentShareListRead,
    /// Update a content share list.
    CmsContentShareListUpdate,
    /// Delete a content share list.
    CmsContentShareListDelete,
    /// Dispatch a content share list.
    CmsContentShareListDispatch,
    /// Cancel a content share list.
    CmsContentShareListCancel,
    /// Create a teacher-uploaded content item.
    CmsTeacherUploadContentCreate,
    /// Read a teacher-uploaded content item.
    CmsTeacherUploadContentRead,
    /// Update a teacher-uploaded content item.
    CmsTeacherUploadContentUpdate,
    /// Delete a teacher-uploaded content item.
    CmsTeacherUploadContentDelete,
    /// Create an admin-uploaded content item.
    CmsUploadContentCreate,
    /// Read an admin-uploaded content item.
    CmsUploadContentRead,
    /// Update an admin-uploaded content item.
    CmsUploadContentUpdate,
    /// Delete an admin-uploaded content item.
    CmsUploadContentDelete,
    /// Create an about-page configuration.
    CmsAboutPageCreate,
    /// Read an about-page configuration.
    CmsAboutPageRead,
    /// Update an about-page configuration.
    CmsAboutPageUpdate,
    /// Delete an about-page configuration.
    CmsAboutPageDelete,
    /// Create a contact-page configuration.
    CmsContactPageCreate,
    /// Read a contact-page configuration.
    CmsContactPageRead,
    /// Update a contact-page configuration.
    CmsContactPageUpdate,
    /// Delete a contact-page configuration.
    CmsContactPageDelete,
    /// Create a course landing page.
    CmsCoursePageCreate,
    /// Read a course landing page.
    CmsCoursePageRead,
    /// Update a course landing page.
    CmsCoursePageUpdate,
    /// Delete a course landing page.
    CmsCoursePageDelete,
    /// Configure the home-page setting.
    CmsHomePageSettingConfigure,
    /// Read the home-page setting.
    CmsHomePageSettingRead,
    /// Delete the home-page setting.
    CmsHomePageSettingDelete,
    /// Create a front-end page record.
    CmsFrontendPageCreate,
    /// Read a front-end page record.
    CmsFrontendPageRead,
    /// Update a front-end page record.
    CmsFrontendPageUpdate,
    /// Delete a front-end page record.
    CmsFrontendPageDelete,
    /// Cross-cutting read capability for the CMS domain (the public
    /// site surface).
    CmsRead,

    // -- Facilities (Phase 8) ---------------------------------------------
    // The 4 Phase 2 `FacilitiesRoom{Create,Read,Update,Delete}`
    // placeholders were deduplicated during implementation;
    // the canonical `FacilitiesRoom{Create,Read,Update,Delete}`
    // variants below use the same wire forms
    // (`Facilities.Room.{Create,Read,Update,Delete}`) as the
    // Phase 2 placeholders. Consumers that referenced the
    // placeholders by name (only
    // `DefaultRoleCatalog::{school_admin,driver}` in this
    // workspace) were updated to the new names. This matches
    // the Phase 8 `FacilitiesRoom*` dedup pattern.
    /// Create a facilities room.
    FacilitiesRoomCreate,
    /// Read a facilities room.
    FacilitiesRoomRead,
    /// Update a facilities room.
    FacilitiesRoomUpdate,
    /// Delete a facilities room.
    FacilitiesRoomDelete,
    /// Create a facilities vehicle.
    FacilitiesVehicleCreate,
    /// Read a facilities vehicle.
    FacilitiesVehicleRead,
    /// Update a facilities vehicle.
    FacilitiesVehicleUpdate,
    /// Delete a facilities vehicle.
    FacilitiesVehicleDelete,
    /// Assign a driver to a vehicle.
    FacilitiesVehicleAssignDriver,
    /// Deactivate a facilities vehicle.
    FacilitiesVehicleDeactivate,
    /// Create a transport route.
    FacilitiesRouteCreate,
    /// Read a transport route.
    FacilitiesRouteRead,
    /// Update a transport route.
    FacilitiesRouteUpdate,
    /// Delete a transport route.
    FacilitiesRouteDelete,
    /// Add a stop to a route.
    FacilitiesRouteAddStop,
    /// Update a stop on a route.
    FacilitiesRouteUpdateStop,
    /// Remove a stop from a route.
    FacilitiesRouteRemoveStop,
    /// Assign a vehicle to a route in an academic year.
    FacilitiesTransportAssignVehicle,
    /// Unassign a vehicle from a route.
    FacilitiesTransportUnassignVehicle,
    /// Assign a student to a vehicle-route pair.
    FacilitiesTransportAssignStudent,
    /// Unassign a student from a vehicle-route pair.
    FacilitiesTransportUnassignStudent,
    /// Read transport assignments.
    FacilitiesTransportRead,
    /// Create a dormitory.
    FacilitiesDormitoryCreate,
    /// Read a dormitory.
    FacilitiesDormitoryRead,
    /// Update a dormitory.
    FacilitiesDormitoryUpdate,
    /// Delete a dormitory.
    FacilitiesDormitoryDelete,
    /// Assign a student to a room.
    FacilitiesRoomAssignStudent,
    /// Unassign a student from a room.
    FacilitiesRoomUnassignStudent,
    /// Create a room type.
    FacilitiesRoomTypeCreate,
    /// Read a room type.
    FacilitiesRoomTypeRead,
    /// Update a room type.
    FacilitiesRoomTypeUpdate,
    /// Delete a room type.
    FacilitiesRoomTypeDelete,
    /// Create an item category.
    FacilitiesItemCategoryCreate,
    /// Read an item category.
    FacilitiesItemCategoryRead,
    /// Update an item category.
    FacilitiesItemCategoryUpdate,
    /// Delete an item category.
    FacilitiesItemCategoryDelete,
    /// Create an inventory item.
    FacilitiesItemCreate,
    /// Read an inventory item.
    FacilitiesItemRead,
    /// Update an inventory item.
    FacilitiesItemUpdate,
    /// Delete an inventory item.
    FacilitiesItemDelete,
    /// Create an item store.
    FacilitiesItemStoreCreate,
    /// Read an item store.
    FacilitiesItemStoreRead,
    /// Update an item store.
    FacilitiesItemStoreUpdate,
    /// Delete an item store.
    FacilitiesItemStoreDelete,
    /// Receive inventory into a store.
    FacilitiesInventoryReceive,
    /// Update a goods-receive note.
    FacilitiesInventoryUpdateReceive,
    /// Cancel a goods-receive note.
    FacilitiesInventoryCancelReceive,
    /// Issue inventory from a store.
    FacilitiesInventoryIssue,
    /// Update a goods-issue note.
    FacilitiesInventoryUpdateIssue,
    /// Return an issued item.
    FacilitiesInventoryReturnIssued,
    /// Sell inventory.
    FacilitiesInventorySell,
    /// Update a sale.
    FacilitiesInventoryUpdateSell,
    /// Cancel a sale.
    FacilitiesInventoryCancelSell,
    /// Refund a sale.
    FacilitiesInventoryRefundSell,
    /// Read inventory.
    FacilitiesInventoryRead,
    /// Create a supplier.
    FacilitiesSupplierCreate,
    /// Read a supplier.
    FacilitiesSupplierRead,
    /// Update a supplier.
    FacilitiesSupplierUpdate,
    /// Delete a supplier.
    FacilitiesSupplierDelete,
    /// Deactivate a supplier.
    FacilitiesSupplierDeactivate,
    /// Create an events-domain calendar entry.
    EventsCalendarCreate,
    /// Read an events-domain calendar entry.
    EventsCalendarRead,
    /// Update an events-domain calendar entry.
    EventsCalendarUpdate,
    /// Delete an events-domain calendar entry.
    EventsCalendarDelete,
    // -- Events domain (Phase 13 net-new) --------------------------------
    // The 4 Phase 2 `EventsCalendar{Create,Read,Update,Delete}`
    // placeholders above are the start point for the Events catalog.
    // The 30 net-new variants below extend the Events catalog with
    // Event, Holiday, Weekend, Incident, IncidentComment, and
    // CalendarSetting. Wire form per
    // `docs/specs/events/permissions.md`:
    // `<Domain>.<Aggregate>.<Action>` (e.g. `Events.Event.Create`).
    // -- Event (5) --
    /// Create a calendar event.
    EventsEventCreate,
    /// Read a calendar event.
    EventsEventRead,
    /// Update a calendar event.
    EventsEventUpdate,
    /// Delete (soft-delete) a calendar event.
    EventsEventDelete,
    /// Publish a calendar event (admin override for cross-role broadcast).
    EventsEventPublish,
    // -- Holiday (4) --
    /// Create a school holiday.
    EventsHolidayCreate,
    /// Read a school holiday.
    EventsHolidayRead,
    /// Update a school holiday.
    EventsHolidayUpdate,
    /// Delete a school holiday.
    EventsHolidayDelete,
    // -- Weekend (5) --
    /// Create a weekend day entry.
    EventsWeekendCreate,
    /// Read a weekend day entry.
    EventsWeekendRead,
    /// Update a weekend day entry.
    EventsWeekendUpdate,
    /// Delete a weekend day entry.
    EventsWeekendDelete,
    /// Batch-configure weekend days.
    EventsWeekendConfigure,
    // -- Incident (9) --
    /// Create (report) an incident.
    EventsIncidentCreate,
    /// Read an incident.
    EventsIncidentRead,
    /// Update an incident.
    EventsIncidentUpdate,
    /// Delete (soft-delete) an incident (admin override).
    EventsIncidentDelete,
    /// Assign an incident to a student or staff member.
    EventsIncidentAssign,
    /// Reassign an incident (update point value).
    EventsIncidentReassign,
    /// Unassign an incident.
    EventsIncidentUnassign,
    /// Comment on an incident.
    EventsIncidentComment,
    /// Resolve an incident.
    EventsIncidentResolve,
    // -- IncidentComment (1) --
    /// Delete (soft-delete) an incident comment (admin override).
    EventsIncidentCommentDelete,
    // -- CalendarSetting (6) --
    /// Create a calendar UI setting.
    EventsCalendarSettingCreate,
    /// Read a calendar UI setting.
    EventsCalendarSettingRead,
    /// Update a calendar UI setting.
    EventsCalendarSettingUpdate,
    /// Enable a calendar UI setting.
    EventsCalendarSettingEnable,
    /// Disable a calendar UI setting.
    EventsCalendarSettingDisable,
    /// Delete a calendar UI setting.
    EventsCalendarSettingDelete,

    // -- Settings (Phase 14 net-new) -------------------------------------
    // Phase 14 ships the full per-aggregate catalog per
    // `docs/specs/settings/permissions.md`. The Phase 2
    // `SettingsManage` placeholder is replaced.
    // GeneralSettings
    /// Read the school's general settings row.
    SettingsGeneralRead,
    /// Update the school's general settings row.
    SettingsGeneralUpdate,
    /// Toggle the school's two-factor authentication.
    SettingsGeneralTwoFactorToggle,
    // Theme
    /// Create a theme.
    SettingsThemeCreate,
    /// Read a theme.
    SettingsThemeRead,
    /// Update a theme.
    SettingsThemeUpdate,
    /// Activate a theme.
    SettingsThemeActivate,
    /// Delete a theme.
    SettingsThemeDelete,
    /// Replicate a theme.
    SettingsThemeReplicate,
    /// Select the active theme for the school.
    SettingsThemeSelect,
    // Color (system)
    /// Create a color (system).
    SettingsColorCreate,
    /// Read a color.
    SettingsColorRead,
    /// Update a color (system).
    SettingsColorUpdate,
    /// Delete a color (system).
    SettingsColorDelete,
    // ColorTheme
    /// Create a color-theme binding.
    SettingsColorThemeCreate,
    /// Read a color-theme binding.
    SettingsColorThemeRead,
    /// Update a color-theme binding.
    SettingsColorThemeUpdate,
    /// Delete a color-theme binding.
    SettingsColorThemeDelete,
    // Language
    /// Add a language.
    SettingsLanguageAdd,
    /// Read a language.
    SettingsLanguageRead,
    /// Update a language.
    SettingsLanguageUpdate,
    /// Delete a language.
    SettingsLanguageDelete,
    /// Activate a language.
    SettingsLanguageActivate,
    /// Select the active language for the school.
    SettingsLanguageSelect,
    // LanguagePhrase
    /// Add a language phrase.
    SettingsLanguagePhraseAdd,
    /// Read a language phrase.
    SettingsLanguagePhraseRead,
    /// Update a language phrase.
    SettingsLanguagePhraseUpdate,
    /// Delete a language phrase.
    SettingsLanguagePhraseDelete,
    /// Translate a language phrase.
    SettingsLanguagePhraseTranslate,
    // BaseGroup / BaseSetup
    /// Add a base group.
    SettingsBaseGroupAdd,
    /// Read a base group.
    SettingsBaseGroupRead,
    /// Update a base group.
    SettingsBaseGroupUpdate,
    /// Delete a base group.
    SettingsBaseGroupDelete,
    /// Add a base setup.
    SettingsBaseSetupAdd,
    /// Read a base setup.
    SettingsBaseSetupRead,
    /// Update a base setup.
    SettingsBaseSetupUpdate,
    /// Delete a base setup.
    SettingsBaseSetupDelete,
    // DateFormat
    /// Add a date format.
    SettingsDateFormatAdd,
    /// Read a date format.
    SettingsDateFormatRead,
    /// Update a date format.
    SettingsDateFormatUpdate,
    /// Delete a date format.
    SettingsDateFormatDelete,
    /// Select the active date format.
    SettingsDateFormatSelect,
    // TimeZone
    /// Select the active time zone.
    SettingsTimeZoneSelect,
    // Session
    /// Select the active academic session.
    SettingsSessionSelect,
    // Style
    /// Create a style.
    SettingsStyleCreate,
    /// Read a style.
    SettingsStyleRead,
    /// Update a style.
    SettingsStyleUpdate,
    /// Activate a style.
    SettingsStyleActivate,
    /// Delete a style.
    SettingsStyleDelete,
    // Background
    /// Create a background setting.
    SettingsBackgroundCreate,
    /// Read a background setting.
    SettingsBackgroundRead,
    /// Update a background setting.
    SettingsBackgroundUpdate,
    /// Delete a background setting.
    SettingsBackgroundDelete,
    // Dashboard
    /// Create a dashboard setting.
    SettingsDashboardCreate,
    /// Read a dashboard setting.
    SettingsDashboardRead,
    /// Update a dashboard setting.
    SettingsDashboardUpdate,
    /// Delete a dashboard setting.
    SettingsDashboardDelete,
    // CustomLink
    /// Read the custom links bundle.
    SettingsCustomLinkRead,
    /// Update the custom links bundle.
    SettingsCustomLinkUpdate,
    /// Reset the custom links bundle to defaults.
    SettingsCustomLinkReset,
    // BehaviorRecord
    /// Read the behavior record settings.
    SettingsBehaviorRecordRead,
    /// Update the behavior record settings.
    SettingsBehaviorRecordUpdate,
    // SetupAdmin
    /// Add a setup admin entry.
    SettingsSetupAdminAdd,
    /// Read a setup admin entry.
    SettingsSetupAdminRead,
    /// Update a setup admin entry.
    SettingsSetupAdminUpdate,
    /// Delete a setup admin entry.
    SettingsSetupAdminDelete,

    // -- Operations (Phase 14 net-new) -----------------------------------
    // Phase 14 ships the full per-aggregate catalog per
    // `docs/specs/operations/permissions.md`. The Phase 2
    // `OperationsManage` placeholder is replaced.
    // Backup
    /// Create a backup.
    OperationsBackupCreate,
    /// Read a backup.
    OperationsBackupRead,
    /// Delete a backup.
    OperationsBackupDelete,
    /// Restore a backup.
    OperationsBackupRestore,
    /// Activate a backup.
    OperationsBackupActivate,
    /// Deactivate a backup.
    OperationsBackupDeactivate,
    // Job (system-tenant for the lifecycle commands)
    /// Schedule a job (system).
    OperationsJobSchedule,
    /// Read a job.
    OperationsJobRead,
    /// Cancel a job (system).
    OperationsJobCancel,
    /// Reserve a job (system).
    OperationsJobReserve,
    /// Complete a job (system).
    OperationsJobComplete,
    /// Fail a job (system).
    OperationsJobFail,
    /// Retry a job (system).
    OperationsJobRetry,
    /// Purge completed jobs.
    OperationsJobPurge,
    // FailedJob
    /// Read a failed job.
    OperationsFailedJobRead,
    /// Retry a failed job (system).
    OperationsFailedJobRetry,
    /// Delete a failed job.
    OperationsFailedJobDelete,
    /// Purge old failed jobs.
    OperationsFailedJobPurge,
    // SystemVersion
    /// Register a system version (system, build-time).
    OperationsVersionRegister,
    /// Read a system version.
    OperationsVersionRead,
    /// Update a system version (system).
    OperationsVersionUpdate,
    // VersionHistory
    /// Record a version history entry (system, build-time).
    OperationsVersionHistoryRecord,
    /// Read version history.
    OperationsVersionHistoryRead,
    // UserLog
    /// Record a user log entry (system).
    OperationsAuditRecord,
    /// Read user logs.
    OperationsAuditRead,
    // Maintenance
    /// Read the maintenance setting.
    OperationsMaintenanceRead,
    /// Configure the maintenance setting.
    OperationsMaintenanceConfigure,
    /// Enable maintenance mode.
    OperationsMaintenanceEnable,
    /// Disable maintenance mode.
    OperationsMaintenanceDisable,
    // Sidebar
    /// Create a sidebar entry.
    OperationsSidebarCreate,
    /// Read sidebar entries.
    OperationsSidebarRead,
    /// Update a sidebar entry.
    OperationsSidebarUpdate,
    /// Delete a sidebar entry.
    OperationsSidebarDelete,
    /// Reorder sidebar entries.
    OperationsSidebarReorder,
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
            Self::LibraryRead
            | Self::LibraryConfigure
            | Self::LibraryReport
            | Self::BookCategoryCreate
            | Self::BookCategoryRead
            | Self::BookCategoryUpdate
            | Self::BookCategoryDelete
            | Self::BookAdd
            | Self::BookRead
            | Self::BookUpdate
            | Self::BookDelete
            | Self::BookAdjustQuantity
            | Self::BookSearch
            | Self::MemberRegister
            | Self::MemberRead
            | Self::MemberUpdate
            | Self::MemberDelete
            | Self::MemberDeactivate
            | Self::MemberReactivate
            | Self::BookIssueIssue
            | Self::BookIssueRead
            | Self::BookIssueReturn
            | Self::BookIssueRenew
            | Self::BookIssueMarkLost
            | Self::BookIssueCalculateFine
            | Self::BookIssueWaiveFine => CapabilityDomain::Library,
            Self::CommunicationMessageCreate
            | Self::CommunicationMessageRead
            | Self::CommunicationMessageUpdate
            | Self::CommunicationMessageDelete => CapabilityDomain::Communication,
            Self::DocumentsFolderCreate
            | Self::DocumentsFolderRead
            | Self::DocumentsFolderUpdate
            | Self::DocumentsFolderDelete
            | Self::FormDownloadUpload
            | Self::FormDownloadUpdate
            | Self::FormDownloadDelete
            | Self::FormDownloadRead
            | Self::PostalDispatchCreate
            | Self::PostalDispatchUpdate
            | Self::PostalDispatchDelete
            | Self::PostalReceiveCreate
            | Self::PostalReceiveUpdate
            | Self::PostalReceiveDelete
            | Self::PostalRead => CapabilityDomain::Documents,
            Self::CmsPageCreate
            | Self::CmsPageRead
            | Self::CmsPageUpdate
            | Self::CmsPageDelete
            | Self::CmsPagePublish
            | Self::CmsPageArchive
            | Self::CmsNewsCreate
            | Self::CmsNewsRead
            | Self::CmsNewsUpdate
            | Self::CmsNewsDelete
            | Self::CmsNewsPublish
            | Self::CmsNewsUnpublish
            | Self::CmsNewsIncrementView
            | Self::CmsNewsCategoryCreate
            | Self::CmsNewsCategoryRead
            | Self::CmsNewsCategoryUpdate
            | Self::CmsNewsCategoryDelete
            | Self::CmsNewsCommentCreate
            | Self::CmsNewsCommentModerate
            | Self::CmsNewsCommentDelete
            | Self::CmsNewsCommentRead
            | Self::CmsNewsPageCreate
            | Self::CmsNewsPageRead
            | Self::CmsNewsPageUpdate
            | Self::CmsNewsPageDelete
            | Self::CmsNoticeBoardCreate
            | Self::CmsNoticeBoardRead
            | Self::CmsNoticeBoardUpdate
            | Self::CmsNoticeBoardDelete
            | Self::CmsNoticeBoardPublish
            | Self::CmsNoticeBoardUnpublish
            | Self::CmsTestimonialCreate
            | Self::CmsTestimonialRead
            | Self::CmsTestimonialUpdate
            | Self::CmsTestimonialDelete
            | Self::CmsHomeSliderCreate
            | Self::CmsHomeSliderRead
            | Self::CmsHomeSliderUpdate
            | Self::CmsHomeSliderDelete
            | Self::CmsSpeechSliderCreate
            | Self::CmsSpeechSliderRead
            | Self::CmsSpeechSliderUpdate
            | Self::CmsSpeechSliderDelete
            | Self::CmsContentCreate
            | Self::CmsContentRead
            | Self::CmsContentUpdate
            | Self::CmsContentDelete
            | Self::CmsContentTypeCreate
            | Self::CmsContentTypeRead
            | Self::CmsContentTypeUpdate
            | Self::CmsContentTypeDelete
            | Self::CmsContentShareListCreate
            | Self::CmsContentShareListRead
            | Self::CmsContentShareListUpdate
            | Self::CmsContentShareListDelete
            | Self::CmsContentShareListDispatch
            | Self::CmsContentShareListCancel
            | Self::CmsTeacherUploadContentCreate
            | Self::CmsTeacherUploadContentRead
            | Self::CmsTeacherUploadContentUpdate
            | Self::CmsTeacherUploadContentDelete
            | Self::CmsUploadContentCreate
            | Self::CmsUploadContentRead
            | Self::CmsUploadContentUpdate
            | Self::CmsUploadContentDelete
            | Self::CmsAboutPageCreate
            | Self::CmsAboutPageRead
            | Self::CmsAboutPageUpdate
            | Self::CmsAboutPageDelete
            | Self::CmsContactPageCreate
            | Self::CmsContactPageRead
            | Self::CmsContactPageUpdate
            | Self::CmsContactPageDelete
            | Self::CmsCoursePageCreate
            | Self::CmsCoursePageRead
            | Self::CmsCoursePageUpdate
            | Self::CmsCoursePageDelete
            | Self::CmsHomePageSettingConfigure
            | Self::CmsHomePageSettingRead
            | Self::CmsHomePageSettingDelete
            | Self::CmsFrontendPageCreate
            | Self::CmsFrontendPageRead
            | Self::CmsFrontendPageUpdate
            | Self::CmsFrontendPageDelete
            | Self::CmsRead => CapabilityDomain::Cms,
            Self::FacilitiesRoomCreate
            | Self::FacilitiesRoomRead
            | Self::FacilitiesRoomUpdate
            | Self::FacilitiesRoomDelete
            | Self::FacilitiesVehicleCreate
            | Self::FacilitiesVehicleRead
            | Self::FacilitiesVehicleUpdate
            | Self::FacilitiesVehicleDelete
            | Self::FacilitiesVehicleAssignDriver
            | Self::FacilitiesVehicleDeactivate
            | Self::FacilitiesRouteCreate
            | Self::FacilitiesRouteRead
            | Self::FacilitiesRouteUpdate
            | Self::FacilitiesRouteDelete
            | Self::FacilitiesRouteAddStop
            | Self::FacilitiesRouteUpdateStop
            | Self::FacilitiesRouteRemoveStop
            | Self::FacilitiesTransportAssignVehicle
            | Self::FacilitiesTransportUnassignVehicle
            | Self::FacilitiesTransportAssignStudent
            | Self::FacilitiesTransportUnassignStudent
            | Self::FacilitiesTransportRead
            | Self::FacilitiesDormitoryCreate
            | Self::FacilitiesDormitoryRead
            | Self::FacilitiesDormitoryUpdate
            | Self::FacilitiesDormitoryDelete
            | Self::FacilitiesRoomAssignStudent
            | Self::FacilitiesRoomUnassignStudent
            | Self::FacilitiesRoomTypeCreate
            | Self::FacilitiesRoomTypeRead
            | Self::FacilitiesRoomTypeUpdate
            | Self::FacilitiesRoomTypeDelete
            | Self::FacilitiesItemCategoryCreate
            | Self::FacilitiesItemCategoryRead
            | Self::FacilitiesItemCategoryUpdate
            | Self::FacilitiesItemCategoryDelete
            | Self::FacilitiesItemCreate
            | Self::FacilitiesItemRead
            | Self::FacilitiesItemUpdate
            | Self::FacilitiesItemDelete
            | Self::FacilitiesItemStoreCreate
            | Self::FacilitiesItemStoreRead
            | Self::FacilitiesItemStoreUpdate
            | Self::FacilitiesItemStoreDelete
            | Self::FacilitiesInventoryReceive
            | Self::FacilitiesInventoryUpdateReceive
            | Self::FacilitiesInventoryCancelReceive
            | Self::FacilitiesInventoryIssue
            | Self::FacilitiesInventoryUpdateIssue
            | Self::FacilitiesInventoryReturnIssued
            | Self::FacilitiesInventorySell
            | Self::FacilitiesInventoryUpdateSell
            | Self::FacilitiesInventoryCancelSell
            | Self::FacilitiesInventoryRefundSell
            | Self::FacilitiesInventoryRead
            | Self::FacilitiesSupplierCreate
            | Self::FacilitiesSupplierRead
            | Self::FacilitiesSupplierUpdate
            | Self::FacilitiesSupplierDelete
            | Self::FacilitiesSupplierDeactivate => CapabilityDomain::Facilities,
            Self::EventsCalendarCreate
            | Self::EventsCalendarRead
            | Self::EventsCalendarUpdate
            | Self::EventsCalendarDelete
            | Self::EventsEventCreate
            | Self::EventsEventRead
            | Self::EventsEventUpdate
            | Self::EventsEventDelete
            | Self::EventsEventPublish
            | Self::EventsHolidayCreate
            | Self::EventsHolidayRead
            | Self::EventsHolidayUpdate
            | Self::EventsHolidayDelete
            | Self::EventsWeekendCreate
            | Self::EventsWeekendRead
            | Self::EventsWeekendUpdate
            | Self::EventsWeekendDelete
            | Self::EventsWeekendConfigure
            | Self::EventsIncidentCreate
            | Self::EventsIncidentRead
            | Self::EventsIncidentUpdate
            | Self::EventsIncidentDelete
            | Self::EventsIncidentAssign
            | Self::EventsIncidentReassign
            | Self::EventsIncidentUnassign
            | Self::EventsIncidentComment
            | Self::EventsIncidentResolve
            | Self::EventsIncidentCommentDelete
            | Self::EventsCalendarSettingCreate
            | Self::EventsCalendarSettingRead
            | Self::EventsCalendarSettingUpdate
            | Self::EventsCalendarSettingEnable
            | Self::EventsCalendarSettingDisable
            | Self::EventsCalendarSettingDelete => CapabilityDomain::Events,
            Self::SettingsGeneralRead
            | Self::SettingsGeneralUpdate
            | Self::SettingsGeneralTwoFactorToggle
            | Self::SettingsThemeCreate
            | Self::SettingsThemeRead
            | Self::SettingsThemeUpdate
            | Self::SettingsThemeActivate
            | Self::SettingsThemeDelete
            | Self::SettingsThemeReplicate
            | Self::SettingsThemeSelect
            | Self::SettingsColorCreate
            | Self::SettingsColorRead
            | Self::SettingsColorUpdate
            | Self::SettingsColorDelete
            | Self::SettingsColorThemeCreate
            | Self::SettingsColorThemeRead
            | Self::SettingsColorThemeUpdate
            | Self::SettingsColorThemeDelete
            | Self::SettingsLanguageAdd
            | Self::SettingsLanguageRead
            | Self::SettingsLanguageUpdate
            | Self::SettingsLanguageDelete
            | Self::SettingsLanguageActivate
            | Self::SettingsLanguageSelect
            | Self::SettingsLanguagePhraseAdd
            | Self::SettingsLanguagePhraseRead
            | Self::SettingsLanguagePhraseUpdate
            | Self::SettingsLanguagePhraseDelete
            | Self::SettingsLanguagePhraseTranslate
            | Self::SettingsBaseGroupAdd
            | Self::SettingsBaseGroupRead
            | Self::SettingsBaseGroupUpdate
            | Self::SettingsBaseGroupDelete
            | Self::SettingsBaseSetupAdd
            | Self::SettingsBaseSetupRead
            | Self::SettingsBaseSetupUpdate
            | Self::SettingsBaseSetupDelete
            | Self::SettingsDateFormatAdd
            | Self::SettingsDateFormatRead
            | Self::SettingsDateFormatUpdate
            | Self::SettingsDateFormatDelete
            | Self::SettingsDateFormatSelect
            | Self::SettingsTimeZoneSelect
            | Self::SettingsSessionSelect
            | Self::SettingsStyleCreate
            | Self::SettingsStyleRead
            | Self::SettingsStyleUpdate
            | Self::SettingsStyleActivate
            | Self::SettingsStyleDelete
            | Self::SettingsBackgroundCreate
            | Self::SettingsBackgroundRead
            | Self::SettingsBackgroundUpdate
            | Self::SettingsBackgroundDelete
            | Self::SettingsDashboardCreate
            | Self::SettingsDashboardRead
            | Self::SettingsDashboardUpdate
            | Self::SettingsDashboardDelete
            | Self::SettingsCustomLinkRead
            | Self::SettingsCustomLinkUpdate
            | Self::SettingsCustomLinkReset
            | Self::SettingsBehaviorRecordRead
            | Self::SettingsBehaviorRecordUpdate
            | Self::SettingsSetupAdminAdd
            | Self::SettingsSetupAdminRead
            | Self::SettingsSetupAdminUpdate
            | Self::SettingsSetupAdminDelete => CapabilityDomain::Settings,
            Self::OperationsBackupCreate
            | Self::OperationsBackupRead
            | Self::OperationsBackupDelete
            | Self::OperationsBackupRestore
            | Self::OperationsBackupActivate
            | Self::OperationsBackupDeactivate
            | Self::OperationsJobSchedule
            | Self::OperationsJobRead
            | Self::OperationsJobCancel
            | Self::OperationsJobReserve
            | Self::OperationsJobComplete
            | Self::OperationsJobFail
            | Self::OperationsJobRetry
            | Self::OperationsJobPurge
            | Self::OperationsFailedJobRead
            | Self::OperationsFailedJobRetry
            | Self::OperationsFailedJobDelete
            | Self::OperationsFailedJobPurge
            | Self::OperationsVersionRegister
            | Self::OperationsVersionRead
            | Self::OperationsVersionUpdate
            | Self::OperationsVersionHistoryRecord
            | Self::OperationsVersionHistoryRead
            | Self::OperationsAuditRecord
            | Self::OperationsAuditRead
            | Self::OperationsMaintenanceRead
            | Self::OperationsMaintenanceConfigure
            | Self::OperationsMaintenanceEnable
            | Self::OperationsMaintenanceDisable
            | Self::OperationsSidebarCreate
            | Self::OperationsSidebarRead
            | Self::OperationsSidebarUpdate
            | Self::OperationsSidebarDelete
            | Self::OperationsSidebarReorder => CapabilityDomain::Operations,
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
            Self::BookAdd
            | Self::BookRead
            | Self::BookUpdate
            | Self::BookDelete
            | Self::BookAdjustQuantity
            | Self::BookSearch => "Book",
            Self::LibraryRead | Self::LibraryConfigure | Self::LibraryReport => "Library",
            Self::BookCategoryCreate
            | Self::BookCategoryRead
            | Self::BookCategoryUpdate
            | Self::BookCategoryDelete => "BookCategory",
            Self::MemberRegister
            | Self::MemberRead
            | Self::MemberUpdate
            | Self::MemberDelete
            | Self::MemberDeactivate
            | Self::MemberReactivate => "Member",
            Self::BookIssueIssue
            | Self::BookIssueRead
            | Self::BookIssueReturn
            | Self::BookIssueRenew
            | Self::BookIssueMarkLost
            | Self::BookIssueCalculateFine
            | Self::BookIssueWaiveFine => "BookIssue",
            Self::CommunicationMessageCreate
            | Self::CommunicationMessageRead
            | Self::CommunicationMessageUpdate
            | Self::CommunicationMessageDelete => "Message",
            Self::DocumentsFolderCreate
            | Self::DocumentsFolderRead
            | Self::DocumentsFolderUpdate
            | Self::DocumentsFolderDelete => "Folder",
            Self::FormDownloadUpload
            | Self::FormDownloadUpdate
            | Self::FormDownloadDelete
            | Self::FormDownloadRead => "FormDownload",
            Self::PostalDispatchCreate
            | Self::PostalDispatchUpdate
            | Self::PostalDispatchDelete => "PostalDispatch",
            Self::PostalReceiveCreate | Self::PostalReceiveUpdate | Self::PostalReceiveDelete => {
                "PostalReceive"
            }
            Self::PostalRead => "Postal",
            Self::CmsPageCreate
            | Self::CmsPageRead
            | Self::CmsPageUpdate
            | Self::CmsPageDelete
            | Self::CmsPagePublish
            | Self::CmsPageArchive => "Page",
            Self::CmsNewsCreate
            | Self::CmsNewsRead
            | Self::CmsNewsUpdate
            | Self::CmsNewsDelete
            | Self::CmsNewsPublish
            | Self::CmsNewsUnpublish
            | Self::CmsNewsIncrementView => "News",
            Self::CmsNewsCategoryCreate
            | Self::CmsNewsCategoryRead
            | Self::CmsNewsCategoryUpdate
            | Self::CmsNewsCategoryDelete => "NewsCategory",
            Self::CmsNewsCommentCreate
            | Self::CmsNewsCommentModerate
            | Self::CmsNewsCommentDelete
            | Self::CmsNewsCommentRead => "NewsComment",
            Self::CmsNewsPageCreate
            | Self::CmsNewsPageRead
            | Self::CmsNewsPageUpdate
            | Self::CmsNewsPageDelete => "NewsPage",
            Self::CmsNoticeBoardCreate
            | Self::CmsNoticeBoardRead
            | Self::CmsNoticeBoardUpdate
            | Self::CmsNoticeBoardDelete
            | Self::CmsNoticeBoardPublish
            | Self::CmsNoticeBoardUnpublish => "NoticeBoard",
            Self::CmsTestimonialCreate
            | Self::CmsTestimonialRead
            | Self::CmsTestimonialUpdate
            | Self::CmsTestimonialDelete => "Testimonial",
            Self::CmsHomeSliderCreate
            | Self::CmsHomeSliderRead
            | Self::CmsHomeSliderUpdate
            | Self::CmsHomeSliderDelete => "HomeSlider",
            Self::CmsSpeechSliderCreate
            | Self::CmsSpeechSliderRead
            | Self::CmsSpeechSliderUpdate
            | Self::CmsSpeechSliderDelete => "SpeechSlider",
            Self::CmsContentCreate
            | Self::CmsContentRead
            | Self::CmsContentUpdate
            | Self::CmsContentDelete => "Content",
            Self::CmsContentTypeCreate
            | Self::CmsContentTypeRead
            | Self::CmsContentTypeUpdate
            | Self::CmsContentTypeDelete => "ContentType",
            Self::CmsContentShareListCreate
            | Self::CmsContentShareListRead
            | Self::CmsContentShareListUpdate
            | Self::CmsContentShareListDelete
            | Self::CmsContentShareListDispatch
            | Self::CmsContentShareListCancel => "ContentShareList",
            Self::CmsTeacherUploadContentCreate
            | Self::CmsTeacherUploadContentRead
            | Self::CmsTeacherUploadContentUpdate
            | Self::CmsTeacherUploadContentDelete => "TeacherUploadContent",
            Self::CmsUploadContentCreate
            | Self::CmsUploadContentRead
            | Self::CmsUploadContentUpdate
            | Self::CmsUploadContentDelete => "UploadContent",
            Self::CmsAboutPageCreate
            | Self::CmsAboutPageRead
            | Self::CmsAboutPageUpdate
            | Self::CmsAboutPageDelete => "AboutPage",
            Self::CmsContactPageCreate
            | Self::CmsContactPageRead
            | Self::CmsContactPageUpdate
            | Self::CmsContactPageDelete => "ContactPage",
            Self::CmsCoursePageCreate
            | Self::CmsCoursePageRead
            | Self::CmsCoursePageUpdate
            | Self::CmsCoursePageDelete => "CoursePage",
            Self::CmsHomePageSettingConfigure
            | Self::CmsHomePageSettingRead
            | Self::CmsHomePageSettingDelete => "HomePageSetting",
            Self::CmsFrontendPageCreate
            | Self::CmsFrontendPageRead
            | Self::CmsFrontendPageUpdate
            | Self::CmsFrontendPageDelete => "FrontendPage",
            Self::CmsRead => "Cms",
            Self::FacilitiesRoomCreate
            | Self::FacilitiesRoomRead
            | Self::FacilitiesRoomUpdate
            | Self::FacilitiesRoomDelete
            | Self::FacilitiesRoomAssignStudent
            | Self::FacilitiesRoomUnassignStudent => "Room",
            Self::FacilitiesVehicleCreate
            | Self::FacilitiesVehicleRead
            | Self::FacilitiesVehicleUpdate
            | Self::FacilitiesVehicleDelete
            | Self::FacilitiesVehicleAssignDriver
            | Self::FacilitiesVehicleDeactivate => "Vehicle",
            Self::FacilitiesRouteCreate
            | Self::FacilitiesRouteRead
            | Self::FacilitiesRouteUpdate
            | Self::FacilitiesRouteDelete
            | Self::FacilitiesRouteAddStop
            | Self::FacilitiesRouteUpdateStop
            | Self::FacilitiesRouteRemoveStop => "Route",
            Self::FacilitiesTransportAssignVehicle
            | Self::FacilitiesTransportUnassignVehicle
            | Self::FacilitiesTransportAssignStudent
            | Self::FacilitiesTransportUnassignStudent
            | Self::FacilitiesTransportRead => "Transport",
            Self::FacilitiesDormitoryCreate
            | Self::FacilitiesDormitoryRead
            | Self::FacilitiesDormitoryUpdate
            | Self::FacilitiesDormitoryDelete => "Dormitory",
            Self::FacilitiesRoomTypeCreate
            | Self::FacilitiesRoomTypeRead
            | Self::FacilitiesRoomTypeUpdate
            | Self::FacilitiesRoomTypeDelete => "RoomType",
            Self::FacilitiesItemCategoryCreate
            | Self::FacilitiesItemCategoryRead
            | Self::FacilitiesItemCategoryUpdate
            | Self::FacilitiesItemCategoryDelete => "ItemCategory",
            Self::FacilitiesItemCreate
            | Self::FacilitiesItemRead
            | Self::FacilitiesItemUpdate
            | Self::FacilitiesItemDelete => "Item",
            Self::FacilitiesItemStoreCreate
            | Self::FacilitiesItemStoreRead
            | Self::FacilitiesItemStoreUpdate
            | Self::FacilitiesItemStoreDelete => "ItemStore",
            Self::FacilitiesInventoryReceive
            | Self::FacilitiesInventoryUpdateReceive
            | Self::FacilitiesInventoryCancelReceive
            | Self::FacilitiesInventoryIssue
            | Self::FacilitiesInventoryUpdateIssue
            | Self::FacilitiesInventoryReturnIssued
            | Self::FacilitiesInventorySell
            | Self::FacilitiesInventoryUpdateSell
            | Self::FacilitiesInventoryCancelSell
            | Self::FacilitiesInventoryRefundSell
            | Self::FacilitiesInventoryRead => "Inventory",
            Self::FacilitiesSupplierCreate
            | Self::FacilitiesSupplierRead
            | Self::FacilitiesSupplierUpdate
            | Self::FacilitiesSupplierDelete
            | Self::FacilitiesSupplierDeactivate => "Supplier",
            Self::EventsCalendarCreate
            | Self::EventsCalendarRead
            | Self::EventsCalendarUpdate
            | Self::EventsCalendarDelete => "Calendar",
            Self::EventsEventCreate
            | Self::EventsEventRead
            | Self::EventsEventUpdate
            | Self::EventsEventDelete
            | Self::EventsEventPublish => "Event",
            Self::EventsHolidayCreate
            | Self::EventsHolidayRead
            | Self::EventsHolidayUpdate
            | Self::EventsHolidayDelete => "Holiday",
            Self::EventsWeekendCreate
            | Self::EventsWeekendRead
            | Self::EventsWeekendUpdate
            | Self::EventsWeekendDelete
            | Self::EventsWeekendConfigure => "Weekend",
            Self::EventsIncidentCreate
            | Self::EventsIncidentRead
            | Self::EventsIncidentUpdate
            | Self::EventsIncidentDelete
            | Self::EventsIncidentAssign
            | Self::EventsIncidentReassign
            | Self::EventsIncidentUnassign
            | Self::EventsIncidentComment
            | Self::EventsIncidentResolve => "Incident",
            Self::EventsIncidentCommentDelete => "IncidentComment",
            Self::EventsCalendarSettingCreate
            | Self::EventsCalendarSettingRead
            | Self::EventsCalendarSettingUpdate
            | Self::EventsCalendarSettingEnable
            | Self::EventsCalendarSettingDisable
            | Self::EventsCalendarSettingDelete => "CalendarSetting",
            Self::SettingsGeneralRead
            | Self::SettingsGeneralUpdate
            | Self::SettingsGeneralTwoFactorToggle => "General",
            Self::SettingsThemeCreate
            | Self::SettingsThemeRead
            | Self::SettingsThemeUpdate
            | Self::SettingsThemeActivate
            | Self::SettingsThemeDelete
            | Self::SettingsThemeReplicate
            | Self::SettingsThemeSelect => "Theme",
            Self::SettingsColorCreate
            | Self::SettingsColorRead
            | Self::SettingsColorUpdate
            | Self::SettingsColorDelete => "Color",
            Self::SettingsColorThemeCreate
            | Self::SettingsColorThemeRead
            | Self::SettingsColorThemeUpdate
            | Self::SettingsColorThemeDelete => "ColorTheme",
            Self::SettingsLanguageAdd
            | Self::SettingsLanguageRead
            | Self::SettingsLanguageUpdate
            | Self::SettingsLanguageDelete
            | Self::SettingsLanguageActivate
            | Self::SettingsLanguageSelect => "Language",
            Self::SettingsLanguagePhraseAdd
            | Self::SettingsLanguagePhraseRead
            | Self::SettingsLanguagePhraseUpdate
            | Self::SettingsLanguagePhraseDelete
            | Self::SettingsLanguagePhraseTranslate => "LanguagePhrase",
            Self::SettingsBaseGroupAdd
            | Self::SettingsBaseGroupRead
            | Self::SettingsBaseGroupUpdate
            | Self::SettingsBaseGroupDelete => "BaseGroup",
            Self::SettingsBaseSetupAdd
            | Self::SettingsBaseSetupRead
            | Self::SettingsBaseSetupUpdate
            | Self::SettingsBaseSetupDelete => "BaseSetup",
            Self::SettingsDateFormatAdd
            | Self::SettingsDateFormatRead
            | Self::SettingsDateFormatUpdate
            | Self::SettingsDateFormatDelete
            | Self::SettingsDateFormatSelect => "DateFormat",
            Self::SettingsTimeZoneSelect => "TimeZone",
            Self::SettingsSessionSelect => "Session",
            Self::SettingsStyleCreate
            | Self::SettingsStyleRead
            | Self::SettingsStyleUpdate
            | Self::SettingsStyleActivate
            | Self::SettingsStyleDelete => "Style",
            Self::SettingsBackgroundCreate
            | Self::SettingsBackgroundRead
            | Self::SettingsBackgroundUpdate
            | Self::SettingsBackgroundDelete => "Background",
            Self::SettingsDashboardCreate
            | Self::SettingsDashboardRead
            | Self::SettingsDashboardUpdate
            | Self::SettingsDashboardDelete => "Dashboard",
            Self::SettingsCustomLinkRead
            | Self::SettingsCustomLinkUpdate
            | Self::SettingsCustomLinkReset => "CustomLink",
            Self::SettingsBehaviorRecordRead | Self::SettingsBehaviorRecordUpdate => {
                "BehaviorRecord"
            }
            Self::SettingsSetupAdminAdd
            | Self::SettingsSetupAdminRead
            | Self::SettingsSetupAdminUpdate
            | Self::SettingsSetupAdminDelete => "SetupAdmin",
            Self::OperationsBackupCreate
            | Self::OperationsBackupRead
            | Self::OperationsBackupDelete
            | Self::OperationsBackupRestore
            | Self::OperationsBackupActivate
            | Self::OperationsBackupDeactivate => "Backup",
            Self::OperationsJobSchedule
            | Self::OperationsJobRead
            | Self::OperationsJobCancel
            | Self::OperationsJobReserve
            | Self::OperationsJobComplete
            | Self::OperationsJobFail
            | Self::OperationsJobRetry
            | Self::OperationsJobPurge => "Job",
            Self::OperationsFailedJobRead
            | Self::OperationsFailedJobRetry
            | Self::OperationsFailedJobDelete
            | Self::OperationsFailedJobPurge => "FailedJob",
            Self::OperationsVersionRegister
            | Self::OperationsVersionRead
            | Self::OperationsVersionUpdate => "Version",
            Self::OperationsVersionHistoryRecord | Self::OperationsVersionHistoryRead => {
                "VersionHistory"
            }
            Self::OperationsAuditRecord | Self::OperationsAuditRead => "Audit",
            Self::OperationsMaintenanceRead
            | Self::OperationsMaintenanceConfigure
            | Self::OperationsMaintenanceEnable
            | Self::OperationsMaintenanceDisable => "Maintenance",
            Self::OperationsSidebarCreate
            | Self::OperationsSidebarRead
            | Self::OperationsSidebarUpdate
            | Self::OperationsSidebarDelete
            | Self::OperationsSidebarReorder => "Sidebar",
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
            | Self::CommunicationMessageCreate
            | Self::DocumentsFolderCreate
            | Self::PostalDispatchCreate
            | Self::PostalReceiveCreate
            | Self::CmsPageCreate
            | Self::CmsNewsCreate
            | Self::CmsNewsCategoryCreate
            | Self::CmsNewsPageCreate
            | Self::CmsNoticeBoardCreate
            | Self::CmsTestimonialCreate
            | Self::CmsHomeSliderCreate
            | Self::CmsSpeechSliderCreate
            | Self::CmsContentCreate
            | Self::CmsContentTypeCreate
            | Self::CmsContentShareListCreate
            | Self::CmsTeacherUploadContentCreate
            | Self::CmsUploadContentCreate
            | Self::CmsAboutPageCreate
            | Self::CmsContactPageCreate
            | Self::CmsCoursePageCreate
            | Self::CmsFrontendPageCreate
            | Self::CmsNewsCommentCreate
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
            | Self::BookRead
            | Self::BookCategoryRead
            | Self::MemberRead
            | Self::BookIssueRead
            | Self::CommunicationMessageRead
            | Self::DocumentsFolderRead
            | Self::FormDownloadRead
            | Self::PostalRead
            | Self::CmsPageRead
            | Self::FacilitiesRoomRead
            | Self::EventsCalendarRead
            | Self::AttendanceStudentRead
            | Self::AttendanceSubjectRead
            | Self::AttendanceStaffRead
            | Self::AttendanceExamRead
            | Self::AttendanceImportRead
            | Self::AttendanceReportRead
            | Self::CmsNewsRead
            | Self::CmsNewsCategoryRead
            | Self::CmsNewsCommentRead
            | Self::CmsNewsPageRead
            | Self::CmsNoticeBoardRead
            | Self::CmsTestimonialRead
            | Self::CmsHomeSliderRead
            | Self::CmsSpeechSliderRead
            | Self::CmsContentRead
            | Self::CmsContentTypeRead
            | Self::CmsContentShareListRead
            | Self::CmsTeacherUploadContentRead
            | Self::CmsUploadContentRead
            | Self::CmsAboutPageRead
            | Self::CmsContactPageRead
            | Self::CmsCoursePageRead
            | Self::CmsHomePageSettingRead
            | Self::CmsFrontendPageRead
            | Self::CmsRead => "Read",
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
            | Self::BookUpdate
            | Self::BookCategoryUpdate
            | Self::MemberUpdate
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
            | Self::CommunicationMessageUpdate
            | Self::DocumentsFolderUpdate
            | Self::FormDownloadUpdate
            | Self::PostalDispatchUpdate
            | Self::PostalReceiveUpdate
            | Self::FacilitiesRoomUpdate
            | Self::EventsCalendarUpdate
            | Self::AttendanceStudentUpdate
            | Self::AttendanceSubjectUpdate
            | Self::AttendanceStaffUpdate
            | Self::AttendanceExamUpdate
            | Self::AttendanceImportUpdate
            | Self::CmsPageUpdate
            | Self::CmsNewsUpdate
            | Self::CmsNewsCategoryUpdate
            | Self::CmsNewsPageUpdate
            | Self::CmsNoticeBoardUpdate
            | Self::CmsTestimonialUpdate
            | Self::CmsHomeSliderUpdate
            | Self::CmsSpeechSliderUpdate
            | Self::CmsContentUpdate
            | Self::CmsContentTypeUpdate
            | Self::CmsContentShareListUpdate
            | Self::CmsTeacherUploadContentUpdate
            | Self::CmsUploadContentUpdate
            | Self::CmsAboutPageUpdate
            | Self::CmsContactPageUpdate
            | Self::CmsCoursePageUpdate
            | Self::CmsFrontendPageUpdate => "Update",
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
            | Self::CommunicationMessageDelete
            | Self::DocumentsFolderDelete
            | Self::FormDownloadDelete
            | Self::PostalDispatchDelete
            | Self::PostalReceiveDelete
            | Self::FacilitiesRoomDelete
            | Self::EventsCalendarDelete
            | Self::AttendanceStudentDelete
            | Self::AttendanceSubjectDelete
            | Self::AttendanceStaffDelete
            | Self::AttendanceExamDelete
            | Self::AttendanceImportDelete
            | Self::BookDelete
            | Self::BookCategoryDelete
            | Self::MemberDelete
            | Self::CmsPageDelete
            | Self::CmsNewsDelete
            | Self::CmsNewsCategoryDelete
            | Self::CmsNewsCommentDelete
            | Self::CmsNewsPageDelete
            | Self::CmsNoticeBoardDelete
            | Self::CmsTestimonialDelete
            | Self::CmsHomeSliderDelete
            | Self::CmsSpeechSliderDelete
            | Self::CmsContentDelete
            | Self::CmsContentTypeDelete
            | Self::CmsContentShareListDelete
            | Self::CmsTeacherUploadContentDelete
            | Self::CmsUploadContentDelete
            | Self::CmsAboutPageDelete
            | Self::CmsContactPageDelete
            | Self::CmsCoursePageDelete
            | Self::CmsHomePageSettingDelete
            | Self::CmsFrontendPageDelete => "Delete",
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
            Self::SettingsGeneralRead | Self::SettingsGeneralUpdate => "General",
            Self::SettingsGeneralTwoFactorToggle => "Toggle",
            Self::SettingsThemeCreate
            | Self::SettingsThemeRead
            | Self::SettingsThemeUpdate
            | Self::SettingsThemeActivate
            | Self::SettingsThemeDelete
            | Self::SettingsThemeReplicate
            | Self::SettingsThemeSelect => "Theme",
            Self::SettingsColorCreate
            | Self::SettingsColorRead
            | Self::SettingsColorUpdate
            | Self::SettingsColorDelete => "Color",
            Self::SettingsColorThemeCreate
            | Self::SettingsColorThemeRead
            | Self::SettingsColorThemeUpdate
            | Self::SettingsColorThemeDelete => "ColorTheme",
            Self::SettingsLanguageAdd
            | Self::SettingsLanguageRead
            | Self::SettingsLanguageUpdate
            | Self::SettingsLanguageDelete
            | Self::SettingsLanguageActivate
            | Self::SettingsLanguageSelect => "Language",
            Self::SettingsLanguagePhraseAdd
            | Self::SettingsLanguagePhraseRead
            | Self::SettingsLanguagePhraseUpdate
            | Self::SettingsLanguagePhraseDelete
            | Self::SettingsLanguagePhraseTranslate => "LanguagePhrase",
            Self::SettingsBaseGroupAdd
            | Self::SettingsBaseGroupRead
            | Self::SettingsBaseGroupUpdate
            | Self::SettingsBaseGroupDelete => "BaseGroup",
            Self::SettingsBaseSetupAdd
            | Self::SettingsBaseSetupRead
            | Self::SettingsBaseSetupUpdate
            | Self::SettingsBaseSetupDelete => "BaseSetup",
            Self::SettingsDateFormatAdd
            | Self::SettingsDateFormatRead
            | Self::SettingsDateFormatUpdate
            | Self::SettingsDateFormatDelete
            | Self::SettingsDateFormatSelect => "DateFormat",
            Self::SettingsTimeZoneSelect => "TimeZone",
            Self::SettingsSessionSelect => "Session",
            Self::SettingsStyleCreate
            | Self::SettingsStyleRead
            | Self::SettingsStyleUpdate
            | Self::SettingsStyleActivate
            | Self::SettingsStyleDelete => "Style",
            Self::SettingsBackgroundCreate
            | Self::SettingsBackgroundRead
            | Self::SettingsBackgroundUpdate
            | Self::SettingsBackgroundDelete => "Background",
            Self::SettingsDashboardCreate
            | Self::SettingsDashboardRead
            | Self::SettingsDashboardUpdate
            | Self::SettingsDashboardDelete => "Dashboard",
            Self::SettingsCustomLinkRead
            | Self::SettingsCustomLinkUpdate
            | Self::SettingsCustomLinkReset => "CustomLink",
            Self::SettingsBehaviorRecordRead | Self::SettingsBehaviorRecordUpdate => {
                "BehaviorRecord"
            }
            Self::SettingsSetupAdminAdd
            | Self::SettingsSetupAdminRead
            | Self::SettingsSetupAdminUpdate
            | Self::SettingsSetupAdminDelete => "SetupAdmin",
            Self::OperationsBackupCreate
            | Self::OperationsBackupRead
            | Self::OperationsBackupDelete
            | Self::OperationsBackupRestore
            | Self::OperationsBackupActivate
            | Self::OperationsBackupDeactivate => "Backup",
            Self::OperationsJobSchedule
            | Self::OperationsJobRead
            | Self::OperationsJobCancel
            | Self::OperationsJobReserve
            | Self::OperationsJobComplete
            | Self::OperationsJobFail
            | Self::OperationsJobRetry
            | Self::OperationsJobPurge => "Job",
            Self::OperationsFailedJobRead
            | Self::OperationsFailedJobRetry
            | Self::OperationsFailedJobDelete
            | Self::OperationsFailedJobPurge => "FailedJob",
            Self::OperationsVersionRegister
            | Self::OperationsVersionRead
            | Self::OperationsVersionUpdate => "Version",
            Self::OperationsVersionHistoryRecord | Self::OperationsVersionHistoryRead => {
                "VersionHistory"
            }
            Self::OperationsAuditRecord | Self::OperationsAuditRead => "Audit",
            Self::OperationsMaintenanceRead
            | Self::OperationsMaintenanceConfigure
            | Self::OperationsMaintenanceEnable
            | Self::OperationsMaintenanceDisable => "Maintenance",
            Self::OperationsSidebarCreate
            | Self::OperationsSidebarRead
            | Self::OperationsSidebarUpdate
            | Self::OperationsSidebarDelete
            | Self::OperationsSidebarReorder => "Sidebar",
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
            Self::FormDownloadUpload => "Upload",
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
            // -- Library (Phase 9) specific action mappings --
            Self::LibraryConfigure => "Configure",
            Self::LibraryReport => "Report",
            Self::BookAdd => "Add",
            Self::BookSearch => "Search",
            Self::BookAdjustQuantity => "AdjustQuantity",
            Self::BookCategoryCreate => "Create",
            Self::MemberRegister => "Register",
            Self::MemberDeactivate => "Deactivate",
            Self::MemberReactivate => "Reactivate",
            Self::BookIssueIssue => "Issue",
            Self::BookIssueReturn => "Return",
            Self::BookIssueRenew => "Renew",
            Self::BookIssueMarkLost => "MarkLost",
            Self::BookIssueCalculateFine => "CalculateFine",
            Self::BookIssueWaiveFine => "WaiveFine",
            Self::LibraryRead => "Read",
            // -- Facilities (Phase 8) -- generic Create/Read/Update/Delete
            Self::FacilitiesVehicleCreate
            | Self::FacilitiesRouteCreate
            | Self::FacilitiesDormitoryCreate
            | Self::FacilitiesRoomTypeCreate
            | Self::FacilitiesItemCategoryCreate
            | Self::FacilitiesItemCreate
            | Self::FacilitiesItemStoreCreate
            | Self::FacilitiesSupplierCreate => "Create",
            Self::FacilitiesVehicleRead
            | Self::FacilitiesRouteRead
            | Self::FacilitiesTransportRead
            | Self::FacilitiesDormitoryRead
            | Self::FacilitiesRoomTypeRead
            | Self::FacilitiesItemCategoryRead
            | Self::FacilitiesItemRead
            | Self::FacilitiesItemStoreRead
            | Self::FacilitiesInventoryRead
            | Self::FacilitiesSupplierRead => "Read",
            Self::FacilitiesVehicleUpdate
            | Self::FacilitiesRouteUpdate
            | Self::FacilitiesDormitoryUpdate
            | Self::FacilitiesRoomTypeUpdate
            | Self::FacilitiesItemCategoryUpdate
            | Self::FacilitiesItemUpdate
            | Self::FacilitiesItemStoreUpdate
            | Self::FacilitiesSupplierUpdate => "Update",
            Self::FacilitiesVehicleDelete
            | Self::FacilitiesRouteDelete
            | Self::FacilitiesDormitoryDelete
            | Self::FacilitiesRoomTypeDelete
            | Self::FacilitiesItemCategoryDelete
            | Self::FacilitiesItemDelete
            | Self::FacilitiesItemStoreDelete
            | Self::FacilitiesSupplierDelete => "Delete",
            // -- Facilities specific verbs --
            Self::FacilitiesVehicleAssignDriver => "AssignDriver",
            Self::FacilitiesVehicleDeactivate => "Deactivate",
            Self::FacilitiesRouteAddStop => "AddStop",
            Self::FacilitiesRouteUpdateStop => "UpdateStop",
            Self::FacilitiesRouteRemoveStop => "RemoveStop",
            Self::FacilitiesTransportAssignVehicle => "AssignVehicle",
            Self::FacilitiesTransportUnassignVehicle => "UnassignVehicle",
            Self::FacilitiesTransportAssignStudent => "AssignStudent",
            Self::FacilitiesTransportUnassignStudent => "UnassignStudent",
            Self::FacilitiesRoomAssignStudent => "AssignStudent",
            Self::FacilitiesRoomUnassignStudent => "UnassignStudent",
            Self::FacilitiesInventoryReceive => "Receive",
            Self::FacilitiesInventoryUpdateReceive => "UpdateReceive",
            Self::FacilitiesInventoryCancelReceive => "CancelReceive",
            Self::FacilitiesInventoryIssue => "Issue",
            Self::FacilitiesInventoryUpdateIssue => "UpdateIssue",
            Self::FacilitiesInventoryReturnIssued => "ReturnIssued",
            Self::FacilitiesInventorySell => "Sell",
            Self::FacilitiesInventoryUpdateSell => "UpdateSell",
            Self::FacilitiesInventoryCancelSell => "CancelSell",
            Self::FacilitiesInventoryRefundSell => "RefundSell",
            Self::FacilitiesSupplierDeactivate => "Deactivate",
            // -- CMS (Phase 12) specific verbs --
            Self::CmsPagePublish => "Publish",
            Self::CmsPageArchive => "Archive",
            Self::CmsNewsPublish => "Publish",
            Self::CmsNewsUnpublish => "Unpublish",
            Self::CmsNewsIncrementView => "IncrementView",
            Self::CmsNewsCommentModerate => "Moderate",
            Self::CmsNoticeBoardPublish => "Publish",
            Self::CmsNoticeBoardUnpublish => "Unpublish",
            Self::CmsContentShareListDispatch => "Dispatch",
            Self::CmsContentShareListCancel => "Cancel",
            Self::CmsHomePageSettingConfigure => "Configure",
            // Phase 13 Events domain net-new
            Self::EventsEventCreate
            | Self::EventsHolidayCreate
            | Self::EventsWeekendCreate
            | Self::EventsIncidentCreate
            | Self::EventsCalendarSettingCreate => "Create",
            Self::EventsEventRead
            | Self::EventsHolidayRead
            | Self::EventsWeekendRead
            | Self::EventsIncidentRead
            | Self::EventsCalendarSettingRead => "Read",
            Self::EventsEventUpdate
            | Self::EventsHolidayUpdate
            | Self::EventsWeekendUpdate
            | Self::EventsIncidentUpdate
            | Self::EventsCalendarSettingUpdate => "Update",
            Self::EventsEventDelete
            | Self::EventsHolidayDelete
            | Self::EventsWeekendDelete
            | Self::EventsIncidentDelete
            | Self::EventsIncidentCommentDelete
            | Self::EventsCalendarSettingDelete => "Delete",
            Self::EventsEventPublish => "Publish",
            Self::EventsWeekendConfigure => "Configure",
            Self::EventsIncidentAssign => "Assign",
            Self::EventsIncidentReassign => "Reassign",
            Self::EventsIncidentUnassign => "Unassign",
            Self::EventsIncidentComment => "Comment",
            Self::EventsIncidentResolve => "Resolve",
            Self::EventsCalendarSettingEnable => "Enable",
            Self::EventsCalendarSettingDisable => "Disable",
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
            Self::LibraryRead => "Library.Library.Read",
            Self::LibraryConfigure => "Library.Library.Configure",
            Self::LibraryReport => "Library.Library.Report",
            Self::BookCategoryCreate => "Library.BookCategory.Create",
            Self::BookCategoryRead => "Library.BookCategory.Read",
            Self::BookCategoryUpdate => "Library.BookCategory.Update",
            Self::BookCategoryDelete => "Library.BookCategory.Delete",
            Self::BookAdd => "Library.Book.Add",
            Self::BookRead => "Library.Book.Read",
            Self::BookUpdate => "Library.Book.Update",
            Self::BookDelete => "Library.Book.Delete",
            Self::BookAdjustQuantity => "Library.Book.AdjustQuantity",
            Self::BookSearch => "Library.Book.Search",
            Self::MemberRegister => "Library.Member.Register",
            Self::MemberRead => "Library.Member.Read",
            Self::MemberUpdate => "Library.Member.Update",
            Self::MemberDelete => "Library.Member.Delete",
            Self::MemberDeactivate => "Library.Member.Deactivate",
            Self::MemberReactivate => "Library.Member.Reactivate",
            Self::BookIssueIssue => "Library.BookIssue.Issue",
            Self::BookIssueRead => "Library.BookIssue.Read",
            Self::BookIssueReturn => "Library.BookIssue.Return",
            Self::BookIssueRenew => "Library.BookIssue.Renew",
            Self::BookIssueMarkLost => "Library.BookIssue.MarkLost",
            Self::BookIssueCalculateFine => "Library.BookIssue.CalculateFine",
            Self::BookIssueWaiveFine => "Library.BookIssue.WaiveFine",
            Self::CommunicationMessageCreate => "Communication.Message.Create",
            Self::CommunicationMessageRead => "Communication.Message.Read",
            Self::CommunicationMessageUpdate => "Communication.Message.Update",
            Self::CommunicationMessageDelete => "Communication.Message.Delete",
            Self::DocumentsFolderCreate => "Documents.Folder.Create",
            Self::DocumentsFolderRead => "Documents.Folder.Read",
            Self::DocumentsFolderUpdate => "Documents.Folder.Update",
            Self::DocumentsFolderDelete => "Documents.Folder.Delete",
            Self::FormDownloadUpload => "Documents.FormDownload.Upload",
            Self::FormDownloadUpdate => "Documents.FormDownload.Update",
            Self::FormDownloadDelete => "Documents.FormDownload.Delete",
            Self::FormDownloadRead => "Documents.FormDownload.Read",
            Self::PostalDispatchCreate => "Documents.PostalDispatch.Create",
            Self::PostalDispatchUpdate => "Documents.PostalDispatch.Update",
            Self::PostalDispatchDelete => "Documents.PostalDispatch.Delete",
            Self::PostalReceiveCreate => "Documents.PostalReceive.Create",
            Self::PostalReceiveUpdate => "Documents.PostalReceive.Update",
            Self::PostalReceiveDelete => "Documents.PostalReceive.Delete",
            Self::PostalRead => "Documents.Postal.Read",
            Self::CmsPageCreate => "Cms.Page.Create",
            Self::CmsPageRead => "Cms.Page.Read",
            Self::CmsPageUpdate => "Cms.Page.Update",
            Self::CmsPageDelete => "Cms.Page.Delete",
            Self::CmsPagePublish => "Cms.Page.Publish",
            Self::CmsPageArchive => "Cms.Page.Archive",
            Self::CmsNewsCreate => "Cms.News.Create",
            Self::CmsNewsRead => "Cms.News.Read",
            Self::CmsNewsUpdate => "Cms.News.Update",
            Self::CmsNewsDelete => "Cms.News.Delete",
            Self::CmsNewsPublish => "Cms.News.Publish",
            Self::CmsNewsUnpublish => "Cms.News.Unpublish",
            Self::CmsNewsIncrementView => "Cms.News.IncrementView",
            Self::CmsNewsCategoryCreate => "Cms.NewsCategory.Create",
            Self::CmsNewsCategoryRead => "Cms.NewsCategory.Read",
            Self::CmsNewsCategoryUpdate => "Cms.NewsCategory.Update",
            Self::CmsNewsCategoryDelete => "Cms.NewsCategory.Delete",
            Self::CmsNewsCommentCreate => "Cms.NewsComment.Create",
            Self::CmsNewsCommentModerate => "Cms.NewsComment.Moderate",
            Self::CmsNewsCommentDelete => "Cms.NewsComment.Delete",
            Self::CmsNewsCommentRead => "Cms.NewsComment.Read",
            Self::CmsNewsPageCreate => "Cms.NewsPage.Create",
            Self::CmsNewsPageRead => "Cms.NewsPage.Read",
            Self::CmsNewsPageUpdate => "Cms.NewsPage.Update",
            Self::CmsNewsPageDelete => "Cms.NewsPage.Delete",
            Self::CmsNoticeBoardCreate => "Cms.NoticeBoard.Create",
            Self::CmsNoticeBoardRead => "Cms.NoticeBoard.Read",
            Self::CmsNoticeBoardUpdate => "Cms.NoticeBoard.Update",
            Self::CmsNoticeBoardDelete => "Cms.NoticeBoard.Delete",
            Self::CmsNoticeBoardPublish => "Cms.NoticeBoard.Publish",
            Self::CmsNoticeBoardUnpublish => "Cms.NoticeBoard.Unpublish",
            Self::CmsTestimonialCreate => "Cms.Testimonial.Create",
            Self::CmsTestimonialRead => "Cms.Testimonial.Read",
            Self::CmsTestimonialUpdate => "Cms.Testimonial.Update",
            Self::CmsTestimonialDelete => "Cms.Testimonial.Delete",
            Self::CmsHomeSliderCreate => "Cms.HomeSlider.Create",
            Self::CmsHomeSliderRead => "Cms.HomeSlider.Read",
            Self::CmsHomeSliderUpdate => "Cms.HomeSlider.Update",
            Self::CmsHomeSliderDelete => "Cms.HomeSlider.Delete",
            Self::CmsSpeechSliderCreate => "Cms.SpeechSlider.Create",
            Self::CmsSpeechSliderRead => "Cms.SpeechSlider.Read",
            Self::CmsSpeechSliderUpdate => "Cms.SpeechSlider.Update",
            Self::CmsSpeechSliderDelete => "Cms.SpeechSlider.Delete",
            Self::CmsContentCreate => "Cms.Content.Create",
            Self::CmsContentRead => "Cms.Content.Read",
            Self::CmsContentUpdate => "Cms.Content.Update",
            Self::CmsContentDelete => "Cms.Content.Delete",
            Self::CmsContentTypeCreate => "Cms.ContentType.Create",
            Self::CmsContentTypeRead => "Cms.ContentType.Read",
            Self::CmsContentTypeUpdate => "Cms.ContentType.Update",
            Self::CmsContentTypeDelete => "Cms.ContentType.Delete",
            Self::CmsContentShareListCreate => "Cms.ContentShareList.Create",
            Self::CmsContentShareListRead => "Cms.ContentShareList.Read",
            Self::CmsContentShareListUpdate => "Cms.ContentShareList.Update",
            Self::CmsContentShareListDelete => "Cms.ContentShareList.Delete",
            Self::CmsContentShareListDispatch => "Cms.ContentShareList.Dispatch",
            Self::CmsContentShareListCancel => "Cms.ContentShareList.Cancel",
            Self::CmsTeacherUploadContentCreate => "Cms.TeacherUploadContent.Create",
            Self::CmsTeacherUploadContentRead => "Cms.TeacherUploadContent.Read",
            Self::CmsTeacherUploadContentUpdate => "Cms.TeacherUploadContent.Update",
            Self::CmsTeacherUploadContentDelete => "Cms.TeacherUploadContent.Delete",
            Self::CmsUploadContentCreate => "Cms.UploadContent.Create",
            Self::CmsUploadContentRead => "Cms.UploadContent.Read",
            Self::CmsUploadContentUpdate => "Cms.UploadContent.Update",
            Self::CmsUploadContentDelete => "Cms.UploadContent.Delete",
            Self::CmsAboutPageCreate => "Cms.AboutPage.Create",
            Self::CmsAboutPageRead => "Cms.AboutPage.Read",
            Self::CmsAboutPageUpdate => "Cms.AboutPage.Update",
            Self::CmsAboutPageDelete => "Cms.AboutPage.Delete",
            Self::CmsContactPageCreate => "Cms.ContactPage.Create",
            Self::CmsContactPageRead => "Cms.ContactPage.Read",
            Self::CmsContactPageUpdate => "Cms.ContactPage.Update",
            Self::CmsContactPageDelete => "Cms.ContactPage.Delete",
            Self::CmsCoursePageCreate => "Cms.CoursePage.Create",
            Self::CmsCoursePageRead => "Cms.CoursePage.Read",
            Self::CmsCoursePageUpdate => "Cms.CoursePage.Update",
            Self::CmsCoursePageDelete => "Cms.CoursePage.Delete",
            Self::CmsHomePageSettingConfigure => "Cms.HomePageSetting.Configure",
            Self::CmsHomePageSettingRead => "Cms.HomePageSetting.Read",
            Self::CmsHomePageSettingDelete => "Cms.HomePageSetting.Delete",
            Self::CmsFrontendPageCreate => "Cms.FrontendPage.Create",
            Self::CmsFrontendPageRead => "Cms.FrontendPage.Read",
            Self::CmsFrontendPageUpdate => "Cms.FrontendPage.Update",
            Self::CmsFrontendPageDelete => "Cms.FrontendPage.Delete",
            Self::CmsRead => "Cms.Read",
            Self::FacilitiesRoomCreate => "Facilities.Room.Create",
            Self::FacilitiesRoomRead => "Facilities.Room.Read",
            Self::FacilitiesRoomUpdate => "Facilities.Room.Update",
            Self::FacilitiesRoomDelete => "Facilities.Room.Delete",
            Self::FacilitiesRoomAssignStudent => "Facilities.Room.AssignStudent",
            Self::FacilitiesRoomUnassignStudent => "Facilities.Room.UnassignStudent",
            Self::FacilitiesVehicleCreate => "Facilities.Vehicle.Create",
            Self::FacilitiesVehicleRead => "Facilities.Vehicle.Read",
            Self::FacilitiesVehicleUpdate => "Facilities.Vehicle.Update",
            Self::FacilitiesVehicleDelete => "Facilities.Vehicle.Delete",
            Self::FacilitiesVehicleAssignDriver => "Facilities.Vehicle.AssignDriver",
            Self::FacilitiesVehicleDeactivate => "Facilities.Vehicle.Deactivate",
            Self::FacilitiesRouteCreate => "Facilities.Route.Create",
            Self::FacilitiesRouteRead => "Facilities.Route.Read",
            Self::FacilitiesRouteUpdate => "Facilities.Route.Update",
            Self::FacilitiesRouteDelete => "Facilities.Route.Delete",
            Self::FacilitiesRouteAddStop => "Facilities.Route.AddStop",
            Self::FacilitiesRouteUpdateStop => "Facilities.Route.UpdateStop",
            Self::FacilitiesRouteRemoveStop => "Facilities.Route.RemoveStop",
            Self::FacilitiesTransportAssignVehicle => "Facilities.Transport.AssignVehicle",
            Self::FacilitiesTransportUnassignVehicle => "Facilities.Transport.UnassignVehicle",
            Self::FacilitiesTransportAssignStudent => "Facilities.Transport.AssignStudent",
            Self::FacilitiesTransportUnassignStudent => "Facilities.Transport.UnassignStudent",
            Self::FacilitiesTransportRead => "Facilities.Transport.Read",
            Self::FacilitiesDormitoryCreate => "Facilities.Dormitory.Create",
            Self::FacilitiesDormitoryRead => "Facilities.Dormitory.Read",
            Self::FacilitiesDormitoryUpdate => "Facilities.Dormitory.Update",
            Self::FacilitiesDormitoryDelete => "Facilities.Dormitory.Delete",
            Self::FacilitiesRoomTypeCreate => "Facilities.RoomType.Create",
            Self::FacilitiesRoomTypeRead => "Facilities.RoomType.Read",
            Self::FacilitiesRoomTypeUpdate => "Facilities.RoomType.Update",
            Self::FacilitiesRoomTypeDelete => "Facilities.RoomType.Delete",
            Self::FacilitiesItemCategoryCreate => "Facilities.ItemCategory.Create",
            Self::FacilitiesItemCategoryRead => "Facilities.ItemCategory.Read",
            Self::FacilitiesItemCategoryUpdate => "Facilities.ItemCategory.Update",
            Self::FacilitiesItemCategoryDelete => "Facilities.ItemCategory.Delete",
            Self::FacilitiesItemCreate => "Facilities.Item.Create",
            Self::FacilitiesItemRead => "Facilities.Item.Read",
            Self::FacilitiesItemUpdate => "Facilities.Item.Update",
            Self::FacilitiesItemDelete => "Facilities.Item.Delete",
            Self::FacilitiesItemStoreCreate => "Facilities.ItemStore.Create",
            Self::FacilitiesItemStoreRead => "Facilities.ItemStore.Read",
            Self::FacilitiesItemStoreUpdate => "Facilities.ItemStore.Update",
            Self::FacilitiesItemStoreDelete => "Facilities.ItemStore.Delete",
            Self::FacilitiesInventoryReceive => "Facilities.Inventory.Receive",
            Self::FacilitiesInventoryUpdateReceive => "Facilities.Inventory.UpdateReceive",
            Self::FacilitiesInventoryCancelReceive => "Facilities.Inventory.CancelReceive",
            Self::FacilitiesInventoryIssue => "Facilities.Inventory.Issue",
            Self::FacilitiesInventoryUpdateIssue => "Facilities.Inventory.UpdateIssue",
            Self::FacilitiesInventoryReturnIssued => "Facilities.Inventory.ReturnIssued",
            Self::FacilitiesInventorySell => "Facilities.Inventory.Sell",
            Self::FacilitiesInventoryUpdateSell => "Facilities.Inventory.UpdateSell",
            Self::FacilitiesInventoryCancelSell => "Facilities.Inventory.CancelSell",
            Self::FacilitiesInventoryRefundSell => "Facilities.Inventory.RefundSell",
            Self::FacilitiesInventoryRead => "Facilities.Inventory.Read",
            Self::FacilitiesSupplierCreate => "Facilities.Supplier.Create",
            Self::FacilitiesSupplierRead => "Facilities.Supplier.Read",
            Self::FacilitiesSupplierUpdate => "Facilities.Supplier.Update",
            Self::FacilitiesSupplierDelete => "Facilities.Supplier.Delete",
            Self::FacilitiesSupplierDeactivate => "Facilities.Supplier.Deactivate",
            Self::EventsCalendarCreate => "Events.Calendar.Create",
            Self::EventsCalendarRead => "Events.Calendar.Read",
            Self::EventsCalendarUpdate => "Events.Calendar.Update",
            Self::EventsCalendarDelete => "Events.Calendar.Delete",
            // Phase 13 Events domain net-new
            Self::EventsEventCreate => "Events.Event.Create",
            Self::EventsEventRead => "Events.Event.Read",
            Self::EventsEventUpdate => "Events.Event.Update",
            Self::EventsEventDelete => "Events.Event.Delete",
            Self::EventsEventPublish => "Events.Event.Publish",
            Self::EventsHolidayCreate => "Events.Holiday.Create",
            Self::EventsHolidayRead => "Events.Holiday.Read",
            Self::EventsHolidayUpdate => "Events.Holiday.Update",
            Self::EventsHolidayDelete => "Events.Holiday.Delete",
            Self::EventsWeekendCreate => "Events.Weekend.Create",
            Self::EventsWeekendRead => "Events.Weekend.Read",
            Self::EventsWeekendUpdate => "Events.Weekend.Update",
            Self::EventsWeekendDelete => "Events.Weekend.Delete",
            Self::EventsWeekendConfigure => "Events.Weekend.Configure",
            Self::EventsIncidentCreate => "Events.Incident.Create",
            Self::EventsIncidentRead => "Events.Incident.Read",
            Self::EventsIncidentUpdate => "Events.Incident.Update",
            Self::EventsIncidentDelete => "Events.Incident.Delete",
            Self::EventsIncidentAssign => "Events.Incident.Assign",
            Self::EventsIncidentReassign => "Events.Incident.Reassign",
            Self::EventsIncidentUnassign => "Events.Incident.Unassign",
            Self::EventsIncidentComment => "Events.Incident.Comment",
            Self::EventsIncidentResolve => "Events.Incident.Resolve",
            Self::EventsIncidentCommentDelete => "Events.IncidentComment.Delete",
            Self::EventsCalendarSettingCreate => "Events.CalendarSetting.Create",
            Self::EventsCalendarSettingRead => "Events.CalendarSetting.Read",
            Self::EventsCalendarSettingUpdate => "Events.CalendarSetting.Update",
            Self::EventsCalendarSettingEnable => "Events.CalendarSetting.Enable",
            Self::EventsCalendarSettingDisable => "Events.CalendarSetting.Disable",
            Self::EventsCalendarSettingDelete => "Events.CalendarSetting.Delete",
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
            Self::SettingsGeneralRead => "Settings.General.Read",
            Self::SettingsGeneralUpdate => "Settings.General.Update",
            Self::SettingsGeneralTwoFactorToggle => "Settings.General.TwoFactor.Toggle",
            Self::SettingsThemeCreate => "Settings.Theme.Create",
            Self::SettingsThemeRead => "Settings.Theme.Read",
            Self::SettingsThemeUpdate => "Settings.Theme.Update",
            Self::SettingsThemeActivate => "Settings.Theme.Activate",
            Self::SettingsThemeDelete => "Settings.Theme.Delete",
            Self::SettingsThemeReplicate => "Settings.Theme.Replicate",
            Self::SettingsThemeSelect => "Settings.Theme.Select",
            Self::SettingsColorCreate => "Settings.Color.Create",
            Self::SettingsColorRead => "Settings.Color.Read",
            Self::SettingsColorUpdate => "Settings.Color.Update",
            Self::SettingsColorDelete => "Settings.Color.Delete",
            Self::SettingsColorThemeCreate => "Settings.ColorTheme.Create",
            Self::SettingsColorThemeRead => "Settings.ColorTheme.Read",
            Self::SettingsColorThemeUpdate => "Settings.ColorTheme.Update",
            Self::SettingsColorThemeDelete => "Settings.ColorTheme.Delete",
            Self::SettingsLanguageAdd => "Settings.Language.Add",
            Self::SettingsLanguageRead => "Settings.Language.Read",
            Self::SettingsLanguageUpdate => "Settings.Language.Update",
            Self::SettingsLanguageDelete => "Settings.Language.Delete",
            Self::SettingsLanguageActivate => "Settings.Language.Activate",
            Self::SettingsLanguageSelect => "Settings.Language.Select",
            Self::SettingsLanguagePhraseAdd => "Settings.LanguagePhrase.Add",
            Self::SettingsLanguagePhraseRead => "Settings.LanguagePhrase.Read",
            Self::SettingsLanguagePhraseUpdate => "Settings.LanguagePhrase.Update",
            Self::SettingsLanguagePhraseDelete => "Settings.LanguagePhrase.Delete",
            Self::SettingsLanguagePhraseTranslate => "Settings.LanguagePhrase.Translate",
            Self::SettingsBaseGroupAdd => "Settings.BaseGroup.Add",
            Self::SettingsBaseGroupRead => "Settings.BaseGroup.Read",
            Self::SettingsBaseGroupUpdate => "Settings.BaseGroup.Update",
            Self::SettingsBaseGroupDelete => "Settings.BaseGroup.Delete",
            Self::SettingsBaseSetupAdd => "Settings.BaseSetup.Add",
            Self::SettingsBaseSetupRead => "Settings.BaseSetup.Read",
            Self::SettingsBaseSetupUpdate => "Settings.BaseSetup.Update",
            Self::SettingsBaseSetupDelete => "Settings.BaseSetup.Delete",
            Self::SettingsDateFormatAdd => "Settings.DateFormat.Add",
            Self::SettingsDateFormatRead => "Settings.DateFormat.Read",
            Self::SettingsDateFormatUpdate => "Settings.DateFormat.Update",
            Self::SettingsDateFormatDelete => "Settings.DateFormat.Delete",
            Self::SettingsDateFormatSelect => "Settings.DateFormat.Select",
            Self::SettingsTimeZoneSelect => "Settings.TimeZone.Select",
            Self::SettingsSessionSelect => "Settings.Session.Select",
            Self::SettingsStyleCreate => "Settings.Style.Create",
            Self::SettingsStyleRead => "Settings.Style.Read",
            Self::SettingsStyleUpdate => "Settings.Style.Update",
            Self::SettingsStyleActivate => "Settings.Style.Activate",
            Self::SettingsStyleDelete => "Settings.Style.Delete",
            Self::SettingsBackgroundCreate => "Settings.Background.Create",
            Self::SettingsBackgroundRead => "Settings.Background.Read",
            Self::SettingsBackgroundUpdate => "Settings.Background.Update",
            Self::SettingsBackgroundDelete => "Settings.Background.Delete",
            Self::SettingsDashboardCreate => "Settings.Dashboard.Create",
            Self::SettingsDashboardRead => "Settings.Dashboard.Read",
            Self::SettingsDashboardUpdate => "Settings.Dashboard.Update",
            Self::SettingsDashboardDelete => "Settings.Dashboard.Delete",
            Self::SettingsCustomLinkRead => "Settings.CustomLink.Read",
            Self::SettingsCustomLinkUpdate => "Settings.CustomLink.Update",
            Self::SettingsCustomLinkReset => "Settings.CustomLink.Reset",
            Self::SettingsBehaviorRecordRead => "Settings.BehaviorRecord.Read",
            Self::SettingsBehaviorRecordUpdate => "Settings.BehaviorRecord.Update",
            Self::SettingsSetupAdminAdd => "Settings.SetupAdmin.Add",
            Self::SettingsSetupAdminRead => "Settings.SetupAdmin.Read",
            Self::SettingsSetupAdminUpdate => "Settings.SetupAdmin.Update",
            Self::SettingsSetupAdminDelete => "Settings.SetupAdmin.Delete",
            Self::OperationsBackupCreate => "Operations.Backup.Create",
            Self::OperationsBackupRead => "Operations.Backup.Read",
            Self::OperationsBackupDelete => "Operations.Backup.Delete",
            Self::OperationsBackupRestore => "Operations.Backup.Restore",
            Self::OperationsBackupActivate => "Operations.Backup.Activate",
            Self::OperationsBackupDeactivate => "Operations.Backup.Deactivate",
            Self::OperationsJobSchedule => "Operations.Job.Schedule",
            Self::OperationsJobRead => "Operations.Job.Read",
            Self::OperationsJobCancel => "Operations.Job.Cancel",
            Self::OperationsJobReserve => "Operations.Job.Reserve",
            Self::OperationsJobComplete => "Operations.Job.Complete",
            Self::OperationsJobFail => "Operations.Job.Fail",
            Self::OperationsJobRetry => "Operations.Job.Retry",
            Self::OperationsJobPurge => "Operations.Job.Purge",
            Self::OperationsFailedJobRead => "Operations.FailedJob.Read",
            Self::OperationsFailedJobRetry => "Operations.FailedJob.Retry",
            Self::OperationsFailedJobDelete => "Operations.FailedJob.Delete",
            Self::OperationsFailedJobPurge => "Operations.FailedJob.Purge",
            Self::OperationsVersionRegister => "Operations.Version.Register",
            Self::OperationsVersionRead => "Operations.Version.Read",
            Self::OperationsVersionUpdate => "Operations.Version.Update",
            Self::OperationsVersionHistoryRecord => "Operations.VersionHistory.Record",
            Self::OperationsVersionHistoryRead => "Operations.VersionHistory.Read",
            Self::OperationsAuditRecord => "Operations.Audit.Record",
            Self::OperationsAuditRead => "Operations.Audit.Read",
            Self::OperationsMaintenanceRead => "Operations.Maintenance.Read",
            Self::OperationsMaintenanceConfigure => "Operations.Maintenance.Configure",
            Self::OperationsMaintenanceEnable => "Operations.Maintenance.Enable",
            Self::OperationsMaintenanceDisable => "Operations.Maintenance.Disable",
            Self::OperationsSidebarCreate => "Operations.Sidebar.Create",
            Self::OperationsSidebarRead => "Operations.Sidebar.Read",
            Self::OperationsSidebarUpdate => "Operations.Sidebar.Update",
            Self::OperationsSidebarDelete => "Operations.Sidebar.Delete",
            Self::OperationsSidebarReorder => "Operations.Sidebar.Reorder",
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
            Self::LibraryRead,
            Self::LibraryConfigure,
            Self::LibraryReport,
            Self::BookCategoryCreate,
            Self::BookCategoryRead,
            Self::BookCategoryUpdate,
            Self::BookCategoryDelete,
            Self::BookAdd,
            Self::BookRead,
            Self::BookUpdate,
            Self::BookDelete,
            Self::BookAdjustQuantity,
            Self::BookSearch,
            Self::MemberRegister,
            Self::MemberRead,
            Self::MemberUpdate,
            Self::MemberDelete,
            Self::MemberDeactivate,
            Self::MemberReactivate,
            Self::BookIssueIssue,
            Self::BookIssueRead,
            Self::BookIssueReturn,
            Self::BookIssueRenew,
            Self::BookIssueMarkLost,
            Self::BookIssueCalculateFine,
            Self::BookIssueWaiveFine,
            Self::CommunicationMessageCreate,
            Self::CommunicationMessageRead,
            Self::CommunicationMessageUpdate,
            Self::CommunicationMessageDelete,
            Self::DocumentsFolderCreate,
            Self::DocumentsFolderRead,
            Self::DocumentsFolderUpdate,
            Self::DocumentsFolderDelete,
            Self::FormDownloadUpload,
            Self::FormDownloadUpdate,
            Self::FormDownloadDelete,
            Self::FormDownloadRead,
            Self::PostalDispatchCreate,
            Self::PostalDispatchUpdate,
            Self::PostalDispatchDelete,
            Self::PostalReceiveCreate,
            Self::PostalReceiveUpdate,
            Self::PostalReceiveDelete,
            Self::PostalRead,
            Self::CmsPageCreate,
            Self::CmsPageRead,
            Self::CmsPageUpdate,
            Self::CmsPageDelete,
            Self::CmsPagePublish,
            Self::CmsPageArchive,
            Self::CmsNewsCreate,
            Self::CmsNewsRead,
            Self::CmsNewsUpdate,
            Self::CmsNewsDelete,
            Self::CmsNewsPublish,
            Self::CmsNewsUnpublish,
            Self::CmsNewsIncrementView,
            Self::CmsNewsCategoryCreate,
            Self::CmsNewsCategoryRead,
            Self::CmsNewsCategoryUpdate,
            Self::CmsNewsCategoryDelete,
            Self::CmsNewsCommentCreate,
            Self::CmsNewsCommentModerate,
            Self::CmsNewsCommentDelete,
            Self::CmsNewsCommentRead,
            Self::CmsNewsPageCreate,
            Self::CmsNewsPageRead,
            Self::CmsNewsPageUpdate,
            Self::CmsNewsPageDelete,
            Self::CmsNoticeBoardCreate,
            Self::CmsNoticeBoardRead,
            Self::CmsNoticeBoardUpdate,
            Self::CmsNoticeBoardDelete,
            Self::CmsNoticeBoardPublish,
            Self::CmsNoticeBoardUnpublish,
            Self::CmsTestimonialCreate,
            Self::CmsTestimonialRead,
            Self::CmsTestimonialUpdate,
            Self::CmsTestimonialDelete,
            Self::CmsHomeSliderCreate,
            Self::CmsHomeSliderRead,
            Self::CmsHomeSliderUpdate,
            Self::CmsHomeSliderDelete,
            Self::CmsSpeechSliderCreate,
            Self::CmsSpeechSliderRead,
            Self::CmsSpeechSliderUpdate,
            Self::CmsSpeechSliderDelete,
            Self::CmsContentCreate,
            Self::CmsContentRead,
            Self::CmsContentUpdate,
            Self::CmsContentDelete,
            Self::CmsContentTypeCreate,
            Self::CmsContentTypeRead,
            Self::CmsContentTypeUpdate,
            Self::CmsContentTypeDelete,
            Self::CmsContentShareListCreate,
            Self::CmsContentShareListRead,
            Self::CmsContentShareListUpdate,
            Self::CmsContentShareListDelete,
            Self::CmsContentShareListDispatch,
            Self::CmsContentShareListCancel,
            Self::CmsTeacherUploadContentCreate,
            Self::CmsTeacherUploadContentRead,
            Self::CmsTeacherUploadContentUpdate,
            Self::CmsTeacherUploadContentDelete,
            Self::CmsUploadContentCreate,
            Self::CmsUploadContentRead,
            Self::CmsUploadContentUpdate,
            Self::CmsUploadContentDelete,
            Self::CmsAboutPageCreate,
            Self::CmsAboutPageRead,
            Self::CmsAboutPageUpdate,
            Self::CmsAboutPageDelete,
            Self::CmsContactPageCreate,
            Self::CmsContactPageRead,
            Self::CmsContactPageUpdate,
            Self::CmsContactPageDelete,
            Self::CmsCoursePageCreate,
            Self::CmsCoursePageRead,
            Self::CmsCoursePageUpdate,
            Self::CmsCoursePageDelete,
            Self::CmsHomePageSettingConfigure,
            Self::CmsHomePageSettingRead,
            Self::CmsHomePageSettingDelete,
            Self::CmsFrontendPageCreate,
            Self::CmsFrontendPageRead,
            Self::CmsFrontendPageUpdate,
            Self::CmsFrontendPageDelete,
            Self::CmsRead,
            Self::FacilitiesRoomCreate,
            Self::FacilitiesRoomRead,
            Self::FacilitiesRoomUpdate,
            Self::FacilitiesRoomDelete,
            Self::FacilitiesVehicleCreate,
            Self::FacilitiesVehicleRead,
            Self::FacilitiesVehicleUpdate,
            Self::FacilitiesVehicleDelete,
            Self::FacilitiesVehicleAssignDriver,
            Self::FacilitiesVehicleDeactivate,
            Self::FacilitiesRouteCreate,
            Self::FacilitiesRouteRead,
            Self::FacilitiesRouteUpdate,
            Self::FacilitiesRouteDelete,
            Self::FacilitiesRouteAddStop,
            Self::FacilitiesRouteUpdateStop,
            Self::FacilitiesRouteRemoveStop,
            Self::FacilitiesTransportAssignVehicle,
            Self::FacilitiesTransportUnassignVehicle,
            Self::FacilitiesTransportAssignStudent,
            Self::FacilitiesTransportUnassignStudent,
            Self::FacilitiesTransportRead,
            Self::FacilitiesDormitoryCreate,
            Self::FacilitiesDormitoryRead,
            Self::FacilitiesDormitoryUpdate,
            Self::FacilitiesDormitoryDelete,
            Self::FacilitiesRoomCreate,
            Self::FacilitiesRoomRead,
            Self::FacilitiesRoomUpdate,
            Self::FacilitiesRoomDelete,
            Self::FacilitiesRoomAssignStudent,
            Self::FacilitiesRoomUnassignStudent,
            Self::FacilitiesRoomTypeCreate,
            Self::FacilitiesRoomTypeRead,
            Self::FacilitiesRoomTypeUpdate,
            Self::FacilitiesRoomTypeDelete,
            Self::FacilitiesItemCategoryCreate,
            Self::FacilitiesItemCategoryRead,
            Self::FacilitiesItemCategoryUpdate,
            Self::FacilitiesItemCategoryDelete,
            Self::FacilitiesItemCreate,
            Self::FacilitiesItemRead,
            Self::FacilitiesItemUpdate,
            Self::FacilitiesItemDelete,
            Self::FacilitiesItemStoreCreate,
            Self::FacilitiesItemStoreRead,
            Self::FacilitiesItemStoreUpdate,
            Self::FacilitiesItemStoreDelete,
            Self::FacilitiesInventoryReceive,
            Self::FacilitiesInventoryUpdateReceive,
            Self::FacilitiesInventoryCancelReceive,
            Self::FacilitiesInventoryIssue,
            Self::FacilitiesInventoryUpdateIssue,
            Self::FacilitiesInventoryReturnIssued,
            Self::FacilitiesInventorySell,
            Self::FacilitiesInventoryUpdateSell,
            Self::FacilitiesInventoryCancelSell,
            Self::FacilitiesInventoryRefundSell,
            Self::FacilitiesInventoryRead,
            Self::FacilitiesSupplierCreate,
            Self::FacilitiesSupplierRead,
            Self::FacilitiesSupplierUpdate,
            Self::FacilitiesSupplierDelete,
            Self::FacilitiesSupplierDeactivate,
            Self::EventsCalendarCreate,
            Self::EventsCalendarRead,
            Self::EventsCalendarUpdate,
            Self::EventsCalendarDelete,
            Self::EventsEventCreate,
            Self::EventsEventRead,
            Self::EventsEventUpdate,
            Self::EventsEventDelete,
            Self::EventsEventPublish,
            Self::EventsHolidayCreate,
            Self::EventsHolidayRead,
            Self::EventsHolidayUpdate,
            Self::EventsHolidayDelete,
            Self::EventsWeekendCreate,
            Self::EventsWeekendRead,
            Self::EventsWeekendUpdate,
            Self::EventsWeekendDelete,
            Self::EventsWeekendConfigure,
            Self::EventsIncidentCreate,
            Self::EventsIncidentRead,
            Self::EventsIncidentUpdate,
            Self::EventsIncidentDelete,
            Self::EventsIncidentAssign,
            Self::EventsIncidentReassign,
            Self::EventsIncidentUnassign,
            Self::EventsIncidentComment,
            Self::EventsIncidentResolve,
            Self::EventsIncidentCommentDelete,
            Self::EventsCalendarSettingCreate,
            Self::EventsCalendarSettingRead,
            Self::EventsCalendarSettingUpdate,
            Self::EventsCalendarSettingEnable,
            Self::EventsCalendarSettingDisable,
            Self::EventsCalendarSettingDelete,
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
            Self::SettingsGeneralRead,
            Self::SettingsGeneralUpdate,
            Self::SettingsGeneralTwoFactorToggle,
            Self::SettingsThemeCreate,
            Self::SettingsThemeRead,
            Self::SettingsThemeUpdate,
            Self::SettingsThemeActivate,
            Self::SettingsThemeDelete,
            Self::SettingsThemeReplicate,
            Self::SettingsThemeSelect,
            Self::SettingsColorCreate,
            Self::SettingsColorRead,
            Self::SettingsColorUpdate,
            Self::SettingsColorDelete,
            Self::SettingsColorThemeCreate,
            Self::SettingsColorThemeRead,
            Self::SettingsColorThemeUpdate,
            Self::SettingsColorThemeDelete,
            Self::SettingsLanguageAdd,
            Self::SettingsLanguageRead,
            Self::SettingsLanguageUpdate,
            Self::SettingsLanguageDelete,
            Self::SettingsLanguageActivate,
            Self::SettingsLanguageSelect,
            Self::SettingsLanguagePhraseAdd,
            Self::SettingsLanguagePhraseRead,
            Self::SettingsLanguagePhraseUpdate,
            Self::SettingsLanguagePhraseDelete,
            Self::SettingsLanguagePhraseTranslate,
            Self::SettingsBaseGroupAdd,
            Self::SettingsBaseGroupRead,
            Self::SettingsBaseGroupUpdate,
            Self::SettingsBaseGroupDelete,
            Self::SettingsBaseSetupAdd,
            Self::SettingsBaseSetupRead,
            Self::SettingsBaseSetupUpdate,
            Self::SettingsBaseSetupDelete,
            Self::SettingsDateFormatAdd,
            Self::SettingsDateFormatRead,
            Self::SettingsDateFormatUpdate,
            Self::SettingsDateFormatDelete,
            Self::SettingsDateFormatSelect,
            Self::SettingsTimeZoneSelect,
            Self::SettingsSessionSelect,
            Self::SettingsStyleCreate,
            Self::SettingsStyleRead,
            Self::SettingsStyleUpdate,
            Self::SettingsStyleActivate,
            Self::SettingsStyleDelete,
            Self::SettingsBackgroundCreate,
            Self::SettingsBackgroundRead,
            Self::SettingsBackgroundUpdate,
            Self::SettingsBackgroundDelete,
            Self::SettingsDashboardCreate,
            Self::SettingsDashboardRead,
            Self::SettingsDashboardUpdate,
            Self::SettingsDashboardDelete,
            Self::SettingsCustomLinkRead,
            Self::SettingsCustomLinkUpdate,
            Self::SettingsCustomLinkReset,
            Self::SettingsBehaviorRecordRead,
            Self::SettingsBehaviorRecordUpdate,
            Self::SettingsSetupAdminAdd,
            Self::SettingsSetupAdminRead,
            Self::SettingsSetupAdminUpdate,
            Self::SettingsSetupAdminDelete,
            Self::OperationsBackupCreate,
            Self::OperationsBackupRead,
            Self::OperationsBackupDelete,
            Self::OperationsBackupRestore,
            Self::OperationsBackupActivate,
            Self::OperationsBackupDeactivate,
            Self::OperationsJobSchedule,
            Self::OperationsJobRead,
            Self::OperationsJobCancel,
            Self::OperationsJobReserve,
            Self::OperationsJobComplete,
            Self::OperationsJobFail,
            Self::OperationsJobRetry,
            Self::OperationsJobPurge,
            Self::OperationsFailedJobRead,
            Self::OperationsFailedJobRetry,
            Self::OperationsFailedJobDelete,
            Self::OperationsFailedJobPurge,
            Self::OperationsVersionRegister,
            Self::OperationsVersionRead,
            Self::OperationsVersionUpdate,
            Self::OperationsVersionHistoryRecord,
            Self::OperationsVersionHistoryRead,
            Self::OperationsAuditRecord,
            Self::OperationsAuditRead,
            Self::OperationsMaintenanceRead,
            Self::OperationsMaintenanceConfigure,
            Self::OperationsMaintenanceEnable,
            Self::OperationsMaintenanceDisable,
            Self::OperationsSidebarCreate,
            Self::OperationsSidebarRead,
            Self::OperationsSidebarUpdate,
            Self::OperationsSidebarDelete,
            Self::OperationsSidebarReorder,
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
            "Library.Library.Read" => Some(Self::LibraryRead),
            "Library.Library.Configure" => Some(Self::LibraryConfigure),
            "Library.Library.Report" => Some(Self::LibraryReport),
            "Library.BookCategory.Create" => Some(Self::BookCategoryCreate),
            "Library.BookCategory.Read" => Some(Self::BookCategoryRead),
            "Library.BookCategory.Update" => Some(Self::BookCategoryUpdate),
            "Library.BookCategory.Delete" => Some(Self::BookCategoryDelete),
            "Library.Book.Add" => Some(Self::BookAdd),
            "Library.Book.Read" => Some(Self::BookRead),
            "Library.Book.Update" => Some(Self::BookUpdate),
            "Library.Book.Delete" => Some(Self::BookDelete),
            "Library.Book.AdjustQuantity" => Some(Self::BookAdjustQuantity),
            "Library.Book.Search" => Some(Self::BookSearch),
            "Library.Member.Register" => Some(Self::MemberRegister),
            "Library.Member.Read" => Some(Self::MemberRead),
            "Library.Member.Update" => Some(Self::MemberUpdate),
            "Library.Member.Delete" => Some(Self::MemberDelete),
            "Library.Member.Deactivate" => Some(Self::MemberDeactivate),
            "Library.Member.Reactivate" => Some(Self::MemberReactivate),
            "Library.BookIssue.Issue" => Some(Self::BookIssueIssue),
            "Library.BookIssue.Read" => Some(Self::BookIssueRead),
            "Library.BookIssue.Return" => Some(Self::BookIssueReturn),
            "Library.BookIssue.Renew" => Some(Self::BookIssueRenew),
            "Library.BookIssue.MarkLost" => Some(Self::BookIssueMarkLost),
            "Library.BookIssue.CalculateFine" => Some(Self::BookIssueCalculateFine),
            "Library.BookIssue.WaiveFine" => Some(Self::BookIssueWaiveFine),
            "Communication.Message.Create" => Some(Self::CommunicationMessageCreate),
            "Communication.Message.Read" => Some(Self::CommunicationMessageRead),
            "Communication.Message.Update" => Some(Self::CommunicationMessageUpdate),
            "Communication.Message.Delete" => Some(Self::CommunicationMessageDelete),
            "Documents.Folder.Create" => Some(Self::DocumentsFolderCreate),
            "Documents.Folder.Read" => Some(Self::DocumentsFolderRead),
            "Documents.Folder.Update" => Some(Self::DocumentsFolderUpdate),
            "Documents.Folder.Delete" => Some(Self::DocumentsFolderDelete),
            "Documents.FormDownload.Upload" => Some(Self::FormDownloadUpload),
            "Documents.FormDownload.Update" => Some(Self::FormDownloadUpdate),
            "Documents.FormDownload.Delete" => Some(Self::FormDownloadDelete),
            "Documents.FormDownload.Read" => Some(Self::FormDownloadRead),
            "Documents.PostalDispatch.Create" => Some(Self::PostalDispatchCreate),
            "Documents.PostalDispatch.Update" => Some(Self::PostalDispatchUpdate),
            "Documents.PostalDispatch.Delete" => Some(Self::PostalDispatchDelete),
            "Documents.PostalReceive.Create" => Some(Self::PostalReceiveCreate),
            "Documents.PostalReceive.Update" => Some(Self::PostalReceiveUpdate),
            "Documents.PostalReceive.Delete" => Some(Self::PostalReceiveDelete),
            "Documents.Postal.Read" => Some(Self::PostalRead),
            "Cms.Page.Create" => Some(Self::CmsPageCreate),
            "Cms.Page.Read" => Some(Self::CmsPageRead),
            "Cms.Page.Update" => Some(Self::CmsPageUpdate),
            "Cms.Page.Delete" => Some(Self::CmsPageDelete),
            "Cms.Page.Publish" => Some(Self::CmsPagePublish),
            "Cms.Page.Archive" => Some(Self::CmsPageArchive),
            "Cms.News.Create" => Some(Self::CmsNewsCreate),
            "Cms.News.Read" => Some(Self::CmsNewsRead),
            "Cms.News.Update" => Some(Self::CmsNewsUpdate),
            "Cms.News.Delete" => Some(Self::CmsNewsDelete),
            "Cms.News.Publish" => Some(Self::CmsNewsPublish),
            "Cms.News.Unpublish" => Some(Self::CmsNewsUnpublish),
            "Cms.News.IncrementView" => Some(Self::CmsNewsIncrementView),
            "Cms.NewsCategory.Create" => Some(Self::CmsNewsCategoryCreate),
            "Cms.NewsCategory.Read" => Some(Self::CmsNewsCategoryRead),
            "Cms.NewsCategory.Update" => Some(Self::CmsNewsCategoryUpdate),
            "Cms.NewsCategory.Delete" => Some(Self::CmsNewsCategoryDelete),
            "Cms.NewsComment.Create" => Some(Self::CmsNewsCommentCreate),
            "Cms.NewsComment.Moderate" => Some(Self::CmsNewsCommentModerate),
            "Cms.NewsComment.Delete" => Some(Self::CmsNewsCommentDelete),
            "Cms.NewsComment.Read" => Some(Self::CmsNewsCommentRead),
            "Cms.NewsPage.Create" => Some(Self::CmsNewsPageCreate),
            "Cms.NewsPage.Read" => Some(Self::CmsNewsPageRead),
            "Cms.NewsPage.Update" => Some(Self::CmsNewsPageUpdate),
            "Cms.NewsPage.Delete" => Some(Self::CmsNewsPageDelete),
            "Cms.NoticeBoard.Create" => Some(Self::CmsNoticeBoardCreate),
            "Cms.NoticeBoard.Read" => Some(Self::CmsNoticeBoardRead),
            "Cms.NoticeBoard.Update" => Some(Self::CmsNoticeBoardUpdate),
            "Cms.NoticeBoard.Delete" => Some(Self::CmsNoticeBoardDelete),
            "Cms.NoticeBoard.Publish" => Some(Self::CmsNoticeBoardPublish),
            "Cms.NoticeBoard.Unpublish" => Some(Self::CmsNoticeBoardUnpublish),
            "Cms.Testimonial.Create" => Some(Self::CmsTestimonialCreate),
            "Cms.Testimonial.Read" => Some(Self::CmsTestimonialRead),
            "Cms.Testimonial.Update" => Some(Self::CmsTestimonialUpdate),
            "Cms.Testimonial.Delete" => Some(Self::CmsTestimonialDelete),
            "Cms.HomeSlider.Create" => Some(Self::CmsHomeSliderCreate),
            "Cms.HomeSlider.Read" => Some(Self::CmsHomeSliderRead),
            "Cms.HomeSlider.Update" => Some(Self::CmsHomeSliderUpdate),
            "Cms.HomeSlider.Delete" => Some(Self::CmsHomeSliderDelete),
            "Cms.SpeechSlider.Create" => Some(Self::CmsSpeechSliderCreate),
            "Cms.SpeechSlider.Read" => Some(Self::CmsSpeechSliderRead),
            "Cms.SpeechSlider.Update" => Some(Self::CmsSpeechSliderUpdate),
            "Cms.SpeechSlider.Delete" => Some(Self::CmsSpeechSliderDelete),
            "Cms.Content.Create" => Some(Self::CmsContentCreate),
            "Cms.Content.Read" => Some(Self::CmsContentRead),
            "Cms.Content.Update" => Some(Self::CmsContentUpdate),
            "Cms.Content.Delete" => Some(Self::CmsContentDelete),
            "Cms.ContentType.Create" => Some(Self::CmsContentTypeCreate),
            "Cms.ContentType.Read" => Some(Self::CmsContentTypeRead),
            "Cms.ContentType.Update" => Some(Self::CmsContentTypeUpdate),
            "Cms.ContentType.Delete" => Some(Self::CmsContentTypeDelete),
            "Cms.ContentShareList.Create" => Some(Self::CmsContentShareListCreate),
            "Cms.ContentShareList.Read" => Some(Self::CmsContentShareListRead),
            "Cms.ContentShareList.Update" => Some(Self::CmsContentShareListUpdate),
            "Cms.ContentShareList.Delete" => Some(Self::CmsContentShareListDelete),
            "Cms.ContentShareList.Dispatch" => Some(Self::CmsContentShareListDispatch),
            "Cms.ContentShareList.Cancel" => Some(Self::CmsContentShareListCancel),
            "Cms.TeacherUploadContent.Create" => Some(Self::CmsTeacherUploadContentCreate),
            "Cms.TeacherUploadContent.Read" => Some(Self::CmsTeacherUploadContentRead),
            "Cms.TeacherUploadContent.Update" => Some(Self::CmsTeacherUploadContentUpdate),
            "Cms.TeacherUploadContent.Delete" => Some(Self::CmsTeacherUploadContentDelete),
            "Cms.UploadContent.Create" => Some(Self::CmsUploadContentCreate),
            "Cms.UploadContent.Read" => Some(Self::CmsUploadContentRead),
            "Cms.UploadContent.Update" => Some(Self::CmsUploadContentUpdate),
            "Cms.UploadContent.Delete" => Some(Self::CmsUploadContentDelete),
            "Cms.AboutPage.Create" => Some(Self::CmsAboutPageCreate),
            "Cms.AboutPage.Read" => Some(Self::CmsAboutPageRead),
            "Cms.AboutPage.Update" => Some(Self::CmsAboutPageUpdate),
            "Cms.AboutPage.Delete" => Some(Self::CmsAboutPageDelete),
            "Cms.ContactPage.Create" => Some(Self::CmsContactPageCreate),
            "Cms.ContactPage.Read" => Some(Self::CmsContactPageRead),
            "Cms.ContactPage.Update" => Some(Self::CmsContactPageUpdate),
            "Cms.ContactPage.Delete" => Some(Self::CmsContactPageDelete),
            "Cms.CoursePage.Create" => Some(Self::CmsCoursePageCreate),
            "Cms.CoursePage.Read" => Some(Self::CmsCoursePageRead),
            "Cms.CoursePage.Update" => Some(Self::CmsCoursePageUpdate),
            "Cms.CoursePage.Delete" => Some(Self::CmsCoursePageDelete),
            "Cms.HomePageSetting.Configure" => Some(Self::CmsHomePageSettingConfigure),
            "Cms.HomePageSetting.Read" => Some(Self::CmsHomePageSettingRead),
            "Cms.HomePageSetting.Delete" => Some(Self::CmsHomePageSettingDelete),
            "Cms.FrontendPage.Create" => Some(Self::CmsFrontendPageCreate),
            "Cms.FrontendPage.Read" => Some(Self::CmsFrontendPageRead),
            "Cms.FrontendPage.Update" => Some(Self::CmsFrontendPageUpdate),
            "Cms.FrontendPage.Delete" => Some(Self::CmsFrontendPageDelete),
            "Cms.Read" => Some(Self::CmsRead),
            "Facilities.Room.Create" => Some(Self::FacilitiesRoomCreate),
            "Facilities.Room.Read" => Some(Self::FacilitiesRoomRead),
            "Facilities.Room.Update" => Some(Self::FacilitiesRoomUpdate),
            "Facilities.Room.Delete" => Some(Self::FacilitiesRoomDelete),
            "Facilities.Room.AssignStudent" => Some(Self::FacilitiesRoomAssignStudent),
            "Facilities.Room.UnassignStudent" => Some(Self::FacilitiesRoomUnassignStudent),
            "Facilities.Vehicle.Create" => Some(Self::FacilitiesVehicleCreate),
            "Facilities.Vehicle.Read" => Some(Self::FacilitiesVehicleRead),
            "Facilities.Vehicle.Update" => Some(Self::FacilitiesVehicleUpdate),
            "Facilities.Vehicle.Delete" => Some(Self::FacilitiesVehicleDelete),
            "Facilities.Vehicle.AssignDriver" => Some(Self::FacilitiesVehicleAssignDriver),
            "Facilities.Vehicle.Deactivate" => Some(Self::FacilitiesVehicleDeactivate),
            "Facilities.Route.Create" => Some(Self::FacilitiesRouteCreate),
            "Facilities.Route.Read" => Some(Self::FacilitiesRouteRead),
            "Facilities.Route.Update" => Some(Self::FacilitiesRouteUpdate),
            "Facilities.Route.Delete" => Some(Self::FacilitiesRouteDelete),
            "Facilities.Route.AddStop" => Some(Self::FacilitiesRouteAddStop),
            "Facilities.Route.UpdateStop" => Some(Self::FacilitiesRouteUpdateStop),
            "Facilities.Route.RemoveStop" => Some(Self::FacilitiesRouteRemoveStop),
            "Facilities.Transport.AssignVehicle" => Some(Self::FacilitiesTransportAssignVehicle),
            "Facilities.Transport.UnassignVehicle" => {
                Some(Self::FacilitiesTransportUnassignVehicle)
            }
            "Facilities.Transport.AssignStudent" => Some(Self::FacilitiesTransportAssignStudent),
            "Facilities.Transport.UnassignStudent" => {
                Some(Self::FacilitiesTransportUnassignStudent)
            }
            "Facilities.Transport.Read" => Some(Self::FacilitiesTransportRead),
            "Facilities.Dormitory.Create" => Some(Self::FacilitiesDormitoryCreate),
            "Facilities.Dormitory.Read" => Some(Self::FacilitiesDormitoryRead),
            "Facilities.Dormitory.Update" => Some(Self::FacilitiesDormitoryUpdate),
            "Facilities.Dormitory.Delete" => Some(Self::FacilitiesDormitoryDelete),
            "Facilities.RoomType.Create" => Some(Self::FacilitiesRoomTypeCreate),
            "Facilities.RoomType.Read" => Some(Self::FacilitiesRoomTypeRead),
            "Facilities.RoomType.Update" => Some(Self::FacilitiesRoomTypeUpdate),
            "Facilities.RoomType.Delete" => Some(Self::FacilitiesRoomTypeDelete),
            "Facilities.ItemCategory.Create" => Some(Self::FacilitiesItemCategoryCreate),
            "Facilities.ItemCategory.Read" => Some(Self::FacilitiesItemCategoryRead),
            "Facilities.ItemCategory.Update" => Some(Self::FacilitiesItemCategoryUpdate),
            "Facilities.ItemCategory.Delete" => Some(Self::FacilitiesItemCategoryDelete),
            "Facilities.Item.Create" => Some(Self::FacilitiesItemCreate),
            "Facilities.Item.Read" => Some(Self::FacilitiesItemRead),
            "Facilities.Item.Update" => Some(Self::FacilitiesItemUpdate),
            "Facilities.Item.Delete" => Some(Self::FacilitiesItemDelete),
            "Facilities.ItemStore.Create" => Some(Self::FacilitiesItemStoreCreate),
            "Facilities.ItemStore.Read" => Some(Self::FacilitiesItemStoreRead),
            "Facilities.ItemStore.Update" => Some(Self::FacilitiesItemStoreUpdate),
            "Facilities.ItemStore.Delete" => Some(Self::FacilitiesItemStoreDelete),
            "Facilities.Inventory.Receive" => Some(Self::FacilitiesInventoryReceive),
            "Facilities.Inventory.UpdateReceive" => Some(Self::FacilitiesInventoryUpdateReceive),
            "Facilities.Inventory.CancelReceive" => Some(Self::FacilitiesInventoryCancelReceive),
            "Facilities.Inventory.Issue" => Some(Self::FacilitiesInventoryIssue),
            "Facilities.Inventory.UpdateIssue" => Some(Self::FacilitiesInventoryUpdateIssue),
            "Facilities.Inventory.ReturnIssued" => Some(Self::FacilitiesInventoryReturnIssued),
            "Facilities.Inventory.Sell" => Some(Self::FacilitiesInventorySell),
            "Facilities.Inventory.UpdateSell" => Some(Self::FacilitiesInventoryUpdateSell),
            "Facilities.Inventory.CancelSell" => Some(Self::FacilitiesInventoryCancelSell),
            "Facilities.Inventory.RefundSell" => Some(Self::FacilitiesInventoryRefundSell),
            "Facilities.Inventory.Read" => Some(Self::FacilitiesInventoryRead),
            "Facilities.Supplier.Create" => Some(Self::FacilitiesSupplierCreate),
            "Facilities.Supplier.Read" => Some(Self::FacilitiesSupplierRead),
            "Facilities.Supplier.Update" => Some(Self::FacilitiesSupplierUpdate),
            "Facilities.Supplier.Delete" => Some(Self::FacilitiesSupplierDelete),
            "Facilities.Supplier.Deactivate" => Some(Self::FacilitiesSupplierDeactivate),
            "Events.Calendar.Create" => Some(Self::EventsCalendarCreate),
            "Events.Calendar.Read" => Some(Self::EventsCalendarRead),
            "Events.Calendar.Update" => Some(Self::EventsCalendarUpdate),
            "Events.Calendar.Delete" => Some(Self::EventsCalendarDelete),
            "Events.Event.Create" => Some(Self::EventsEventCreate),
            "Events.Event.Read" => Some(Self::EventsEventRead),
            "Events.Event.Update" => Some(Self::EventsEventUpdate),
            "Events.Event.Delete" => Some(Self::EventsEventDelete),
            "Events.Event.Publish" => Some(Self::EventsEventPublish),
            "Events.Holiday.Create" => Some(Self::EventsHolidayCreate),
            "Events.Holiday.Read" => Some(Self::EventsHolidayRead),
            "Events.Holiday.Update" => Some(Self::EventsHolidayUpdate),
            "Events.Holiday.Delete" => Some(Self::EventsHolidayDelete),
            "Events.Weekend.Create" => Some(Self::EventsWeekendCreate),
            "Events.Weekend.Read" => Some(Self::EventsWeekendRead),
            "Events.Weekend.Update" => Some(Self::EventsWeekendUpdate),
            "Events.Weekend.Delete" => Some(Self::EventsWeekendDelete),
            "Events.Weekend.Configure" => Some(Self::EventsWeekendConfigure),
            "Events.Incident.Create" => Some(Self::EventsIncidentCreate),
            "Events.Incident.Read" => Some(Self::EventsIncidentRead),
            "Events.Incident.Update" => Some(Self::EventsIncidentUpdate),
            "Events.Incident.Delete" => Some(Self::EventsIncidentDelete),
            "Events.Incident.Assign" => Some(Self::EventsIncidentAssign),
            "Events.Incident.Reassign" => Some(Self::EventsIncidentReassign),
            "Events.Incident.Unassign" => Some(Self::EventsIncidentUnassign),
            "Events.Incident.Comment" => Some(Self::EventsIncidentComment),
            "Events.Incident.Resolve" => Some(Self::EventsIncidentResolve),
            "Events.IncidentComment.Delete" => Some(Self::EventsIncidentCommentDelete),
            "Events.CalendarSetting.Create" => Some(Self::EventsCalendarSettingCreate),
            "Events.CalendarSetting.Read" => Some(Self::EventsCalendarSettingRead),
            "Events.CalendarSetting.Update" => Some(Self::EventsCalendarSettingUpdate),
            "Events.CalendarSetting.Enable" => Some(Self::EventsCalendarSettingEnable),
            "Events.CalendarSetting.Disable" => Some(Self::EventsCalendarSettingDisable),
            "Events.CalendarSetting.Delete" => Some(Self::EventsCalendarSettingDelete),
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
            "Settings.General.Read" => Some(Self::SettingsGeneralRead),
            "Settings.General.Update" => Some(Self::SettingsGeneralUpdate),
            "Settings.General.TwoFactor.Toggle" => Some(Self::SettingsGeneralTwoFactorToggle),
            "Settings.Theme.Create" => Some(Self::SettingsThemeCreate),
            "Settings.Theme.Read" => Some(Self::SettingsThemeRead),
            "Settings.Theme.Update" => Some(Self::SettingsThemeUpdate),
            "Settings.Theme.Activate" => Some(Self::SettingsThemeActivate),
            "Settings.Theme.Delete" => Some(Self::SettingsThemeDelete),
            "Settings.Theme.Replicate" => Some(Self::SettingsThemeReplicate),
            "Settings.Theme.Select" => Some(Self::SettingsThemeSelect),
            "Settings.Color.Create" => Some(Self::SettingsColorCreate),
            "Settings.Color.Read" => Some(Self::SettingsColorRead),
            "Settings.Color.Update" => Some(Self::SettingsColorUpdate),
            "Settings.Color.Delete" => Some(Self::SettingsColorDelete),
            "Settings.ColorTheme.Create" => Some(Self::SettingsColorThemeCreate),
            "Settings.ColorTheme.Read" => Some(Self::SettingsColorThemeRead),
            "Settings.ColorTheme.Update" => Some(Self::SettingsColorThemeUpdate),
            "Settings.ColorTheme.Delete" => Some(Self::SettingsColorThemeDelete),
            "Settings.Language.Add" => Some(Self::SettingsLanguageAdd),
            "Settings.Language.Read" => Some(Self::SettingsLanguageRead),
            "Settings.Language.Update" => Some(Self::SettingsLanguageUpdate),
            "Settings.Language.Delete" => Some(Self::SettingsLanguageDelete),
            "Settings.Language.Activate" => Some(Self::SettingsLanguageActivate),
            "Settings.Language.Select" => Some(Self::SettingsLanguageSelect),
            "Settings.LanguagePhrase.Add" => Some(Self::SettingsLanguagePhraseAdd),
            "Settings.LanguagePhrase.Read" => Some(Self::SettingsLanguagePhraseRead),
            "Settings.LanguagePhrase.Update" => Some(Self::SettingsLanguagePhraseUpdate),
            "Settings.LanguagePhrase.Delete" => Some(Self::SettingsLanguagePhraseDelete),
            "Settings.LanguagePhrase.Translate" => Some(Self::SettingsLanguagePhraseTranslate),
            "Settings.BaseGroup.Add" => Some(Self::SettingsBaseGroupAdd),
            "Settings.BaseGroup.Read" => Some(Self::SettingsBaseGroupRead),
            "Settings.BaseGroup.Update" => Some(Self::SettingsBaseGroupUpdate),
            "Settings.BaseGroup.Delete" => Some(Self::SettingsBaseGroupDelete),
            "Settings.BaseSetup.Add" => Some(Self::SettingsBaseSetupAdd),
            "Settings.BaseSetup.Read" => Some(Self::SettingsBaseSetupRead),
            "Settings.BaseSetup.Update" => Some(Self::SettingsBaseSetupUpdate),
            "Settings.BaseSetup.Delete" => Some(Self::SettingsBaseSetupDelete),
            "Settings.DateFormat.Add" => Some(Self::SettingsDateFormatAdd),
            "Settings.DateFormat.Read" => Some(Self::SettingsDateFormatRead),
            "Settings.DateFormat.Update" => Some(Self::SettingsDateFormatUpdate),
            "Settings.DateFormat.Delete" => Some(Self::SettingsDateFormatDelete),
            "Settings.DateFormat.Select" => Some(Self::SettingsDateFormatSelect),
            "Settings.TimeZone.Select" => Some(Self::SettingsTimeZoneSelect),
            "Settings.Session.Select" => Some(Self::SettingsSessionSelect),
            "Settings.Style.Create" => Some(Self::SettingsStyleCreate),
            "Settings.Style.Read" => Some(Self::SettingsStyleRead),
            "Settings.Style.Update" => Some(Self::SettingsStyleUpdate),
            "Settings.Style.Activate" => Some(Self::SettingsStyleActivate),
            "Settings.Style.Delete" => Some(Self::SettingsStyleDelete),
            "Settings.Background.Create" => Some(Self::SettingsBackgroundCreate),
            "Settings.Background.Read" => Some(Self::SettingsBackgroundRead),
            "Settings.Background.Update" => Some(Self::SettingsBackgroundUpdate),
            "Settings.Background.Delete" => Some(Self::SettingsBackgroundDelete),
            "Settings.Dashboard.Create" => Some(Self::SettingsDashboardCreate),
            "Settings.Dashboard.Read" => Some(Self::SettingsDashboardRead),
            "Settings.Dashboard.Update" => Some(Self::SettingsDashboardUpdate),
            "Settings.Dashboard.Delete" => Some(Self::SettingsDashboardDelete),
            "Settings.CustomLink.Read" => Some(Self::SettingsCustomLinkRead),
            "Settings.CustomLink.Update" => Some(Self::SettingsCustomLinkUpdate),
            "Settings.CustomLink.Reset" => Some(Self::SettingsCustomLinkReset),
            "Settings.BehaviorRecord.Read" => Some(Self::SettingsBehaviorRecordRead),
            "Settings.BehaviorRecord.Update" => Some(Self::SettingsBehaviorRecordUpdate),
            "Settings.SetupAdmin.Add" => Some(Self::SettingsSetupAdminAdd),
            "Settings.SetupAdmin.Read" => Some(Self::SettingsSetupAdminRead),
            "Settings.SetupAdmin.Update" => Some(Self::SettingsSetupAdminUpdate),
            "Settings.SetupAdmin.Delete" => Some(Self::SettingsSetupAdminDelete),
            "Operations.Backup.Create" => Some(Self::OperationsBackupCreate),
            "Operations.Backup.Read" => Some(Self::OperationsBackupRead),
            "Operations.Backup.Delete" => Some(Self::OperationsBackupDelete),
            "Operations.Backup.Restore" => Some(Self::OperationsBackupRestore),
            "Operations.Backup.Activate" => Some(Self::OperationsBackupActivate),
            "Operations.Backup.Deactivate" => Some(Self::OperationsBackupDeactivate),
            "Operations.Job.Schedule" => Some(Self::OperationsJobSchedule),
            "Operations.Job.Read" => Some(Self::OperationsJobRead),
            "Operations.Job.Cancel" => Some(Self::OperationsJobCancel),
            "Operations.Job.Reserve" => Some(Self::OperationsJobReserve),
            "Operations.Job.Complete" => Some(Self::OperationsJobComplete),
            "Operations.Job.Fail" => Some(Self::OperationsJobFail),
            "Operations.Job.Retry" => Some(Self::OperationsJobRetry),
            "Operations.Job.Purge" => Some(Self::OperationsJobPurge),
            "Operations.FailedJob.Read" => Some(Self::OperationsFailedJobRead),
            "Operations.FailedJob.Retry" => Some(Self::OperationsFailedJobRetry),
            "Operations.FailedJob.Delete" => Some(Self::OperationsFailedJobDelete),
            "Operations.FailedJob.Purge" => Some(Self::OperationsFailedJobPurge),
            "Operations.Version.Register" => Some(Self::OperationsVersionRegister),
            "Operations.Version.Read" => Some(Self::OperationsVersionRead),
            "Operations.Version.Update" => Some(Self::OperationsVersionUpdate),
            "Operations.VersionHistory.Record" => Some(Self::OperationsVersionHistoryRecord),
            "Operations.VersionHistory.Read" => Some(Self::OperationsVersionHistoryRead),
            "Operations.Audit.Record" => Some(Self::OperationsAuditRecord),
            "Operations.Audit.Read" => Some(Self::OperationsAuditRead),
            "Operations.Maintenance.Read" => Some(Self::OperationsMaintenanceRead),
            "Operations.Maintenance.Configure" => Some(Self::OperationsMaintenanceConfigure),
            "Operations.Maintenance.Enable" => Some(Self::OperationsMaintenanceEnable),
            "Operations.Maintenance.Disable" => Some(Self::OperationsMaintenanceDisable),
            "Operations.Sidebar.Create" => Some(Self::OperationsSidebarCreate),
            "Operations.Sidebar.Read" => Some(Self::OperationsSidebarRead),
            "Operations.Sidebar.Update" => Some(Self::OperationsSidebarUpdate),
            "Operations.Sidebar.Delete" => Some(Self::OperationsSidebarDelete),
            "Operations.Sidebar.Reorder" => Some(Self::OperationsSidebarReorder),
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
    fn library_capabilities_round_trip_and_resolve_to_library_domain() {
        let mut count = 0u32;
        for c in Capability::all() {
            let s = c.as_str();
            if s.starts_with("Library.") {
                let parsed = Capability::from_str(s).unwrap();
                assert_eq!(parsed, *c, "round-trip failed for {s}");
                assert_eq!(
                    c.domain(),
                    CapabilityDomain::Library,
                    "domain mismatch for {s}"
                );
                count += 1;
            }
        }
        assert_eq!(
            count, 26,
            "expected 26 Library.* capabilities (4 Phase 2 placeholders deduplicated during implementation; got {count})"
        );
    }

    #[test]
    fn documents_capabilities_round_trip_and_resolve_to_documents_domain() {
        let mut count = 0u32;
        for c in Capability::all() {
            let s = c.as_str();
            if s.starts_with("Documents.") {
                let parsed = Capability::from_str(s).unwrap();
                assert_eq!(parsed, *c, "round-trip failed for {s}");
                assert_eq!(
                    c.domain(),
                    CapabilityDomain::Documents,
                    "domain mismatch for {s}"
                );
                count += 1;
            }
        }
        assert_eq!(
            count, 15,
            "expected 15 Documents.* capabilities (4 Phase 2 placeholders + 11 Phase 11 net-new; got {count})"
        );
    }

    #[test]
    fn cms_capabilities_round_trip_and_resolve_to_cms_domain() {
        let mut count = 0u32;
        for c in Capability::all() {
            let s = c.as_str();
            if s.starts_with("Cms.") && !s.starts_with("Cms.Read") {
                let parsed = Capability::from_str(s).unwrap();
                assert_eq!(parsed, *c, "round-trip failed for {s}");
                assert_eq!(c.domain(), CapabilityDomain::Cms, "domain mismatch for {s}");
                count += 1;
            }
        }
        // Phase 12: 4 placeholders (CmsPageCreate/Read/Update/Delete)
        // + ~80 net-new across 20 aggregates (one aggregate
        // typically has Create/Read/Update/Delete; News adds
        // Publish/Unpublish/IncrementView; NoticeBoard adds
        // Publish/Unpublish; HomePageSetting adds Configure).
        assert!(
            count >= 80,
            "expected >= 80 Cms.* capabilities (got {count})"
        );
    }

    #[test]
    fn events_capabilities_round_trip_and_resolve_to_events_domain() {
        let mut count = 0u32;
        for c in Capability::all() {
            let s = c.as_str();
            if s.starts_with("Events.") {
                let parsed = Capability::from_str(s).unwrap();
                assert_eq!(parsed, *c, "round-trip failed for {s}");
                assert_eq!(
                    c.domain(),
                    CapabilityDomain::Events,
                    "domain mismatch for {s}"
                );
                count += 1;
            }
        }
        // Phase 13: 4 placeholders (EventsCalendarCreate/Read/Update/Delete)
        // + 30 net-new across 6 aggregates (Event/Holiday/Weekend/Incident/
        // IncidentComment/CalendarSetting).
        assert!(
            count >= 30,
            "expected >= 30 Events.* capabilities (got {count})"
        );
    }

    #[test]
    fn settings_capabilities_round_trip_and_resolve_to_settings_domain() {
        // Phase 14: 66 Settings.* capabilities per
        // `docs/specs/settings/permissions.md`. The Phase 2
        // `SettingsManage` placeholder is REPLACED by the
        // per-aggregate catalog.
        let mut count = 0u32;
        let mut seen_wires: std::collections::HashSet<String> = std::collections::HashSet::new();
        for c in Capability::all() {
            let s = c.as_str();
            if s.starts_with("Settings.") {
                let parsed = Capability::from_str(s).unwrap();
                assert_eq!(parsed, *c, "round-trip failed for {s}");
                assert_eq!(
                    c.domain(),
                    CapabilityDomain::Settings,
                    "domain mismatch for {s}"
                );
                assert!(
                    s.chars().all(|c| c.is_ascii_alphanumeric() || c == '.'),
                    "wire string {s:?} is not dot-separated ASCII"
                );
                assert!(
                    !s.starts_with('.') && !s.ends_with('.'),
                    "wire string {s:?} has a leading or trailing dot"
                );
                assert!(
                    seen_wires.insert(s.to_owned()),
                    "duplicate wire string: {s:?}"
                );
                count += 1;
            }
        }
        assert!(
            count >= 66,
            "expected >= 66 Settings.* capabilities (got {count})"
        );
    }

    #[test]
    fn operations_capabilities_round_trip_and_resolve_to_operations_domain() {
        // Phase 14: 34 Operations.* capabilities per
        // `docs/specs/operations/permissions.md`. The Phase 2
        // `OperationsManage` placeholder is REPLACED by the
        // per-aggregate catalog.
        let mut count = 0u32;
        let mut seen_wires: std::collections::HashSet<String> = std::collections::HashSet::new();
        for c in Capability::all() {
            let s = c.as_str();
            if s.starts_with("Operations.") {
                let parsed = Capability::from_str(s).unwrap();
                assert_eq!(parsed, *c, "round-trip failed for {s}");
                assert_eq!(
                    c.domain(),
                    CapabilityDomain::Operations,
                    "domain mismatch for {s}"
                );
                assert!(
                    s.chars().all(|c| c.is_ascii_alphanumeric() || c == '.'),
                    "wire string {s:?} is not dot-separated ASCII"
                );
                assert!(
                    !s.starts_with('.') && !s.ends_with('.'),
                    "wire string {s:?} has a leading or trailing dot"
                );
                assert!(
                    seen_wires.insert(s.to_owned()),
                    "duplicate wire string: {s:?}"
                );
                count += 1;
            }
        }
        assert!(
            count >= 34,
            "expected >= 34 Operations.* capabilities (got {count})"
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
            // Two-segment exception: `Rbac.Bootstrap`. Every
            // other capability uses the three-segment form
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
                    || s == "Rbac.Bootstrap",
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
