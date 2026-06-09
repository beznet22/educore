# CMS Domain Overview

## Purpose

The CMS domain owns the school's public-facing content surface: the
website pages, news, content uploads (study material, assignments,
syllabi), testimonials, sliders, course pages, and the per-page
configurations (about, contact, news, home). It also owns the
`NoticeBoard` aggregate — the school-side notice board surfaced on
the public site, distinct from the `Notice` aggregate in the
communication domain which is the staff-and-guardian messaging
fabric.

The domain is intentionally **port-agnostic**. It models the records
and the lifecycle. Surface rendering to the public web is performed
by consumer adapters.

## Responsibilities

- Page publication: editable pages with title, slug, body, status
  (`draft` / `published`), and home-page flag.
- News publication: news entries with category, body, publish date,
  image, comment toggle, and global/auto-approve flags.
- News categories: a taxonomy of news categories per school.
- News comments: per-news threaded comments, with moderation.
- News page configuration: a CMS-style landing page for the news
  section.
- Notice board (school-side): the public-site notice board, with
  title, message, date, publish date, and audience.
- Testimonial curation: name, designation, institution, image,
  description, and star rating.
- Home slider: image and link for the home page slider.
- Speech slider: leadership message (also tracked by the
  communication domain; the CMS owns the public-page rendering
  reference).
- Content uploads: study material, assignments, syllabi, and
  other downloads, with type, role/class availability, and file.
- Content types: a taxonomy of content types per school.
- Content sharing: a bulk-share list of content items to roles,
  classes, sections, or individual users, with a validity window.
- Course pages: course landing pages with title, description, image,
  button text/URL, and parent/child relationship.
- About page, contact page, home page setting: per-page
  configuration records.
- Frontend page: a generic page record for static front-end pages.

## Boundaries

The CMS domain does **not** own:

- The `Notice` aggregate used for staff-and-guardian messaging. That
  is the communication domain. The CMS owns a separate `NoticeBoard`
  aggregate scoped to the public site.
- File bytes. The file storage port holds the bytes; the domain holds
  only `FileReference`s.
- The HTML / CSS / JS rendering of pages. The rendering is a port
  adapter (a static-site generator, a server-rendered template, or
  a headless CMS adapter).
- Authentication of public-site visitors. The CMS domain exposes
  read-only queries that the public-site adapter invokes.
- Bulk user management. The platform layer owns users.

The CMS domain **does** provide identifier types and value objects
that other domains depend on: `PageId`, `NewsId`, `NewsCategoryId`,
`NewsCommentId`, `NoticeBoardId` (public-site), `TestimonialId`,
`HomeSliderId`, `SpeechSliderId` (CMS-side), `ContentId`,
`ContentTypeId`, `ContentShareListId`, `CoursePageId`, `AboutPageId`,
`ContactPageId`, `HomePageSettingId`, `FrontendPageId`.

## Dependencies

- `smsengine-core` — error types, result, identifier trait.
- `smsengine-platform` — `SchoolId`, `UserId`, `TenantContext`.
- `smsengine-rbac` — capability checks.
- `smsengine-events` — domain event publishing.
- `smsengine-academic` — `ClassId`, `SectionId` for content
  availability scope (read-only references).

## Domain Invariants

1. Every `Page`, `News`, `Testimonial`, `Content`, `CoursePage`, and
   `NoticeBoard` is anchored to a school.
2. A `Page` has a unique `slug` within a school when the slug is set.
3. A `Page` has a `Status` of `draft` or `published`. A draft is
   invisible to the public site.
4. A `Page` may be flagged as `home_page`. At most one page per
   school may be the home page.
5. A `Page` may be flagged as `is_default`. Default pages are
   pre-installed templates.
6. A `News` is anchored to a `NewsCategory` and a school.
7. A `News` has a `Status` flag (`active_status`). A disabled news is
   hidden from the public site.
8. A `News` has a `Status` of `Published` or `Pending`. A pending
   news is hidden until moderation approves.
