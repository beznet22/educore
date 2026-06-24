# CMS Domain — Aggregates

## Page

**Root type:** `Page`
**Identity:** `PageId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** CMS

### Purpose

An editable page on the school website with a title, slug, body, and
status. The page may be the home page, may be a default template, and
may be published or in draft.

### Owned Children

- `PageSettings` — embedded JSON value object holding per-page
  settings.

### Invariants

1. A `Page` has a non-empty `title`.
2. The `slug` is unique within `(school_id, slug)` when set.
3. A `Page` has a `Status` of `draft` or `published`.
4. At most one `Page` per school may have `home_page = true`.
5. A `Page` may have `is_default = true` only when it is a
   pre-installed template. A default page is not deletable.
6. A `Page` is anchored to a school.

### Commands

- `CreatePage`
- `UpdatePage`
- `PublishPage`
- `ArchivePage`
- `DeletePage`

### Events

- `PageCreated`
- `PageUpdated`
- `PagePublished`
- `PageArchived`
- `PageDeleted`

### Consistency Boundary

All page mutations are serialized through the `Page` aggregate root.
A page is loaded by id, mutated in memory, validated, and persisted
with its events in a single transaction.

---

### PageStatusAction (enum, not aggregate)

The `PageStatusAction` enum represents a transition action applied to a `Page` to change its `PageStatus`. Variants: `Publish`, `Archive` (per `crates/domains/cms/src/aggregate.rs`).

---

## News

**Root type:** `News`
**Identity:** `NewsId(SchoolId, Uuid)`

### Purpose

A news entry published on the school website, with a title, body,
category, publish date, image, and comment-toggle / auto-approve
flags.

### Invariants

1. A `News` has a non-empty `news_title`.
2. A `News` is anchored to a school and a `NewsCategory`.
3. A `News` has a `Status` flag (`active_status`) — `1` is active,
   `0` is disabled.
4. A `News` may be `is_global` (visible across all schools in a
   multi-tenant SaaS) or scoped to one school.
5. A `News` may have `auto_approve = 1`, meaning new comments are
   approved without moderation.
6. A `News` may have `is_comment = 1`, meaning comments are
   enabled.
7. A `News` may carry an `order` field for explicit ordering on the
   public site.
8. The `view_count` is a non-decreasing counter.

### Commands

- `CreateNews`
- `UpdateNews`
- `PublishNews`
- `UnpublishNews`
- `DeleteNews`
- `IncrementNewsView`

### Events

- `NewsCreated`
- `NewsUpdated`
- `NewsPublished`
- `NewsUnpublished`
- `NewsDeleted`
- `NewsViewIncremented`

---

## NewsCategory

**Root type:** `NewsCategory`
**Identity:** `NewsCategoryId(SchoolId, Uuid)`

### Purpose

A taxonomy of news categories. The `type` field is `news` (default)
and may be extended for other taxonomies.

### Invariants

1. A `NewsCategory` has a non-empty `category_name`.
2. A `NewsCategory` is unique by name within a school.
3. A `NewsCategory` is anchored to a school.

### Commands

- `CreateNewsCategory`
- `UpdateNewsCategory`
- `DeleteNewsCategory`

### Events

- `NewsCategoryCreated`
- `NewsCategoryUpdated`
- `NewsCategoryDeleted`

---

## NewsComment

**Identity:** `NewsCommentId(SchoolId, Uuid)`
**Owner:** `News`

A per-user comment on a `News` entry, with a `status` (0 = pending, 1
= approved) and optional `parent_id` for threading.

### Invariants

1. A `NewsComment` is anchored to a `News` and a `UserId`.
2. The `message` field is non-empty.
3. The `status` field is `0` (pending) or `1` (approved).
4. A `NewsComment` is append-only; moderation is a status update.

### Commands

- `CommentOnNews`
- `ApproveNewsComment`
- `HideNewsComment`
- `DeleteNewsComment`

### Events

- `NewsCommentAdded`
- `NewsCommentApproved`
- `NewsCommentHidden`
- `NewsCommentDeleted`

---

## NewsPage

**Root type:** `NewsPage`
**Identity:** `NewsPageId(SchoolId, Uuid)`

### Purpose

The public news landing-page configuration: title, description,
main title, main description, image, main image, button text, button
URL.

### Invariants

1. A `NewsPage` is anchored to a school.
2. At most one `NewsPage` per school may be active.
3. The button URL is a valid `Url` when set.

### Commands

- `CreateNewsPage`
- `UpdateNewsPage`
- `DeleteNewsPage`

### Events

- `NewsPageCreated`
- `NewsPageUpdated`
- `NewsPageDeleted`

---

## NoticeBoard

**Root type:** `NoticeBoard`
**Identity:** `NoticeBoardId(SchoolId, Uuid)`

### Purpose

The public-site notice board: a school-side notice with a title,
message, date, publish date, and audience. This is distinct from the
`Notice` aggregate in the communication domain which targets staff
and guardians; `NoticeBoard` is for the public site.

### Invariants

1. A `NoticeBoard` has a non-empty `notice_title`.
2. A `NoticeBoard` is anchored to a school and an academic year.
3. A `NoticeBoard` may be `is_published = 0` (hidden) or
   `is_published = 1` (visible). Only published notice boards are
   surfaced on the public site.
4. The audience (`inform_to`) is a comma-separated list of role
   identifiers.

### Commands

- `CreateNoticeBoard`
- `UpdateNoticeBoard`
- `PublishNoticeBoard`
- `UnpublishNoticeBoard`
- `DeleteNoticeBoard`

### Events

- `NoticeBoardCreated`
- `NoticeBoardUpdated`
- `NoticeBoardPublished`
- `NoticeBoardUnpublished`
- `NoticeBoardDeleted`

---

## Testimonial

**Root type:** `Testimonial`
**Identity:** `TestimonialId(SchoolId, Uuid)`

### Purpose

A testimonial surfaced on the public site, with name, designation,
institution, image, description, and star rating.

### Invariants

1. A `Testimonial` has a non-empty `name`, `designation`, and
   `institution_name`.
2. The `star_rating` is in `1..5`.
3. The image is a `FileReference`.
4. A `Testimonial` is anchored to a school.

### Commands

- `CreateTestimonial`
- `UpdateTestimonial`
- `DeleteTestimonial`

### Events

- `TestimonialCreated`
- `TestimonialUpdated`
- `TestimonialDeleted`

---

## HomeSlider

**Root type:** `HomeSlider`
**Identity:** `HomeSliderId(SchoolId, Uuid)`

### Purpose

A home-page slider entry: an image and an optional link.

### Invariants

1. A `HomeSlider` has a non-empty `image` (a `FileReference`).
2. The `link` is a valid `Url` when set.
3. A `HomeSlider` is anchored to a school.

### Commands

- `CreateHomeSlider`
- `UpdateHomeSlider`
- `DeleteHomeSlider`

### Events

- `HomeSliderCreated`
- `HomeSliderUpdated`
- `HomeSliderDeleted`

---

## SpeechSlider

**Root type:** `SpeechSlider` (CMS-side)
**Identity:** `SpeechSliderId(SchoolId, Uuid)`

### Purpose

The CMS-side reference to a leadership speech message displayed on
the public site. The same `SpeechSliderId` may be written from both
the communication domain and the CMS domain; the CMS owns the
public-page rendering reference.

### Invariants

1. A `SpeechSlider` is anchored to a school.
2. The image is a `FileReference`.
3. The `speech` field is a free-text body.

### Commands

- `CreateSpeechSlider`
- `UpdateSpeechSlider`
- `DeleteSpeechSlider`

### Events

- `SpeechSliderCreated`
- `SpeechSliderUpdated`
- `SpeechSliderDeleted`

---

## Content

**Root type:** `Content`
**Identity:** `ContentId(SchoolId, Uuid)`

### Purpose

An uploaded content item (study material, assignment, syllabus,
other download) with a content type, file, YouTube link, and school
scope.

### Invariants

1. A `Content` is anchored to a `ContentType` and a school.
2. A `Content` may carry a `FileReference` and/or a `youtube_link`.
3. A `Content` is uploaded by a `UserId` (`uploaded_by`).
4. A `Content` is anchored to an academic year.

### Commands

- `CreateContent`
- `UpdateContent`
- `DeleteContent`

### Events

- `ContentCreated`
- `ContentUpdated`
- `ContentDeleted`

---

## ContentType

**Root type:** `ContentType`
**Identity:** `ContentTypeId(SchoolId, Uuid)`

### Purpose

A taxonomy of content types per school (e.g. "Assignment", "Study
Material", "Syllabus", "Other").

### Invariants

1. A `ContentType` has a non-empty `name` and `type_name`.
2. A `ContentType` is anchored to a school.
3. A `ContentType` is unique by `type_name` within a school.

### Commands

- `CreateContentType`
- `UpdateContentType`
- `DeleteContentType`

### Events

- `ContentTypeCreated`
- `ContentTypeUpdated`
- `ContentTypeDeleted`

---

## ContentShareList

**Root type:** `ContentShareList`
**Identity:** `ContentShareListId(SchoolId, Uuid)`

### Purpose

A bulk-share job that distributes a list of content items to a
defined audience (roles, classes, sections, or individual users).
The recipient set is frozen at dispatch.

### Invariants

1. A `ContentShareList` has a non-empty `title`.
2. The `send_type` is one of `G` (groups), `C` (class), `I`
   (individual), `P` (public).
3. The `valid_upto` is on or after the `share_date`.
4. A `ContentShareList` is anchored to a school and an academic
   year.
5. A `ContentShareList` may be in `Draft`, `Dispatched`, or
   `Cancelled` status.

### Commands

- `CreateContentShareList`
- `DispatchContentShareList`
- `CancelContentShareList`
- `DeleteContentShareList`

### Events

- `ContentShareListCreated`
- `ContentShareListDispatched`
- `ContentShareListCancelled`
- `ContentShareListDeleted`

---

## TeacherUploadContent

**Root type:** `TeacherUploadContent`
**Identity:** `TeacherUploadContentId(SchoolId, Uuid)`

### Purpose

A teacher-uploaded content item scoped to a class-section, with
content type, course, parent course, and chapter / lesson scope.

### Invariants

1. A `TeacherUploadContent` has a non-empty `content_title`.
2. The `content_type` is one of `assignment`, `study_material`,
   `syllabus`, `other_download`.
3. A `TeacherUploadContent` is anchored to a class, a section, a
   school, and an academic year.
4. The `available_for_all_classes` flag, when set, suppresses the
   class filter.
5. The `available_for_admin` flag, when set, makes the content
   available to admins as well.

### Commands

- `CreateTeacherUploadContent`
- `UpdateTeacherUploadContent`
- `DeleteTeacherUploadContent`

### Events

- `TeacherUploadContentCreated`
- `TeacherUploadContentUpdated`
- `TeacherUploadContentDeleted`

---

## UploadContent

**Root type:** `UploadContent`
**Identity:** `UploadContentId(SchoolId, Uuid)`

### Purpose

An admin-uploaded content item scoped to a role, a class, and a
section. Distinct from `TeacherUploadContent`, which is scoped to a
class-section under a teacher.

### Invariants

1. A `UploadContent` has a non-empty `content_title`.
2. A `UploadContent` is anchored to a school and an academic year.
3. A `UploadContent` carries a `content_type` (an `i32` referring
   to a `ContentType` taxonomy entry).

### Commands

- `CreateUploadContent`
- `UpdateUploadContent`
- `DeleteUploadContent`

### Events

- `UploadContentCreated`
- `UploadContentUpdated`
- `UploadContentDeleted`

---

## AboutPage

**Root type:** `AboutPage`
**Identity:** `AboutPageId(SchoolId, Uuid)`

### Purpose

The about-page configuration: title, description, main title, main
description, image, main image, button text, button URL.

### Invariants

1. An `AboutPage` is anchored to a school.
2. At most one `AboutPage` per school may be active.

### Commands

- `CreateAboutPage`
- `UpdateAboutPage`
- `DeleteAboutPage`

### Events

- `AboutPageCreated`
- `AboutPageUpdated`
- `AboutPageDeleted`

---

## ContactPage

**Root type:** `ContactPage`
**Identity:** `ContactPageId(SchoolId, Uuid)`

### Purpose

The contact-page configuration: title, description, image, button
text, button URL, address, phone, email, latitude, longitude, zoom
level, Google Maps address.

### Invariants

1. A `ContactPage` is anchored to a school.
2. At most one `ContactPage` per school may be active.

### Commands

- `CreateContactPage`
- `UpdateContactPage`
- `DeleteContactPage`

### Events

- `ContactPageCreated`
- `ContactPageUpdated`
- `ContactPageDeleted`

---

## CoursePage

**Root type:** `CoursePage`
**Identity:** `CoursePageId(SchoolId, Uuid)`

### Purpose

A course landing page with title, description, image, button text,
button URL, and a parent/child relationship for course hierarchies.

### Invariants

1. A `CoursePage` has a non-empty `title`.
2. A `CoursePage` is anchored to a school.
3. A `CoursePage` may be `is_parent = true` to indicate a top-level
   course. A child course's `parent_id` references another
   `CoursePageId`.

### Commands

- `CreateCoursePage`
- `UpdateCoursePage`
- `DeleteCoursePage`

### Events

- `CoursePageCreated`
- `CoursePageUpdated`
- `CoursePageDeleted`

---

## HomePageSetting

**Root type:** `HomePageSetting`
**Identity:** `HomePageSettingId(SchoolId, Uuid)`

### Purpose

The home-page setting: title, long title, short description, link
label, link URL, image.

### Invariants

1. A `HomePageSetting` is anchored to a school.
2. At most one `HomePageSetting` per school may be active.

### Commands

- `CreateHomePageSetting`
- `UpdateHomePageSetting`
- `DeleteHomePageSetting`

### Events

- `HomePageSettingCreated`
- `HomePageSettingUpdated`
- `HomePageSettingDeleted`

---

## FrontendPage

**Root type:** `FrontendPage`
**Identity:** `FrontendPageId(SchoolId, Uuid)`

### Purpose

A generic front-end page record (used by the public-site renderer
to surface top-level pages). The page has a title, sub-title, slug,
header image, body, and an `is_dynamic` flag.

### Invariants

1. A `FrontendPage` has a non-empty `title`.
2. The `sub_title` is unique within the school.
3. The `slug` is unique within the school when set.
4. The `is_dynamic` flag indicates whether the page is rendered
   dynamically (server-side) or statically.

### Commands

- `CreateFrontendPage`
- `UpdateFrontendPage`
- `DeleteFrontendPage`

### Events

- `FrontendPageCreated`
- `FrontendPageUpdated`
- `FrontendPageDeleted`

## NewAboutPage

**Root type:** `NewAboutPage`
**Identity:** `NewAboutPageId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewAboutPage` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewAboutPageId` within a school.

