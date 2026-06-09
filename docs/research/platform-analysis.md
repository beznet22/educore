# Platform Domain — Business Analysis

## Purpose

The platform domain is the engine's foundational
layer. It owns the multi-tenant substrate: schools,
users, custom fields, lookup data, modules, and the
global registry. Every other domain depends on
platform for `SchoolId`, `UserId`, `TenantContext`.

This document describes how multi-school tenancy and
user management work in real schools, with the
edge cases that real schools hit.

## Key Concepts

- **School** — the tenant root. Identity, profile,
  contact, package, lifecycle.
- **User** — a person who interacts with the system
  (staff, student, parent, external).
- **OtpCode** — a one-time password for user-facing
  flows (parent login, password reset).
- **Course** — an online course offered by the
  school (e.g. "Spoken English").
- **CourseCategory** — a grouping for courses.
- **CustomField** — a custom field definition
  attached to an entity.
- **CustomFieldValue** — a per-entity field value.
- **ChartOfAccount** — an income/expense head
  (consumed by the finance domain).
- **BaseGroup** / **BaseSetup** — configurable
  lookup values.
- **Module** — a top-level functional area
  ("Academics", "Finance", "HR").
- **ModuleLink** — a menu item within a module.
- **AddOn** — a registered add-on.
- **Country** / **Continent** / **Currency** /
  **TimeZone** / **Language** — global lookup data.
- **Visitor** — a visitor log entry (front office).
- **ToDo** — a to-do item.
- **Instruction** — a front-office instruction.
- **ExpertTeacher** — a featured staff member.
- **PhotoGallery** / **VideoGallery** — public
  media.
- **SocialMediaIcon** — a social link.
- **HeaderMenuManager** — a public-header menu
  entry.
- **FrontendPermission** — a public-facing
  permission flag.
- **AmountTransfer** — a fund transfer between
  accounts.

## Real-World Scenarios

### School Onboarding

A new school joins the platform:

1. The platform admin (or the school's
   representative) provides the school details:
   - School name.
   - School code.
   - Address.
   - Contact (phone, email).
   - Package / subscription tier.
   - Admin contact.
2. The platform admin issues the onboarding
   command (`Platform.School.Onboard`).
3. The engine creates the `School` aggregate
   with a unique `SchoolId`.
4. The engine seeds the default roles
   (SuperAdmin for the platform, SchoolAdmin
   for the school).
5. The engine seeds the default settings
   (theme, language, currency, time zone).
6. The engine seeds the default academic
   structures (current academic year, default
   classes if opted in).
7. The school admin user is bound to the
   SchoolAdmin role.
8. The engine emits `Platform.School.Onboarded`.

A real school onboarding is a multi-step
process that may span days. The engine's
onboarding is resumable; a failed step does
not roll back the school.

### User Registration

A user is added to a school:

1. The user provides the details (name, contact,
   role).
2. The engine creates the `User` aggregate with
   a unique `UserId`.
3. The user is bound to a role (e.g. Teacher,
   Student, Parent).
4. The user receives a welcome email with
   activation instructions.
5. The user sets their password (or signs in
   via OAuth / SAML).

In real schools, user registration is
**role-driven**. A Teacher is registered by
the HR admin; a Student is registered via
admission; a Parent is registered by the
school admin or self-registers via the parent
portal.

### Multi-Role User

A user may have multiple roles in the same
school. A teacher who is also a parent has
both Teacher and Parent roles. The engine's
`User` aggregate supports many-to-many role
bindings.

The `TenantContext` resolves the active role
based on the action: a `Mark.Create` command
activates the Teacher role; a
`FeesAssign.Read` command for the child
activates the Parent role.

### User Suspension

A user is suspended (e.g. disciplinary):

1. The school admin suspends the user.
2. The user's status becomes `Suspended`.
3. The user's portal access is revoked.
4. The user cannot log in or invoke commands.
5. The engine emits `UserSuspended`.

The user can be reinstated (status
`Reinstated`).

### OTP-Based Login

A parent logs in via OTP:

1. The parent provides their phone number.
2. The engine issues an `OtpCode` with a 5-
   minute expiry.
3. The OTP is sent via SMS.
4. The parent enters the OTP.
5. The engine verifies the OTP and issues a
   session.

The engine's `OtpCode` aggregate is one-time
use; the OTP is invalidated after use.

### Password Reset

A user forgets their password:

1. The user requests a password reset.
2. The engine issues a reset token (in the
   consumer's port, e.g. via email).
3. The user clicks the link and sets a new
   password.
4. The engine's `User.password_hash` is
   updated.
5. The old sessions are invalidated.

The engine's `PasswordResetRequest` aggregate
captures the request (in the operations
domain).

### Two-Factor Authentication

A school enables 2FA for the admin role:

1. The user enables 2FA (via authenticator
   app or SMS).
2. The engine stores the 2FA secret.
3. On login, the user provides the second
   factor.
4. The engine verifies the second factor.

The engine's `TwoFactorSetting` aggregate
captures the policy (per the RBAC domain).

### Custom Fields

A school adds a custom field to the student
profile: "Allergies":

1. The school admin defines the custom field
   with the form name, the label, the type
   (text, number, date, select), and the
   options (for select).
2. The engine creates the `CustomField`
   aggregate.
3. The student admission form shows the new
   field.
4. The engine captures the value in the
   `CustomFieldValue` aggregate.

In real schools, custom fields are used to
capture school-specific data without code
changes.

### Chart of Accounts

A school configures its chart of accounts:

- **Income heads**: Tuition, Exam, Transport,
  Hostel, Donation, Sale, Investment.
- **Expense heads**: Salary, Electricity,
  Water, Internet, Stationery, Maintenance,
  Vendor, Bank Charges.

The platform domain's `ChartOfAccount`
aggregate captures the heads. The finance
domain consumes them.

### Modules and Add-Ons

A school enables the "Online Course" module:

1. The platform admin registers the module
   in the engine's catalog.
2. The school admin enables the module for
   the school.
3. The engine emits `ModuleEnabled`.
4. The platform domain's module list shows
   the enabled module.
5. The sidebar projection includes the
   module's menu items.

In real schools, modules are
**per-tenant**. A school that does not need
"Online Course" does not enable it; the
sidebar and the API surface do not include
it.

### Front-Office Operations

A school's front office has:

- **Visitor log** — every visitor to the
  school is recorded (name, purpose, time
  in / out).
- **To-dos** — staff's personal to-do lists.
- **Instructions** — front-office
  instructions displayed on the reception
  screen.
- **Expert teachers** — featured staff on
  the public website.

The platform domain's `Visitor`, `ToDo`,
`Instruction`, and `ExpertTeacher`
aggregates capture these.

### Public Website

A school's public website has:

- **Photo gallery** — public photos.
- **Video gallery** — public videos.
- **Header menu** — the top navigation.
- **Social media icons** — Facebook, Twitter,
  Instagram, YouTube links.

The platform domain's `PhotoGallery`,
`VideoGallery`, `HeaderMenuManager`, and
`SocialMediaIcon` aggregates capture these.

### Public Permissions

A school's public website has
**public-facing permissions** that control
which pages are visible to the public. The
platform domain's `FrontendPermission`
aggregate captures these.

### Lookup Data

A school uses global lookup data:

- **Countries** — for student nationality.
- **Continents** — for grouping countries.
- **Currencies** — for fees and payments.
- **Time zones** — for event scheduling.
- **Languages** — for multi-lingual support.

The platform domain's `Country`,
`Continent`, `Currency`, `TimeZone`, and
`Language` aggregates are global (not
tenant-scoped). They are seeded by the
bootstrap school at engine initialization.

## Business Rules

1. Every aggregate is anchored to a `SchoolId`.
   A `School` itself is the root of its own
   tenant; an aggregate without a `SchoolId`
   is global (`Country`, `Currency`).
2. A `User` belongs to exactly one `School`.
3. A `User::email` is unique within
   `(school_id, lower(email))`.
4. A `User::phone_number` is unique within
   `(school_id, normalized_phone)`.
5. A `School::domain` is unique across the
   platform.
6. A `School::school_code` is unique across
   the platform.
7. A `Module` is registered once and
   referenced by id; a `ModuleLink` is
   registered once per parent module.
8. A `CustomField` is unique by
   `(school_id, form_name, label)`.
9. A `CustomFieldValue` is unique by
   `(custom_field_id, entity_type, entity_id)`.
10. A `Course` is unique by `(school_id, title)`.
11. A `CourseCategory` is unique by
    `(school_id, category_name)`.
12. A `ChartOfAccount` head is unique by
    `(school_id, lower(head))` and has type
    `Expense` or `Income`.
13. A `Country` is unique by `code`; a
    `Currency` is unique by `code`; a
    `TimeZone` is unique by `code`; a
    `Language` is unique by `code`.
14. A `User::usertype` is one of the closed
    enum values and determines the default
    role binding at registration.
15. The bootstrap `School` (id
    `PLATFORM_BOOTSTRAP`) is the engine's
    seed school and cannot be deleted; it is
    the parent of seed data.

## Edge Cases

### School with No Academic Year

A new school is onboarded but has not yet
configured the academic year. The engine
allows the school to exist; the academic
domain's commands require an academic year
and will fail until one is configured.

### User with Multiple Roles in Different Schools

A teacher works in two schools (a network
of schools). The teacher's user account
exists in both schools. The
`TenantContext` resolves the active role
based on the active school.

### User Suspended Across All Schools

A user is suspended at the platform level.
The user cannot log in to any school. The
engine's `Platform.User.Suspend` command
with `cross_tenant = true` enforces this.

### School Domain Change

A school changes its domain name (e.g.
`oldschool.com` to `newschool.com`). The
admin updates the school's domain. The
engine's `School.domain` is unique; the
change is auditable.

### Custom Field Deleted with Values

A school admin deletes a custom field. The
field's values are cascade-deleted (or
retained with `is_orphan = true`, per the
school's policy). The engine supports both
modes.

### Module Disabled

A school disables a module. The module's
data is soft-archived; the module's
commands are rejected; the sidebar no
longer shows the module. The data is
retained for re-enable or export.

### Visitor Log Retention

A school has a visitor log retention policy
of 12 months. The engine's purge job
deletes entries older than 12 months. The
audit log retains a summary count.

### Amount Transfer with Different Currencies

A school transfers funds between two
accounts in different currencies. The
engine records the transfer with the
source amount, the destination amount, and
the exchange rate at the transfer date.

### Country with Multiple Languages

A country has multiple official languages
(e.g. Switzerland has German, French,
Italian, Romansh). The `Country.languages`
field is a list.

### Time Zone with DST

A time zone observes daylight saving time.
The engine's `TimeZone` carries the DST
rules. The engine uses the time zone for
event scheduling and timestamp display.

## Notes for SMSengine Implementation

- The **platform** crate depends only on
  `smsengine-core`. It is the foundational
  crate; no other domain depends on
  platform.
- The platform domain provides `SchoolId`,
  `UserId`, `TenantContext` — the typed
  identifiers that every other domain uses.
- The platform domain's **custom fields**
  are a generic mechanism. Every domain
  can attach custom fields to its
  aggregates.
- The platform domain's **modules** drive
  the sidebar and the API surface. A
  consumer enables / disables modules per
  school.
- The platform domain's **lookup data** is
  global (not tenant-scoped). The bootstrap
  school seeds the data.
- The platform domain's **front-office**
  features are isolated from the
  operational data. The visitor log does
  not affect academic, finance, or HR.
- The platform domain's **public
  website** features (galleries, menus,
  social icons) are isolated from the
  operational data. They are managed by
  the school admin but are read by the
  public (no authentication).
- The platform domain's **amount
  transfer** is a per-school record. The
  finance domain subscribes to
  `AmountTransferCreated` and records the
  bank statements.
- The platform domain's **audit log** is
  the same engine-wide audit log. Every
  platform event is recorded.