9. A `News` may be `is_global` (visible across all schools in a
   multi-tenant SaaS) or scoped to one school.
10. A `News` may have `auto_approve` comments enabled. When disabled,
    comments are queued for moderation.
11. A `NewsComment` is anchored to a `News` and a `UserId`. Comments
    are append-only and may be hidden by moderation.
12. A `Content` is anchored to a `ContentType` and a school.
13. A `Content` has an `available_for_role`, `available_for_class`,
    and `available_for_section` to scope visibility. A content with
    all three null is unavailable.
14. A `ContentShareList` is a bulk-share job with a frozen
    recipient set at dispatch time.
15. A `CoursePage` may have a `parent_id` to model a course hierarchy.
16. A `Testimonial` has a `star_rating` in `1..5`.
17. A `SpeechSlider` is anchored to a school and carries a name,
    designation, free-text speech, and image.

## Aggregate Roots

| Aggregate                | Root Type             | Purpose                                        |
| ------------------------ | --------------------- | ---------------------------------------------- |
| Page                     | `Page`                | Editable page with slug and status             |
| News                     | `News`                | News entry with category and body              |
| NewsCategory             | `NewsCategory`        | News category taxonomy                         |
| NewsComment              | `NewsComment`         | Comment on a news entry                        |
| NewsPage                 | `NewsPage`            | Public news landing-page configuration         |
| NoticeBoard              | `NoticeBoard`         | Public-site notice board (school-side)         |
| Testimonial              | `Testimonial`         | Testimonial with rating                        |
| HomeSlider               | `HomeSlider`          | Home-page slider image and link                |
| SpeechSlider             | `SpeechSlider`        | Public-page leadership message (CMS-side)      |
| Content                  | `Content`             | Uploaded content item                          |
| ContentType              | `ContentType`         | Content type taxonomy                          |
| ContentShareList         | `ContentShareList`    | Bulk-share list of content items               |
| TeacherUploadContent     | `TeacherUploadContent`| Teacher-uploaded content (per class-section)   |
| UploadContent            | `UploadContent`       | Admin-uploaded content (per role/class)        |
| AboutPage                | `AboutPage`           | About-page configuration                       |
| ContactPage              | `ContactPage`         | Contact-page configuration                     |
| CoursePage               | `CoursePage`          | Course landing page                            |
| HomePageSetting          | `HomePageSetting`     | Home-page setting                              |
| FrontendPage             | `FrontendPage`        | Generic front-end page record                  |

Each aggregate is documented in detail under
`docs/specs/cms/aggregates.md`.

## Cross-Domain Impact

When a `Page` is published, the CMS domain emits `PagePublished`. The
public-site port adapter re-renders the affected page.

When a `News` is published, the CMS domain emits `NewsPublished`. The
search index port may index the news.

When a `Content` is shared via a `ContentShareList`, the CMS domain
emits `ContentShared`. The communication domain may subscribe to
dispatch a notification to the share audience.

When a `Testimonial` is created, the CMS domain emits
`TestimonialCreated`. The public-site port adapter surfaces it on
the testimonial widget.

When a `HomeSlider` is updated, the CMS domain emits
`HomeSliderUpdated`. The public-site port adapter rotates the slider.

## Consumers

- Web admin UI (compose pages, news, content, configure home).
- Web public site (render pages, news, testimonials, slider).
- Mobile parent app (view news, download content).
- Mobile teacher app (upload content, view news).
- AI agent (publish pages, moderate news comments, share content,
  configure home page).

## Anti-Goals

- The CMS domain does not present data to humans. It exposes
  commands, events, and queries.
- The CMS domain does not implement a file storage backend. Files
  are held in the file storage port.
- The CMS domain does not implement an HTML/CSS renderer. Rendering
  is a port adapter.
- The CMS domain does not own the `Notice` aggregate used for
  staff-and-guardian messaging; that is the communication domain.