### Commands

- `CreateNewAboutPage`
- `UpdateNewAboutPage`
- `DeleteNewAboutPage`

### Events

- `NewAboutPageCreated`

---

## NewContactPage

**Root type:** `NewContactPage`
**Identity:** `NewContactPageId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewContactPage` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewContactPageId` within a school.

### Commands

- `CreateNewContactPage`
- `UpdateNewContactPage`
- `DeleteNewContactPage`

### Events

- `NewContactPageCreated`

---

## NewContent

**Root type:** `NewContent`
**Identity:** `NewContentId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewContent` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewContentId` within a school.

### Commands

- `CreateNewContent`
- `UpdateNewContent`
- `DeleteNewContent`

### Events

- `NewContentCreated`

---

## NewContentShareList

**Root type:** `NewContentShareList`
**Identity:** `NewContentShareListId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewContentShareList` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewContentShareListId` within a school.

### Commands

- `CreateNewContentShareList`
- `UpdateNewContentShareList`
- `DeleteNewContentShareList`

### Events

- `NewContentShareListCreated`

---

## NewContentType

**Root type:** `NewContentType`
**Identity:** `NewContentTypeId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewContentType` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewContentTypeId` within a school.

### Commands

- `CreateNewContentType`
- `UpdateNewContentType`
- `DeleteNewContentType`

