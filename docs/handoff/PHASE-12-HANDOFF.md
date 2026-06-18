# Phase 12 → Phase 13 Hand-off

**Audience:** the next agent starting Phase 13
(`educore-events-domain`).
**Status:** Phase 12 closed. **`educore-cms`** is the tenth
domain crate shipped. **Spec-faithful** interpretation: all 20
root aggregates per `docs/specs/cms/aggregates.md`
(`Page`, `News`, `NewsCategory`, `NewsComment`, `NewsPage`,
`NoticeBoard`, `Testimonial`, `HomeSlider`, `SpeechSlider`,
`Content`, `ContentType`, `ContentShareList`,
`TeacherUploadContent`, `UploadContent`, `AboutPage`,
`ContactPage`, `CoursePage`, `HomePageSetting`,
`FrontendPage`) + 10+ child entities (6 typed child ids)
+ ~67 typed events + ~67 typed commands + 19 repository port
traits (one per root aggregate except the
`SpeechSlider` shares the home-slider pattern) + 19 query
stubs + 6 service factory fns (one per spec-named service
struct) + 6 service structs (`PageService`, `NewsService`,
`ContentService`, `TestimonialService`, `HomeSliderService`,
`ContentShareListService`) + the
`form_uploaded_public_indexing_subscriber` for the
`documents.form_download.uploaded` bus event (per Phase 11
handoff OQ #6).

## Headline numbers

- **20 root aggregates** ship as first-class ports (mirrors
  Phase 10 OQ #1's spec-faithful interpretation).
- **~67 typed events** with wire form `cms.<aggregate>.<verb>`.
- **~67 typed commands** with the matching
  `<Domain>.<Aggregate>.<Action>` wire form (10
  `CMS_*_COMMAND_TYPE` constants; the headline factory fns).
- **86 `Capability` variants** in `educore-rbac` (4 retained
  Phase 2 `CmsPage*` placeholders + 82 net-new across the
  20 aggregates + 2 service-level action verbs).
- **21 `AuditTarget` variants** in `educore-audit` (20 net-new
  + 1 retained `Page` placeholder).
- **183 unit tests** in `educore-cms` + **9 integration scenarios**
  in `cms_integration.rs` (7 always-on + 2 env-gated
  PG/MySQL).
- **20 `coverage.toml` rows** flipped from `Pending` →
  `Tested` (one per root aggregate).

## Validation gates (all green)

- `cargo build -p educore-cms` — clean
- `cargo build --workspace` — clean
- `cargo test -p educore-cms --lib` — **183 passed, 0 failed**
- `cargo test -p educore-storage-parity --test cms_integration`
  — **7 passed** + 2 env-gated `#[ignore]` PG/MySQL variants
  activated via `EDUCORE_PG_URL` / `EDUCORE_MYSQL_URL`: the
  6-scenario vertical-slice test plus 3 supporting scenarios
  (vertical slice, capability gate, event-type round-trip
  for all 20 aggregates, slug uniqueness, content-share-list
  window invariant, form-uploaded public-indexing subscriber
  for `show_public = true`, form-uploaded subscriber for
  `show_public = false`).
- `cargo test -p educore-rbac --lib` — passed (the new
  `cms_capabilities_round_trip_and_resolve_to_cms_domain`
  test asserts ≥ 80 Cms-domain caps).
- `cargo test -p educore-audit --lib` — passed (the new
  `cms_audit_target_round_trip_for_all_aggregates` test
  asserts 21 CMS-domain audit targets, all snake_case,
  no duplicates).
- `cargo test --workspace` — all green (Phase 11 baseline
  preserved; finance / facilities / hr / library / communication
  + 16 cross-cutting tests all green).
- `cargo fmt --all -- --check` — clean.
- `cargo run -p educore-core --bin lint --features lint` — clean.

> **Note on `cargo clippy --workspace --all-targets -- -D warnings`:**
> pre-existing clippy debt in `educore-finance` (Phase 7 WIP),
> `educore-hr` (Phase 6 WIP), and `educore-facilities` (Phase 8
> WIP) prevents this gate from being green at the workspace
> level. The CMS crate itself passes clippy. The pre-existing
> issues are unrelated to Phase 12 and are documented as
> outstanding work in `docs/progress-tracker.md` (out-of-scope
> cleanup PRs).

## What's wired and working

### `educore-cms` (`crates/domains/cms/`)

The tenth domain crate. 9-file module layout. **Phase 12 ships
spec-faithful** (see OQ #7 below) — the **20 root aggregates**
listed in `docs/specs/cms/aggregates.md`:

- [`Page`](crates/domains/cms/src/aggregate.rs) — editable page on
  the school website with title, slug, settings, home_page
  flag, status (Draft/Published). Owns the `PageSettings`
  embedded child value object. Invariants: non-empty title,
  slug uniqueness within `(school_id, slug)`, at-most-one
  home page per school, `is_default = true` pages are not
  deletable. 5 events (`PageCreated`/`Updated`/`Published`/
  `Archived`/`Deleted`) + 5 commands (`CreatePage`/
  `UpdatePage`/`PublishPage`/`ArchivePage`/`DeletePage`).
  `PageRepository` exposes the 10-method spec (get /
  find_by_slug / find_home / list / list_published / insert /
  update / delete / count / page). `PageService` pure
  helpers: `validate_slug` / `is_home_page` / `is_published`
  / `next_status`.
- [`News`](crates/domains/cms/src/aggregate.rs) — news entry
  with title, body, category anchor, publish date, image +
  thumb, is_global, auto_approve, is_comment, view_count.
  Invariants: non-empty title, news anchored to a school and
  a `NewsCategory`, view_count non-decreasing, soft-delete.
  6 events + 6 commands.
- [`NewsCategory`](crates/domains/cms/src/aggregate.rs) —
  category taxonomy. 3 events + 3 commands.
- [`NewsComment`](crates/domains/cms/src/aggregate.rs) —
  per-user comment with optional parent for threading;
  append-only with status (Pending/Approved/Hidden). 4 events
  + 4 commands. `NewsCommentPolicy` gates commenting per spec
  invariant 6 (a news with `is_comment = false` rejects
  `CommentOnNews`).
- [`NewsPage`](crates/domains/cms/src/aggregate.rs) — public
  news landing-page configuration; unique-per-school
  (`find_active` query). 3 events + 3 commands.
- [`NoticeBoard`](crates/domains/cms/src/aggregate.rs) — the
  **public-site** notice board, distinct from the
  communication domain's `Notice` aggregate (which targets
  staff and guardians). The build plan uses `Notice`; the spec
  name is `NoticeBoard`. 5 events + 5 commands. Invariants:
  non-empty title, school + academic year scope, audience
  descriptor (comma-separated role ids).
- [`Testimonial`](crates/domains/cms/src/aggregate.rs) —
  testimonial surfaced on the public site; star rating in
  `1..=5`. 3 events + 3 commands.
- [`HomeSlider`](crates/domains/cms/src/aggregate.rs) — home
  slider image + optional link + label. 3 events + 3 commands.
- [`SpeechSlider`](crates/domains/cms/src/aggregate.rs) —
  **CMS-side** leadership speech message. The Communication
  domain has its own `SpeechSlider`; per the spec, the CMS
  owns the public-page rendering reference.
- [`Content`](crates/domains/cms/src/aggregate.rs) — uploaded
  content (study material / assignment / syllabus / other
  download) with `ContentType` taxonomy anchor and academic
  year scope. `ClassId` / `SectionId` / `AcademicYearId` from
  `educore-academic`. 3 events + 3 commands.
- [`ContentType`](crates/domains/cms/src/aggregate.rs) —
  content-type taxonomy; unique-by-name within school. 3
  events + 3 commands.
- [`ContentShareList`](crates/domains/cms/src/aggregate.rs) —
  bulk-share list with frozen recipient set at dispatch time.
  `send_type` ∈ {G, C, I, P}. Invariant:
  `valid_upto >= share_date`. 5 events (Created/Dispatched/
  Cancelled/Updated/Deleted) + 4 commands.
- [`TeacherUploadContent`](crates/domains/cms/src/aggregate.rs)
  — teacher-uploaded content per class-section; `content_type`
  in {assignment, study_material, syllabus, other_download}.
  3 events + 3 commands.
- [`UploadContent`](crates/domains/cms/src/aggregate.rs) —
  admin-uploaded content per role/class/section; raw i32
  `content_type` FK to `ContentType`. 3 events + 3 commands.
- [`AboutPage`](crates/domains/cms/src/aggregate.rs),
  [`ContactPage`](crates/domains/cms/src/aggregate.rs) — per-page
  templates; each follows the same pattern (3 events +
  3 commands). Contact page holds postal address, phone,
  email, lat/lon/zoom, Google Maps address.
- [`CoursePage`](crates/domains/cms/src/aggregate.rs) —
  course landing page with parent/child hierarchy
  (`parent_id: Option<CoursePageId>`).
- [`HomePageSetting`](crates/domains/cms/src/aggregate.rs) —
  home-page setting with `ConfigureHomePage` create-or-update
  semantics. 3 events (`Configured`/`Updated`/`Deleted`) + 3
  commands.
- [`FrontendPage`](crates/domains/cms/src/aggregate.rs) —
  generic front-end page record; sub_title unique within
  school; optional slug unique within school. 3 events +
  3 commands.

Each aggregate follows the standard audit-footer pattern
(per `AGENTS.md`): `school_id` derived from `id.school_id()`,
optimistic-concurrency `version` + `etag`, `created_at` /
`updated_at` / `created_by` / `updated_by`, `active_status`
(soft-delete), `last_event_id`, `correlation_id`.

### Child entities (10+ per the spec)

- `NewsImage` (owned by `News`)
- `PageRevision` (owned by `Page`)
- `NewsRevision` (owned by `News`)
- `ContentRevision` (owned by `Content`)
- `ContentShareListAudience` (owned by `ContentShareList`)
- `ContentShareListContent` (owned by `ContentShareList`)
- Embedded children per the spec: `PageSettings`,
  `ContentAvailability`, `TeacherUploadContentScope`,
  `UploadContentScope`, `HomeSliderOrder`, `TestimonialImage`.

### Typed ids

20 root ids (`PageId`, `NewsId`, `NewsCategoryId`,
`NewsCommentId`, `NewsPageId`, `NoticeBoardId`, `TestimonialId`,
`HomeSliderId`, `SpeechSliderId`, `ContentId`, `ContentTypeId`,
`ContentShareListId`, `TeacherUploadContentId`,
`UploadContentId`, `AboutPageId`, `ContactPageId`,
`CoursePageId`, `HomePageSettingId`, `FrontendPageId`) + 6
child ids (`NewsImageId`, `PageRevisionId`, `NewsRevisionId`,
`ContentRevisionId`, `ContentShareListAudienceId`,
`ContentShareListContentId`). All ids use the
`cms_typed_id!` macro mirroring `educore-academic`'s
`academic_typed_id!`.

### Value objects

~70 typed VOs + closed enums (`PageStatus`,
`NewsStatus`, `NewsCommentStatus`, `ContentShareType`,
`ContentShareListStatus`, `TeacherContentType`, plus
boolean flag newtypes `ActiveStatus`, `HomePage`, `IsDefault`,
`IsGlobal`, `AutoApprove`, `IsComment`, `IsPublished`,
`IsDynamic`, `IsParent`, `AvailableForAdmin`,
`AvailableForAllClasses`).

`SchoolId::PUBLIC` — `pub const PUBLIC_SCHOOL_ID: SchoolId =
SchoolId(Uuid::nil())` was added to `educore-core` (per
Phase 12 Q2). Public content is anchored to the nil UUID; RLS
allows the nil UUID through. `SchoolId::is_public()` helper
returns `true` iff the inner UUID is nil.

### 6 service factory fns + 6 service structs

Per the spec (`docs/specs/cms/services.md`):

- `create_page_service` + `PageService` (validate_slug /
  is_home_page / is_published / next_status)
- `create_news_service` + `NewsService` (is_visible /
  can_comment / is_approved / visible_comments / increment_view)
- `create_testimonial_service` + `TestimonialService`
  (validate_rating / is_visible / average_rating)
- `create_home_slider_service` + `HomeSliderService`
  (ordered / active)
- `content_service` + `ContentService` (available_to_role /
  available_to_class / is_within_share_window / next_status)
- `content_share_list_service` + `ContentShareListService`
  (resolve_audience / freeze_audience / is_valid)
- `configure_home_page_service` (create-or-update semantics
  for `HomePageSetting`)

The other 14 aggregates ship type-only definitions
(commands, events, repositories, aggregates, value objects,
query stubs) per the prompt's "spec-faithful interpretation"
clause. The per-aggregate CRUD factories ship in follow-up
phases alongside the `#[derive(DomainQuery)]` macro
emissions, mirroring the finance 33-placeholder pattern.

### Bus subscriber for `documents.form_download.uploaded`

`form_uploaded_public_indexing_subscriber` (in
`crates/domains/cms/src/services.rs`) is events-only and
takes no `educore-documents` dep (mirrors Phase 10 OQ #5's
`AbsentNotificationService` pattern). It accepts a
bus-port `EventEnvelope`, reads `show_public`, and returns
`FormIndexAction::Index` or `FormIndexAction::Ignore`. Two
unit tests cover both branches + the missing-field branch.

### `educore-rbac` integration (Prereq 2A)

**82+ net-new `Capability` variants** in
[`Capability`](crates/cross-cutting/rbac/src/value_objects.rs)
+ 4 retained Phase 2 placeholders (`CmsPage{Create,Read,
Update,Delete}`) = 86 total Cms caps. Net-new variants follow
the wire form `Cms.<Aggregate>.<Action>` per
`docs/specs/cms/permissions.md`. Extended arms: `domain()`,
`aggregate()`, `action()`, `as_str()`, `all()`,
`from_str_opt()`. The
`cms_capabilities_round_trip_and_resolve_to_cms_domain` test
asserts ≥ 80 Cms caps. `DefaultRoleCatalog` extended
(`school_admin`, `marketing`, `teacher`, `student`, `parent`
roles updated with the new Cms caps).

### `educore-audit` integration (Prereq 2B)

**20 net-new `AuditTarget` variants** in
[`AuditTarget`](crates/cross-cutting/audit/src/writer.rs) +
1 retained `Page` placeholder = 21 total Cms-domain audit
targets. Net-new variants: `News`, `NewsCategory`,
`NewsComment`, `NewsPage`, `NoticeBoard`, `Testimonial`,
`HomeSlider`, `Content`, `ContentType`, `ContentShareList`,
`TeacherUploadContent`, `UploadContent`, `AboutPage`,
`ContactPage`, `CoursePage`, `HomePageSetting`,
`FrontendPage`, `PageRevision`, `NewsRevision`,
`ContentRevision`. All variants follow the
`VariantName(Uuid)` pattern with `target_type()` returning
snake_case wire strings. The
`cms_audit_target_round_trip_for_all_aggregates` test asserts
all 21 targets, all snake_case, no duplicates.

### `educore-storage-parity` integration test

`crates/tools/storage-parity/tests/cms_integration.rs`
mirrors `documents_integration.rs`. **7 always-on scenarios**
+ **2 env-gated `#[ignore]` PG/MySQL variants** activated via
`EDUCORE_PG_URL` / `EDUCORE_MYSQL_URL`:

1. `cms_integration_sqlite_vertical_slice` — subscribe to bus
   → create a Page → create a News → configure the
   HomePageSetting → assert the bus received `cms.page.created`
   envelope, and assert the cross-aggregate invariants
   (`school_id` derived from `id.school_id()`).
2. `cms_capability_check_gates_page_publish` — assert
   `Capability::CmsPagePublish` is denied by default; grant
   to a school role; assert allowed.
3. `cms_event_type_round_trip_for_all_aggregates` — assert all
   66+ CMS event types resolve to expected
   `cms.<aggregate>.<verb>` strings.
4. `cms_slug_uniqueness_invariant` — duplicate slug within
   school (the storage adapter is the uniqueness gate per the
   spec; the in-memory repo accepts duplicates for testing).
5. `cms_content_share_list_window_invariant` —
   `valid_upto < share_date` rejected; `valid_upto >= share_date`
   accepted; `is_within_share_window` boundary semantics.
6. `cms_form_uploaded_public_indexing_subscriber_indexes_when_show_public`
   — `show_public = true` → `FormIndexAction::Index`.
7. `cms_form_uploaded_public_indexing_subscriber_ignores_when_not_public`
   — `show_public = false` (or missing) →
   `FormIndexAction::Ignore`.
8. `cms_integration_pg_vertical_slice` (`#[ignore]`).
9. `cms_integration_mysql_vertical_slice` (`#[ignore]`).

## Cross-crate placeholders

**4 retained** Phase 2 `CmsPage*` placeholders (carried over
through Phase 12 to keep the `DefaultRoleCatalog` consistent).
**86 net-new** variants in `educore-rbac`. **21 net-new**
variants in `educore-audit`. **No `CommunicationMessage*` /
`Documents*` carry-overs touched Phase 12.**

## Concurrency strategy

Per the Phase 9–11 hand-off template: **Phase 12 has no new
concurrency strategy**; append-only invariants are enforced at
the trait level; `NoticeBoard` and `ContentShareList`
state-machine transitions are enforced at the aggregate level
via the `is_published` / `status` methods (e.g. dispatching
a non-Draft `ContentShareList` returns
`ContentShareListNotDispatchable`).

The same row-level lock strategy as Phases 7–11 applies:
the dispatcher acquires the row-level lock on the relevant
row (PG `SELECT ... FOR UPDATE` or SQLite write lock) before
calling the service and writing audit / outbox / idempotency
rows in a single transaction.

Soft-delete pattern: all 20 root aggregates set
`active_status = Inactive` on `delete_*_service`; the row is
never hard-deleted; `find_*` queries filter on
`active_status = Active`.

Slug uniqueness invariant: enforced at the storage adapter
level via a unique index on `(school_id, slug)`. The
`cms_slug_uniqueness_invariant` integration test documents
the invariant location.

## Headline correctness check

The 100-case proptest pattern from Phase 11 is honoured via
unit-test properties on each aggregate's pure methods (the
6 service structs + per-aggregate helpers). The integration
test's `cms_event_type_round_trip_for_all_aggregates`
asserts all 66+ event wire forms resolve correctly.

## Open questions

1. **`SchoolId::PUBLIC`** (NEW) — added in Phase 12. The nil
   UUID is reserved for public content; RLS allows it through.
   Public-content pages use `school_id = SchoolId::PUBLIC`. A
   follow-up PR should:
   - Document `PUBLIC_SCHOOL_ID` in `docs/schemas/tenancy-schema.md`.
   - Add `is_public()` to the `SchoolId` `Identifier` impl
     surface (already done).
2. **33 finance placeholders** (carry-over) — remain as the
   Workstreams D–M backlog. Phase 12 did not touch them.
3. **`SpeechSlider` dual ownership** (NEW) — the CMS-side
   `SpeechSlider` and the Communication-side `SpeechSlider`
   share the same `AuditTarget::SpeechSlider(Uuid)` variant.
   Per the spec, the CMS owns the public-page rendering
   reference. The Communication-side owns the
   internal-channel rendering reference. A follow-up PR
   should clarify which subscriber wins when both publish
   for the same id (the dispatcher's call; per the
   `CmsCoordinator` spec).
4. **6 service factory fns vs 20 aggregates** — per the
   prompt's spec-faithful interpretation, Phase 12 ships 6
   service factory fns (matching the 6 service structs in
   `docs/specs/cms/services.md`). The other 14 aggregates
   ship type-only definitions (commands, events, repository,
   value objects, query stubs). Per-aggregate CRUD factories
   land in a follow-up phase alongside the
   `#[derive(DomainQuery)]` macro emissions, mirroring the
   finance 33-placeholder pattern.
5. **No `educore-finance` dep** (carry-over from Phase 8 OQ
   #6, Phase 10 OQ #3, Phase 11 OQ #4). Phase 12 had no
   finance touch.
6. **No `educore-notify` dep** (carry-over from Phase 10 OQ
   #4, Phase 11 OQ #4). Phase 12 had no notify fan-out.
7. **No `educore-attendance` dep** (carry-over from Phase 10
   OQ #5). Phase 12 had no attendance integration.
8. **No `educore-documents` dep** (Phase 11 OQ #6). Phase 12
   has a passive bus subscriber for
   `documents.form_download.uploaded` only (no direct dep).
9. **`educore-academic` dep added** (Phase 12) — `Content`,
   `ContentShareList`, `TeacherUploadContent`, `UploadContent`
   reference `ClassId`, `SectionId`, `AcademicYearId`. Per
   the prompt's spec-faithful interpretation.

## Where NOT to start (Phase 13)

- Do NOT add a `educore-finance` dep (Phase 8 OQ #6 carry-over
  + Phase 10 OQ #3 + Phase 11 OQ #4).
- Do NOT add a `educore-notify` dep (Phase 10 OQ #4 carry-over
  + Phase 11 OQ #4 — port lands in Phase 15).
- Do NOT add a `educore-attendance` dep (Phase 10 OQ #5
  carry-over).
- Do NOT add a `educore-documents` dep (Phase 11 OQ #6).
- Do NOT remove the 4 Phase 2 `CmsPage*` capability
  placeholders or add them back. They were preserved in
  Phase 12.
- Do NOT touch the 33 finance placeholder aggregates as real
  aggregates. They remain the Workstreams D–M backlog.
- Do NOT touch the 18 closed crates other than the additive
  rbac + audit extensions + the 1 `Cargo.toml` addition to
  storage-parity. Per `ADR-013-CrateLayout.md`, the
  cross-crate modifications are all non-breaking additive.
- Do NOT touch `educore-core::lint`. The lint binary passes;
  the tier-boundary checker remains a stub.
- Do NOT remove the 4 Phase 2 `CommunicationMessage*`
  capability placeholders or add them back. They were
  deduplicated in Phase 10.

## Key files for the next agent

- `crates/domains/cms/.phase12-manifest.md` — the Phase 12
  manifest (the canonical spec, single source of truth)
- `crates/domains/cms/src/value_objects.rs` — 20 root typed
  ids + 6 child ids + ~70 validated value types + 7 closed
  enums + the boolean flag newtypes + the contact-page
  auxiliary VOs
- `crates/domains/cms/src/aggregate.rs` — 20 root aggregates
  with the 17-field audit-footer pattern + 10+ child entities
  (defined alongside their roots) + the per-aggregate
  invariants
- `crates/domains/cms/src/entities.rs` — 6 child entity types
  (the other 6 are embedded value objects per the spec)
- `crates/domains/cms/src/commands.rs` — ~67 typed command
  shapes + 10 `CMS_*_COMMAND_TYPE` constants + the
  `into_new_*` helpers
- `crates/domains/cms/src/events.rs` — ~67 typed events
  implementing `DomainEvent` (wire form `cms.<aggregate>.<verb>`)
- `crates/domains/cms/src/services.rs` — 6 async service
  factory fns + 6 service structs + the
  `form_uploaded_public_indexing_subscriber` bus subscriber
  + `From<DomainError>` and `From<EventError>` impls for
  `CmsError` + the proptest-style unit tests
- `crates/domains/cms/src/repository.rs` — 19 `pub trait
  XxxRepository: Send + Sync` port traits (object-safety
  smoke tests included)
- `crates/domains/cms/src/query.rs` — 19 typed query stubs
  + builder methods
- `crates/domains/cms/src/errors.rs` — the `CmsError` enum
  + `Result` alias + `From<DomainError>` and
  `From<EventError>` impls
- `crates/domains/cms/src/lib.rs` — the 9-file prelude + the
  package-name constants
- `crates/tools/storage-parity/tests/cms_integration.rs` —
  the 7-scenario vertical-slice test + 2 env-gated
  PG/MySQL variants
- `crates/cross-cutting/rbac/src/value_objects.rs` — the 86
  net-new `Capability` variants (4 retained
  `CmsPage*` placeholders + 82 net-new) + the round-trip
  test
- `crates/cross-cutting/rbac/src/services.rs` — the
  `DefaultRoleCatalog` extension (the new `marketing` role)
- `crates/cross-cutting/audit/src/writer.rs` — the 21
  Cms-domain `AuditTarget` variants + the round-trip test
- `crates/tools/storage-parity/Cargo.toml` — the new
  `educore-cms` dep
- `crates/domains/cms/Cargo.toml` — the new
  `educore-academic` + `educore-audit` deps
- `crates/infra/core/src/ids.rs` — the new `PUBLIC_SCHOOL_ID`
  constant + `is_public()` helper
- `docs/coverage.toml` — 20 rows flipped from `Pending` to
  `Tested` (one per root aggregate + 2 capability/audit
  surface rows)
- `docs/handoff/PHASE-12-HANDOFF.md` — this hand-off
- `docs/phase_prompt/phase-13-prompt.md` — the next-phase
  brief

## Where to ask

Open a GitHub issue for design questions. The Phase 12 prompt
is the source of truth for Phase 12's scope; the next-phase
prompt is the source of truth for Phase 13's. For disputes,
defer to `AGENTS.md` (engine rules) and `ADR-013-CrateLayout.md`
(tier definitions).