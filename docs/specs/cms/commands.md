# CMS Domain — Commands

Commands describe intent. They are validated, authorized, and dispatched
to the relevant aggregate. Every command produces zero or more events
that are recorded in the event log.

All commands carry a `TenantContext` (school + actor + correlation) and
are rejected if the actor lacks the required capability.

## CreatePage

```rust
pub struct CreatePageCommand {
    pub tenant: TenantContext,
    pub name: String,
    pub title: PageTitle,
    pub description: Option<PageDescription>,
    pub slug: Option<Slug>,
    pub settings: Option<PageSettings>,
    pub home_page: bool,
    pub is_default: bool,
}
```

**Capability:** `Page.Create`
**Pre-conditions:**
- The `slug` is unique within the school when set.
- At most one page per school has `home_page = true`.

**Effects:** Creates a `Page` in `Draft` status and emits
`PageCreated`.

## UpdatePage

```rust
pub struct UpdatePageCommand {
    pub tenant: TenantContext,
    pub page_id: PageId,
    pub title: Option<PageTitle>,
    pub description: Option<PageDescription>,
    pub slug: Option<Slug>,
    pub settings: Option<PageSettings>,
    pub home_page: Option<bool>,
}
```

**Capability:** `Page.Update`
**Effects:** Emits `PageUpdated`.

## PublishPage

```rust
pub struct PublishPageCommand {
    pub tenant: TenantContext,
    pub page_id: PageId,
}
```

**Capability:** `Page.Publish`
**Effects:** Status moves to `Published`; emits `PagePublished`. The
public-site port adapter re-renders.

## ArchivePage

```rust
pub struct ArchivePageCommand {
    pub tenant: TenantContext,
    pub page_id: PageId,
}
```

**Capability:** `Page.Archive`
**Effects:** Status moves to `Draft`; emits `PageArchived`. The
public-site port adapter removes the page from public listings.

## DeletePage

```rust
pub struct DeletePageCommand {
    pub tenant: TenantContext,
    pub page_id: PageId,
}
```

**Capability:** `Page.Delete`
**Pre-conditions:** The page is not `is_default = true`.
**Effects:** Emits `PageDeleted`.

## CreateNews

```rust
pub struct CreateNewsCommand {
    pub tenant: TenantContext,
    pub news_title: NewsTitle,
    pub category_id: NewsCategoryId,
    pub image: Option<FileReference>,
    pub image_thumb: Option<FileReference>,
    pub news_body: NewsBody,
    pub publish_date: PublishDate,
    pub is_global: IsGlobal,
    pub auto_approve: AutoApprove,
    pub is_comment: IsComment,
    pub order: Option<String>,
}
```

**Capability:** `News.Create`
**Effects:** Creates a `News` and emits `NewsCreated`.

## UpdateNews

```rust
pub struct UpdateNewsCommand {
    pub tenant: TenantContext,
    pub news_id: NewsId,
    pub news_title: Option<NewsTitle>,
    pub category_id: Option<NewsCategoryId>,
    pub image: Option<FileReference>,
    pub image_thumb: Option<FileReference>,
    pub news_body: Option<NewsBody>,
    pub publish_date: Option<PublishDate>,
    pub is_global: Option<IsGlobal>,
    pub auto_approve: Option<AutoApprove>,
    pub is_comment: Option<IsComment>,
    pub order: Option<String>,
}
```

**Capability:** `News.Update`
**Effects:** Emits `NewsUpdated`.

## PublishNews

```rust
pub struct PublishNewsCommand {
    pub tenant: TenantContext,
    pub news_id: NewsId,
}
```

**Capability:** `News.Publish`
**Effects:** Emits `NewsPublished`. The public-site port adapter
surfaces the news.

## UnpublishNews

```rust
pub struct UnpublishNewsCommand {
    pub tenant: TenantContext,
    pub news_id: NewsId,
}
```

**Capability:** `News.Unpublish`
**Effects:** Emits `NewsUnpublished`.

## DeleteNews

```rust
pub struct DeleteNewsCommand {
    pub tenant: TenantContext,
    pub news_id: NewsId,
}
```

