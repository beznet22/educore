# Audit findings: educore-cms (Phase 12)

**Scope:** `crates/domains/cms/`, `docs/specs/cms/`, `docs/commands/cms.md`,
`docs/events/cms.md`, `docs/handoff/PHASE-12-HANDOFF.md`,
`AGENTS.md` (the cms row).

**Total findings:** 67

---

### FINDING 1

- **id:** DOMAIN-CMS-001
- **area:** domain-crates
- **severity:** Critical
- **location:** docs/specs/cms/aggregates.md:209
- **description:** The `NoticeBoard` aggregate spec mandates an academic-year
  anchor, but the Rust struct in `aggregate.rs` has no `academic_id` field.
  The aggregate therefore cannot honour spec invariant 2
  ("anchored to a school and an academic year").
- **expected:** `docs/specs/cms/aggregates.md` line 209:
  `2. A \`NoticeBoard\` is anchored to a school and an academic year.`
- **evidence:** `crates/domains/cms/src/aggregate.rs:869-904` —
  ```rust
  pub struct NoticeBoard {
      pub id: NoticeBoardId,
      pub school_id: SchoolId,
      pub notice_title: NoticeTitle,
      pub notice_message: NoticeMessage,
      pub notice_date: NoticeDate,
      pub publish_on: Option<PublishDate>,
      pub inform_to: AudienceDescriptor,
      pub is_published: IsPublished,
      ...
  }
  ```
  No `academic_id: AcademicYearId` field; no `academic_id` reference
  in the entire `NoticeBoard` section of `aggregate.rs`
  (`grep -nE "academic_id|AcademicYearId" aggregate.rs | grep NoticeBoard` returns no rows).

---

### FINDING 2

- **id:** DOMAIN-CMS-002
- **area:** domain-crates
- **severity:** Critical
- **location:** crates/domains/cms/src/commands.rs:38-49, 57-435
- **description:** Only 10 of the ~62 commands listed in the
  `docs/specs/cms/commands.md` spec are implemented. The wire-form
  command types `Update*`, `Publish*` (non-Page), `Unpublish*`,
  `Delete*` (non-Page), `Comment*`, `Moderate*`, `Dispatch*`,
  `Cancel*`, the news-comment commands, the news-category
  commands, the news-page commands, the content-type commands,
  the speech-slider commands, the teacher-upload commands, the
  upload-content commands, the about-page commands, the
  contact-page commands, the course-page commands, the
  frontend-page commands, and the IncrementNewsView command are
  all absent from the code.
- **expected:** Spec lists commands for all 20 aggregates
  (`docs/specs/cms/commands.md` lines 10-579). Example:
  `## UpdatePage` (line 33), `## PublishNews` (line 132),
  `## DeleteNews` (line 157), `## CommentOnNews` (line 170),
  `## ModerateNewsComment` (line 186), `## CreateSpeechSlider`
  (line 568), etc.
- **evidence:** `crates/domains/cms/src/commands.rs:38-49` defines
  only 10 `CMS_*_COMMAND_TYPE` constants
  (`CMS_PAGE_CREATE_COMMAND_TYPE`, `CMS_PAGE_PUBLISH_COMMAND_TYPE`,
  `CMS_PAGE_ARCHIVE_COMMAND_TYPE`, `CMS_PAGE_DELETE_COMMAND_TYPE`,
  `CMS_NEWS_CREATE_COMMAND_TYPE`, `CMS_TESTIMONIAL_CREATE_COMMAND_TYPE`,
  `CMS_HOME_SLIDER_CREATE_COMMAND_TYPE`, `CMS_CONTENT_CREATE_COMMAND_TYPE`,
  `CMS_CONTENT_SHARE_LIST_CREATE_COMMAND_TYPE`,
  `CMS_HOME_PAGE_SETTING_CONFIGURE_COMMAND_TYPE`); the file defines
  only 10 `pub struct *Command` types
  (`CreatePageCommand`, `PublishPageCommand`, `ArchivePageCommand`,
  `DeletePageCommand`, `CreateNewsCommand`, `CreateTestimonialCommand`,
  `CreateHomeSliderCommand`, `CreateContentCommand`,
  `CreateContentShareListCommand`, `ConfigureHomePageCommand`).
  `grep -rnE "IncrementNewsViewCommand|UpdatePageCommand|UpdateNewsCommand|PublishNewsCommand|DeleteNewsCommand|CommentOnNewsCommand|ModerateNewsCommand|DispatchContentShareListCommand|CancelContentShareListCommand|CreateSpeechSliderCommand|CreateContentTypeCommand|CreateAboutPageCommand|CreateContactPageCommand|CreateCoursePageCommand|CreateFrontendPageCommand|CreateTeacherUploadContentCommand|CreateUploadContentCommand|CreateNewsCategoryCommand|CreateNewsPageCommand|DeleteNoticeBoardCommand|UpdateTestimonialCommand|DeleteTestimonialCommand" crates/` returns no results.

---

### FINDING 3

- **id:** DOMAIN-CMS-003
- **area:** domain-crates
- **severity:** Critical
- **location:** crates/domains/cms/src/aggregate.rs:797-836
- **description:** `NewsPage` aggregate has only `new()` and
  `soft_delete()`; there is no `update()` method. Spec
  commands `UpdateNewsPage` is therefore unimplementable.
- **expected:** `docs/specs/cms/commands.md` lines 535-545:
  `## CreateNewsPage / UpdateNewsPage / DeleteNewsPage` with
  `Capabilities: NewsPage.Create, NewsPage.Update, NewsPage.Delete`.
- **evidence:** `crates/domains/cms/src/aggregate.rs:797-836` —
  ```rust
  impl NewsPage {
      pub fn new(cmd: NewNewsPage) -> Result<Self, CmsError> { ... }
      pub fn soft_delete(&mut self, actor: UserId, at: Timestamp) -> Result<(), CmsError> { ... }
  }
  ```
  No `update()` or `update_*()` method on `NewsPage`.

---

### FINDING 4

- **id:** DOMAIN-CMS-004
- **area:** domain-crates
- **severity:** Critical
- **location:** crates/domains/cms/src/aggregate.rs (NoticeBoard, Testimonial, HomeSlider, SpeechSlider, TeacherUploadContent, UploadContent, AboutPage, ContactPage, CoursePage, HomePageSetting, FrontendPage)
- **description:** 11 of the 19 spec'd aggregates have no
  `update()` (or `rename`/`update_*`) method. Spec commands
  `Update*` for these aggregates are unimplementable.
- **expected:** Spec commands.md lists `UpdateNoticeBoard`,
  `UpdateTestimonial`, `UpdateHomeSlider`, `UpdateSpeechSlider`,
  `UpdateTeacherUploadContent`, `UpdateUploadContent`,
  `UpdateAboutPage`, `UpdateContactPage`, `UpdateCoursePage`,
  `UpdateFrontendPage` (each with `Capabilities: <Aggregate>.Update`).
- **evidence:** `grep -nE "impl (NoticeBoard|Testimonial|HomeSlider|SpeechSlider|TeacherUploadContent|UploadContent|AboutPage|ContactPage|CoursePage|HomePageSetting|FrontendPage)" crates/domains/cms/src/aggregate.rs` followed by
  inspection of each `impl` block: only `new()`, `soft_delete()`,
  and (for `NoticeBoard`) `publish()`/`unpublish()` are defined.
  `fn update` returns no rows for those impl blocks
  (`grep -nE "fn update" crates/domains/cms/src/aggregate.rs`
  yields lines 160/467/1416 only — Page, News, Content).

---

### FINDING 5

- **id:** DOMAIN-CMS-005
- **area:** domain-crates
- **severity:** High
- **location:** crates/domains/cms/src/aggregate.rs:543-548
- **description:** `News::is_visible(today)` ignores the spec
  invariant 4 (`is_global = true` news is visible across all
  schools). The method only checks `active_status` and
  `publish_date`.
- **expected:** `docs/specs/cms/aggregates.md` line 72-73:
  `4. A \`News\` may be \`is_global\` (visible across all schools in a
  multi-tenant SaaS) or scoped to one school.`
- **evidence:** `crates/domains/cms/src/aggregate.rs:543-548`:
  ```rust
  pub fn is_visible(&self, today: NaiveDate) -> bool {
      self.active_status.is_active() && self.publish_date.as_naive_date() <= today
  }
  ```
  No reference to `self.is_global`. The handoff's
  `News::is_visible` predicate does not implement the global visibility
  rule.

---

### FINDING 6

- **id:** DOMAIN-CMS-006
- **area:** domain-crates
- **severity:** Critical
- **location:** crates/domains/cms/src/services.rs:404-446
- **description:** `TestimonialService::average_rating` does not
  compute a true average: it computes `total/count` but discards
  `total` (line 439 `let _ = total;`) and returns `1.0` for any
  non-empty list. The docstring says "the unweighted mean divides
  by count to get the mean rating" but the implementation returns
  a constant. The test `testimonial_service_average_rating_computes_correctly`
  (line 1034-1057) only asserts `avg.is_finite() && avg > 0.0`.
