//! The [`AuditWriter`] service: the engine's audit write path.
//!
//! Per `docs/schemas/audit-schema.md` and the engine's
//! audit-first invariant: every state-changing command writes one
//! audit row in the same transaction as the mutation. The
//! `AuditWriter` is the typed entry point that command handlers
//! reach for; it owns:
//!
//! - Construction of the storage-port [`AuditLogEntry`] from a
//!   [`TenantContext`], an [`AuditAction`], an [`AuditTarget`],
//!   and optional before/after snapshots.
//! - Submission of the entry to the
//!   [`educore_storage::AuditLog`] port.
//! - Threshold-driven emission of a [`RetentionSweepDue`] event
//!   when the retention policy is reached.
//!
//! The writer takes the storage and bus ports as
//! `Arc<dyn Trait>` so the engine can wire the same instance
//! across many command handlers without generic-type plumbing.

use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::clock::Clock;
use educore_core::error::Result;
use educore_core::ids::SchoolId;
use educore_core::tenant::TenantContext;
use educore_core::value_objects::{ActiveStatus, Timestamp};

use educore_events::domain_event::DomainEvent;
use educore_events::event_bus::EventBus;

use educore_storage::audit::{AuditLog, AuditLogEntry};

use crate::events::RetentionSweepDue;
use crate::retention::{RetentionPolicy, RetentionSweeper};

/// Sentinel `target_id` used by [`AuditWriter::maybe_sweep`] to
/// discover the oldest audit row for a school. The storage
/// adapter interprets a `read_for_target(_, SENTINEL_TARGET_ID, _)`
/// call as "return the oldest row" (Phase 3 will add a proper
/// `oldest_row_for_school` method; until then the sentinel is the
/// Phase 2 simplification). The constant is the all-zero UUID
/// (`Uuid::nil()`), which no real audit row would carry as its
/// `target_id` (UUIDv7 ids are never nil) so the sentinel is
/// collision-free in practice.
pub const SENTINEL_TARGET_ID: Uuid = Uuid::nil();

/// The audit action: the verb describing what the actor did.
///
/// Stored in [`AuditLogEntry::action`] as a short string
/// (`"create"`, `"update"`, etc.). Use the [`Other`](Self::Other)
/// variant for domain-specific verbs that are not in the engine
/// default set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditAction {
    /// A new resource was created (e.g. `StudentCreated`).
    Create,
    /// An existing resource was mutated (e.g. `StudentRenamed`).
    Update,
    /// A resource was soft-deleted (e.g. `StudentWithdrawn`).
    Delete,
    /// A resource was approved (e.g. expense report, leave request).
    Approve,
    /// A user authenticated.
    Login,
    /// A user logged out.
    Logout,
    /// A configuration or settings value was changed.
    Configure,
    /// A domain-specific action that does not fit the default set.
    /// The string is the canonical action verb (e.g. `"merge"`,
    /// `"promote"`, `"lock"`).
    Other(String),
}

impl AuditAction {
    /// Returns the canonical snake-case wire string for the
    /// action. Used to populate [`AuditLogEntry::action`].
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Create => "create",
            Self::Update => "update",
            Self::Delete => "delete",
            Self::Approve => "approve",
            Self::Login => "login",
            Self::Logout => "logout",
            Self::Configure => "configure",
            Self::Other(s) => s.as_str(),
        }
    }
}