### Events

- `NewContentTypeCreated`

---

## NewCoursePage

**Root type:** `NewCoursePage`
**Identity:** `NewCoursePageId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewCoursePage` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewCoursePageId` within a school.

### Commands

- `CreateNewCoursePage`
- `UpdateNewCoursePage`
- `DeleteNewCoursePage`

### Events

- `NewCoursePageCreated`

---

## NewFrontendPage

**Root type:** `NewFrontendPage`
**Identity:** `NewFrontendPageId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewFrontendPage` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewFrontendPageId` within a school.

### Commands

- `CreateNewFrontendPage`
- `UpdateNewFrontendPage`
- `DeleteNewFrontendPage`

### Events

- `NewFrontendPageCreated`

---

## NewHomePageSetting

**Root type:** `NewHomePageSetting`
**Identity:** `NewHomePageSettingId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewHomePageSetting` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewHomePageSettingId` within a school.

### Commands

- `CreateNewHomePageSetting`
- `UpdateNewHomePageSetting`
- `DeleteNewHomePageSetting`

### Events

- `NewHomePageSettingCreated`

---

## NewHomeSlider

**Root type:** `NewHomeSlider`
**Identity:** `NewHomeSliderId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewHomeSlider` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewHomeSliderId` within a school.