**Capability:** `News.Delete`
**Effects:** Emits `NewsDeleted`. Soft delete; the audit record
remains.

## CommentOnNews

```rust
pub struct CommentOnNewsCommand {
    pub tenant: TenantContext,
    pub news_id: NewsId,
    pub message: CommentMessage,
    pub parent_id: Option<NewsCommentId>,
}
```

**Capability:** `NewsComment.Create`
**Pre-conditions:** The news has `is_comment = true`.
**Effects:** Emits `NewsCommentAdded`. The comment is created in
`Pending` status when `auto_approve = false`; otherwise in `Approved`.

## ModerateNewsComment

```rust
pub struct ModerateNewsCommentCommand {
    pub tenant: TenantContext,
    pub news_comment_id: NewsCommentId,
    pub action: NewsCommentModerationAction, // Approve | Hide
}
```

**Capability:** `NewsComment.Moderate`
**Effects:** Emits `NewsCommentApproved` or `NewsCommentHidden`.

## DeleteNewsComment

```rust
pub struct DeleteNewsCommentCommand {
    pub tenant: TenantContext,
    pub news_comment_id: NewsCommentId,
}
```

**Capability:** `NewsComment.Delete`
**Effects:** Emits `NewsCommentDeleted`.

## CreateNoticeBoard

```rust
pub struct CreateNoticeBoardCommand {
    pub tenant: TenantContext,
    pub notice_title: String,
    pub notice_message: String,
    pub notice_date: NoticeDate,
    pub publish_on: Option<PublishDate>,
    pub inform_to: AudienceDescriptor,
}
```

**Capability:** `NoticeBoard.Create`
**Effects:** Emits `NoticeBoardCreated`.

## PublishNoticeBoard

```rust
pub struct PublishNoticeBoardCommand {
    pub tenant: TenantContext,
    pub notice_board_id: NoticeBoardId,
}
```

**Capability:** `NoticeBoard.Publish`
**Effects:** Status moves to `is_published = true`; emits
`NoticeBoardPublished`.

## UpdateNoticeBoard / UnpublishNoticeBoard / DeleteNoticeBoard

```rust
pub struct UpdateNoticeBoardCommand { ... }
pub struct UnpublishNoticeBoardCommand { ... }
pub struct DeleteNoticeBoardCommand { ... }
```

**Capabilities:** `NoticeBoard.Update`, `NoticeBoard.Unpublish`,
`NoticeBoard.Delete`.

## CreateTestimonial

```rust
pub struct CreateTestimonialCommand {
    pub tenant: TenantContext,
    pub name: PersonName,
    pub designation: String,
    pub institution_name: String,
    pub image: FileReference,
    pub description: TestimonialDescription,
    pub star_rating: StarRating,
}
```

**Capability:** `Testimonial.Create`
**Pre-conditions:** `star_rating` is in `1..5`.
**Effects:** Emits `TestimonialCreated`.

## UpdateTestimonial / DeleteTestimonial

```rust
pub struct UpdateTestimonialCommand { ... }
pub struct DeleteTestimonialCommand { ... }
```

**Capabilities:** `Testimonial.Update`, `Testimonial.Delete`.

## CreateHomeSlider

```rust
pub struct CreateHomeSliderCommand {
    pub tenant: TenantContext,
    pub image: FileReference,
    pub link: Option<Url>,
}
```

**Capability:** `HomeSlider.Create`
**Effects:** Emits `HomeSliderCreated`.

## UpdateHomeSlider

```rust
pub struct UpdateHomeSliderCommand {
    pub tenant: TenantContext,
    pub home_slider_id: HomeSliderId,
    pub image: Option<FileReference>,
    pub link: Option<Url>,
}
```

**Capability:** `HomeSlider.Update`
**Effects:** Emits `HomeSliderUpdated`.

## DeleteHomeSlider

```rust
pub struct DeleteHomeSliderCommand {
    pub tenant: TenantContext,
    pub home_slider_id: HomeSliderId,
}
```

**Capability:** `HomeSlider.Delete`
**Effects:** Emits `HomeSliderDeleted`.

## ConfigureHomePage