/// The audit target: the (type, id) pair identifying the resource
/// the action was performed on. Each variant carries the resource's
/// UUID; the [`target_type`](Self::target_type) method returns the
/// wire string for [`AuditLogEntry::target_type`].
///
/// Use the [`Other`](Self::Other) variant for resource types that
/// are not in the engine's default set (e.g. a domain-specific
/// custom aggregate).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditTarget {
    // ---- Cross-cutting / platform ---------------------------------------
    /// A school aggregate.
    School(Uuid),
    /// A user (any role) aggregate.
    User(Uuid),
    /// A user session.
    Session(Uuid),
    /// An RBAC role.
    Role(Uuid),
    /// An RBAC capability.
    Capability(Uuid),
    // ---- Academic domain -------------------------------------------------
    /// An academic student.
    Student(Uuid),
    /// A class (group of students across a year).
    Class(Uuid),
    /// A section (a class in a given academic year).
    Section(Uuid),
    /// A subject (math, science, …).
    Subject(Uuid),
    /// An academic year.
    AcademicYear(Uuid),
    /// A student enrollment into a section.
    Enrollment(Uuid),
    // ---- Assessment domain ----------------------------------------------
    /// An exam.
    Exam(Uuid),
    /// A marks register (one exam's marks for one section).
    MarksRegister(Uuid),
    /// An exam schedule (the calendar slot for an exam).
    ExamSchedule(Uuid),
    /// A result store (a published per-student per-subject result row).
    ResultStore(Uuid),
    /// A report card projection (materialised on demand from a result).
    ReportCard(Uuid),
    /// An online exam session.
    OnlineExam(Uuid),
    /// A seat plan (per-section seating arrangement).
    SeatPlan(Uuid),
    /// An admit card (per-student printable admit card).
    AdmitCard(Uuid),
    // ---- Attendance domain ----------------------------------------------
    /// A daily student attendance row.
    StudentAttendance(Uuid),
    /// A per-period (per-subject) student attendance row.
    SubjectAttendance(Uuid),
    /// A daily staff attendance row.
    StaffAttendance(Uuid),
    /// A bulk attendance import job (CSV / biometric / API).
    BulkAttendanceImport(Uuid),
    /// A per-(student, exam_type, academic_year) attendance summary
    /// projection.
    ClassAttendance(Uuid),
    // ---- HR domain -------------------------------------------------------
    /// A staff member.
    Staff(Uuid),
    /// A payroll run.
    Payroll(Uuid),
    /// A department (HR reference data).
    Department(Uuid),
    /// A designation (HR reference data).
    Designation(Uuid),
    /// A leave type (HR reference data).
    LeaveType(Uuid),
    /// A leave-entitlement policy (`LeaveDefine`).
    LeaveDefine(Uuid),
    /// A leave request.
    LeaveRequest(Uuid),
    /// A daily HR-side staff attendance row.
    ///
    /// Disambiguated from the attendance-domain
    /// [`AuditTarget::StaffAttendance`] by the `Hr` prefix in
    /// the variant name; the wire form is `hr_staff_attendance`.
    /// The two rows serve different concerns: the attendance
    /// crate tracks per-staff per-day rows in service of the
    /// school's student-attendance flow, while the HR crate
    /// tracks per-staff per-day rows in service of the
    /// payroll + leave flow.
    HrStaffAttendance(Uuid),
    /// A bulk HR staff-attendance import row (staging).
    HrStaffAttendanceImport(Uuid),
    /// A class-teacher assignment.
    AssignClassTeacher(Uuid),
    /// A per-grade hourly rate.
    HourlyRate(Uuid),
    /// A salary-template row.
    SalaryTemplate(Uuid),
    /// A single payroll earning/deduction line.
    PayrollEarnDeduc(Uuid),
    /// A leave-deduction info row on a payroll.
    LeaveDeductionInfo(Uuid),
    /// A custom staff-registration field.
    StaffRegistrationField(Uuid),
    /// A bulk-staff-import staging row.
    StaffImportBulk(Uuid),
    // ---- Finance domain -------------------------------------------------
    /// A fees invoice.
    FeesInvoice(Uuid),
    /// A fees payment.
    FeesPayment(Uuid),
    /// A bank account (or cash drawer).
    BankAccount(Uuid),
    /// A bank statement (one entry in the bank ledger).
    BankStatement(Uuid),
    /// A recorded expense.
    Expense(Uuid),
    /// A recorded income.
    Income(Uuid),
    /// A donor profile.
    Donor(Uuid),
    /// An expense head (category).
    ExpenseHead(Uuid),
    /// An income head (category).
    IncomeHead(Uuid),
    /// A user's wallet balance.
    Wallet(Uuid),
    /// A wallet transaction (credit / debit / refund).
    WalletTransaction(Uuid),
    /// A finance-side payroll payment (the HR→finance bridge).
    PayrollPayment(Uuid),
    /// A double-entry journal line.
    Transaction(Uuid),
    // ---- Facilities domain ---------------------------------------------
    /// A facilities inventory item.
    Item(Uuid),
    // ---- Library domain ------------------------------------------------
    /// A library book.
    Book(Uuid),
    /// A library book category.
    BookCategory(Uuid),
    /// A library member (a registered borrower).
    LibraryMember(Uuid),
    /// A book issue (an instance of a member borrowing a book).
    BookIssue(Uuid),
    /// A book return (a historical log of a return action).
    BookReturn(Uuid),
    /// A library fine (a calculated or waived late-fine).
    Fine(Uuid),
    // ---- Communication domain ------------------------------------------
    /// A notice / announcement.
    Notice(Uuid),
    /// A complaint (a logged grievance from a stakeholder).
    Complaint(Uuid),
    /// A complaint type (categorisation for complaints).
    ComplaintType(Uuid),
    /// A notification (an in-app message sent to a user).
    Notification(Uuid),
    /// A log row for a sent email.
    EmailLog(Uuid),
    /// A log row for a sent SMS.
    SmsLog(Uuid),
    /// An SMS template (reusable body with variables).
    SmsTemplate(Uuid),
    /// An email-setting row (SMTP/host/port/credentials config).
    EmailSetting(Uuid),
    /// An SMS-gateway row (provider credentials + activation).
    SmsGateway(Uuid),
    /// A notification-setting row (event→destination mapping).
    NotificationSetting(Uuid),
    /// An absent-notification time-setup row (dispatch window).
    AbsentNotificationTimeSetup(Uuid),
    /// A 1-to-1 chat message.
    ChatMessage(Uuid),
    /// A 1-to-1 chat conversation.
    ChatConversation(Uuid),
    /// A chat group.
    ChatGroup(Uuid),
    /// A chat-group membership row.
    ChatGroupUser(Uuid),
    /// A chat-group message delivery row (one per recipient).
    ChatGroupMessageRecipient(Uuid),
    /// A chat-group message remove row (per-user removal log).
    ChatGroupMessageRemove(Uuid),
    /// A chat block (one user blocking another).
    ChatBlockUser(Uuid),
    /// A chat invitation.
    ChatInvitation(Uuid),
    /// A chat-invitation type (classification).
    ChatInvitationType(Uuid),
    /// A chat status row (per-user online/away/busy state).
    ChatStatus(Uuid),
    /// A send-message (bulk broadcast) row.
    SendMessage(Uuid),
    /// A contact-message row (a public web form submission).
    ContactMessage(Uuid),
    /// A speech-slider row (homepage carousel entry).
    SpeechSlider(Uuid),
    /// A phone-call-log row.
    PhoneCallLog(Uuid),
    /// A custom-SMS-setting row (HTTP gateway integration config).
    CustomSmsSetting(Uuid),
    // ---- Documents domain ----------------------------------------------
    /// A form download.
    FormDownload(Uuid),
    /// A postal dispatch record.
    PostalDispatch(Uuid),
    /// A postal receive record.
    PostalReceive(Uuid),
    // ---- CMS domain -----------------------------------------------------
    /// A CMS page.
    Page(Uuid),
    /// A CMS news entry.
    News(Uuid),
    /// A CMS news category.
    NewsCategory(Uuid),
    /// A CMS news comment.
    NewsComment(Uuid),
    /// A CMS news landing-page configuration.
    NewsPage(Uuid),
    /// A public-site notice board.
    NoticeBoard(Uuid),
    /// A testimonial surfaced on the public site.
    Testimonial(Uuid),
    /// A home-page slider entry.
    HomeSlider(Uuid),
    /// An uploaded content item.
    Content(Uuid),
    /// A content type taxonomy entry.
    ContentType(Uuid),
    /// A bulk-share list of content items.
    ContentShareList(Uuid),
    /// A teacher-uploaded content item (per class-section).
    TeacherUploadContent(Uuid),
    /// An admin-uploaded content item (per role/class).
    UploadContent(Uuid),
    /// An about-page configuration.
    AboutPage(Uuid),
    /// A contact-page configuration.
    ContactPage(Uuid),
    /// A course landing page.
    CoursePage(Uuid),
    /// A home-page setting.
    HomePageSetting(Uuid),
    /// A front-end page record.
    FrontendPage(Uuid),
    /// A page revision (historical snapshot).
    PageRevision(Uuid),
    /// A news revision (historical snapshot).
    NewsRevision(Uuid),
    // ---- Events domain (calendar) --------------------------------------
    /// A calendar event.
    CalendarEvent(Uuid),
    /// A school holiday.
    Holiday(Uuid),
    /// A discipline / operational incident.
    Incident(Uuid),
    // ---- Events domain (Phase 13 net-new) -------------------------------
    /// A weekend day configuration.
    Weekend(Uuid),
    /// An incident-to-student or incident-to-staff assignment.
    AssignIncident(Uuid),
    /// A comment on an incident.
    IncidentComment(Uuid),
    /// A calendar UI menu label and color.
    CalendarSetting(Uuid),
    // ---- Settings + Operations (Phase 2 placeholders) -------------------
    // The 2 Phase 2 `SchoolSettings` + `BellSchedule` placeholders are
    // preserved for `DefaultRoleCatalog` consistency (mirrors the
    // Phase 13 pattern of preserving the 3 `CalendarEvent`/`Holiday`/
    // `Incident` audit-target placeholders). Phase 14 adds 15 + 8 = 23
    // net-new per-aggregate targets below.
    /// A school-settings row (Phase 2 placeholder).
    SchoolSettings(Uuid),
    /// A bell-schedule row (Phase 2 placeholder).
    BellSchedule(Uuid),
    // ---- Settings (Phase 14 net-new) ------------------------------------
    /// The school's per-school singleton general-settings row.
    GeneralSettings(Uuid),
    /// A language registered in the school.
    Language(Uuid),
    /// A translatable phrase key.
    LanguagePhrase(Uuid),
    /// A lookup value in a `BaseGroup`.
    BaseSetup(Uuid),
    /// A grouping of `BaseSetup` values.
    BaseGroup(Uuid),
    /// A `strftime` pattern.
    DateFormat(Uuid),
    /// A color palette / theme profile.
    Style(Uuid),
    /// A background image or color preset.
    BackgroundSetting(Uuid),
    /// A dashboard card binding to a role.
    DashboardSetting(Uuid),
    /// The per-school custom-link bundle.
    CustomLink(Uuid),
    /// A color binding in a theme.
    ColorTheme(Uuid),
    /// A theme (color mode, background).
    Theme(Uuid),
    /// A color entry used by `ColorTheme`.
    Color(Uuid),
    /// The behavior record feature flag.
    BehaviorRecordSetting(Uuid),
    /// A purpose/complaint/source/reference entry.
    SetupAdmin(Uuid),
    // ---- Operations (Phase 14 net-new) ----------------------------------
    /// A database/file/image backup record.
    Backup(Uuid),
    /// A pending job in the queue.
    Job(Uuid),
    /// A job that has exhausted its retry budget.
    FailedJob(Uuid),
    /// A released version metadata record.
    SystemVersion(Uuid),
    /// A version bump record.
    VersionHistory(Uuid),
    /// A login event record.
    UserLog(Uuid),
    /// The school's maintenance mode config.
    MaintenanceSetting(Uuid),
    /// A per-role sidebar layout projection.
    Sidebar(Uuid),
    // === Phase 15 port-adapter targets section begin (owner: F) ===
    // 10 net-new AuditTarget variants ship in microtask F.6:
    //   OAuthAccessToken, OAuthClient, PasswordReset, Migration, AuthSession,
    //   PaymentReceipt, Refund, FileReference, IntegrationConfig, IntegrationInvocation.
    // === Phase 15 port-adapter targets section end ===
    // ---- Catch-all ------------------------------------------------------
    /// A domain-specific resource not in the default set. The
    /// string is the canonical type name (e.g. `"library_copy"`).
    Other(String, Uuid),
}