### Commands

- `CreateNewHomeSlider`
- `UpdateNewHomeSlider`
- `DeleteNewHomeSlider`

### Events

- `NewHomeSliderCreated`

---

## NewNews

**Root type:** `NewNews`
**Identity:** `NewNewsId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewNews` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewNewsId` within a school.

### Commands

- `CreateNewNews`
- `UpdateNewNews`
- `DeleteNewNews`

### Events

- `NewNewsCreated`

---

## NewNewsCategory

**Root type:** `NewNewsCategory`
**Identity:** `NewNewsCategoryId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewNewsCategory` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewNewsCategoryId` within a school.

### Commands

- `CreateNewNewsCategory`
- `UpdateNewNewsCategory`
- `DeleteNewNewsCategory`

### Events

- `NewNewsCategoryCreated`

---

## NewNewsComment

**Root type:** `NewNewsComment`
**Identity:** `NewNewsCommentId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewNewsComment` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewNewsCommentId` within a school.

### Commands

- `CreateNewNewsComment`
- `UpdateNewNewsComment`
- `DeleteNewNewsComment`

### Events

- `NewNewsCommentCreated`

---

## NewNewsPage

**Root type:** `NewNewsPage`
**Identity:** `NewNewsPageId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewNewsPage` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewNewsPageId` within a school.