```rust
pub struct ConfigureHomePageCommand {
    pub tenant: TenantContext,
    pub title: HomePageTitle,
    pub long_title: Option<HomePageLongTitle>,
    pub short_description: Option<HomePageShortDescription>,
    pub link_label: Option<HomeSliderLinkLabel>,
    pub link_url: Option<Url>,
    pub image: Option<FileReference>,
}
```

**Capability:** `HomePageSetting.Configure`
**Effects:** Emits `HomePageSettingCreated` or `HomePageSettingUpdated`
depending on whether the school already has a setting.

## CreateContent

```rust
pub struct CreateContentCommand {
    pub tenant: TenantContext,
    pub file_name: String,
    pub file_size: i64,
    pub content_type_id: ContentTypeId,
    pub youtube_link: Option<YoutubeLink>,
    pub upload_file: Option<FileReference>,
}
```

**Capability:** `Content.Create`
**Effects:** Emits `ContentCreated`.

## UpdateContent / DeleteContent

```rust
pub struct UpdateContentCommand { ... }
pub struct DeleteContentCommand { ... }
```

**Capabilities:** `Content.Update`, `Content.Delete`.

## CreateContentShareList

```rust
pub struct CreateContentShareListCommand {
    pub tenant: TenantContext,
    pub title: ContentShareListTitle,
    pub share_date: ShareDate,
    pub valid_upto: ValidUntil,
    pub description: Option<String>,
    pub send_type: ContentShareType,
    pub content_ids: Vec<ContentId>,
    pub gr_role_ids: Option<Vec<RoleId>>,
    pub ind_user_ids: Option<Vec<UserId>>,
    pub class_id: Option<ClassId>,
    pub section_ids: Option<Vec<SectionId>>,
    pub url: Option<Url>,
}
```

**Capability:** `ContentShareList.Create`
**Pre-conditions:** `valid_upto ≥ share_date`.
**Effects:** Emits `ContentShareListCreated`.

## DispatchContentShareList

```rust
pub struct DispatchContentShareListCommand {
    pub tenant: TenantContext,
    pub content_share_list_id: ContentShareListId,
}
```

**Capability:** `ContentShareList.Dispatch`
**Effects:** Freezes the recipient set; emits `ContentShareListDispatched`.
The communication domain may subscribe to dispatch notifications.

## CancelContentShareList / DeleteContentShareList

```rust
pub struct CancelContentShareListCommand { ... }
pub struct DeleteContentShareListCommand { ... }
```

**Capabilities:** `ContentShareList.Cancel`, `ContentShareList.Delete`.

## CreateTeacherUploadContent

```rust
pub struct CreateTeacherUploadContentCommand {
    pub tenant: TenantContext,
    pub content_title: ContentTitle,
    pub content_type: TeacherContentType, // assignment, study_material, syllabus, other_download
    pub available_for_admin: AvailableForAdmin,
    pub available_for_all_classes: AvailableForAllClasses,
    pub upload_date: UploadDate,
    pub description: Option<ContentDescription>,
    pub source_url: Option<SourceUrl>,
    pub upload_file: Option<FileReference>,
    pub course_id: Option<i32>,
    pub parent_course_id: Option<i32>,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub chapter_id: Option<i64>,
    pub lesson_id: Option<i64>,
    pub parent_id: Option<i32>,
}
```

**Capability:** `TeacherUploadContent.Create`
**Effects:** Emits `TeacherUploadContentCreated`.

## UpdateTeacherUploadContent / DeleteTeacherUploadContent

```rust
pub struct UpdateTeacherUploadContentCommand { ... }
pub struct DeleteTeacherUploadContentCommand { ... }
```

**Capabilities:** `TeacherUploadContent.Update`,
`TeacherUploadContent.Delete`.

## CreateUploadContent

```rust
pub struct CreateUploadContentCommand {
    pub tenant: TenantContext,
    pub content_title: ContentTitle,
    pub content_type: i32, // FK to ContentType
    pub available_for_role: Option<i32>,
    pub available_for_class: Option<i32>,
    pub available_for_section: Option<i32>,
    pub upload_date: UploadDate,
    pub description: Option<ContentDescription>,
    pub upload_file: Option<FileReference>,
}
```