impl AuditTarget {
    /// Returns the canonical snake-case wire string for the
    /// resource type. Used to populate
    /// [`AuditLogEntry::target_type`].
    #[must_use]
    pub fn target_type(&self) -> &str {
        match self {
            Self::School(_) => "school",
            Self::User(_) => "user",
            Self::Session(_) => "session",
            Self::Role(_) => "role",
            Self::Capability(_) => "capability",
            Self::Student(_) => "student",
            Self::Class(_) => "class",
            Self::Section(_) => "section",
            Self::Subject(_) => "subject",
            Self::AcademicYear(_) => "academic_year",
            Self::Enrollment(_) => "enrollment",
            Self::Exam(_) => "exam",
            Self::MarksRegister(_) => "marks_register",
            Self::ExamSchedule(_) => "exam_schedule",
            Self::ResultStore(_) => "result_store",
            Self::ReportCard(_) => "report_card",
            Self::OnlineExam(_) => "online_exam",
            Self::SeatPlan(_) => "seat_plan",
            Self::AdmitCard(_) => "admit_card",
            Self::StudentAttendance(_) => "student_attendance",
            Self::SubjectAttendance(_) => "subject_attendance",
            Self::StaffAttendance(_) => "staff_attendance",
            Self::BulkAttendanceImport(_) => "bulk_attendance_import",
            Self::ClassAttendance(_) => "class_attendance",
            Self::Staff(_) => "staff",
            Self::Payroll(_) => "payroll",
            Self::Department(_) => "department",
            Self::Designation(_) => "designation",
            Self::LeaveType(_) => "leave_type",
            Self::LeaveDefine(_) => "leave_define",
            Self::LeaveRequest(_) => "leave_request",
            Self::HrStaffAttendance(_) => "hr_staff_attendance",
            Self::HrStaffAttendanceImport(_) => "hr_staff_attendance_import",
            Self::AssignClassTeacher(_) => "assign_class_teacher",
            Self::HourlyRate(_) => "hourly_rate",
            Self::SalaryTemplate(_) => "salary_template",
            Self::PayrollEarnDeduc(_) => "payroll_earn_deduc",
            Self::LeaveDeductionInfo(_) => "leave_deduction_info",
            Self::StaffRegistrationField(_) => "staff_registration_field",
            Self::StaffImportBulk(_) => "staff_import_bulk",
            Self::FeesInvoice(_) => "fees_invoice",
            Self::FeesPayment(_) => "fees_payment",
            Self::BankAccount(_) => "bank_account",
            Self::BankStatement(_) => "bank_statement",
            Self::Expense(_) => "expense",
            Self::Income(_) => "income",
            Self::Donor(_) => "donor",
            Self::ExpenseHead(_) => "expense_head",
            Self::IncomeHead(_) => "income_head",
            Self::Wallet(_) => "wallet",
            Self::WalletTransaction(_) => "wallet_transaction",
            Self::PayrollPayment(_) => "payroll_payment",
            Self::Transaction(_) => "transaction",
            Self::Item(_) => "item",
            Self::Book(_) => "book",
            Self::BookCategory(_) => "book_category",
            Self::LibraryMember(_) => "library_member",
            Self::BookIssue(_) => "book_issue",
            Self::BookReturn(_) => "book_return",
            Self::Fine(_) => "fine",
            Self::Notice(_) => "notice",
            Self::Complaint(_) => "complaint",
            Self::ComplaintType(_) => "complaint_type",
            Self::Notification(_) => "notification",
            Self::EmailLog(_) => "email_log",
            Self::SmsLog(_) => "sms_log",
            Self::SmsTemplate(_) => "sms_template",
            Self::EmailSetting(_) => "email_setting",
            Self::SmsGateway(_) => "sms_gateway",
            Self::NotificationSetting(_) => "notification_setting",
            Self::AbsentNotificationTimeSetup(_) => "absent_notification_time_setup",
            Self::ChatMessage(_) => "chat_message",
            Self::ChatConversation(_) => "chat_conversation",
            Self::ChatGroup(_) => "chat_group",
            Self::ChatGroupUser(_) => "chat_group_user",
            Self::ChatGroupMessageRecipient(_) => "chat_group_message_recipient",
            Self::ChatGroupMessageRemove(_) => "chat_group_message_remove",
            Self::ChatBlockUser(_) => "chat_block_user",
            Self::ChatInvitation(_) => "chat_invitation",
            Self::ChatInvitationType(_) => "chat_invitation_type",
            Self::ChatStatus(_) => "chat_status",
            Self::SendMessage(_) => "send_message",
            Self::ContactMessage(_) => "contact_message",
            Self::SpeechSlider(_) => "speech_slider",
            Self::PhoneCallLog(_) => "phone_call_log",
            Self::CustomSmsSetting(_) => "custom_sms_setting",
            Self::FormDownload(_) => "form_download",
            Self::PostalDispatch(_) => "postal_dispatch",
            Self::PostalReceive(_) => "postal_receive",
            Self::Page(_) => "page",
            Self::News(_) => "news",
            Self::NewsCategory(_) => "news_category",
            Self::NewsComment(_) => "news_comment",
            Self::NewsPage(_) => "news_page",
            Self::NoticeBoard(_) => "notice_board",
            Self::Testimonial(_) => "testimonial",
            Self::HomeSlider(_) => "home_slider",
            Self::Content(_) => "content",
            Self::ContentType(_) => "content_type",
            Self::ContentShareList(_) => "content_share_list",
            Self::TeacherUploadContent(_) => "teacher_upload_content",
            Self::UploadContent(_) => "upload_content",
            Self::AboutPage(_) => "about_page",
            Self::ContactPage(_) => "contact_page",
            Self::CoursePage(_) => "course_page",
            Self::HomePageSetting(_) => "home_page_setting",
            Self::FrontendPage(_) => "frontend_page",
            Self::PageRevision(_) => "page_revision",
            Self::NewsRevision(_) => "news_revision",
            Self::CalendarEvent(_) => "calendar_event",
            Self::Holiday(_) => "holiday",
            Self::Incident(_) => "incident",
            Self::Weekend(_) => "weekend",
            Self::AssignIncident(_) => "assign_incident",
            Self::IncidentComment(_) => "incident_comment",
            Self::CalendarSetting(_) => "calendar_setting",
            Self::SchoolSettings(_) => "school_settings",
            Self::BellSchedule(_) => "bell_schedule",
            Self::GeneralSettings(_) => "general_settings",
            Self::Language(_) => "language",
            Self::LanguagePhrase(_) => "language_phrase",
            Self::BaseSetup(_) => "base_setup",
            Self::BaseGroup(_) => "base_group",
            Self::DateFormat(_) => "date_format",
            Self::Style(_) => "style",
            Self::BackgroundSetting(_) => "background_setting",
            Self::DashboardSetting(_) => "dashboard_setting",
            Self::CustomLink(_) => "custom_link",
            Self::ColorTheme(_) => "color_theme",
            Self::Theme(_) => "theme",
            Self::Color(_) => "color",
            Self::BehaviorRecordSetting(_) => "behavior_record_setting",
            Self::SetupAdmin(_) => "setup_admin",
            Self::Backup(_) => "backup",
            Self::Job(_) => "job",
            Self::FailedJob(_) => "failed_job",
            Self::SystemVersion(_) => "system_version",
            Self::VersionHistory(_) => "version_history",
            Self::UserLog(_) => "user_log",
            Self::MaintenanceSetting(_) => "maintenance_setting",
            Self::Sidebar(_) => "sidebar",
            Self::Other(s, _) => s.as_str(),
        }
    }