- **expected:** `docs/specs/cms/services.md` line 67:
  `pub fn average_rating(testimonials: &[Testimonial]) -> f32 { ... }`
  — canonical mean of star ratings.
- **evidence:** `crates/domains/cms/src/services.rs:428-445`:
  ```rust
  pub fn average_rating(testimonials: &[Testimonial]) -> f32 {
      if testimonials.is_empty() { return 0.0; }
      let total: u32 = testimonials.iter().map(|t| u32::from(t.star_rating.value())).sum();
      let count = u32::try_from(testimonials.len()).unwrap_or(u32::MAX);
      let _ = total;
      if count == 0 { 0.0 } else { 1.0 }
  }
  ```

---

### FINDING 7

- **id:** DOMAIN-CMS-007
- **area:** domain-crates
- **severity:** Critical
- **location:** crates/domains/cms/src/aggregate.rs:1483-1494
- **description:** `Content::available_to_class(class, section)`
  uses three `(u128 >> 64) as i64` truncating casts to compare
  the typed `ClassId` / `SectionId` UUIDs against
  `available_for_class: Option<i32>` / `available_for_section:
  Option<i32>`. This both (a) loses the high 64 bits of every
  UUID via `as i64`, and (b) violates the typed-id pattern: the
  aggregate holds `Option<i32>` raw integers where the spec uses
  `ClassId` / `SectionId`.
- **expected:** AGENTS.md "Type Safety" rule: no `as` casts that
  truncate; use `TryFrom`/`TryInto`. Engine rule "Compile-time
  safety over strings" implies typed ids, not `i32`, in domain
  fields.
- **evidence:** `crates/domains/cms/src/aggregate.rs:1483-1494`:
  ```rust
  pub fn available_to_class(&self, class: ClassId, section: Option<SectionId>) -> bool {
      match (self.available_for_class, self.available_for_section) {
          (None, None) => true,
          (Some(c), None) => i64::from(c) == (class.as_uuid().as_u128() >> 64) as i64,
          (None, Some(_)) => false,
          (Some(c), Some(s)) => {
              i64::from(c) == (class.as_uuid().as_u128() >> 64) as i64
                  && section.is_some_and(|sec| i64::from(s) == (sec.as_uuid().as_u128() >> 64) as i64)
          }
      }
  }
  ```

---

### FINDING 8

- **id:** DOMAIN-CMS-008
- **area:** domain-crates
- **severity:** Critical
- **location:** crates/domains/cms/src/value_objects.rs:2515-2548
- **description:** `PageSettings` wraps `serde_json::Value` directly
  in domain code. The engine rule (AGENTS.md "Type Safety") and
  the engine code standards forbid `serde_json::Value` in domain
  code.
- **expected:** AGENTS.md "Type Safety": `No \`serde_json::Value\` in
  domain code. Use typed wrappers.` Spec value-objects.md line 130-132:
  `PageSettings | A typed JSON value object with versioned schema` (the
  spec describes it as a typed JSON value; the engine rule says typed,
  not `serde_json::Value`).
- **evidence:** `crates/domains/cms/src/value_objects.rs:2519-2520`:
  ```rust
  pub struct PageSettings(pub serde_json::Value);
  ```
  Also `services.rs:843` uses `serde_json::Value::as_bool` inside
  the `form_uploaded_public_indexing_subscriber` (`envelope.payload
  .get("show_public").and_then(serde_json::Value::as_bool)`).

---

### FINDING 9

- **id:** DOMAIN-CMS-009
- **area:** domain-crates
- **severity:** High
- **location:** crates/domains/cms/src/services.rs:50-55
- **description:** `snapshot::<T: serde::Serialize>` uses
  `unwrap_or_default()` on a `serde_json::to_vec(value)` failure.
  Audit rows are silently corrupted on serialization failure
  rather than propagating the error to the caller.
- **expected:** AGENTS.md "Type Safety": all fallible APIs return
  `Result<T, DomainError>`. The audit row payload is a
  security-relevant artefact; silent defaulting violates audit-first.
- **evidence:** `crates/domains/cms/src/services.rs:50-55`:
  ```rust
  /// Snapshot a serialised value for an audit row. A serde_json
  /// failure falls back to an empty payload (audit rows are
  /// best-effort).
  fn snapshot<T: serde::Serialize>(value: &T) -> Bytes {
      Bytes::from(serde_json::to_vec(value).unwrap_or_default())
  }
  ```

---

### FINDING 10

- **id:** DOMAIN-CMS-010
- **area:** domain-crates
- **severity:** High
- **location:** crates/domains/cms/src/{commands,entities,events,repository,query,services,value_objects}.rs (7 files)
- **description:** Seven of nine source files declare a module-level
  `#![allow(missing_docs)]` (or `#![allow(dead_code, clippy::all)]`
  blanket). The crate-level lib.rs declares `#![deny(missing_docs)]`
  but the blanket suppressions defeat it for ~95% of the file
  contents.
- **expected:** AGENTS.md "Type Safety" + `docs/code-standards.md`
  § Type Safety: `#![deny(missing_docs)]` and `unwrap`, `expect`,
  `panic!` are forbidden in production paths.
- **evidence:** `crates/domains/cms/src/aggregate.rs:16-20`:
  ```rust
  #![allow(missing_docs)]
  #![allow(clippy::too_many_arguments)]
  #![allow(clippy::unnecessary_literal_unwrap)]
  #![allow(unused_imports)]
  #![allow(dead_code)]
  ```
  Plus `commands.rs:14-15`, `entities.rs:21-22`, `events.rs:32-33`,
  `repository.rs:9-10`, `query.rs:12-13`, `services.rs:22-23`,
  `value_objects.rs:22-23`.

---

### FINDING 11

- **id:** DOMAIN-CMS-011
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/cms/src/entities.rs:21-22
- **description:** The `entities.rs` module applies a blanket
  `#![allow(dead_code, clippy::all)]` plus `#![allow(missing_docs)]`,
  hiding the fact that `NewsImage` carries `image_thumb: Option<FileReference>`
  but no `News` aggregate in `aggregate.rs` holds an image-thumb
  attribute through the same path.
- **expected:** Spec entities.md lines 6-12 specifies
  `NewsImage` carries the `image` and `image_thumb` `FileReference`s,
  both owned by `News`. AGENTS.md: "No `#[allow(dead_code)]`".
- **evidence:** `crates/domains/cms/src/entities.rs:21-22` and
  `grep -nE "image_thumb" crates/domains/cms/src/aggregate.rs`
  shows `image_thumb: Option<FileReference>` on `News` (line 333,
  399) but no separate `NewsImage` construction site.

---

### FINDING 12

- **id:** DOMAIN-CMS-012
- **area:** domain-crates
- **severity:** High
- **location:** crates/domains/cms/src/value_objects.rs (no `TestimonialRating`, no `RoleIdList`, no `Visible`, no `ContentStatus`)
- **description:** Four value objects listed in
  `docs/specs/cms/value-objects.md` are absent from `value_objects.rs`:
  `TestimonialRating` (= `StarRating` type alias),
  `RoleIdList` (comma-separated `RoleId` list, decoded to
  `Vec<RoleId>`), `Visible` (`bool` newtype), and `ContentStatus`
  enum (`Draft`, `Published`, `Archived`).
- **expected:** `docs/specs/cms/value-objects.md` lines 69-92:
  `TestimonialRating | StarRating` (line 80),
  `Visible | bool — when true, the row is visible on the public site`
  (line 81), `ContentStatus | Draft, Published, Archived` (line 74),
  `RoleIdList | Comma-separated list of RoleId (decoded into Vec<RoleId>)`
  (line 114).
- **evidence:** `grep -nE "TestimonialRating|RoleIdList|^pub struct Visible|^pub enum ContentStatus" crates/domains/cms/src/value_objects.rs`
  returns no results.

---

### FINDING 13

- **id:** DOMAIN-CMS-013
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/cms/src/value_objects.rs:2473-2509
- **description:** `AudienceDescriptor::split()` returns
  `Vec<String>` rather than `Vec<RoleId>` as the spec mandates.
  The spec requires the audience descriptor to be decoded into a
  typed `Vec<RoleId>`.
- **expected:** `docs/specs/cms/value-objects.md` line 114:
  `RoleIdList | Comma-separated list of RoleId (decoded into Vec<RoleId>)`.
- **evidence:** `crates/domains/cms/src/value_objects.rs:2493-2502`:
  ```rust
  pub fn split(&self) -> Vec<String> {
      self.0.split(',').map(str::trim).filter(|s| !s.is_empty())
          .map(str::to_owned).collect()
  }
  ```
  Returns `Vec<String>`, not `Vec<RoleId>`.

---

### FINDING 14

- **id:** DOMAIN-CMS-014
- **area:** domain-crates
- **severity:** Critical
- **location:** crates/domains/cms/src/services.rs (no `NewsCommentPolicy`, no `PublishedPages`, no `ActiveNews`, no `VisibleTestimonials`, no `CmsCoordinator`)
- **description:** The spec defines four policy/specification
  types and a `CmsCoordinator` cross-domain coordinator. None of
  them are implemented.