**Capability:** `UploadContent.Create`
**Effects:** Emits `UploadContentCreated`.

## UpdateUploadContent / DeleteUploadContent

```rust
pub struct UpdateUploadContentCommand { ... }
pub struct DeleteUploadContentCommand { ... }
```

**Capabilities:** `UploadContent.Update`, `UploadContent.Delete`.

## CreateAboutPage / UpdateAboutPage / DeleteAboutPage

```rust
pub struct CreateAboutPageCommand { ... }
pub struct UpdateAboutPageCommand { ... }
pub struct DeleteAboutPageCommand { ... }
```

**Capabilities:** `AboutPage.Create`, `AboutPage.Update`,
`AboutPage.Delete`.

## CreateContactPage / UpdateContactPage / DeleteContactPage

```rust
pub struct CreateContactPageCommand { ... }
pub struct UpdateContactPageCommand { ... }
pub struct DeleteContactPageCommand { ... }
```

**Capabilities:** `ContactPage.Create`, `ContactPage.Update`,
`ContactPage.Delete`.

## CreateCoursePage / UpdateCoursePage / DeleteCoursePage

```rust
pub struct CreateCoursePageCommand {
    pub tenant: TenantContext,
    pub title: CoursePageTitle,
    pub description: Option<CoursePageDescription>,
    pub main_title: Option<String>,
    pub main_description: Option<String>,
    pub image: Option<FileReference>,
    pub main_image: Option<FileReference>,
    pub button_text: Option<ButtonText>,
    pub button_url: Option<ButtonUrl>,
    pub is_parent: IsParent,
    pub parent_id: Option<CoursePageId>,
}

pub struct UpdateCoursePageCommand { ... }
pub struct DeleteCoursePageCommand { ... }
```

**Capabilities:** `CoursePage.Create`, `CoursePage.Update`,
`CoursePage.Delete`.

## CreateFrontendPage / UpdateFrontendPage / DeleteFrontendPage

```rust
pub struct CreateFrontendPageCommand {
    pub tenant: TenantContext,
    pub title: PageTitle,
    pub sub_title: PageSubTitle,
    pub slug: Option<Slug>,
    pub header_image: Option<FileReference>,
    pub details: Option<PageDescription>,
    pub is_dynamic: IsDynamic,
}

pub struct UpdateFrontendPageCommand { ... }
pub struct DeleteFrontendPageCommand { ... }
```

**Capabilities:** `FrontendPage.Create`, `FrontendPage.Update`,
`FrontendPage.Delete`.

## CreateNewsPage / UpdateNewsPage / DeleteNewsPage

```rust
pub struct CreateNewsPageCommand { ... }
pub struct UpdateNewsPageCommand { ... }
pub struct DeleteNewsPageCommand { ... }
```

**Capabilities:** `NewsPage.Create`, `NewsPage.Update`,
`NewsPage.Delete`.

## CreateNewsCategory / UpdateNewsCategory / DeleteNewsCategory

```rust
pub struct CreateNewsCategoryCommand { ... }
pub struct UpdateNewsCategoryCommand { ... }
pub struct DeleteNewsCategoryCommand { ... }
```

**Capabilities:** `NewsCategory.Create`, `NewsCategory.Update`,
`NewsCategory.Delete`.

## CreateContentType / UpdateContentType / DeleteContentType

```rust
pub struct CreateContentTypeCommand { ... }
pub struct UpdateContentTypeCommand { ... }
pub struct DeleteContentTypeCommand { ... }
```

**Capabilities:** `ContentType.Create`, `ContentType.Update`,
`ContentType.Delete`.

## CreateSpeechSlider / UpdateSpeechSlider / DeleteSpeechSlider

```rust
pub struct CreateSpeechSliderCommand { ... }
pub struct UpdateSpeechSliderCommand { ... }
pub struct DeleteSpeechSliderCommand { ... }
```

**Capabilities:** `SpeechSlider.Create`, `SpeechSlider.Update`,
`SpeechSlider.Delete`. Note: a separate `SpeechSlider` is also owned
by the communication domain; the consumer may wire the two to a
single adapter for the public-site rendering.
