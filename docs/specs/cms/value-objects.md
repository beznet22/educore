# CMS Domain — Value Objects

Value objects are immutable, validated at construction, and have no
identity. They are compared by value.

## Identifiers

All identifiers in the CMS domain are typed and tenant-scoped. The
generic `Id<S, T>` wrapper carries the `SchoolId` of the owning school
and the local id (`Uuid`).

| Identifier                  | Backing Type            | Source Column                  |
| --------------------------- | ----------------------- | ------------------------------ |
| `PageId`                    | `Id<Page>`              | `cms_pages.id`                 |
| `NewsId`                    | `Id<News>`              | `cms_news.id`                  |
| `NewsCategoryId`            | `Id<NewsCategory>`      | `cms_news_categories.id`       |
| `NewsCommentId`             | `Id<NewsComment>`       | `cms_news_comments.id`         |
| `NewsPageId`                | `Id<NewsPage>`          | `cms_news_pages.id`            |
| `NoticeBoardId`             | `Id<NoticeBoard>`       | `cms_notice_boards.id`         |
| `TestimonialId`             | `Id<Testimonial>`       | `cms_testimonials.id`          |
| `HomeSliderId`              | `Id<HomeSlider>`        | `cms_home_sliders.id`          |
| `SpeechSliderId`            | `Id<SpeechSlider>`      | `cms_speech_sliders.id`        |
| `ContentId`                 | `Id<Content>`           | `cms_contents.id`              |
| `ContentTypeId`             | `Id<ContentType>`       | `cms_content_types.id`         |
| `ContentShareListId`        | `Id<ContentShareList>`  | `cms_content_share_lists.id`   |
| `TeacherUploadContentId`    | `Id<TeacherUploadContent>` | `cms_teacher_upload_contents.id` |
| `UploadContentId`           | `Id<UploadContent>`     | `cms_upload_contents.id`        |
| `AboutPageId`               | `Id<AboutPage>`         | `cms_about_pages.id`            |
| `ContactPageId`             | `Id<ContactPage>`       | `cms_contact_pages.id`          |
| `CoursePageId`              | `Id<CoursePage>`        | `cms_course_pages.id`           |
| `HomePageSettingId`         | `Id<HomePageSetting>`   | `cms_home_page_settings.id`     |
| `FrontendPageId`            | `Id<FrontendPage>`      | `cms_frontend_pages.id`                  |

## Names and Free Text

| Type                  | Constraints                                                       |
| --------------------- | ----------------------------------------------------------------- |
| `PageTitle`           | 1..191 chars                                                      |
| `PageSubTitle`        | 1..191 chars, unique within school                                |
| `PageDescription`     | 1..5000 chars                                                     |
| `NewsTitle`           | 1..191 chars                                                      |
| `NewsBody`            | 1..65535 chars                                                    |
| `CategoryName`        | 1..191 chars                                                       |
| `TestimonialDescription` | 1..5000 chars                                                 |
| `SpeechText`          | 1..5000 chars                                                     |
| `ContentTitle`        | 1..200 chars                                                      |
| `ContentDescription`  | 1..500 chars                                                      |
| `ContentShareListTitle` | 1..191 chars                                                    |
| `CoursePageTitle`     | 1..191 chars                                                      |
| `CoursePageDescription` | 1..5000 chars                                                   |
| `HomeSliderLinkLabel` | 1..255 chars                                                      |
| `HomePageTitle`       | 1..255 chars                                                      |
| `HomePageLongTitle`   | 1..255 chars                                                      |
| `HomePageShortDescription` | 1..5000 chars                                               |
| `CommentMessage`      | 1..5000 chars                                                     |

## URLs, Files, and Slugs

| Type                 | Constraints                                                       |
| -------------------- | ----------------------------------------------------------------- |
| `Slug`               | URL-safe slug, 1..200 chars, `[a-z0-9-]`                          |
| `Url`                | Validated URL, max 2048 chars                                     |
| `FileReference`      | From `educore-platform`                                           |
| `YoutubeLink`        | URL or null; validated as a YouTube URL when present              |
| `SourceUrl`          | URL or null                                                       |
| `ButtonText`         | 1..191 chars                                                      |
| `ButtonUrl`          | `Url`                                                             |

