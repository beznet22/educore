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

    // -- Finance (Phase 7 placeholders) ---------------------------------
    /// Create a finance invoice. Placeholder for the finance domain.
    FinanceInvoiceCreate,
    /// Read a finance invoice.
    FinanceInvoiceRead,
    /// Update a finance invoice.
    FinanceInvoiceUpdate,
    /// Delete / void a finance invoice.
    FinanceInvoiceDelete,

    // -- HR (Phase 6 placeholders) --------------------------------------
    /// Create a staff member. Placeholder for the HR domain.
    HrStaffCreate,
    /// Read a staff member.
    HrStaffRead,
    /// Update a staff member.
    HrStaffUpdate,
    /// Delete / deactivate a staff member.
    HrStaffDelete,

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
            Self::FinanceInvoiceCreate
            | Self::FinanceInvoiceRead
            | Self::FinanceInvoiceUpdate
            | Self::FinanceInvoiceDelete => CapabilityDomain::Finance,
            Self::HrStaffCreate | Self::HrStaffRead | Self::HrStaffUpdate | Self::HrStaffDelete => {
                CapabilityDomain::Hr
            }
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
            Self::FinanceInvoiceCreate
            | Self::FinanceInvoiceRead
            | Self::FinanceInvoiceUpdate
            | Self::FinanceInvoiceDelete => "Invoice",
            Self::HrStaffCreate | Self::HrStaffRead | Self::HrStaffUpdate | Self::HrStaffDelete => {
                "Staff"
            }
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
            | Self::FinanceInvoiceCreate
            | Self::HrStaffCreate
            | Self::LibraryBookCreate
            | Self::CommunicationMessageCreate
            | Self::DocumentsFolderCreate
            | Self::CmsPageCreate
            | Self::FacilitiesRoomCreate
            | Self::EventsCalendarCreate => "Create",
            Self::PlatformSchoolRead
            | Self::PlatformUserRead
            | Self::RbacRoleRead
            | Self::RbacCapabilityRead
            | Self::AcademicStudentRead
            | Self::AcademicClassRead
            | Self::FinanceInvoiceRead
            | Self::HrStaffRead
            | Self::LibraryBookRead
            | Self::CommunicationMessageRead
            | Self::DocumentsFolderRead
            | Self::CmsPageRead
            | Self::FacilitiesRoomRead
            | Self::EventsCalendarRead => "Read",
            Self::PlatformSchoolUpdate
            | Self::PlatformUserUpdate
            | Self::RbacRoleUpdate
            | Self::RbacCapabilityUpdateMetadata
            | Self::AcademicStudentUpdate
            | Self::AcademicClassUpdate
            | Self::FinanceInvoiceUpdate
            | Self::HrStaffUpdate
            | Self::LibraryBookUpdate
            | Self::CommunicationMessageUpdate
            | Self::DocumentsFolderUpdate
            | Self::CmsPageUpdate
            | Self::FacilitiesRoomUpdate
            | Self::EventsCalendarUpdate => "Update",
            Self::PlatformSchoolDelete
            | Self::PlatformUserDelete
            | Self::RbacRoleDelete
            | Self::AcademicStudentDelete
            | Self::AcademicClassDelete
            | Self::FinanceInvoiceDelete
            | Self::HrStaffDelete
            | Self::LibraryBookDelete
            | Self::CommunicationMessageDelete
            | Self::DocumentsFolderDelete
            | Self::CmsPageDelete
            | Self::FacilitiesRoomDelete
            | Self::EventsCalendarDelete => "Delete",
            Self::RbacRoleManage => "Manage",
            Self::RbacRoleClone => "Clone",
            Self::RbacCapabilityAssign => "Assign",
            Self::RbacCapabilityRevoke => "Revoke",
            Self::RbacBootstrap => "Bootstrap",
            Self::SettingsManage => "Manage",
            Self::OperationsManage => "Manage",
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
            Self::FinanceInvoiceCreate => "Finance.Invoice.Create",
            Self::FinanceInvoiceRead => "Finance.Invoice.Read",
            Self::FinanceInvoiceUpdate => "Finance.Invoice.Update",
            Self::FinanceInvoiceDelete => "Finance.Invoice.Delete",
            Self::HrStaffCreate => "Hr.Staff.Create",
            Self::HrStaffRead => "Hr.Staff.Read",
            Self::HrStaffUpdate => "Hr.Staff.Update",
            Self::HrStaffDelete => "Hr.Staff.Delete",
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
            Self::FinanceInvoiceCreate,
            Self::FinanceInvoiceRead,
            Self::FinanceInvoiceUpdate,
            Self::FinanceInvoiceDelete,
            Self::HrStaffCreate,
            Self::HrStaffRead,
            Self::HrStaffUpdate,
            Self::HrStaffDelete,
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
            "Finance.Invoice.Create" => Some(Self::FinanceInvoiceCreate),
            "Finance.Invoice.Read" => Some(Self::FinanceInvoiceRead),
            "Finance.Invoice.Update" => Some(Self::FinanceInvoiceUpdate),
            "Finance.Invoice.Delete" => Some(Self::FinanceInvoiceDelete),
            "Hr.Staff.Create" => Some(Self::HrStaffCreate),
            "Hr.Staff.Read" => Some(Self::HrStaffRead),
            "Hr.Staff.Update" => Some(Self::HrStaffUpdate),
            "Hr.Staff.Delete" => Some(Self::HrStaffDelete),
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
            // `<Domain>.<Aggregate>.<Action>`.
            assert!(
                parts.len() >= 2,
                "expected at least 2 segments for {c:?}, got {s:?}"
            );
            let last = parts[parts.len() - 1];
            let action = c.action();
            assert!(
                last == action
                    || last.starts_with(action)
                    || s == "Rbac.Bootstrap"
                    || s == "Settings.Manage"
                    || s == "Operations.Manage",
                "action mismatch for {c:?}: wire={last:?} action={action:?}"
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
