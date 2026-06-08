# CMS Domain — Tables

The CMS domain is backed by the following tables. Each table maps to
one or more aggregates; the `aggregate` column tells you which
aggregate owns the row.

| Table                                  | Aggregate            | Notes                                            |
| -------------------------------------- | -------------------- | ------------------------------------------------ |
| `infixedu__pages`                      | Page                 | Editable page with slug and status               |
| `sm_pages`                             | FrontendPage         | Generic front-end page record                    |
| `sm_news`                              | News                 | News entry with category and body                |
| `sm_news_categories`                   | NewsCategory         | News category taxonomy                           |
| `sm_news_comments`                     | NewsComment          | Comment on a news entry                          |
| `sm_news_pages`                        | NewsPage             | Public news landing-page configuration           |
| `sm_notice_boards`                     | NoticeBoard          | Public-site notice board                         |
| `sm_testimonials`                      | Testimonial          | Testimonial with rating                          |
| `home_sliders`                         | HomeSlider           | Home-page slider image and link                  |
| `speech_sliders`                       | SpeechSlider         | Public-page leadership message (CMS-side)        |
| `contents`                             | Content              | Uploaded content item                            |
| `content_types`                        | ContentType (alt)    | Content type taxonomy                            |
| `sm_content_types`                     | ContentType          | Content type taxonomy (scoped)                   |
| `content_share_lists`                  | ContentShareList     | Bulk-share list of content items                 |
| `sm_teacher_upload_contents`           | TeacherUploadContent | Teacher-uploaded content (per class-section)     |
| `sm_upload_contents`                   | UploadContent        | Admin-uploaded content (per role/class)          |
| `sm_about_pages`                       | AboutPage            | About-page configuration                         |
| `sm_contact_pages`                     | ContactPage          | Contact-page configuration                       |
| `sm_course_pages`                      | CoursePage           | Course landing page                              |
| `sm_home_page_settings`                | HomePageSetting      | Home-page setting                                |

## Notes

- Every school-scoped table includes `school_id` for multi-tenant
  isolation. The `school_id` is `NOT NULL DEFAULT 1` for the bootstrap
  school.
- Every school-scoped table includes `academic_id` referencing
  `sm_academic_years`. The CMS domain uses `academic_id` to scope
  content and content-share lists. The `infixedu__pages` and
  `sm_pages` tables do not include `academic_id`; the scope is
  per-school only.
- Every table includes `created_at`, `updated_at`, `created_by`,
  `updated_by`, `active_status` (where applicable). These are managed
  by the engine's storage adapter.
- The `infixedu__pages` table uses an `enum('draft', 'published')`
  for `status`. Other tables use `tinyint` flags.
- The `sm_news_comments` table does not include `school_id`; the
  scope is derived from the `News` aggregate. Consumers MUST add a
  row-level filter on `school_id` for news comment queries, derived
  from the parent news.
- The `sm_news_pages`, `sm_about_pages`, `sm_contact_pages`, and
  `sm_home_page_settings` tables are unique per school (one row per
  school). The domain enforces "at most one active" through
  `find_active` queries.
- The `sm_teacher_upload_contents` table is the only CMS table with
  `chapter_id` and `lesson_id` references; these reference the
  academic domain's lesson and topic aggregates. The CMS does not
  enforce referential integrity on these columns; the academic
  domain's storage adapter does.
- File references (`file` column) and URLs (`link` column) are
  captured as plain strings at the persistence boundary. The domain
  enforces their shape through value objects at construction time.