## Status Enums

| Type                    | Values                                                              |
| ----------------------- | ------------------------------------------------------------------- |
| `PageStatus`            | `Draft`, `Published`                                                |
| `ContentStatus`         | `Draft`, `Published`, `Archived`                                    |
| `NewsStatus`            | `Active`, `Disabled`                                                |
| `NewsCommentStatus`     | `Pending`, `Approved`, `Hidden`                                     |
| `ContentShareType`      | `Groups` (`G`), `Class` (`C`), `Individual` (`I`), `Public` (`P`)    |
| `ContentShareListStatus`| `Draft`, `Dispatched`, `Cancelled`                                  |
| `StarRating`            | `u8` in `1..5`                                                       |
| `TestimonialRating`     | `StarRating`                                                        |
| `Visible`               | `bool` — when `true`, the row is visible on the public site         |
| `IsParent`              | `bool` — when `true`, the course page is a top-level parent         |
| `IsDefault`             | `bool` — when `true`, the page is a pre-installed template           |
| `IsGlobal`              | `bool` — when `true`, the news is visible across all schools        |
| `AutoApprove`           | `bool` — when `true`, comments are auto-approved                    |
| `IsComment`             | `bool` — when `true`, comments are enabled on the news              |
| `IsPublished`           | `bool` — when `true`, the notice board is visible on the public site|
| `IsDynamic`             | `bool` — when `true`, the front-end page is rendered dynamically    |
| `ActiveStatus`          | `bool` — soft-delete flag                                            |
| `AvailableForAdmin`     | `bool`                                                              |
| `AvailableForAllClasses`| `bool`                                                              |

## Address, Contact, and Maps

| Type                 | Constraints                                                       |
| -------------------- | ----------------------------------------------------------------- |
| `PostalAddress`      | 1..191 chars                                                       |
| `PhoneNumber`        | E.164 format preferred; alternative national formats accepted     |
| `EmailAddress`       | RFC 5322 with length cap 200                                      |
| `Latitude`           | 1..191 chars; validated as a latitude string                       |
| `Longitude`          | 1..191 chars; validated as a longitude string                      |
| `ZoomLevel`          | `i32` in `0..21`                                                  |
| `GoogleMapAddress`   | 1..191 chars                                                       |

## Audience

| Type                  | Notes                                                              |
| --------------------- | ------------------------------------------------------------------ |
| `RoleId`              | From `educore-rbac`                                                |
| `ClassId`             | From `educore-academic`                                            |
| `SectionId`           | From `educore-academic`                                            |
| `UserId`              | From `educore-platform`                                            |
| `AudienceDescriptor`  | `Vec<RoleId>` OR `ClassId`+`SectionId` OR `Vec<UserId>` OR `Public` |
| `RoleIdList`          | Comma-separated list of `RoleId` (decoded into `Vec<RoleId>`)     |

## Time and Schedule

| Type                  | Notes                                                              |
| --------------------- | ------------------------------------------------------------------ |
| `PublishDate`         | `NaiveDate`                                                        |
| `NoticeDate`          | `NaiveDate`                                                        |
| `ShareDate`           | `NaiveDate`                                                        |
| `ValidUntil`          | `NaiveDate`                                                        |
| `UploadDate`          | `NaiveDate`                                                        |
| `AcademicYearId`      | From `educore-academic`                                            |
| `TenantContext`       | `(SchoolId, UserId, ...)` from `educore-platform`                 |

## Page Settings

| Type                  | Notes                                                              |
| --------------------- | ------------------------------------------------------------------ |
| `PageSettings`        | A typed JSON value with versioned schema                           |

## Validation Rules

All value objects implement `Validate` and refuse construction when
validation fails:

```rust
pub trait Validate {
    fn validate(&self) -> Result<(), ValueError>;
}
```

Construction is the only entry point:

```rust
let title = NewsTitle::new("Annual sports day announced")?;
```

Parsing returns `Result<NewsTitle, ValueError>`. There are no setters
that bypass validation.