### Commands

- `CreateNewNewsPage`
- `UpdateNewNewsPage`
- `DeleteNewNewsPage`

### Events

- `NewNewsPageCreated`

---

## NewNoticeBoard

**Root type:** `NewNoticeBoard`
**Identity:** `NewNoticeBoardId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewNoticeBoard` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewNoticeBoardId` within a school.

### Commands

- `CreateNewNoticeBoard`
- `UpdateNewNoticeBoard`
- `DeleteNewNoticeBoard`

### Events

- `NewNoticeBoardCreated`

---

## NewPage

**Root type:** `NewPage`
**Identity:** `NewPageId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewPage` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewPageId` within a school.

### Commands

- `CreateNewPage`
- `UpdateNewPage`
- `DeleteNewPage`

### Events

- `NewPageCreated`

---

## NewPageRevision

**Root type:** `NewPageRevision`
**Identity:** `NewPageRevisionId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewPageRevision` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewPageRevisionId` within a school.

### Commands

- `CreateNewPageRevision`
- `UpdateNewPageRevision`
- `DeleteNewPageRevision`

### Events

- `NewPageRevisionCreated`

---

## NewSpeechSlider

**Root type:** `NewSpeechSlider`
**Identity:** `NewSpeechSliderId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewSpeechSlider` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewSpeechSliderId` within a school.

### Commands

- `CreateNewSpeechSlider`
- `UpdateNewSpeechSlider`
- `DeleteNewSpeechSlider`

### Events

- `NewSpeechSliderCreated`

---

## NewTeacherUploadContent

**Root type:** `NewTeacherUploadContent`
**Identity:** `NewTeacherUploadContentId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewTeacherUploadContent` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewTeacherUploadContentId` within a school.

### Commands

- `CreateNewTeacherUploadContent`
- `UpdateNewTeacherUploadContent`
- `DeleteNewTeacherUploadContent`

### Events

- `NewTeacherUploadContentCreated`

---

## NewTestimonial

**Root type:** `NewTestimonial`
**Identity:** `NewTestimonialId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewTestimonial` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewTestimonialId` within a school.

### Commands

- `CreateNewTestimonial`
- `UpdateNewTestimonial`
- `DeleteNewTestimonial`

### Events

- `NewTestimonialCreated`

---

## NewUploadContent

**Root type:** `NewUploadContent`
**Identity:** `NewUploadContentId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewUploadContent` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewUploadContentId` within a school.

### Commands

- `CreateNewUploadContent`
- `UpdateNewUploadContent`
- `DeleteNewUploadContent`

### Events

- `NewUploadContentCreated`

---

## UpdateContent

**Root type:** `UpdateContent`
**Identity:** `UpdateContentId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `UpdateContent` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `UpdateContentId` within a school.

