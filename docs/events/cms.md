# CMS Domain — Events

Quick reference of every event the CMS domain emits. Events are
immutable, append-only records. Every event carries a typed
`EventEnvelope` and is durably persisted to the aggregate's event
log. Most events are observed by the public-site port adapter for
rendering.

| Event                              | Aggregate               | Subscribers                              | Description                                                            | Durable? | Replicated? | Replayable? |
| ---------------------------------- | ----------------------- | ---------------------------------------- | ---------------------------------------------------------------------- | -------- | ----------- | ----------- |
| `PageCreated`                      | `Page`                  | —                                        | A new page was created.                                                | yes      | yes         | yes         |
| `PageUpdated`                      | `Page`                  | —                                        | A page was patched.                                                    | yes      | yes         | yes         |
| `PagePublished`                    | `Page`                  | public-site port adapter                 | A page was published.                                                  | yes      | yes         | yes         |
| `PageArchived`                     | `Page`                  | public-site port adapter                 | A page was archived.                                                   | yes      | yes         | yes         |
| `PageDeleted`                      | `Page`                  | —                                        | A page was soft-deleted.                                               | yes      | yes         | yes         |
| `NewsCreated`                      | `News`                  | —                                        | A news article was created.                                            | yes      | yes         | yes         |
| `NewsUpdated`                      | `News`                  | —                                        | A news article was patched.                                            | yes      | yes         | yes         |
| `NewsPublished`                    | `News`                  | public-site port adapter                 | A news article was published.                                          | yes      | yes         | yes         |
| `NewsUnpublished`                  | `News`                  | public-site port adapter                 | A news article was unpublished.                                        | yes      | yes         | yes         |
| `NewsDeleted`                      | `News`                  | —                                        | A news article was soft-deleted.                                       | yes      | yes         | yes         |
| `NewsViewIncremented`              | `News`                  | —                                        | A news view was recorded.                                              | yes      | yes         | yes         |
| `NewsCommentAdded`                 | `NewsComment`           | `communication`                          | A comment was added to a news article.                                 | yes      | yes         | yes         |
| `NewsCommentApproved`              | `NewsComment`           | public-site port adapter                 | A comment was approved.                                                | yes      | yes         | yes         |
| `NewsCommentHidden`                | `NewsComment`           | public-site port adapter                 | A comment was hidden.                                                  | yes      | yes         | yes         |
| `NewsCommentDeleted`               | `NewsComment`           | —                                        | A comment was soft-deleted.                                            | yes      | yes         | yes         |
| `NewsPageCreated`                  | `NewsPage`              | public-site port adapter                 | The news listing page was created.                                     | yes      | yes         | yes         |
| `NewsPageUpdated`                  | `NewsPage`              | public-site port adapter                 | The news listing page was patched.                                     | yes      | yes         | yes         |
| `NewsPageDeleted`                  | `NewsPage`              | —                                        | The news listing page was soft-deleted.                                | yes      | yes         | yes         |
| `NoticeBoardCreated`               | `NoticeBoard`           | —                                        | A notice board entry was created.                                      | yes      | yes         | yes         |
| `NoticeBoardUpdated`               | `NoticeBoard`           | —                                        | A notice board entry was patched.                                      | yes      | yes         | yes         |
| `NoticeBoardPublished`             | `NoticeBoard`           | public-site port adapter                 | A notice board entry was published.                                    | yes      | yes         | yes         |
| `NoticeBoardUnpublished`           | `NoticeBoard`           | public-site port adapter                 | A notice board entry was unpublished.                                  | yes      | yes         | yes         |
| `NoticeBoardDeleted`               | `NoticeBoard`           | —                                        | A notice board entry was soft-deleted.                                 | yes      | yes         | yes         |
| `TestimonialCreated`               | `Testimonial`           | public-site port adapter                 | A testimonial was created.                                            | yes      | yes         | yes         |
| `TestimonialUpdated`               | `Testimonial`           | public-site port adapter                 | A testimonial was patched.                                            | yes      | yes         | yes         |
| `TestimonialDeleted`               | `Testimonial`           | —                                        | A testimonial was soft-deleted.                                       | yes      | yes         | yes         |
| `HomeSliderCreated`                | `HomeSlider`            | public-site port adapter                 | A home slider was created.                                             | yes      | yes         | yes         |
| `HomeSliderUpdated`                | `HomeSlider`            | public-site port adapter                 | A home slider was patched.                                             | yes      | yes         | yes         |
| `HomeSliderDeleted`                | `HomeSlider`            | —                                        | A home slider was soft-deleted.                                        | yes      | yes         | yes         |
| `SpeechSliderCreated`              | `SpeechSlider` (CMS-side) | public-site port adapter               | A speech slider card was created.                                      | yes      | yes         | yes         |
| `SpeechSliderUpdated`              | `SpeechSlider` (CMS-side) | public-site port adapter               | A speech slider card was patched.                                      | yes      | yes         | yes         |
| `SpeechSliderDeleted`              | `SpeechSlider` (CMS-side) | —                                      | A speech slider card was soft-deleted.                                 | yes      | yes         | yes         |
| `ContentCreated`                   | `Content`               | —                                        | A content item was created.                                            | yes      | yes         | yes         |
| `ContentUpdated`                   | `Content`               | —                                        | A content item was patched.                                            | yes      | yes         | yes         |
| `ContentDeleted`                   | `Content`               | —                                        | A content item was soft-deleted.                                       | yes      | yes         | yes         |
| `ContentTypeCreated`               | `ContentType`           | —                                        | A content type was created.                                            | yes      | yes         | yes         |
| `ContentTypeUpdated`               | `ContentType`           | —                                        | A content type was patched.                                            | yes      | yes         | yes         |
| `ContentTypeDeleted`               | `ContentType`           | —                                        | A content type was soft-deleted.                                       | yes      | yes         | yes         |
| `ContentShareListCreated`          | `ContentShareList`      | —                                        | A content share list was created.                                      | yes      | yes         | yes         |
| `ContentShareListDispatched`       | `ContentShareList`      | `communication`                          | A content share list was dispatched.                                   | yes      | yes         | yes         |
| `ContentShareListCancelled`        | `ContentShareList`      | —                                        | A content share list was cancelled.                                    | yes      | yes         | yes         |
| `ContentShareListDeleted`          | `ContentShareList`      | —                                        | A content share list was soft-deleted.                                 | yes      | yes         | yes         |
| `TeacherUploadContentCreated`      | `TeacherUploadContent`  | `communication`                          | Teacher-authored content was created.                                  | yes      | yes         | yes         |
| `TeacherUploadContentUpdated`      | `TeacherUploadContent`  | —                                        | Teacher-authored content was patched.                                  | yes      | yes         | yes         |
| `TeacherUploadContentDeleted`      | `TeacherUploadContent`  | —                                        | Teacher-authored content was soft-deleted.                             | yes      | yes         | yes         |
| `UploadContentCreated`             | `UploadContent`         | —                                        | A file upload was created.                                            | yes      | yes         | yes         |
| `UploadContentUpdated`             | `UploadContent`         | —                                        | A file upload was patched.                                            | yes      | yes         | yes         |
| `UploadContentDeleted`             | `UploadContent`         | —                                        | A file upload was soft-deleted.                                       | yes      | yes         | yes         |
| `AboutPageCreated`                 | `AboutPage`             | public-site port adapter                 | The About page was created.                                            | yes      | yes         | yes         |
| `AboutPageUpdated`                 | `AboutPage`             | public-site port adapter                 | The About page was patched.                                            | yes      | yes         | yes         |
| `AboutPageDeleted`                 | `AboutPage`             | —                                        | The About page was soft-deleted.                                       | yes      | yes         | yes         |
| `ContactPageCreated`               | `ContactPage`           | public-site port adapter                 | The Contact page was created.                                          | yes      | yes         | yes         |
| `ContactPageUpdated`               | `ContactPage`           | public-site port adapter                 | The Contact page was patched.                                          | yes      | yes         | yes         |
| `ContactPageDeleted`               | `ContactPage`           | —                                        | The Contact page was soft-deleted.                                     | yes      | yes         | yes         |
| `CoursePageCreated`                | `CoursePage`            | public-site port adapter                 | A course page was created.                                             | yes      | yes         | yes         |
| `CoursePageUpdated`                | `CoursePage`            | public-site port adapter                 | A course page was patched.                                             | yes      | yes         | yes         |
| `CoursePageDeleted`                | `CoursePage`            | —                                        | A course page was soft-deleted.                                        | yes      | yes         | yes         |
| `HomePageSettingCreated`           | `HomePageSetting`       | public-site port adapter                 | The home page setting was created.                                     | yes      | yes         | yes         |
| `HomePageSettingUpdated`           | `HomePageSetting`       | public-site port adapter                 | The home page setting was patched.                                     | yes      | yes         | yes         |
| `HomePageSettingDeleted`           | `HomePageSetting`       | —                                        | The home page setting was soft-deleted.                                | yes      | yes         | yes         |
| `FrontendPageCreated`              | `FrontendPage`          | public-site port adapter                 | A dynamic public page was created.                                     | yes      | yes         | yes         |
| `FrontendPageUpdated`              | `FrontendPage`          | public-site port adapter                 | A dynamic public page was patched.                                     | yes      | yes         | yes         |
| `FrontendPageDeleted`              | `FrontendPage`          | —                                        | A dynamic public page was soft-deleted.                                | yes      | yes         | yes         |
| `NewsCategoryCreated`              | `NewsCategory`          | public-site port adapter                 | A news category was created.                                           | yes      | yes         | yes         |
| `NewsCategoryUpdated`              | `NewsCategory`          | public-site port adapter                 | A news category was patched.                                           | yes      | yes         | yes         |
| `NewsCategoryDeleted`              | `NewsCategory`          | —                                        | A news category was soft-deleted.                                      | yes      | yes         | yes         |

**See also:** `docs/specs/cms/events.md` for full Rust struct
definitions, the canonical `EventEnvelope`, and per-event
subscribers.
