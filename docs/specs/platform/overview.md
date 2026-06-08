# Platform Domain Overview

## Purpose

The platform domain is the foundational layer of the engine. It owns
the multi-tenant substrate: schools, users, courses, custom fields,
charts of accounts, plugins, addons, modules, and the global lookup
tables (countries, currencies, languages, time zones, continents).
Every other domain depends on platform for `SchoolId`, `UserId`,
`TenantContext`, and the capability-check plumbing.

## Responsibilities

- Multi-tenant anchoring: every aggregate is rooted in a `SchoolId`.
- School identity, profile, package, and lifecycle.
- User identity, profile, contact, status, and authentication hooks.
- OTP issuance and verification for the user-facing flows.
- Course catalog (online courses) and course category grouping.
- Custom field definitions and their values attached to entities.
- Chart of accounts (the head list for the finance domain).
- Module and addon registry; per-school module enablement.
- Front-office content (photo gallery, video gallery, header menu,
  instructions, expert teachers, social media icons).
- Operational side entities (visitors, to-dos, comments, tags).
- Global lookup data (countries, continents, currencies, time zones,
  languages).
- Frontend permission rules and amount transfer records.

## Boundaries

The platform domain does **not** own:

- Academic structures (classes, sections, subjects) — see
  `specs/academic/`.
- Permissions, roles, and two-factor policies — see `specs/rbac/`.
- School-wide settings, themes, languages — see `specs/settings/`.
- Backups, jobs, system versions, audit logs — see
  `specs/operations/`.
- Authentication mechanisms (password hashing, JWT, OAuth) — these
  are port concerns.

The platform domain **does** provide:

- `SchoolId`, `UserId`, `TenantContext`.
- A `Module` / `ModuleLink` registry consumed by the RBAC domain
  to render menus and dashboards.
- A `CustomField` engine consumed by every domain that needs
  per-entity extension points.
- A `ChartOfAccount` registry consumed by the finance domain.
- A `Country`, `Currency`, `Language`, `TimeZone` registry consumed
  by every domain that needs locale data.

## Dependencies

- `smscore-core` — error types, identifier trait.
- No other engine domains.

## Domain Invariants

1. Every aggregate is anchored to a `SchoolId`. A `School` itself
   is the root of its own tenant; an aggregate without a `SchoolId`
   is global (e.g. `Country`, `Currency`).
2. A `User` belongs to exactly one `School`.
3. A `User::email` is unique within `(school_id, lower(email))`.
4. A `User::phone_number` is unique within `(school_id,
   normalized_phone)`.
5. A `School::domain` is unique across the platform.
6. A `School::school_code` is unique across the platform.
7. A `Module` is registered once and referenced by id; a
   `ModuleLink` is registered once per parent module.
8. A `CustomField` is unique by `(school_id, form_name, label)`.
9. A `CustomFieldValue` is unique by
   `(custom_field_id, entity_type, entity_id)`.
10. A `Course` is unique by `(school_id, title)`.
11. A `CourseCategory` is unique by `(school_id, category_name)`.
12. A `ChartOfAccount` head is unique by `(school_id, lower(head))`
    and has type `Expense` or `Income`.
13. A `Country` is unique by `code`; a `Currency` is unique by
    `code`; a `TimeZone` is unique by `code`; a `Language` is
    unique by `code`.
14. A `User::usertype` is one of the closed enum values and
    determines the default role binding at registration.
15. The bootstrap `School` (id 1) is the engine's seed school
    and cannot be deleted; it is the parent of seed data.

## Aggregate Roots