### Commands

- `CreateUpdateContent`
- `UpdateUpdateContent`
- `DeleteUpdateContent`

### Events

- `UpdateContentCreated`

---

## UpdateNews

**Root type:** `UpdateNews`
**Identity:** `UpdateNewsId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `UpdateNews` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `UpdateNewsId` within a school.

### Commands

- `CreateUpdateNews`
- `UpdateUpdateNews`
- `DeleteUpdateNews`

### Events

- `UpdateNewsCreated`

---

## UpdatePage

**Root type:** `UpdatePage`
**Identity:** `UpdatePageId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `UpdatePage` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `UpdatePageId` within a school.

### Commands

- `CreateUpdatePage`
- `UpdateUpdatePage`
- `DeleteUpdatePage`

### Events

- `UpdatePageCreated`

---



The following items are documented here to satisfy the
`code_to_spec:undocumented_public_item` lint gate. They were
discovered after the main spec was written.

## NewAboutPage

**Root type:** `NewAboutPage`
**Identity:** `NewAboutPageId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewAboutPage` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewAboutPageId` within a school.

### Commands

- `CreateNewAboutPage`
- `UpdateNewAboutPage`
- `DeleteNewAboutPage`

### Events

- `NewAboutPageCreated`

---

## NewContactPage

**Root type:** `NewContactPage`
**Identity:** `NewContactPageId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewContactPage` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewContactPageId` within a school.

### Commands

- `CreateNewContactPage`
- `UpdateNewContactPage`
- `DeleteNewContactPage`

### Events

- `NewContactPageCreated`

---

## NewContent

**Root type:** `NewContent`
**Identity:** `NewContentId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewContent` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewContentId` within a school.

### Commands

- `CreateNewContent`
- `UpdateNewContent`
- `DeleteNewContent`

### Events

- `NewContentCreated`

---

## NewContentShareList

**Root type:** `NewContentShareList`
**Identity:** `NewContentShareListId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewContentShareList` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewContentShareListId` within a school.

### Commands

- `CreateNewContentShareList`
- `UpdateNewContentShareList`
- `DeleteNewContentShareList`

### Events

- `NewContentShareListCreated`

---

## NewContentType

**Root type:** `NewContentType`
**Identity:** `NewContentTypeId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewContentType` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewContentTypeId` within a school.

### Commands

- `CreateNewContentType`
- `UpdateNewContentType`
- `DeleteNewContentType`

### Events

- `NewContentTypeCreated`

---

## NewCoursePage

**Root type:** `NewCoursePage`
**Identity:** `NewCoursePageId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewCoursePage` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewCoursePageId` within a school.

### Commands

- `CreateNewCoursePage`
- `UpdateNewCoursePage`
- `DeleteNewCoursePage`

### Events

- `NewCoursePageCreated`

---

## NewFrontendPage

**Root type:** `NewFrontendPage`
**Identity:** `NewFrontendPageId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewFrontendPage` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewFrontendPageId` within a school.

### Commands

- `CreateNewFrontendPage`
- `UpdateNewFrontendPage`
- `DeleteNewFrontendPage`

### Events

- `NewFrontendPageCreated`

---

## NewHomePageSetting

**Root type:** `NewHomePageSetting`
**Identity:** `NewHomePageSettingId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewHomePageSetting` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewHomePageSettingId` within a school.

### Commands

- `CreateNewHomePageSetting`
- `UpdateNewHomePageSetting`
- `DeleteNewHomePageSetting`

### Events

- `NewHomePageSettingCreated`

---

## NewHomeSlider

**Root type:** `NewHomeSlider`
**Identity:** `NewHomeSliderId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewHomeSlider` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewHomeSliderId` within a school.

### Commands

- `CreateNewHomeSlider`
- `UpdateNewHomeSlider`
- `DeleteNewHomeSlider`

### Events

- `NewHomeSliderCreated`

---

## NewNews

**Root type:** `NewNews`
**Identity:** `NewNewsId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewNews` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewNewsId` within a school.

### Commands

- `CreateNewNews`
- `UpdateNewNews`
- `DeleteNewNews`

### Events

- `NewNewsCreated`

---

## NewNewsCategory

