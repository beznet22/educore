# CMS Domain — Tables

The CMS domain is backed by the following tables. Each table maps
to one or more aggregates; the `aggregate` column tells you which
aggregate owns the row.

| Table                          | Aggregate            | Notes                                            |
| ------------------------------ | -------------------- | ------------------------------------------------ |
| `cms_pages`                    | Page                 | Editable page with slug and status               |
| `cms_frontend_pages`            | FrontendPage         | Generic front-end page record                    |
| `cms_news`                     | News                 | News entry with category and body                |
| `cms_news_categories`          | NewsCategory         | News category taxonomy                           |
| `cms_news_comments`            | NewsComment          | Comment on a news entry                          |
| `cms_news_pages`               | NewsPage             | Public news landing-page configuration           |
| `cms_notice_boards`            | NoticeBoard          | Public-site notice board                         |
| `cms_testimonials`             | Testimonial          | Testimonial with rating                          |
| `cms_home_sliders`             | HomeSlider           | Home-page slider image and link                  |
| `cms_speech_sliders`           | SpeechSlider         | Public-page leadership message                    |
| `cms_contents`                 | Content              | Uploaded content item                            |
| `cms_content_types`            | ContentType          | Content type taxonomy (scoped)                   |
| `cms_content_share_lists`      | ContentShareList     | Bulk-share list of content items                 |
| `cms_teacher_upload_contents`  | TeacherUploadContent | Teacher-uploaded content (per class-section)     |
| `cms_upload_contents`          | UploadContent        | Admin-uploaded content (per role/class)          |
| `cms_about_pages`              | AboutPage            | About-page configuration                         |
| `cms_contact_pages`            | ContactPage          | Contact-page configuration                       |
| `cms_course_pages`             | CoursePage           | Course landing page                              |
| `cms_home_page_settings`       | HomePageSetting      | Home-page setting                                |

## Notes

- Every school-scoped table includes `school_id` for multi-tenant
  isolation. The `school_id` is `CHAR(36) NOT NULL` (UUIDv7) for
  the active school.
- Every school-scoped table includes `academic_id` referencing
  `academic_academic_years`. The CMS domain uses `academic_id` to
  scope content and content-share lists. The `cms_pages` table
  does not include `academic_id`; the scope is per-school only.
- Every table includes `id`, `created_at`, `updated_at`,
  `created_by`, `updated_by`, `active_status`, `version`, `etag`,
  `last_event_id`, `correlation_id`, `source`. The seven engine
  invariants per `docs/schemas/database-schema.md` § 2, § 5, § 9.
- The `cms_pages` table uses `VARCHAR(16) NOT NULL` for `status`
  with a `CHECK IN ('draft', 'published')` constraint. Other tables
  use `TINYINT` / `BOOLEAN` flags.
- The `cms_news_comments` table's `school_id` is derived from the
  parent news entry. Consumers MUST filter on `school_id` for
  news comment queries.
- The `cms_news_pages`, `cms_about_pages`, `cms_contact_pages`,
  and `cms_home_page_settings` tables are unique per school (one
  row per school). The domain enforces "at most one active" through
  `find_active` queries.
- The `cms_teacher_upload_contents` table is the only CMS table
  with `chapter_id` and `lesson_id` references; these reference
  the academic domain's lesson and topic aggregates. The CMS does
  not enforce referential integrity on these columns; the academic
  domain's storage adapter does.
- File references (`file` column) and URLs (`link` column) are
  captured as plain strings at the persistence boundary. The
  domain enforces their shape through value objects at construction
  time.

## Cross-Domain Tables (Referenced)

| Table                          | Owning Domain | Notes                                  |
| ------------------------------ | ------------- | -------------------------------------- |
| `platform_schools`             | platform      | Tenant anchor (FK target)              |
| `academic_academic_years`      | academic      | Referenced by `academic_id` FKs        |
| `academic_lessons`             | academic      | Referenced by `cms_teacher_upload_contents` |
| `academic_lesson_topic_details`| academic      | Referenced by `cms_teacher_upload_contents` |