| Aggregate                 | Root Type            | Purpose                                         |
| ------------------------- | -------------------- | ----------------------------------------------- |
| School                    | `School`             | Tenant root; identity and lifecycle             |
| User                      | `User`               | Actor identity, profile, contact, status        |
| OtpCode                   | `OtpCode`            | A one-time password issued to a user            |
| Course                    | `Course`             | An online course                                |
| CourseCategory            | `CourseCategory`     | A grouping for courses                          |
| CoursePage                | `CoursePage`         | A landing-page entry for a course               |
| CustomField               | `CustomField`        | A field definition on a form                    |
| CustomFieldValue          | `CustomFieldValue`   | A per-entity field value                        |
| ChartOfAccount            | `ChartOfAccount`     | An income/expense head                          |
| BaseGroup                 | `BaseGroup`          | A grouping for base setups                      |
| BaseSetup                 | `BaseSetup`          | A configurable lookup value                     |
| Module                    | `Module`             | A top-level functional area                     |
| ModuleLink                | `ModuleLink`         | A menu item within a module                     |
| AddOn                     | `AddOn`              | A registered add-on                             |
| ModuleManager             | `ModuleManager`      | A registered module manager                     |
| ModuleStudentParentInfo   | `ModuleStudentParentInfo` | A per-school student/parent menu visibility |
| TimeZone                  | `TimeZone`           | A timezone entry                                |
| Country                   | `Country`            | A country entry                                 |
| Continent                 | `Continent`          | A continent entry                               |
| Currency                  | `Currency`           | A currency with format settings                 |
| Language                  | `Language`           | A language entry                                |
| SocialMediaIcon           | `SocialMediaIcon`    | A social link with icon                         |
| HeaderMenuManager         | `HeaderMenuManager`  | A public-header menu entry                      |
| PhotoGallery              | `PhotoGallery`       | A photo gallery                                 |
| VideoGallery              | `VideoGallery`       | A video gallery                                 |
| Visitor                   | `Visitor`            | A visitor log entry                             |
| ToDo                      | `ToDo`               | A to-do item                                    |
| Instruction               | `Instruction`        | A front-office instruction                      |
| ExpertTeacher             | `ExpertTeacher`      | A featured staff member                         |
| FrontendPermission        | `FrontendPermission` | A public-facing permission flag                 |
| AmountTransfer            | `AmountTransfer`     | A fund transfer between accounts                |
| Plugin                    | `Plugin`             | A registered front-office plugin                |
| Comment                   | `Comment`            | A comment with tagged attachments               |
| CommentTag                | `CommentTag`         | A tag value for comments                        |
| CommentPivot              | `CommentPivot`       | The join between comments and tags              |
| PersonalAccessToken       | `PersonalAccessToken`| A personal access token                         |
| VideoUpload               | `VideoUpload`        | A class-section video upload                    |

Each aggregate is documented in detail under
`docs/specs/platform/aggregates.md`.

## Cross-Domain Impact

When a `School` is created, the platform domain emits
`SchoolCreated`. The following domains may subscribe:

- `rbac` — seeds the bootstrap role catalog and assigns the
  school admin user to `SuperAdmin`.
- `settings` — seeds the default `GeneralSettings` row.
- `academic` — seeds the first `AcademicYear`.

When a `User` is registered, the platform domain emits
`UserRegistered`. The RBAC domain subscribes to bind the user to
their default role.

When a `User` is deactivated, the platform domain emits
`UserDeactivated`. The RBAC domain revokes all session tokens
and forces a re-authentication on next access.

## Subscribers

- `rbac` subscribes to `SchoolCreated` and `UserRegistered`.
- `settings` subscribes to `SchoolCreated` and `ModuleEnabled`.

## Consumers

- Web admin UI (school, user, course, plugin management).
- Mobile apps (user profile, course catalog, OTP verification).
- Public website (gallery, header menu, video gallery).
- AI agents (capability checks; user lookup; course queries).
- Automation systems (visitor logs, amount transfers, to-dos).

## Anti-Goals

- The platform domain does not implement authentication. It
  exposes commands that *require* authentication but does not
  validate credentials.
- The platform domain does not render the public website. It
  owns the data; consumers render.
- The platform domain does not implement payment processing. It
  records amount transfers; the payment port processes them.
- The platform domain does not store or render student academic
  data. That is the academic domain.
- The platform domain does not manage roles or capabilities. It
  only carries the `role_id` reference; the RBAC domain owns the
  catalog.