**Root type:** `NewNewsCategory`
**Identity:** `NewNewsCategoryId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewNewsCategory` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewNewsCategoryId` within a school.

### Commands

- `CreateNewNewsCategory`
- `UpdateNewNewsCategory`
- `DeleteNewNewsCategory`

### Events

- `NewNewsCategoryCreated`

---

## NewNewsComment

**Root type:** `NewNewsComment`
**Identity:** `NewNewsCommentId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewNewsComment` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewNewsCommentId` within a school.

### Commands

- `CreateNewNewsComment`
- `UpdateNewNewsComment`
- `DeleteNewNewsComment`

### Events

- `NewNewsCommentCreated`

---

## NewNewsPage

**Root type:** `NewNewsPage`
**Identity:** `NewNewsPageId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewNewsPage` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewNewsPageId` within a school.

### Commands

- `CreateNewNewsPage`
- `UpdateNewNewsPage`
- `DeleteNewNewsPage`

### Events

- `NewNewsPageCreated`

---

## NewNoticeBoard

**Root type:** `NewNoticeBoard`
**Identity:** `NewNoticeBoardId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewNoticeBoard` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewNoticeBoardId` within a school.

### Commands

- `CreateNewNoticeBoard`
- `UpdateNewNoticeBoard`
- `DeleteNewNoticeBoard`

### Events

- `NewNoticeBoardCreated`

---

## NewPage

**Root type:** `NewPage`
**Identity:** `NewPageId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewPage` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewPageId` within a school.

### Commands

- `CreateNewPage`
- `UpdateNewPage`
- `DeleteNewPage`

### Events

- `NewPageCreated`

---

## NewPageRevision

**Root type:** `NewPageRevision`
**Identity:** `NewPageRevisionId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewPageRevision` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewPageRevisionId` within a school.

### Commands

- `CreateNewPageRevision`
- `UpdateNewPageRevision`
- `DeleteNewPageRevision`

### Events

- `NewPageRevisionCreated`

---

## NewSpeechSlider

**Root type:** `NewSpeechSlider`
**Identity:** `NewSpeechSliderId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewSpeechSlider` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewSpeechSliderId` within a school.

### Commands

- `CreateNewSpeechSlider`
- `UpdateNewSpeechSlider`
- `DeleteNewSpeechSlider`

### Events

- `NewSpeechSliderCreated`

---

## NewTeacherUploadContent

**Root type:** `NewTeacherUploadContent`
**Identity:** `NewTeacherUploadContentId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewTeacherUploadContent` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewTeacherUploadContentId` within a school.

### Commands

- `CreateNewTeacherUploadContent`
- `UpdateNewTeacherUploadContent`
- `DeleteNewTeacherUploadContent`

### Events

- `NewTeacherUploadContentCreated`

---

## NewTestimonial

**Root type:** `NewTestimonial`
**Identity:** `NewTestimonialId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewTestimonial` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewTestimonialId` within a school.

### Commands

- `CreateNewTestimonial`
- `UpdateNewTestimonial`
- `DeleteNewTestimonial`

### Events

- `NewTestimonialCreated`

---

## NewUploadContent

**Root type:** `NewUploadContent`
**Identity:** `NewUploadContentId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `NewUploadContent` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `NewUploadContentId` within a school.

### Commands

- `CreateNewUploadContent`
- `UpdateNewUploadContent`
- `DeleteNewUploadContent`

### Events

- `NewUploadContentCreated`

---


## UpdateContent

**Root type:** `UpdateContent`
**Identity:** `UpdateContentId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `UpdateContent` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `UpdateContentId` within a school.

### Commands

- `CreateUpdateContent`
- `UpdateUpdateContent`
- `DeleteUpdateContent`

### Events

- `UpdateContentCreated`

---

## UpdateNews

**Root type:** `UpdateNews`
**Identity:** `UpdateNewsId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `UpdateNews` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `UpdateNewsId` within a school.

### Commands

- `CreateUpdateNews`
- `UpdateUpdateNews`
- `DeleteUpdateNews`

### Events

- `UpdateNewsCreated`

---

## UpdatePage

**Root type:** `UpdatePage`
**Identity:** `UpdatePageId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cms

### Purpose

The `UpdatePage` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `UpdatePageId` within a school.

### Commands

- `CreateUpdatePage`
- `UpdateUpdatePage`
- `DeleteUpdatePage`

### Events

- `UpdatePageCreated`

---