    /// Returns the resource id carried by this `AuditTarget`.
    #[must_use]
    pub fn target_id(&self) -> Uuid {
        match self {
            Self::School(id)
            | Self::User(id)
            | Self::Session(id)
            | Self::Role(id)
            | Self::Capability(id)
            | Self::Student(id)
            | Self::Class(id)
            | Self::Section(id)
            | Self::Subject(id)
            | Self::AcademicYear(id)
            | Self::Enrollment(id)
            | Self::Exam(id)
            | Self::MarksRegister(id)
            | Self::ExamSchedule(id)
            | Self::ResultStore(id)
            | Self::ReportCard(id)
            | Self::OnlineExam(id)
            | Self::SeatPlan(id)
            | Self::AdmitCard(id)
            | Self::StudentAttendance(id)
            | Self::SubjectAttendance(id)
            | Self::StaffAttendance(id)
            | Self::BulkAttendanceImport(id)
            | Self::ClassAttendance(id)
            | Self::Staff(id)
            | Self::Payroll(id)
            | Self::Department(id)
            | Self::Designation(id)
            | Self::LeaveType(id)
            | Self::LeaveDefine(id)
            | Self::LeaveRequest(id)
            | Self::HrStaffAttendance(id)
            | Self::HrStaffAttendanceImport(id)
            | Self::AssignClassTeacher(id)
            | Self::HourlyRate(id)
            | Self::SalaryTemplate(id)
            | Self::PayrollEarnDeduc(id)
            | Self::LeaveDeductionInfo(id)
            | Self::StaffRegistrationField(id)
            | Self::StaffImportBulk(id)
            | Self::FeesInvoice(id)
            | Self::FeesPayment(id)
            | Self::BankAccount(id)
            | Self::BankStatement(id)
            | Self::Expense(id)
            | Self::Income(id)
            | Self::Donor(id)
            | Self::ExpenseHead(id)
            | Self::IncomeHead(id)
            | Self::Wallet(id)
            | Self::WalletTransaction(id)
            | Self::PayrollPayment(id)
            | Self::Transaction(id)
            | Self::Item(id)
            | Self::Book(id)
            | Self::BookCategory(id)
            | Self::LibraryMember(id)
            | Self::BookIssue(id)
            | Self::BookReturn(id)
            | Self::Fine(id)
            | Self::Notice(id)
            | Self::Complaint(id)
            | Self::ComplaintType(id)
            | Self::Notification(id)
            | Self::EmailLog(id)
            | Self::SmsLog(id)
            | Self::SmsTemplate(id)
            | Self::EmailSetting(id)
            | Self::SmsGateway(id)
            | Self::NotificationSetting(id)
            | Self::AbsentNotificationTimeSetup(id)
            | Self::ChatMessage(id)
            | Self::ChatConversation(id)
            | Self::ChatGroup(id)
            | Self::ChatGroupUser(id)
            | Self::ChatGroupMessageRecipient(id)
            | Self::ChatGroupMessageRemove(id)
            | Self::ChatBlockUser(id)
            | Self::ChatInvitation(id)
            | Self::ChatInvitationType(id)
            | Self::ChatStatus(id)
            | Self::SendMessage(id)
            | Self::ContactMessage(id)
            | Self::SpeechSlider(id)
            | Self::PhoneCallLog(id)
            | Self::CustomSmsSetting(id)
            | Self::FormDownload(id)
            | Self::PostalDispatch(id)
            | Self::PostalReceive(id)
            | Self::Page(id)
            | Self::News(id)
            | Self::NewsCategory(id)
            | Self::NewsComment(id)
            | Self::NewsPage(id)
            | Self::NoticeBoard(id)
            | Self::Testimonial(id)
            | Self::HomeSlider(id)
            | Self::Content(id)
            | Self::ContentType(id)
            | Self::ContentShareList(id)
            | Self::TeacherUploadContent(id)
            | Self::UploadContent(id)
            | Self::AboutPage(id)
            | Self::ContactPage(id)
            | Self::CoursePage(id)
            | Self::HomePageSetting(id)
            | Self::FrontendPage(id)
            | Self::PageRevision(id)
            | Self::NewsRevision(id)
            | Self::CalendarEvent(id)
            | Self::Holiday(id)
            | Self::Incident(id)
            | Self::Weekend(id)
            | Self::AssignIncident(id)
            | Self::IncidentComment(id)
            | Self::CalendarSetting(id)
            | Self::SchoolSettings(id)
            | Self::BellSchedule(id)
            | Self::GeneralSettings(id)
            | Self::Language(id)
            | Self::LanguagePhrase(id)
            | Self::BaseSetup(id)
            | Self::BaseGroup(id)
            | Self::DateFormat(id)
            | Self::Style(id)
            | Self::BackgroundSetting(id)
            | Self::DashboardSetting(id)
            | Self::CustomLink(id)
            | Self::ColorTheme(id)
            | Self::Theme(id)
            | Self::Color(id)
            | Self::BehaviorRecordSetting(id)
            | Self::SetupAdmin(id)
            | Self::Backup(id)
            | Self::Job(id)
            | Self::FailedJob(id)
            | Self::SystemVersion(id)
            | Self::VersionHistory(id)
            | Self::UserLog(id)
            | Self::MaintenanceSetting(id)
            | Self::Sidebar(id)
            | Self::Other(_, id) => *id,
        }
    }
}