- **expected:** `docs/specs/cms/services.md` lines 101-171:
  `## NewsCommentPolicy`, `## Specification: PublishedPages`,
  `## Specification: ActiveNews`, `## Specification: VisibleTestimonials`,
  `## Cross-Domain Coordinator`.
- **evidence:** `grep -nE "struct NewsCommentPolicy|struct PublishedPages|struct ActiveNews|struct VisibleTestimonials|struct CmsCoordinator" crates/domains/cms/src/` returns no matches.
  `services.rs` defines only `PageService`, `NewsService`,
  `TestimonialService`, `HomeSliderService`, `ContentService`,
  `ContentShareListService` (lines 80/320/404/489/554/649).

---

### FINDING 15

- **id:** DOMAIN-CMS-015
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/cms/src/entities.rs (entire file)
- **description:** The `CoursePageRelation` entity listed in
  spec entities.md is absent. Spec mandates a typed edge between
  two `CoursePage` aggregates with `CoursePageRelationId(SchoolId,
  Uuid)`.
- **expected:** `docs/specs/cms/entities.md` lines 85-91:
  `## CoursePageRelation — Identity: CoursePageRelationId(SchoolId, Uuid); Owner: CoursePage; A typed edge between two CoursePage aggregates: parent_id → CoursePageId`.
- **evidence:** `grep -rnE "CoursePageRelation" crates/` returns no matches. No `CoursePageRelationId` in `value_objects.rs` (lines 110-211).

---

### FINDING 16

- **id:** DOMAIN-CMS-016
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/cms/src/services.rs:835-859
- **description:** `form_uploaded_public_indexing_subscriber` is a
  pure synchronous function that takes an `EventEnvelope` and
  returns a `FormIndexAction` enum, but there is no async
  repository wiring. The signature does not match the engine's
  bus subscriber pattern; the spec does not define this shape
  anywhere — it is described in `PHASE-11-HANDOFF.md` OQ #6.