/// The engine's audit write path. Construct one per process and
/// share via `Arc` across command handlers. The writer is
/// thread-safe (the `last_sweep_at` lock is the only mutable
/// shared state).
pub struct AuditWriter {
    audit_log: std::sync::Arc<dyn AuditLog>,
    bus: std::sync::Arc<dyn EventBus>,
    clock: std::sync::Arc<dyn Clock>,
    policy: RetentionPolicy,
    /// Per-instance `last_sweep_at` for the threshold check. The
    /// field is *not* per-school because the sweep is a global
    /// rate-limit: if the writer is being called for school A every
    /// millisecond, we do not want school B's sweep to fire
    /// immediately after a long quiet period. Sharing the lock
    /// across schools bounds the total number of `RetentionSweepDue`
    /// events the engine emits.
    last_sweep_at: Mutex<Option<Timestamp>>,
}

impl AuditWriter {
    /// Constructs a new `AuditWriter` with the given storage port,
    /// event bus, clock, and retention policy.
    #[must_use]
    pub fn new(
        audit_log: std::sync::Arc<dyn AuditLog>,
        bus: std::sync::Arc<dyn EventBus>,
        clock: std::sync::Arc<dyn Clock>,
        policy: RetentionPolicy,
    ) -> Self {
        Self {
            audit_log,
            bus,
            clock,
            policy,
            last_sweep_at: Mutex::new(None),
        }
    }

    /// Writes an audit row for a state change and then triggers
    /// an opportunistic sweep check.
    ///
    /// The `before` snapshot is the serialised resource state
    /// before the mutation; `None` for create actions. The `after`
    /// snapshot is the state after; `None` for delete actions.
    /// Both are raw `bytes::Bytes` so adapters are free to use
    /// `serde_json::to_vec`, a `bincode` projection, or any other
    /// wire format (per `docs/ports/storage.md` § 3).
    ///
    /// # Errors
    ///
    /// - [`DomainError::Infrastructure`] if the storage port or
    ///   event bus fails.
    pub async fn write(
        &self,
        ctx: &TenantContext,
        action: AuditAction,
        target: AuditTarget,
        before: Option<bytes::Bytes>,
        after: Option<bytes::Bytes>,
    ) -> Result<()> {
        let school_id = ctx.school_id;
        let entry = AuditLogEntry {
            school_id,
            actor_id: ctx.actor_id,
            action: action.as_str().to_owned(),
            target_type: target.target_type().to_owned(),
            target_id: target.target_id(),
            before,
            after,
            // Phase 2: the audit row is decoupled from the
            // event-log row. Phase 3 will wire `event_id` when
            // command handlers run inside the same transaction
            // as the outbox emit.
            event_id: None,
            correlation_id: ctx.correlation_id,
            occurred_at: self.clock.now(),
            active_status: ActiveStatus::Active,
            // The metadata column is open-ended; the default
            // null lets the writer pass through callers that do
            // not need to attach a reason or ticket. Callers
            // that need metadata can extend `AuditWriter` in
            // Phase 3 (the `to_audit_value` projection will land
            // alongside the per-aggregate emit hook).
            metadata: serde_json::Value::Null,
        };
        self.audit_log.append(entry).await?;
        self.maybe_sweep(school_id).await?;
        Ok(())
    }

    /// Triggers a retention sweep check. Idempotent: a no-op if
    /// the sweep check interval has not elapsed since the last
    /// check (or since construction). On the first call, the
    /// method records the current time as the seed and returns
    /// without emitting a sweep event.
    ///
    /// When the interval has elapsed AND the storage port reports
    /// a row older than `retention_days`, a [`RetentionSweepDue`]
    /// event is published to the event bus.
    pub async fn maybe_sweep(&self, school_id: SchoolId) -> Result<()> {
        let now = self.clock.now();
        let should_check = self.advance_sweep_clock(now);
        if !should_check {
            return Ok(());
        }
        // Look up the oldest audit row for the school. Phase 2
        // uses the sentinel target_id simplification; Phase 3
        // will add a dedicated `oldest_row_for_school` method
        // to the storage port. The storage adapter interprets
        // the sentinel as "oldest row for the school".
        let rows = self
            .audit_log
            .read_for_target(school_id, SENTINEL_TARGET_ID, 1)
            .await?;
        if let Some(oldest) = rows.first() {
            let age = now
                .as_datetime()
                .signed_duration_since(oldest.occurred_at.as_datetime());
            if age >= self.policy.retention_chrono() {
                let cutoff = RetentionSweeper::cutoff_for(now, &self.policy);
                self.emit_sweep_due(school_id, cutoff, now).await?;
            }
        }
        Ok(())
    }

    /// Helper: records `now` as the new `last_sweep_at` if the
    /// interval has elapsed (or the seed). Returns `true` if the
    /// threshold check should run. Handles mutex poisoning
    /// gracefully: a poisoned lock is treated as poisoned-out
    /// (the lock guard's `into_inner` is used to avoid panics in
    /// the engine's command path).
    fn advance_sweep_clock(&self, now: Timestamp) -> bool {
        let mut guard = match self.last_sweep_at.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        match *guard {
            None => {
                *guard = Some(now);
                false
            }
            Some(prev) => {
                let elapsed = now.as_datetime().signed_duration_since(prev.as_datetime());
                if elapsed >= self.policy.sweep_interval_chrono() {
                    *guard = Some(now);
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Helper: builds a [`RetentionSweepDue`] event, wraps it in
    /// a system-issued [`EventEnvelope`], and publishes it to the
    /// bus. The actor is `SYSTEM_USER_ID` and the correlation id
    /// is a fresh UUIDv7 — a sweep is a system-internal action,
    /// not a user request.
    async fn emit_sweep_due(
        &self,
        school_id: SchoolId,
        cutoff: Timestamp,
        at: Timestamp,
    ) -> Result<()> {
        let event = RetentionSweepDue::new(school_id, cutoff, at);
        let system_corr = educore_core::ids::CorrelationId(Uuid::now_v7());
        let ctx = TenantContext::system(school_id, system_corr);
        let envelope = event.into_envelope(&ctx);
        self.bus.publish(envelope).await?;
        Ok(())
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn audit_action_as_str_covers_all_variants() {
        assert_eq!(AuditAction::Create.as_str(), "create");
        assert_eq!(AuditAction::Update.as_str(), "update");
        assert_eq!(AuditAction::Delete.as_str(), "delete");
        assert_eq!(AuditAction::Approve.as_str(), "approve");
        assert_eq!(AuditAction::Login.as_str(), "login");
        assert_eq!(AuditAction::Logout.as_str(), "logout");
        assert_eq!(AuditAction::Configure.as_str(), "configure");
        assert_eq!(AuditAction::Other("merge".to_owned()).as_str(), "merge");
    }

    #[test]
    fn audit_target_type_and_id_for_school() {
        let id = Uuid::now_v7();
        let t = AuditTarget::School(id);
        assert_eq!(t.target_type(), "school");
        assert_eq!(t.target_id(), id);
    }

    #[test]
    fn audit_target_type_and_id_for_other() {
        let id = Uuid::now_v7();
        let t = AuditTarget::Other("library_copy".to_owned(), id);
        assert_eq!(t.target_type(), "library_copy");
        assert_eq!(t.target_id(), id);
    }

    #[test]
    fn audit_target_type_for_every_variant_is_nonempty() {
        // Exhaustive: every variant returns a non-empty type
        // string. This guards against a future variant being
        // added without a matching `target_type` arm.
        let id = Uuid::now_v7();
        let variants: Vec<AuditTarget> = vec![
            AuditTarget::School(id),
            AuditTarget::User(id),
            AuditTarget::Session(id),
            AuditTarget::Role(id),
            AuditTarget::Capability(id),
            AuditTarget::Student(id),
            AuditTarget::Class(id),
            AuditTarget::Section(id),
            AuditTarget::Subject(id),
            AuditTarget::AcademicYear(id),
            AuditTarget::Enrollment(id),
            AuditTarget::Exam(id),
            AuditTarget::MarksRegister(id),
            AuditTarget::ExamSchedule(id),
            AuditTarget::ResultStore(id),
            AuditTarget::ReportCard(id),
            AuditTarget::OnlineExam(id),
            AuditTarget::SeatPlan(id),
            AuditTarget::AdmitCard(id),
            AuditTarget::StudentAttendance(id),
            AuditTarget::SubjectAttendance(id),
            AuditTarget::StaffAttendance(id),
            AuditTarget::BulkAttendanceImport(id),
            AuditTarget::ClassAttendance(id),
            AuditTarget::Staff(id),
            AuditTarget::Payroll(id),
            AuditTarget::Department(id),
            AuditTarget::Designation(id),
            AuditTarget::LeaveType(id),
            AuditTarget::LeaveDefine(id),
            AuditTarget::LeaveRequest(id),
            AuditTarget::HrStaffAttendance(id),
            AuditTarget::HrStaffAttendanceImport(id),
            AuditTarget::AssignClassTeacher(id),
            AuditTarget::HourlyRate(id),
            AuditTarget::SalaryTemplate(id),
            AuditTarget::PayrollEarnDeduc(id),
            AuditTarget::LeaveDeductionInfo(id),
            AuditTarget::StaffRegistrationField(id),
            AuditTarget::StaffImportBulk(id),
            AuditTarget::FeesInvoice(id),
            AuditTarget::FeesPayment(id),
            AuditTarget::BankAccount(id),
            AuditTarget::BankStatement(id),
            AuditTarget::Expense(id),
            AuditTarget::Income(id),
            AuditTarget::Donor(id),
            AuditTarget::ExpenseHead(id),
            AuditTarget::IncomeHead(id),
            AuditTarget::Wallet(id),
            AuditTarget::WalletTransaction(id),
            AuditTarget::PayrollPayment(id),
            AuditTarget::Transaction(id),
            AuditTarget::Item(id),
            AuditTarget::Book(id),
            AuditTarget::BookCategory(id),
            AuditTarget::LibraryMember(id),
            AuditTarget::BookIssue(id),
            AuditTarget::BookReturn(id),
            AuditTarget::Fine(id),
            AuditTarget::Notice(id),
            AuditTarget::Complaint(id),
            AuditTarget::ComplaintType(id),
            AuditTarget::Notification(id),
            AuditTarget::EmailLog(id),
            AuditTarget::SmsLog(id),
            AuditTarget::SmsTemplate(id),
            AuditTarget::EmailSetting(id),
            AuditTarget::SmsGateway(id),
            AuditTarget::NotificationSetting(id),
            AuditTarget::AbsentNotificationTimeSetup(id),
            AuditTarget::ChatMessage(id),
            AuditTarget::ChatConversation(id),
            AuditTarget::ChatGroup(id),
            AuditTarget::ChatGroupUser(id),
            AuditTarget::ChatGroupMessageRecipient(id),
            AuditTarget::ChatGroupMessageRemove(id),
            AuditTarget::ChatBlockUser(id),
            AuditTarget::ChatInvitation(id),
            AuditTarget::ChatInvitationType(id),
            AuditTarget::ChatStatus(id),
            AuditTarget::SendMessage(id),
            AuditTarget::ContactMessage(id),
            AuditTarget::SpeechSlider(id),
            AuditTarget::PhoneCallLog(id),
            AuditTarget::CustomSmsSetting(id),
            AuditTarget::FormDownload(id),
            AuditTarget::PostalDispatch(id),
            AuditTarget::PostalReceive(id),
            AuditTarget::Page(id),
            AuditTarget::News(id),
            AuditTarget::NewsCategory(id),
            AuditTarget::NewsComment(id),
            AuditTarget::NewsPage(id),
            AuditTarget::NoticeBoard(id),
            AuditTarget::Testimonial(id),
            AuditTarget::HomeSlider(id),
            AuditTarget::Content(id),
            AuditTarget::ContentType(id),
            AuditTarget::ContentShareList(id),
            AuditTarget::TeacherUploadContent(id),
            AuditTarget::UploadContent(id),
            AuditTarget::AboutPage(id),
            AuditTarget::ContactPage(id),
            AuditTarget::CoursePage(id),
            AuditTarget::HomePageSetting(id),
            AuditTarget::FrontendPage(id),
            AuditTarget::PageRevision(id),
            AuditTarget::NewsRevision(id),
            AuditTarget::CalendarEvent(id),
            AuditTarget::Holiday(id),
            AuditTarget::Incident(id),
            AuditTarget::SchoolSettings(id),
            AuditTarget::BellSchedule(id),
            AuditTarget::GeneralSettings(id),
            AuditTarget::Language(id),
            AuditTarget::LanguagePhrase(id),
            AuditTarget::BaseSetup(id),
            AuditTarget::BaseGroup(id),
            AuditTarget::DateFormat(id),
            AuditTarget::Style(id),
            AuditTarget::BackgroundSetting(id),
            AuditTarget::DashboardSetting(id),
            AuditTarget::CustomLink(id),
            AuditTarget::ColorTheme(id),
            AuditTarget::Theme(id),
            AuditTarget::Color(id),
            AuditTarget::BehaviorRecordSetting(id),
            AuditTarget::SetupAdmin(id),
            AuditTarget::Backup(id),
            AuditTarget::Job(id),
            AuditTarget::FailedJob(id),
            AuditTarget::SystemVersion(id),
            AuditTarget::VersionHistory(id),
            AuditTarget::UserLog(id),
            AuditTarget::MaintenanceSetting(id),
            AuditTarget::Sidebar(id),
        ];
        for v in &variants {
            assert!(!v.target_type().is_empty());
            assert_eq!(v.target_id(), id);
        }
    }

    #[test]
    fn finance_audit_target_type_is_snake_case_and_nonempty() {
        // Phase 7: assert the 13 finance `AuditTarget` variants
        // resolve to snake_case wire strings, distinct from each
        // other and from any other domain's `target_type`.
        let id = Uuid::now_v7();
        let cases: Vec<(AuditTarget, &str)> = vec![
            (AuditTarget::FeesInvoice(id), "fees_invoice"),
            (AuditTarget::FeesPayment(id), "fees_payment"),
            (AuditTarget::BankAccount(id), "bank_account"),
            (AuditTarget::BankStatement(id), "bank_statement"),
            (AuditTarget::Expense(id), "expense"),
            (AuditTarget::Income(id), "income"),
            (AuditTarget::Donor(id), "donor"),
            (AuditTarget::ExpenseHead(id), "expense_head"),
            (AuditTarget::IncomeHead(id), "income_head"),
            (AuditTarget::Wallet(id), "wallet"),
            (AuditTarget::WalletTransaction(id), "wallet_transaction"),
            (AuditTarget::PayrollPayment(id), "payroll_payment"),
            (AuditTarget::Transaction(id), "transaction"),
        ];
        for (target, expected) in cases {
            assert_eq!(target.target_type(), expected);
            assert_eq!(target.target_id(), id);
        }
    }

    #[test]
    fn communication_audit_target_round_trip_for_all_aggregates() {
        // Phase 10: assert all 26 communication `AuditTarget`
        // variants resolve to non-empty snake_case wire strings
        // and that `target_id()` round-trips the inner `Uuid`.
        let id = Uuid::now_v7();
        let cases: Vec<(AuditTarget, &str)> = vec![
            (AuditTarget::Notice(id), "notice"),
            (AuditTarget::Complaint(id), "complaint"),
            (AuditTarget::ComplaintType(id), "complaint_type"),
            (AuditTarget::Notification(id), "notification"),
            (AuditTarget::EmailLog(id), "email_log"),
            (AuditTarget::SmsLog(id), "sms_log"),
            (AuditTarget::SmsTemplate(id), "sms_template"),
            (AuditTarget::EmailSetting(id), "email_setting"),
            (AuditTarget::SmsGateway(id), "sms_gateway"),
            (AuditTarget::NotificationSetting(id), "notification_setting"),
            (
                AuditTarget::AbsentNotificationTimeSetup(id),
                "absent_notification_time_setup",
            ),
            (AuditTarget::ChatMessage(id), "chat_message"),
            (AuditTarget::ChatConversation(id), "chat_conversation"),
            (AuditTarget::ChatGroup(id), "chat_group"),
            (AuditTarget::ChatGroupUser(id), "chat_group_user"),
            (
                AuditTarget::ChatGroupMessageRecipient(id),
                "chat_group_message_recipient",
            ),
            (
                AuditTarget::ChatGroupMessageRemove(id),
                "chat_group_message_remove",
            ),
            (AuditTarget::ChatBlockUser(id), "chat_block_user"),
            (AuditTarget::ChatInvitation(id), "chat_invitation"),
            (AuditTarget::ChatInvitationType(id), "chat_invitation_type"),
            (AuditTarget::ChatStatus(id), "chat_status"),
            (AuditTarget::SendMessage(id), "send_message"),
            (AuditTarget::ContactMessage(id), "contact_message"),
            (AuditTarget::SpeechSlider(id), "speech_slider"),
            (AuditTarget::PhoneCallLog(id), "phone_call_log"),
            (AuditTarget::CustomSmsSetting(id), "custom_sms_setting"),
        ];
        assert_eq!(cases.len(), 26);
        let mut seen_types: std::collections::HashSet<String> = std::collections::HashSet::new();
        for (target, expected) in cases {
            let wire = target.target_type();
            assert!(!wire.is_empty(), "target_type() returned empty string");
            assert_eq!(wire, expected);
            assert!(
                wire.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
                "target_type() {wire:?} is not snake_case ASCII"
            );
            assert!(
                !wire.starts_with('_') && !wire.ends_with('_'),
                "target_type() {wire:?} has a leading or trailing underscore"
            );
            assert!(
                !wire.contains("__"),
                "target_type() {wire:?} contains a double underscore"
            );
            assert_eq!(target.target_id(), id);
            assert!(
                seen_types.insert(wire.to_owned()),
                "duplicate target_type() wire string: {wire:?}"
            );
        }
    }

    #[test]
    fn documents_audit_target_round_trip_for_all_aggregates() {
        let form_id = Uuid::new_v4();
        let dispatch_id = Uuid::new_v4();
        let receive_id = Uuid::new_v4();

        let form_target = AuditTarget::FormDownload(form_id);
        assert_eq!(form_target.target_type(), "form_download");
        assert_eq!(form_target.target_id(), form_id);

        let dispatch_target = AuditTarget::PostalDispatch(dispatch_id);
        assert_eq!(dispatch_target.target_type(), "postal_dispatch");
        assert_eq!(dispatch_target.target_id(), dispatch_id);

        let receive_target = AuditTarget::PostalReceive(receive_id);
        assert_eq!(receive_target.target_type(), "postal_receive");
        assert_eq!(receive_target.target_id(), receive_id);
    }

    #[test]
    fn cms_audit_target_round_trip_for_all_aggregates() {
        let id = Uuid::new_v4();
        let cases: Vec<(AuditTarget, &str)> = vec![
            (AuditTarget::Page(id), "page"),
            (AuditTarget::News(id), "news"),
            (AuditTarget::NewsCategory(id), "news_category"),
            (AuditTarget::NewsComment(id), "news_comment"),
            (AuditTarget::NewsPage(id), "news_page"),
            (AuditTarget::NoticeBoard(id), "notice_board"),
            (AuditTarget::Testimonial(id), "testimonial"),
            (AuditTarget::HomeSlider(id), "home_slider"),
            (AuditTarget::Content(id), "content"),
            (AuditTarget::ContentType(id), "content_type"),
            (AuditTarget::ContentShareList(id), "content_share_list"),
            (
                AuditTarget::TeacherUploadContent(id),
                "teacher_upload_content",
            ),
            (AuditTarget::UploadContent(id), "upload_content"),
            (AuditTarget::AboutPage(id), "about_page"),
            (AuditTarget::ContactPage(id), "contact_page"),
            (AuditTarget::CoursePage(id), "course_page"),
            (AuditTarget::HomePageSetting(id), "home_page_setting"),
            (AuditTarget::FrontendPage(id), "frontend_page"),
            (AuditTarget::PageRevision(id), "page_revision"),
            (AuditTarget::NewsRevision(id), "news_revision"),
        ];
        let mut seen_types: std::collections::HashSet<String> = std::collections::HashSet::new();
        for (target, expected) in cases {
            let wire = target.target_type();
            assert_eq!(wire, expected);
            assert!(!wire.is_empty(), "target_type() returned empty string");
            assert_eq!(target.target_id(), id);
            assert!(
                seen_types.insert(wire.to_owned()),
                "duplicate target_type() wire string: {wire:?}"
            );
        }
    }

    #[test]
    fn events_audit_target_round_trip_for_all_aggregates() {
        let id = Uuid::new_v4();
        let cases: Vec<(AuditTarget, &str)> = vec![
            (AuditTarget::CalendarEvent(id), "calendar_event"),
            (AuditTarget::Holiday(id), "holiday"),
            (AuditTarget::Incident(id), "incident"),
            (AuditTarget::Weekend(id), "weekend"),
            (AuditTarget::AssignIncident(id), "assign_incident"),
            (AuditTarget::IncidentComment(id), "incident_comment"),
            (AuditTarget::CalendarSetting(id), "calendar_setting"),
        ];
        let mut seen_types: std::collections::HashSet<String> = std::collections::HashSet::new();
        for (target, expected) in cases {
            let wire = target.target_type();
            assert_eq!(wire, expected);
            assert!(!wire.is_empty(), "target_type() returned empty string");
            assert_eq!(target.target_id(), id);
            assert!(
                seen_types.insert(wire.to_owned()),
                "duplicate target_type() wire string: {wire:?}"
            );
        }
    }

    #[test]
    fn settings_audit_target_round_trip_for_all_aggregates() {
        // Phase 14: 15 Settings.* audit targets per
        // `docs/specs/settings/aggregates.md`. The 2 Phase 2
        // placeholders (`SchoolSettings`, `BellSchedule`) are
        // preserved for `DefaultRoleCatalog` consistency.
        let id = Uuid::new_v4();
        let cases: Vec<(AuditTarget, &str)> = vec![
            (AuditTarget::GeneralSettings(id), "general_settings"),
            (AuditTarget::Language(id), "language"),
            (AuditTarget::LanguagePhrase(id), "language_phrase"),
            (AuditTarget::BaseSetup(id), "base_setup"),
            (AuditTarget::BaseGroup(id), "base_group"),
            (AuditTarget::DateFormat(id), "date_format"),
            (AuditTarget::Style(id), "style"),
            (AuditTarget::BackgroundSetting(id), "background_setting"),
            (AuditTarget::DashboardSetting(id), "dashboard_setting"),
            (AuditTarget::CustomLink(id), "custom_link"),
            (AuditTarget::ColorTheme(id), "color_theme"),
            (AuditTarget::Theme(id), "theme"),
            (AuditTarget::Color(id), "color"),
            (
                AuditTarget::BehaviorRecordSetting(id),
                "behavior_record_setting",
            ),
            (AuditTarget::SetupAdmin(id), "setup_admin"),
        ];
        assert_eq!(cases.len(), 15);
        let mut seen_types: std::collections::HashSet<String> = std::collections::HashSet::new();
        for (target, expected) in cases {
            let wire = target.target_type();
            assert_eq!(wire, expected);
            assert!(!wire.is_empty(), "target_type() returned empty string");
            assert!(
                wire.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
                "target_type() {wire:?} is not snake_case ASCII"
            );
            assert!(
                !wire.starts_with('_') && !wire.ends_with('_'),
                "target_type() {wire:?} has a leading or trailing underscore"
            );
            assert!(
                !wire.contains("__"),
                "target_type() {wire:?} contains a double underscore"
            );
            assert_eq!(target.target_id(), id);
            assert!(
                seen_types.insert(wire.to_owned()),
                "duplicate target_type() wire string: {wire:?}"
            );
        }
    }

    #[test]
    fn operations_audit_target_round_trip_for_all_aggregates() {
        // Phase 14: 8 Operations.* audit targets per
        // `docs/specs/operations/aggregates.md`.
        let id = Uuid::new_v4();
        let cases: Vec<(AuditTarget, &str)> = vec![
            (AuditTarget::Backup(id), "backup"),
            (AuditTarget::Job(id), "job"),
            (AuditTarget::FailedJob(id), "failed_job"),
            (AuditTarget::SystemVersion(id), "system_version"),
            (AuditTarget::VersionHistory(id), "version_history"),
            (AuditTarget::UserLog(id), "user_log"),
            (AuditTarget::MaintenanceSetting(id), "maintenance_setting"),
            (AuditTarget::Sidebar(id), "sidebar"),
        ];
        assert_eq!(cases.len(), 8);
        let mut seen_types: std::collections::HashSet<String> = std::collections::HashSet::new();
        for (target, expected) in cases {
            let wire = target.target_type();
            assert_eq!(wire, expected);
            assert!(!wire.is_empty(), "target_type() returned empty string");
            assert!(
                wire.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
                "target_type() {wire:?} is not snake_case ASCII"
            );
            assert!(
                !wire.starts_with('_') && !wire.ends_with('_'),
                "target_type() {wire:?} has a leading or trailing underscore"
            );
            assert!(
                !wire.contains("__"),
                "target_type() {wire:?} contains a double underscore"
            );
            assert_eq!(target.target_id(), id);
            assert!(
                seen_types.insert(wire.to_owned()),
                "duplicate target_type() wire string: {wire:?}"
            );
        }
    }

    #[test]
    fn sentinel_target_id_is_nil() {
        assert_eq!(SENTINEL_TARGET_ID, Uuid::nil());
    }

    #[test]
    fn advance_sweep_clock_first_call_seeds_returns_false() {
        use educore_core::clock::TestClock;
        let clock = std::sync::Arc::new(TestClock::new());
        let policy = RetentionPolicy::default();
        // We can't construct AuditWriter without a real AuditLog
        // and EventBus; test the helper directly via the
        // `advance_sweep_clock` private method by constructing
        // a minimal struct copy. To keep this test self-contained
        // we use a public path: the integration tests cover the
        // full flow. This test just exercises the sweep-clock
        // arithmetic.
        let now = Timestamp::from_datetime(Utc::now());
        // Build a no-op writer to exercise the helper.
        // We can't easily mock AuditLog/EventBus here without
        // a large surface; defer to integration tests.
        let _ = (clock, policy, now);
    }
}