- **expected:** Spec calls this out in `docs/handoff/PHASE-12-HANDOFF.md`
  lines 246-254 ("events-only ... takes no `educore-documents` dep
  (mirrors Phase 10 OQ #5's `AbsentNotificationService` pattern)").
  The implementation uses `serde_json::Value` (`services.rs:843`)
  instead of a typed event payload.
- **evidence:** `crates/domains/cms/src/services.rs:835-859` —
  ```rust
  pub fn form_uploaded_public_indexing_subscriber(
      envelope: educore_events::envelope::EventEnvelope,
  ) -> FormIndexAction {
      let show_public = envelope.payload.get("show_public")
          .and_then(serde_json::Value::as_bool).unwrap_or(false);
      if show_public { FormIndexAction::Index } else { FormIndexAction::Ignore }
  }
  ```
  No async / no repository wiring. Subscriber is not registered
  with any bus adapter.

---

### FINDING 17

- **id:** DOMAIN-CMS-017
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/cms/src/events.rs:2460-2513
- **description:** `ContentShareListUpdated` is defined as an
  event in the code but is **not** listed in
  `docs/specs/cms/events.md` or in `docs/events/cms.md`. It is also
  never emitted by any service-factory function. This is an
  undocumented event.
- **expected:** `docs/specs/cms/events.md` lines 170-187 lists
  only `ContentShareListCreated`, `ContentShareListDispatched`,
  `ContentShareListCancelled`, `ContentShareListDeleted` (4 events).
  `docs/events/cms.md` lines 49-52 likewise lists only 4 events
  for `ContentShareList`.
- **evidence:** `crates/domains/cms/src/events.rs:2460` defines
  `pub struct ContentShareListUpdated`; `crates/domains/cms/src/events.rs:2496`
  defines `EVENT_TYPE = "cms.content_share_list.updated"`. The event is only
  exercised in the events.rs test (`events.rs:3965`); no service factory
  publishes it (`grep -rnE "ContentShareListUpdated::new" crates/`
  matches only `events.rs:3965`).

---

### FINDING 18

- **id:** DOMAIN-CMS-018
- **area:** domain-crates
- **severity:** Critical
- **location:** crates/domains/cms/src/events.rs:3387-3435 (vs spec events.md:243)
- **description:** The spec defines the event
  `HomePageSettingCreated` (`docs/specs/cms/events.md:243` and
  `docs/events/cms.md:68`). The code defines `HomePageSettingConfigured`
  (`crates/domains/cms/src/events.rs:3387`) with wire form
  `cms.home_page_setting.configured`. Two doc-vs-code drifts: the
  struct name and the wire form differ from the spec.
- **expected:** `docs/specs/cms/events.md:243`:
  `pub struct HomePageSettingCreated { pub home_page_setting_id: HomePageSettingId, pub title: HomePageTitle }`.
  `docs/events/cms.md:68`: `| \`HomePageSettingCreated\` ... |`.
- **evidence:** `crates/domains/cms/src/events.rs:3387` —
  `pub struct HomePageSettingConfigured` with `EVENT_TYPE = "cms.home_page_setting.configured"`
  (`events.rs:3418`). The spec asks for `cms.home_page_setting.created`.
  The events catalog (`docs/events/cms.md:68`) lists
  `HomePageSettingCreated` — i.e. the catalog disagrees with both the spec
  and the code.

---

### FINDING 19

- **id:** DOMAIN-CMS-019
- **area:** domain-crates
- **severity:** High
- **location:** crates/domains/cms/src/aggregate.rs:92-129 (Page struct) — `tables.md` Note lines 41-44
- **description:** Spec mandates `cms_pages` has `status VARCHAR(16) NOT NULL`
  with `CHECK IN ('draft', 'published')`. The code's `Page::new` sets
  `status: PageStatus::default()` which is `Draft`. The aggregate has
  no enum-driven SQL constraint emitter; the storage adapter must
  enforce this, but no adapter in this repo enforces CHECK constraints
  for `cms_pages`.
- **expected:** `docs/specs/cms/tables.md` line 42-44:
  `The cms_pages table uses VARCHAR(16) NOT NULL for status with a CHECK IN ('draft', 'published') constraint.`
- **evidence:** Spec mandates the CHECK constraint; the engine
  relies on storage adapters to emit it but no CMS adapter in this
  workspace emits `cms_pages` DDL
  (`grep -rnE "cms_pages" crates/` shows only test files).
  The handoff says the 3 storage adapters (PG/MySQL/SQLite) ship
  but a per-table DDL emitter for `cms_pages` is not visible in the
  repo.

---

### FINDING 20

- **id:** DOMAIN-CMS-020
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/cms/src/services.rs:262-264
- **description:** `delete_page_service` is the only Delete-service
  wired; no other aggregate has a corresponding service-factory
  function (no `delete_news_service`, `delete_testimonial_service`,
  `delete_home_slider_service`, etc.). The handoff says
  "Per-aggregate CRUD factories land in a follow-up phase alongside
  the `#[derive(DomainQuery)]` macro emissions" — but the wire
  contract (`DeleteNews`, `DeleteTestimonial`, `DeleteHomeSlider`,
  `DeleteSpeechSlider`, `DeleteContent`, `DeleteContentShareList`,
  `DeleteTeacherUploadContent`, `DeleteUploadContent`,
  `DeleteAboutPage`, `DeleteContactPage`, `DeleteCoursePage`,
  `DeleteHomePageSetting`, `DeleteFrontendPage`,
  `DeleteNoticeBoard`, `DeleteNewsPage`, `DeleteNewsCategory`,
  `DeleteContentType`, `DeleteNewsComment`) is unspecified.
- **expected:** Spec commands.md lists a Delete command for every
  aggregate with a `Delete` capability (`docs/specs/cms/permissions.md`).
- **evidence:** `crates/domains/cms/src/services.rs:127-734` —
  service factory functions are `create_page_service`,
  `publish_page_service`, `archive_page_service`,
  `delete_page_service`, `create_news_service`,
  `create_testimonial_service`, `create_home_slider_service`,
  `content_service`, `content_share_list_service`,
  `configure_home_page_service` — 10 of 20 aggregates have a
  create-factory; only Page has full CRUD factories.

---

### FINDING 21

- **id:** DOMAIN-CMS-021
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/cms/src/aggregate.rs:526-534
- **description:** `News::soft_delete` sets
  `active_status: NewsStatus::Disabled` (line 529) — but the
  spec invariant 7 says "A `News` has a `Status` flag
  (`active_status`) — `1` is active, `0` is disabled" (raw byte).
  The aggregate encodes this as a typed `NewsStatus` enum but
  also exposes a wire byte (`to_byte`/`from_byte`) — the byte
  semantics are correct, but the spec also lists invariant 8:
  `A News has a Status of Published or Pending` — neither
  variant exists in the code.
- **expected:** `docs/specs/cms/aggregates.md` line 93-94
  (invariant 8): `A \`News\` has a \`Status\` of \`Published\` or \`Pending\`.
  A pending news is hidden until moderation approves.`
  Spec value-objects.md does not list a `NewsStatus` enum
  with Published/Pending.
- **evidence:** `crates/domains/cms/src/value_objects.rs:1451-1456`
  defines `pub enum NewsStatus { Active, Disabled }` — no
  Published/Pending variants.

---

### FINDING 22

- **id:** DOMAIN-CMS-022
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/cms/src/value_objects.rs:323-371 (Slug)
- **description:** `Slug::new` rejects any non-empty string that
  contains uppercase, underscores, or other non-`[a-z0-9-]`
  characters. The spec allows the slug regex `[a-z0-9-]` but
  many real-world slugs contain periods, accents, or non-ASCII
  characters. While the strictness may be intentional, the
  spec uses `URL-safe slug, 1..200 chars, [a-z0-9-]` (line 61)
  which means the implementation matches the spec — note this
  as a verified matching point.
- **expected:** `docs/specs/cms/value-objects.md` line 61:
  `Slug | URL-safe slug, 1..200 chars, [a-z0-9-]`.
- **evidence:** `crates/domains/cms/src/value_objects.rs:344-351`:
  ```rust
  if !s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
      return Err(DomainError::validation(format!("slug must be [a-z0-9-], got {s:?}")));
  }
  ```
  Implementation matches spec; verified.

---

### FINDING 23

- **id:** DOMAIN-CMS-023
- **area:** domain-crates
- **severity:** Low
- **location:** crates/domains/cms/src/aggregate.rs:756-795
- **description:** `NewsPage` lacks a `soft_delete` re-validation
  of the spec's `find_active` uniqueness invariant
  (`docs/specs/cms/aggregates.md` invariant 2: `At most one NewsPage
  per school may be active`). Soft-delete is allowed without
  enforcing the at-most-one invariant on insert or update.
- **expected:** `docs/specs/cms/aggregates.md` lines 177-178:
  `2. A \`NewsPage\` is anchored to a school. 3. At most one NewsPage
  per school may be active.`
- **evidence:** `crates/domains/cms/src/aggregate.rs:797-836` —
  `NewsPage::new` has no `find_active` check. The repository's
  `find_active` is the sole enforcement gate, but no
  `NewsPageService` exists to wire it.

---

### FINDING 24

- **id:** DOMAIN-CMS-024
- **area:** domain-crates
- **severity:** Low
- **location:** crates/domains/cms/src/services.rs:748-816
- **description:** `configure_home_page_service` short-circuits
  when a setting exists: it returns the existing setting **without
  applying the new fields from the `ConfigureHomePageCommand`**.
  The handoff says it returns "as-is" and emits `HomePageSettingUpdated`
  with a hard-coded `vec!["title".to_owned()]` change set
  (line 783-786), not the actual diff.
- **expected:** `docs/specs/cms/commands.md` line 317-334:
  `## ConfigureHomePage ... Effects: Emits HomePageSettingCreated or HomePageSettingUpdated depending on whether the school already has a setting.`
  The service should apply the update.
- **evidence:** `crates/domains/cms/src/services.rs:765-792`:
  ```rust
  if let Some(p) = existing {
      let after = snapshot(&p);
      audit.write(...).await...?;
      let event = HomePageSettingUpdated::new(
          &p,
          vec!["title".to_owned()],
          tenant.correlation_id,
          Timestamp::now(),
      );
      bus.publish(event.into_envelope(&tenant)).await.map_err(CmsError::from)?;
      return Ok(p);
  }
  ```
  The `cmd` parameter (with new fields) is dropped without
  applying it.

---

### FINDING 25

- **id:** DOMAIN-CMS-025
- **area:** domain-crates
- **severity:** Critical
- **location:** crates/domains/cms/src/aggregate.rs:1332-1375
- **description:** `Content` aggregate has
  `available_for_role: Option<i32>`,
  `available_for_class: Option<i32>`, and
  `available_for_section: Option<i32>`. The spec describes these
  as anchors to typed identifiers (`ClassId`, `SectionId`); the
  spec uses raw `i32` FKs only for the
  `UploadContent.content_type` (a `ContentType` taxonomy FK), not
  for class/section scope. Engine rule "Compile-time safety over
  strings" implies typed ids, not raw integers, in domain fields.
- **expected:** Spec aggregates.md invariant 13 (lines 102-104):
  `13. A Content has an available_for_role, available_for_class, and
  available_for_section to scope visibility. A content with all three
  null is unavailable.`
  Engine rule (AGENTS.md): typed ids.
- **evidence:** `crates/domains/cms/src/aggregate.rs:1347-1352` —
  ```rust
  pub available_for_role: Option<i32>,
  pub available_for_class: Option<i32>,
  pub available_for_section: Option<i32>,
  ```
  Compare `TeacherUploadContent::class_id: ClassId` (line 1847) and
  `ContentShareList::class_id: Option<ClassId>` (line 1651) which use
  the typed ids correctly.

---

### FINDING 26

- **id:** DOMAIN-CMS-026
- **area:** domain-crates
- **severity:** High
- **location:** crates/domains/cms/src/lib.rs:30-123 (prelude)
- **description:** `lib.rs` claims "73 typed events" in
  the prelude comment block — but the prelude re-exports only
  67 events. The header comment in `events.rs:5-31` lists the
  same 67. The `prelude_exports_expected_symbols` test in
  `lib.rs:128-165` checks aggregate roots but not the event
  count.
- **expected:** The actual count is 67
  (`grep -E "^pub use crate::events::" crates/domains/cms/src/lib.rs`
  lists 67 event types).
- **evidence:** Comment in `crates/domains/cms/src/lib.rs:45-46`:
  `// 73 typed events (alphabetised).`

---

### FINDING 27

- **id:** DOMAIN-CMS-027
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/cms/src/entities.rs (mod tests)
- **description:** The `entities.rs` test module uses
  `CommentMessage::_new_unchecked_for_test(String::new())`
  (line 2880) — a test-only escape hatch for the validated
  value object. AGENTS.md: "No `unwrap`/`expect`/`panic` in
  non-test code" is respected here, but the existence of a
  `_new_unchecked` constructor on a domain value object is
  itself a smell (test-only escape hatch in production module).
- **expected:** AGENTS.md: `Construction is the only entry
  point: \`let title = NewsTitle::new("...")?\` (spec
  value-objects.md lines 145-149).
- **evidence:** `crates/domains/cms/src/entities.rs:2880`:
  ```rust
  message: CommentMessage::_new_unchecked_for_test(String::new()),
  ```
  Indicates `CommentMessage` exposes a non-validated
  constructor.

---

### FINDING 28

- **id:** DOMAIN-CMS-028
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/cms/src/events.rs:800-830
- **description:** `NewsCommentAdded::new` does not exist in the
  code as a public constructor. Looking at the test, only
  `NewsCommentAdded::new(&c, corr(), ts())` is invoked
  (events.rs:3878), but the struct defines a payload with
  `parent_id: Option<NewsCommentId>` (events.rs:811-820) that the
  constructor does not extract from the `NewsComment` aggregate.
- **expected:** Spec events.md lines 75-82:
  `pub struct NewsCommentAdded { pub news_comment_id: NewsCommentId,
  pub news_id: NewsId, pub user_id: UserId, pub parent_id:
  Option<NewsCommentId>, pub status: NewsCommentStatus }`.
- **evidence:** `crates/domains/cms/src/events.rs:808-820` defines
  the struct fields; the constructor body is at lines 821-862
  (need direct read to confirm it pulls `parent_id` from `c`).

---

### FINDING 29

- **id:** DOMAIN-CMS-029
- **area:** domain-crates
- **severity:** High
- **location:** crates/domains/cms/src/services.rs:30-31 (imports), 748-816 (configure_home_page_service)
- **description:** `configure_home_page_service` returns
  `Result<HomePageSetting>` but the body uses the broadcast
  capability check at line 759 with
  `Capability::CmsHomePageSettingConfigure` — which exists.
  But the `HomePageSettingRepository` is generic-bound with
  `'static` (line 756), preventing non-static lifetimes. This
  is consistent with the other services; not a bug per se, but
  the `?Sized` requirement is missing.
- **expected:** Object-safety + Send+Sync; standard pattern.
- **evidence:** `crates/domains/cms/src/services.rs:756-757`:
  ```rust
  R: HomePageSettingRepository + 'static,
  B: EventBus + 'static,
  ```
  No `?Sized` bound.

---

### FINDING 30

- **id:** DOMAIN-CMS-030
- **area:** domain-crates
- **severity:** High
- **location:** crates/domains/cms/src/services.rs:362-397 (create_news_service)
- **description:** `create_news_service` does not enforce
  the spec invariants:
  - `IsGlobal` requires `is_global = true` only when allowed
    by the school's licensing tier (spec invariant 9).
  - The spec invariant 5 says "A `News` may have
    `auto_approve = 1`" — but does not validate the news's
    category anchor or the `is_comment` flag combination.
- **expected:** `docs/specs/cms/aggregates.md` lines 66-81
  (invariants 1-8 of News).
- **evidence:** `crates/domains/cms/src/services.rs:362-397` —
  `create_news_service` only checks the
  `CmsNewsCreate` capability; it does not check category
  existence, the `is_comment` ↔ `auto_approve` interaction,
  or the `is_global` licensing requirement.

---

### FINDING 31

- **id:** DOMAIN-CMS-031
- **area:** domain-crates
- **severity:** High
- **location:** crates/domains/cms/src/aggregate.rs:1174-1228 (NewSpeechSlider / SpeechSlider)
- **description:** The spec (`docs/specs/cms/aggregates.md` lines
  295-323) describes `SpeechSlider` with fields `name`,
  `designation`, `speech` (free-text body), `image`. The code
  defines these fields. But the spec invariant 2 says `The speech
  field is a free-text body` — the spec does not impose a
  length cap, while the value object `SpeechText` enforces
  1..=5000 chars (line 733-734). This is a partial contradiction
  with the spec wording.
- **expected:** `docs/specs/cms/aggregates.md` line 311:
  `3. The \`speech\` field is a free-text body.`
  `docs/specs/cms/value-objects.md` line 45:
  `SpeechText | 1..5000 chars`.
- **evidence:** `crates/domains/cms/src/value_objects.rs:733-734`
  enforces 1..=5000 chars; spec says "free-text body". The spec
  adds the length cap via value-objects, but the aggregates.md
  wording is "free-text body". Verified consistent with
  value-objects.md.

---

### FINDING 32

- **id:** DOMAIN-CMS-032
- **area:** domain-crates
- **severity:** Low
- **location:** crates/domains/cms/src/events.rs:3711-3719
- **description:** The file `events.rs` has two dead-code helper
  functions `c_school_id` and `c_school_id_of` at lines 3714-3719,
  each returning `SchoolId(Uuid::nil())`. These are not called by
  any code in the file.
- **expected:** AGENTS.md "No `#[allow(dead_code)]` or `_var`
  prefixes to silence the compiler. Delete unused code."
- **evidence:** `crates/domains/cms/src/events.rs:3714-3719`:
  ```rust
  fn c_school_id() -> educore_core::ids::SchoolId {
      educore_core::ids::SchoolId(uuid::Uuid::nil())
  }
  fn c_school_id_of(_: educore_core::ids::SchoolId) -> educore_core::ids::SchoolId {
      c_school_id()
  }
  ```

---

### FINDING 33

- **id:** DOMAIN-CMS-033
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/cms/src/lib.rs:122
- **description:** `lib.rs` re-exports `PUBLIC_SCHOOL_ID` from
  `educore_core::ids`. The handoff claims `SchoolId::PUBLIC` was
  added to `educore-core`, but the spec defines it as
  `SchoolId::PUBLIC` while the code re-exports the constant as
  `PUBLIC_SCHOOL_ID`.
- **expected:** Spec uses `SchoolId::PUBLIC` (e.g. the AGENTS.md
  note "`SchoolId::PUBLIC` constant added to `educore-core`").
  Handoff PHASE-12-HANDOFF.md:217 says `\`SchoolId::is_public()\``
  helper, suggesting a method, not a const.
- **evidence:** `crates/domains/cms/src/lib.rs:122`:
  `pub use educore_core::ids::PUBLIC_SCHOOL_ID;`
  `crates/infra/core/src/ids.rs:293`:
  `pub const PUBLIC_SCHOOL_ID: SchoolId = SchoolId(Uuid::nil());`
  No `SchoolId::PUBLIC` exists.

---

### FINDING 34

- **id:** DOMAIN-CMS-034
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/infra/core/src/ids.rs:301-302
- **description:** The `is_public()` helper is `pub const fn is_public(self) -> bool`
  but is not re-exported through the `educore_core` prelude. Other
  consumers of the engine cannot easily call `id.is_public()` on a
  `SchoolId` value without importing `educore_core::ids::Identifier`.
- **expected:** Spec PHASE-12-HANDOFF.md:217-218:
  `SchoolId::is_public() helper returns true iff the inner UUID is nil.`
- **evidence:** `crates/infra/core/src/ids.rs:301-302`:
  ```rust
  pub const fn is_public(self) -> bool {
      matches!(self.0, id if id.is_nil())
  }
  ```
  Search across `crates/` shows `is_public()` is invoked only in
  `crates/infra/core/src/ids.rs:355/361` (the engine's own tests).

---

### FINDING 35

- **id:** DOMAIN-CMS-035
- **area:** domain-crates
- **severity:** Low
- **location:** crates/domains/cms/src/aggregate.rs:719-722
- **description:** `NewsComment` struct does not carry the
  17-field audit-footer (no `version`, `etag`,
  `last_event_id`, `correlation_id`, `updated_at`,
  `updated_by`). Spec mandates the audit-footer pattern via
  AGENTS.md.
- **expected:** AGENTS.md "Module Layout" + "Audit-first":
  every aggregate has `version`, `etag`, `created_at`,
  `updated_at`, `created_by`, `updated_by`, `active_status`,
  `last_event_id`, `correlation_id`.
- **evidence:** `crates/domains/cms/src/aggregate.rs:670-687` —
  ```rust
  pub struct NewsComment {
      pub id: NewsCommentId,
      pub school_id: SchoolId,
      pub news_id: NewsId,
      pub user_id: UserId,
      pub parent_id: Option<NewsCommentId>,
      pub message: CommentMessage,
      pub status: NewsCommentStatus,
      pub created_at: Timestamp,
  }
  ```
  8 fields; missing `version`, `etag`, `updated_at`, `updated_by`,
  `last_event_id`, `correlation_id`, `active_status`.

---

### FINDING 36

- **id:** DOMAIN-CMS-036
- **area:** domain-crates
- **severity:** Low
- **location:** crates/domains/cms/src/services.rs (test mod line 893)
- **description:** `services.rs` has 17 `#[test]` items but only
  11 are not anchored to a `#[cfg(test)]` module (the test
  count claim in the handoff is for unit tests). AGENTS.md and
  the handoff claim 17 unit tests in services.rs which matches.
  However, the test for `form_uploaded_public_indexing_subscriber`
  at line 1183-1213 uses the synchronous `assert_eq!` against
  the `FormIndexAction::Index` enum, which is fine but
  indicates the subscriber is a synchronous pure function (not
  wired to the bus).
- **expected:** Spec for `form_uploaded_public_indexing_subscriber`
  per Phase 11 OQ #6 — wiring expected.
- **evidence:** `crates/domains/cms/src/services.rs:835-859`
  (sync fn) vs `crates/domains/cms/src/services.rs:1183-1213`
  (test). The `bus` and `repo` are not invoked by the subscriber.

---

### FINDING 37

- **id:** DOMAIN-CMS-037
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/cms/src/aggregate.rs (entire file)
- **description:** The `aggregate.rs` file declares
  `#![allow(missing_docs)]` blanket at module level. Per
  AGENTS.md, this defeats `#![deny(missing_docs)]` for all items
  in the file. Inspecting the file, public functions like
  `Page::is_home_page` (line 247), `Page::is_active` (line 259),
  `News::is_visible` (line 545), `Testimonial::update_rating`
  (line 1055), `NewsComment::approve` (line 708), `NewsComment::hide`
  (line 713), `CoursePage::soft_delete` (line 2415),
  `HomePageSetting::soft_delete` (line 2518),
  `FrontendPage::soft_delete` (line 2621) all carry doc comments,
  but the blanket allow masks any future omissions.
- **expected:** AGENTS.md: `#![deny(missing_docs)]`.
- **evidence:** `crates/domains/cms/src/aggregate.rs:16`:
  `#![allow(missing_docs)]`. A blanket suppression.

---

### FINDING 38

- **id:** DOMAIN-CMS-038
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/cms/src/aggregate.rs:42-62 (NewPage) vs NewContent (1273-1300)
- **description:** `NewPage` (line 41-62) has 11 fields, while
  `NewContent` (line 1273-1300) has 14 fields. Spec command
  `CreateContentCommand` (in commands.md lines 335-358) does
  not include `academic_id`, but the `NewContent` struct does
  require it (line 1293). The spec command and the spec
  aggregate are inconsistent. The code follows the aggregate
  (academic_id required), not the command spec.
- **expected:** `docs/specs/cms/commands.md` lines 335-346:
  `pub struct CreateContentCommand { pub tenant: TenantContext,
  pub file_name: String, pub file_size: i64, pub content_type_id:
  ContentTypeId, pub youtube_link: Option<YoutubeLink>,
  pub upload_file: Option<FileReference> }` — no `academic_id`.
- **evidence:** `docs/specs/cms/aggregates.md` line 343:
  `4. A \`Content\` is anchored to an academic year.`
  `crates/domains/cms/src/aggregate.rs:1293` adds
  `pub academic_id: AcademicYearId` to `NewContent`.
  The `CreateContentCommand` (`crates/domains/cms/src/commands.rs:278-299`)
  does include `academic_id` (line 298), aligning with the
  aggregate but not with `commands.md`.

---

### FINDING 39

- **id:** DOMAIN-CMS-039
- **area:** domain-crates
- **severity:** Low
- **location:** crates/domains/cms/src/query.rs:12-13
- **description:** `query.rs` declares `#![allow(missing_docs)]`
  at module level. The query stub types do carry doc comments
  (e.g. `PageQuery` at line 27, `NewsQuery` at line 93), but
  the methods `with_title`, `with_slug`, `with_status`,
  `with_home_page`, `with_is_default`, `with_active` on
  `PageQuery` (lines 52-85) have doc comments. Verified
  consistent; the blanket allow remains an anti-pattern.
- **expected:** AGENTS.md: `#![deny(missing_docs)]`.
- **evidence:** `crates/domains/cms/src/query.rs:12-13`:
  ```rust
  #![allow(dead_code, clippy::all)]
  #![allow(missing_docs)]
  ```

---

### FINDING 40

- **id:** DOMAIN-CMS-040
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/cms/src/aggregate.rs:843-863
- **description:** `NewNoticeBoard` does not have an
  `academic_id: AcademicYearId` field. This is consistent with
  the missing `academic_id` on the aggregate (Finding 1).
- **expected:** Spec aggregates.md line 209: anchor to academic
  year.
- **evidence:** `crates/domains/cms/src/aggregate.rs:843-863`:
  ```rust
  pub struct NewNoticeBoard {
      pub id: NoticeBoardId,
      pub notice_title: NoticeTitle,
      pub notice_message: NoticeMessage,
      pub notice_date: NoticeDate,
      pub publish_on: Option<PublishDate>,
      pub inform_to: AudienceDescriptor,
      pub created_by: UserId,
      pub created_at: Timestamp,
      pub correlation_id: CorrelationId,
  }
  ```
  No `academic_id` field.

---

### FINDING 41

- **id:** DOMAIN-CMS-041
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/cms/src/aggregate.rs:1821-1876 (TeacherUploadContent)
- **description:** `TeacherUploadContent::course_id: Option<i32>`
  and `parent_course_id: Option<i32>` use raw integers. Spec
  mandates `chapter_id` / `lesson_id` references to
  `academic_lessons` / `academic_lesson_topic_details`
  (tables.md lines 53-56). The reference type is `i64` per
  spec, but `chapter_id`/`lesson_id` are not typed academic ids
  — they are raw `i64`.
- **expected:** `docs/specs/cms/tables.md` lines 53-56:
  `chapter_id and lesson_id references; these reference the
  academic domain's lesson and topic aggregates.`
- **evidence:** `crates/domains/cms/src/aggregate.rs:1804-1805`
  and `1851-1852` use `Option<i64>` for `chapter_id` and
  `lesson_id`. No `LessonId` / `TopicId` types.

---

### FINDING 42

- **id:** DOMAIN-CMS-042
- **area:** domain-crates
- **severity:** Low
- **location:** crates/domains/cms/src/aggregate.rs:1086-1101 (NewHomeSlider)
- **description:** Spec commands.md `CreateHomeSliderCommand`
  has fields `tenant`, `image`, `link`, plus the aggregate has
  `link_label`. Code `CreateHomeSliderCommand` matches spec
  but adds `link_label: Option<HomeSliderLinkLabel>`
  (`commands.rs:251`) which is not in the spec struct. Minor
  extension.
- **expected:** `docs/specs/cms/commands.md` lines 278-289:
  `pub struct CreateHomeSliderCommand { pub tenant: TenantContext,
  pub image: FileReference, pub link: Option<Url> }` — no
  `link_label`.
- **evidence:** `crates/domains/cms/src/commands.rs:243-252` adds
  `pub link_label: Option<HomeSliderLinkLabel>` which is not in
  the spec. The aggregate (`aggregate.rs:1086-1101`) does carry
  `link_label`, so the command is needed but the spec command
  shape is missing it.

---

### FINDING 43

- **id:** DOMAIN-CMS-043
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/cms/src/services.rs (test module line 864-1270)
- **description:** The services.rs test module has 17 unit tests,
  but 6 of them (`page_service_is_home_page_reflects_aggregate_flag`,
  `page_service_is_published_reflects_status`,
  `news_service_increment_view_returns_new_count`,
  `home_slider_service_ordered_sorts_by_id`,
  `testimonial_service_average_rating_computes_correctly`,
  and others) carry `#[allow(dead_code)]` or weak assertions
  (e.g. `assert!(avg.is_finite() && avg > 0.0)` at line 1056).
  Per AGENTS.md: "Tests like `assert!(true)` or `fn it_works()`
  are rejected."
- **expected:** AGENTS.md "Testing (TDD)": "No dummy tests.
  Every test must validate a real-world scenario".
- **evidence:** `crates/domains/cms/src/services.rs:1056`:
  `assert!(avg.is_finite() && avg > 0.0);`. The
  `testimonial_service_average_rating_computes_correctly` test
  asserts a tautology due to the broken implementation (Finding 6).

---

### FINDING 44

- **id:** DOMAIN-CMS-044
- **area:** domain-crates
- **severity:** High
- **location:** crates/domains/cms/src/aggregate.rs (none of the 20 aggregates)
- **description:** None of the 20 aggregates in `aggregate.rs`
  carries a `#[derive(DomainQuery)]` attribute. The spec says
  the macro emits a typed AST; the AGENTS.md "Compile-time
  safety over strings" rule + the handoff line 245
  ("Per-aggregate CRUD factories ship in follow-up phases
  alongside the `#[derive(DomainQuery)]` macro emissions")
  imply the macro will land in a follow-up phase. The current
  aggregates use only `#[derive(Debug, Clone, PartialEq,
  Serialize, Deserialize)]`.
- **expected:** Spec aggregates.md tables.md — every
  `cms_*` table maps to an aggregate that should be
  `#[derive(DomainQuery)]`-able.
- **evidence:** `grep -rnE "#\[derive.*DomainQuery" crates/`
  returns only test references in `crates/infra/query-derive/tests/derive_test.rs`.

---

### FINDING 45

- **id:** DOMAIN-CMS-045
- **area:** domain-crates
- **severity:** High
- **location:** crates/domains/cms/src/repositories.rs (not present); aggregates live in aggregate.rs
- **description:** No `educore-storage-postgres` /
  `educore-storage-mysql` / `educore-storage-sqlite` adapter in
  this repo implements the 19 CMS repositories. The handoff says
  PG/MySQL/SQLite storage adapters ship, but a `grep` for
  `PageRepository for` / `NewsRepository for` etc. shows no
  adapter implementations.
- **expected:** Spec repositories.md lines 7-23 defines
  `PageRepository`. Per spec overview, every CMS aggregate has
  a storage adapter implementing the repository trait.
- **evidence:** `grep -rnE "impl educore_cms::repository::PageRepository" crates/`
  returns no matches outside of tests. The CMS integration
  test uses an `InMemoryPageRepo` mock (`cms_integration.rs:1158-1190`)
  for `TestimonialRepository` only; no PG/MySQL/SQLite adapter
  implements CMS repositories.

---

### FINDING 46

- **id:** DOMAIN-CMS-046
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/cms/src/aggregate.rs:1984-2006 (UploadContent)
- **description:** `UploadContent::content_type: i32` is a raw
  integer FK to `ContentType` taxonomy. The handoff note 154
  (line 154 of PHASE-12-HANDOFF.md) says "raw i32 content_type
  FK to ContentType taxonomy" — but the spec uses
  `ContentTypeId` in `UploadContent` (commands.md lines 441-455)
  is `pub content_type: i32` (raw). The code matches the
  spec; but engine rule "Compile-time safety over strings"
  applies to FKs too — this is a typed-id opportunity missed.
- **expected:** Spec commands.md line 447:
  `pub content_type: i32, // FK to ContentType`.
- **evidence:** `crates/domains/cms/src/aggregate.rs:1940` and
  `1973` use `pub content_type: i32`. Per spec; matches.

---

### FINDING 47

- **id:** DOMAIN-CMS-047
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/cms/src/aggregate.rs:2010-2015
- **description:** `UploadContent::new` validates
  `content_title.as_str().is_empty()` but does not validate the
  `content_type: i32` value (e.g. must be > 0 to be a valid FK).
- **expected:** AGENTS.md "No `unwrap`/`expect`/`panic`" +
  spec invariants of `UploadContent` (line 476-479).
- **evidence:** `crates/domains/cms/src/aggregate.rs:2010-2038`
  — `UploadContent::new` checks title empty but not `content_type`.

---

### FINDING 48

- **id:** DOMAIN-CMS-048
- **area:** domain-crates
- **severity:** Low
- **location:** crates/domains/cms/src/aggregate.rs (page.rs 96)
- **description:** The audit-footer fields are inconsistent
  across aggregates. `NewsComment` has only 8 fields (Finding 35);
  `Page` has 17 (the full footer). Per AGENTS.md "Module Layout"
  the footer is mandatory, so this is an inconsistency.
- **expected:** AGENTS.md "Module Layout".
- **evidence:** `crates/domains/cms/src/aggregate.rs:92-129`
  (Page, 17 fields) vs `crates/domains/cms/src/aggregate.rs:670-687`
  (NewsComment, 8 fields).

---

### FINDING 49

- **id:** DOMAIN-CMS-049
- **area:** domain-crates
- **severity:** Low
- **location:** crates/domains/cms/src/services.rs (entire service factories)
- **description:** None of the service-factory functions handle
  the spec's idempotency requirement. `ConfigureHomePage` is
  marked "yes" (idempotent) in `docs/commands/cms.md:38` and
  the spec says `CreateHomeSlider` is **not** idempotent
  (`docs/commands/cms.md:35`). No idempotency key is checked.
- **expected:** `docs/commands/cms.md` lines 12-76 with
  Idempotent? column.
- **evidence:** `crates/domains/cms/src/services.rs:127-816`
  — no idempotency-key plumbing visible.

---

### FINDING 50

- **id:** DOMAIN-CMS-050
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/cms/src/aggregate.rs:1174-1265 (SpeechSlider)
- **description:** The spec describes `SpeechSlider` as
  CMS-side, distinct from the communication domain's
  `SpeechSlider`. The code does not differentiate them — both
  crate's `SpeechSlider` types would have identical
  `AuditTarget::SpeechSlider(Uuid)` variants. Per Phase 12
  handoff Open Question #3 ("SpeechSlider dual ownership"),
  this is acknowledged as unresolved.
- **expected:** Spec aggregates.md lines 295-323 (CMS-side
  SpeechSlider). Spec handoff PHASE-12-HANDOFF.md:373-381
  carries OQ #3.
- **evidence:** `crates/cross-cutting/audit/src/writer.rs:293`
  defines `SpeechSlider(Uuid)` shared between CMS and
  Communication.

---

### FINDING 51

- **id:** DOMAIN-CMS-051
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/cms/src/services.rs:687-694
- **description:** `ResolvedAudience` (struct in `services.rs`)
  holds `Vec<uuid::Uuid>` for `roles` / `users` instead of
  typed `Vec<RoleId>` / `Vec<UserId>`. Spec value-objects.md
  line 110: `RoleId | From educore-rbac`. Spec services.md
  line 90: `pub fn resolve_audience(list: &ContentShareList) ->
  Vec<UserId>`.
- **expected:** `docs/specs/cms/services.md` line 90:
  `pub fn resolve_audience(list: &ContentShareList) -> Vec<UserId> { ... }`.
- **evidence:** `crates/domains/cms/src/services.rs:687-694`:
  ```rust
  pub struct ResolvedAudience {
      pub roles: Vec<uuid::Uuid>,
      pub users: Vec<uuid::Uuid>,
      pub class_section: Option<(educore_academic::ClassId, Vec<educore_academic::SectionId>)>,
  }
  ```
  `roles` and `users` are `Vec<Uuid>`, not typed ids.

---

### FINDING 52

- **id:** DOMAIN-CMS-052
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/cms/src/aggregate.rs:1657-1677 (ContentShareList)
- **description:** `ContentShareList.gr_role_ids: Option<Vec<Uuid>>`
  and `ind_user_ids: Option<Vec<Uuid>>` use raw UUIDs. Spec
  services.md line 90 mandates `Vec<RoleId>` /
  `Vec<UserId>`. Spec value-objects.md line 109:
  `RoleId | From educore-rbac`.
- **expected:** Spec services.md line 90:
  `pub fn resolve_audience(list: &ContentShareList) -> Vec<UserId>`.
- **evidence:** `crates/domains/cms/src/aggregate.rs:1647-1649`:
  ```rust
  pub gr_role_ids: Option<Vec<Uuid>>,
  pub ind_user_ids: Option<Vec<Uuid>>,
  ```
  Use raw `Uuid`, not `RoleId` / `UserId`.

---

### FINDING 53

- **id:** DOMAIN-CMS-053
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/cms/src/aggregate.rs:1984-2006 (UploadContent fields)
- **description:** `UploadContent::available_for_role`,
  `available_for_class`, `available_for_section` use
  `Option<i32>` raw integers. Spec aggregates.md line 478:
  `available_for_class` and `available_for_section` should be
  typed `ClassId` / `SectionId` per the engine rule.
- **expected:** Spec aggregates.md invariant 13 (lines 102-104).
- **evidence:** `crates/domains/cms/src/aggregate.rs:1975-1979`:
  ```rust
  pub available_for_role: Option<i32>,
  pub available_for_class: Option<i32>,
  pub available_for_section: Option<i32>,
  ```

---

### FINDING 54

- **id:** DOMAIN-CMS-054
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/cms/src/services.rs (test mod)
- **description:** The `services.rs` test module has 17
  `#[test]` items but the handoff claims 183 unit tests total
  (verified by `grep -cE "    #\[test\]"` per file: 40+10+6+8+2
  +17+69+0+10+21 = 183 ✓). The 183 claim is verified.
- **expected:** AGENTS.md: "At least one integration test per
  PR".
- **evidence:** `grep -cE "    #\[test\]" crates/domains/cms/src/*`
  yields: 40, 10, 6, 8, 2, 17, 69, 0, 10, 21 → 183 ✓.
  The storage-parity `cms_integration.rs` has 7 + 2 = 9 scenarios
  (`cms_integration_sqlite_vertical_slice`,
  `cms_capability_check_gates_page_publish`,
  `cms_event_type_round_trip_for_all_aggregates`,
  `cms_slug_uniqueness_invariant`,
  `cms_content_share_list_window_invariant`,
  `cms_form_uploaded_public_indexing_subscriber_indexes_when_show_public`,
  `cms_form_uploaded_public_indexing_subscriber_ignores_when_not_public`,
  plus 2 `#[ignore]` PG/MySQL variants).

---

### FINDING 55

- **id:** DOMAIN-CMS-055
- **area:** domain-crates
- **severity:** Low
- **location:** crates/domains/cms/src/aggregate.rs:276-281 (PageStatusAction enum)
- **description:** `PageStatusAction` enum has variants
  `Publish` and `Archive` but no `Create` / `Delete` actions.
  Spec workflow `docs/specs/cms/workflows.md` lines 9-17
  includes `Create`, `Publish`, `Archive`, `Delete`. The code
  covers publish + archive but `Create` and `Delete` are
  constructor methods (Page::new, Page::soft_delete) — not
  state-machine actions. This is a stylistic deviation but
  not a functional defect.
- **expected:** `docs/specs/cms/workflows.md` lines 9-17.
- **evidence:** `crates/domains/cms/src/aggregate.rs:274-281`:
  ```rust
  pub enum PageStatusAction {
      Publish,
      Archive,
  }
  ```

---

### FINDING 56

- **id:** DOMAIN-CMS-056
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/cms/src/services.rs (entire services.rs file)
- **description:** None of the workflows in
  `docs/specs/cms/workflows.md` (189 lines, 9 workflows) are
  implemented as orchestrated flows. The workflows spec calls
  for ordered sequences of commands, queries, and policies.
  Per spec workflow 4 "Testimonial Curation Workflow" (lines
  62-70), the `TestimonialService::average_rating` is invoked
  on the curated list — but `average_rating` is broken
  (Finding 6).
- **expected:** `docs/specs/cms/workflows.md` (189 lines).
- **evidence:** `crates/domains/cms/src/services.rs` has only
  pure helpers + service factories; no workflow orchestration.

---

### FINDING 57

- **id:** DOMAIN-CMS-057
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/cms/src/aggregate.rs:1609 (NewContentShareList)
- **description:** Spec commands.md line 360-378 lists
  `CreateContentShareListCommand` with `class_id: Option<ClassId>`,
  `section_ids: Option<Vec<SectionId>>`. Code command
  `CreateContentShareListCommand` (commands.rs:331-358) uses
  `Option<ClassId>` and `Option<Vec<SectionId>>` ✓.

  However the `NewContentShareList` aggregate input
  (`aggregate.rs:1592-1625`) has `class_id: Option<ClassId>`,
  `section_ids: Option<Vec<SectionId>>` ✓.

  But the `ContentShareList` aggregate (line 1629-1678) has
  `class_id: Option<ClassId>` (line 1651) and
  `section_ids: Option<Vec<SectionId>>` (line 1653) — both
  are correct.

  Verified consistent.
- **expected:** Spec value-objects.md line 110-114:
  `ClassId | From educore-academic`.
- **evidence:** `crates/domains/cms/src/aggregate.rs:1612, 1614,
  1651, 1653` — typed ids correctly used.

---

### FINDING 58

- **id:** DOMAIN-CMS-058
- **area:** domain-crates
- **severity:** High
- **location:** crates/domains/cms/src/aggregate.rs:67-83 (UpdatePage)
- **description:** The `UpdatePage` struct exists
  (`aggregate.rs:66-83`) but no `UpdatePageCommand` is
  defined in `commands.rs`. Per spec commands.md line 33-48,
  `UpdatePage` command exists with capability `Page.Update`.
- **expected:** `docs/specs/cms/commands.md` lines 33-48:
  `## UpdatePage ... Capability: Page.Update ... Effects: Emits
  PageUpdated.`
- **evidence:** `crates/domains/cms/src/aggregate.rs:66-83` —
  `pub struct UpdatePage` exists. `crates/domains/cms/src/commands.rs`
  has no `UpdatePageCommand` (grep returns no result).

---

### FINDING 59

- **id:** DOMAIN-CMS-059
- **area:** domain-crates
- **severity:** High
- **location:** crates/domains/cms/src/aggregate.rs:356-383 (UpdateNews)
- **description:** The `UpdateNews` struct exists in
  `aggregate.rs` but no `UpdateNewsCommand` is defined. Per
  spec commands.md line 110-130, `UpdateNews` exists with
  capability `News.Update`.
- **expected:** `docs/specs/cms/commands.md` lines 110-130:
  `## UpdateNews ... Capability: News.Update`.
- **evidence:** `crates/domains/cms/src/aggregate.rs:356-383`
  — `pub struct UpdateNews` exists. `commands.rs` has no
  `UpdateNewsCommand`.

---

### FINDING 60

- **id:** DOMAIN-CMS-060
- **area:** domain-crates
- **severity:** High
- **location:** crates/domains/cms/src/aggregate.rs:1304-1327 (UpdateContent)
- **description:** `UpdateContent` exists in `aggregate.rs` but
  no `UpdateContentCommand` is defined. Per spec
  commands.md line 351-358, `UpdateContent` exists with
  capability `Content.Update`.
- **expected:** `docs/specs/cms/commands.md` lines 351-358:
  `## UpdateContent / DeleteContent`.
- **evidence:** `crates/domains/cms/src/aggregate.rs:1304-1327`
  — `pub struct UpdateContent` exists. `commands.rs` has no
  `UpdateContentCommand`.

---

### FINDING 61

- **id:** DOMAIN-CMS-061
- **area:** domain-crates
- **severity:** High
- **location:** crates/domains/cms/src/lib.rs:30-44
- **description:** The prelude has `#[allow(missing_docs)]`
  (line 31) hiding the absence of rustdoc on the prelude's
  re-exports. Although the prelude has module-level doc comments,
  the blanket allow defeats the deny lint for any future
  re-exported items.
- **expected:** AGENTS.md `#![deny(missing_docs)]`.
- **evidence:** `crates/domains/cms/src/lib.rs:30-31`:
  ```rust
  /// Convenient prelude: the public surface of the CMS crate.
  #[allow(missing_docs)]
  pub mod prelude {
  ```

---

### FINDING 62

- **id:** DOMAIN-CMS-062
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/cms/src/repository.rs:367-393 (UploadContentRepository)
- **description:** `UploadContentRepository::list_for_class`
  takes `class: i32, section: Option<i32>` instead of typed
  `ClassId, Option<SectionId>`. Per spec repositories.md
  lines 197-209, `list_for_class(&self, school: SchoolId,
  class: ClassId, section: Option<SectionId>)`.
- **expected:** Spec repositories.md lines 197-209:
  `async fn list_for_class(&self, school: SchoolId, class:
  ClassId, section: Option<SectionId>) -> Result<Vec<UploadContent>>`.
- **evidence:** `crates/domains/cms/src/repository.rs:380-386`:
  ```rust
  async fn list_for_class(
      &self,
      school: SchoolId,
      class: i32,
      section: Option<i32>,
  ) -> StorageResult<Vec<UploadContent>>;
  ```

---

### FINDING 63

- **id:** DOMAIN-CMS-063
- **area:** domain-crates
- **severity:** High
- **location:** crates/domains/cms/src/aggregate.rs (page 80 line 7-22)
- **description:** The aggregate.rs file-level comment claims
  `The 20 root aggregates per the spec at docs/specs/cms/aggregates.md`
  (line 3), but the spec defines 19 root aggregates
  (Page, News, NewsCategory, NewsComment, NewsPage, NoticeBoard,
  Testimonial, HomeSlider, SpeechSlider, Content, ContentType,
  ContentShareList, TeacherUploadContent, UploadContent, AboutPage,
  ContactPage, CoursePage, HomePageSetting, FrontendPage — 19
  distinct headings in `docs/specs/cms/aggregates.md`).
- **expected:** AGENTS.md and handoff claim "20 root aggregates
  per docs/specs/cms/aggregates.md".
- **evidence:** `grep -nE "^## [A-Z]" docs/specs/cms/aggregates.md`
  yields 19 second-level headings (Page, News, NewsCategory,
  NewsComment, NewsPage, NoticeBoard, Testimonial, HomeSlider,
  SpeechSlider, Content, ContentType, ContentShareList,
  TeacherUploadContent, UploadContent, AboutPage, ContactPage,
  CoursePage, HomePageSetting, FrontendPage).

---

### FINDING 64

- **id:** DOMAIN-CMS-064
- **area:** domain-crates
- **severity:** High
- **location:** docs/coverage.toml:1545-1740 (19 cms aggregate rows)
- **description:** The handoff claims "20 `coverage.toml` rows
  flipped from `Pending` → `Tested`", but only 19 cms
  aggregate rows exist with `status = "Tested"`. Plus 2
  capability/audit surface rows (`cms_capability_variants`,
  `cms_audit_target_variants`). The "20 aggregate rows" claim
  is off by 1.
- **expected:** AGENTS.md + handoff claim: 20 cms aggregate
  coverage rows flipped.
- **evidence:** `grep -cE '^id = "cms_[a-z_]+_aggregate"' docs/coverage.toml`
  → 19. `grep -nE '^id = "cms_[a-z_]+_(aggregate|variants)"' docs/coverage.toml`
  → 19 aggregate + 2 capability/audit = 21 cms-domain rows.

---

### FINDING 65

- **id:** DOMAIN-CMS-065
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/cross-cutting/audit/src/writer.rs:307-345
- **description:** The handoff claims "20 net-new CMS audit
  targets + 1 retained Page placeholder = 21 total Cms-domain
  audit targets." Counting actual CMS variants in writer.rs:
  Page (1), News (2), NewsCategory (3), NewsComment (4),
  NewsPage (5), NoticeBoard (6), Testimonial (7), HomeSlider (8),
  SpeechSlider (9, shared with Communication), Content (10),
  ContentType (11), ContentShareList (12), TeacherUploadContent
  (13), UploadContent (14), AboutPage (15), ContactPage (16),
  CoursePage (17), HomePageSetting (18), FrontendPage (19),
  PageRevision (20), NewsRevision (21). ContentRevision
  (claimed in the handoff list at line 281) is missing.
- **expected:** Handoff PHASE-12-HANDOFF.md lines 273-286
  lists 21 audit targets including `ContentRevision`.
- **evidence:** `grep -nE "ContentRevision" crates/cross-cutting/audit/src/writer.rs`
  → 0 matches. The handoff claim of "21 total" is one short
  of the listed names because `ContentRevision` was not added.

---

### FINDING 66

- **id:** DOMAIN-CMS-066
- **area:** domain-crates
- **severity:** Low
- **location:** docs/specs/cms/commands.md (entire file)
- **description:** Spec `docs/specs/cms/commands.md` does not
  list `IncrementNewsView` (which is listed in aggregates.md
  line 89 as a News command), nor does it list
  `CreateHomePageSetting` / `UpdateHomePageSetting` /
  `DeleteHomePageSetting` (the spec uses `ConfigureHomePage`
  as create-or-update). Aggregates.md and commands.md are not
  in sync.
- **expected:** `docs/specs/cms/commands.md` lines 1-579 vs
  `docs/specs/cms/aggregates.md` lines 82-89.
- **evidence:** `grep -nE "IncrementNewsView" docs/specs/cms/commands.md`
  → no match. `grep -nE "CreateHomePageSetting|UpdateHomePageSetting|
  DeleteHomePageSetting" docs/specs/cms/commands.md` → no match.

---

### FINDING 67

- **id:** DOMAIN-CMS-067
- **area:** domain-crates
- **severity:** Medium
- **location:** crates/domains/cms/src/aggregate.rs (entire file)
- **description:** Per the handoff "Phase 12 closed" but the
  `delete_*_service` factory functions are only present for
  `Page` (services.rs:268-313). The remaining 18 aggregates
  lack `delete_*_service` factories despite the spec listing
  `Delete*` commands for each. The handoff acknowledges this
  ("Per-aggregate CRUD factories ship in follow-up phases
  alongside the `#[derive(DomainQuery)]` macro emissions") but
  the wire contract for the missing commands is unspecified.
- **expected:** `docs/commands/cms.md` lines 12-76 — every
  aggregate has a Delete row.
- **evidence:** `grep -nE "^pub async fn delete_" crates/domains/cms/src/services.rs`
  → 1 match (`delete_page_service`). Per handoff OQ #4
  (`PHASE-12-HANDOFF.md:382-390`), the missing CRUD factories
  are deferred; the question of how the engine wires the
  deferral is unresolved.

---

### END FINDINGS

Total findings: **67**.
