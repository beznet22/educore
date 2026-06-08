# CMS Domain — Events

Domain events describe facts that have already happened. They are
immutable, append-only records used for cross-domain integration, audit,
and event sourcing.

All events implement:

```rust
pub trait DomainEvent: Serialize + DeserializeOwned + Send + Sync {
    const TYPE: &'static str;
    fn aggregate_id(&self) -> Uuid;
    fn school_id(&self) -> SchoolId;
    fn occurred_at(&self) -> Timestamp;
}
```

The event envelope wraps the event with metadata:

```rust
pub struct EventEnvelope<E> {
    pub event_id: EventId,
    pub event_type: &'static str,
    pub school_id: SchoolId,
    pub aggregate_id: Uuid,
    pub aggregate_type: &'static str,
    pub actor_id: UserId,
    pub correlation_id: CorrelationId,
    pub causation_id: Option<EventId>,
    pub occurred_at: Timestamp,
    pub payload: E,
}
```

## Page Lifecycle

```rust
pub struct PageCreated {
    pub page_id: PageId,
    pub title: PageTitle,
    pub slug: Option<Slug>,
    pub home_page: bool,
}

pub struct PageUpdated { pub page_id: PageId, pub changes: Vec<&'static str> }
pub struct PagePublished { pub page_id: PageId, pub published_by: UserId, pub published_at: Timestamp }
pub struct PageArchived { pub page_id: PageId, pub archived_by: UserId }
pub struct PageDeleted { pub page_id: PageId, pub deleted_by: UserId }
```

**Subscribers:**
- The public-site port adapter re-renders on `PagePublished` /
  `PageArchived`.

## News Lifecycle

```rust
pub struct NewsCreated {
    pub news_id: NewsId,
    pub news_title: NewsTitle,
    pub category_id: NewsCategoryId,
    pub publish_date: PublishDate,
    pub is_global: bool,
}

pub struct NewsUpdated { pub news_id: NewsId, pub changes: Vec<&'static str> }
pub struct NewsPublished { pub news_id: NewsId, pub published_by: UserId, pub published_at: Timestamp }
pub struct NewsUnpublished { pub news_id: NewsId, pub unpublished_by: UserId }
pub struct NewsDeleted { pub news_id: NewsId, pub deleted_by: UserId }
pub struct NewsViewIncremented { pub news_id: NewsId, pub new_count: i64 }
```

## News Comment Lifecycle

```rust
pub struct NewsCommentAdded {
    pub news_comment_id: NewsCommentId,
    pub news_id: NewsId,
    pub user_id: UserId,
    pub parent_id: Option<NewsCommentId>,
    pub status: NewsCommentStatus,
}

pub struct NewsCommentApproved { pub news_comment_id: NewsCommentId, pub approved_by: UserId }
pub struct NewsCommentHidden { pub news_comment_id: NewsCommentId, pub hidden_by: UserId }
pub struct NewsCommentDeleted { pub news_comment_id: NewsCommentId, pub deleted_by: UserId }
```

## News Page

```rust
pub struct NewsPageCreated { pub news_page_id: NewsPageId, pub title: PageTitle }
pub struct NewsPageUpdated { pub news_page_id: NewsPageId, pub changes: Vec<&'static str> }
pub struct NewsPageDeleted { pub news_page_id: NewsPageId }
```

## Notice Board (Public Site)

```rust
pub struct NoticeBoardCreated {
    pub notice_board_id: NoticeBoardId,
    pub notice_title: String,
    pub notice_date: NoticeDate,
    pub publish_on: Option<PublishDate>,
}

pub struct NoticeBoardUpdated { pub notice_board_id: NoticeBoardId, pub changes: Vec<&'static str> }
pub struct NoticeBoardPublished { pub notice_board_id: NoticeBoardId, pub published_by: UserId }
pub struct NoticeBoardUnpublished { pub notice_board_id: NoticeBoardId, pub unpublished_by: UserId }
pub struct NoticeBoardDeleted { pub notice_board_id: NoticeBoardId, pub deleted_by: UserId }
```

## Testimonial

```rust
pub struct TestimonialCreated {
    pub testimonial_id: TestimonialId,
    pub name: PersonName,
    pub designation: String,
    pub star_rating: StarRating,
}

pub struct TestimonialUpdated { pub testimonial_id: TestimonialId, pub changes: Vec<&'static str> }
pub struct TestimonialDeleted { pub testimonial_id: TestimonialId, pub deleted_by: UserId }
```

## Home Slider

```rust
pub struct HomeSliderCreated { pub home_slider_id: HomeSliderId, pub image: FileReference, pub link: Option<Url> }
pub struct HomeSliderUpdated { pub home_slider_id: HomeSliderId, pub changes: Vec<&'static str> }
pub struct HomeSliderDeleted { pub home_slider_id: HomeSliderId, pub deleted_by: UserId }
```

## Speech Slider (CMS-Side)

```rust
pub struct SpeechSliderCreated { pub speech_slider_id: SpeechSliderId, pub name: PersonName, pub designation: String }
pub struct SpeechSliderUpdated { pub speech_slider_id: SpeechSliderId, pub changes: Vec<&'static str> }
pub struct SpeechSliderDeleted { pub speech_slider_id: SpeechSliderId, pub deleted_by: UserId }
```

## Content Lifecycle

```rust
pub struct ContentCreated {
    pub content_id: ContentId,
    pub content_type_id: ContentTypeId,
    pub file_name: String,
    pub file_size: i64,
    pub youtube_link: Option<YoutubeLink>,
    pub uploaded_by: UserId,
}

pub struct ContentUpdated { pub content_id: ContentId, pub changes: Vec<&'static str> }
pub struct ContentDeleted { pub content_id: ContentId, pub deleted_by: UserId }
```

## Content Type

```rust
pub struct ContentTypeCreated { pub content_type_id: ContentTypeId, pub type_name: String }
pub struct ContentTypeUpdated { pub content_type_id: ContentTypeId, pub changes: Vec<&'static str> }
pub struct ContentTypeDeleted { pub content_type_id: ContentTypeId, pub deleted_by: UserId }
```

## Content Share List

```rust
pub struct ContentShareListCreated {
    pub content_share_list_id: ContentShareListId,
    pub title: ContentShareListTitle,
    pub share_date: ShareDate,
    pub valid_upto: ValidUntil,
    pub send_type: ContentShareType,
    pub content_count: u32,
}

pub struct ContentShareListDispatched {
    pub content_share_list_id: ContentShareListId,
    pub recipient_count: u32,
    pub dispatched_at: Timestamp,
}

pub struct ContentShareListCancelled { pub content_share_list_id: ContentShareListId, pub reason: Option<String> }
pub struct ContentShareListDeleted { pub content_share_list_id: ContentShareListId, pub deleted_by: UserId }
```

**Subscribers:**
- The communication domain may subscribe to dispatch notifications to
  the share audience.

## Teacher Upload Content

```rust
pub struct TeacherUploadContentCreated {
    pub teacher_upload_content_id: TeacherUploadContentId,
    pub content_title: ContentTitle,
    pub content_type: TeacherContentType,
    pub class_id: ClassId,
    pub section_id: SectionId,
}

pub struct TeacherUploadContentUpdated { pub teacher_upload_content_id: TeacherUploadContentId, pub changes: Vec<&'static str> }
pub struct TeacherUploadContentDeleted { pub teacher_upload_content_id: TeacherUploadContentId, pub deleted_by: UserId }
```

## Upload Content

```rust
pub struct UploadContentCreated { pub upload_content_id: UploadContentId, pub content_title: ContentTitle, pub content_type: i32 }
pub struct UploadContentUpdated { pub upload_content_id: UploadContentId, pub changes: Vec<&'static str> }
pub struct UploadContentDeleted { pub upload_content_id: UploadContentId, pub deleted_by: UserId }
```

## About Page

```rust
pub struct AboutPageCreated { pub about_page_id: AboutPageId, pub title: PageTitle }
pub struct AboutPageUpdated { pub about_page_id: AboutPageId, pub changes: Vec<&'static str> }
pub struct AboutPageDeleted { pub about_page_id: AboutPageId, pub deleted_by: UserId }
```

## Contact Page

```rust
pub struct ContactPageCreated { pub contact_page_id: ContactPageId, pub title: PageTitle }
pub struct ContactPageUpdated { pub contact_page_id: ContactPageId, pub changes: Vec<&'static str> }
pub struct ContactPageDeleted { pub contact_page_id: ContactPageId, pub deleted_by: UserId }
```

## Course Page

```rust
pub struct CoursePageCreated { pub course_page_id: CoursePageId, pub title: CoursePageTitle, pub is_parent: bool, pub parent_id: Option<CoursePageId> }
pub struct CoursePageUpdated { pub course_page_id: CoursePageId, pub changes: Vec<&'static str> }
pub struct CoursePageDeleted { pub course_page_id: CoursePageId, pub deleted_by: UserId }
```

## Home Page Setting

```rust
pub struct HomePageSettingCreated { pub home_page_setting_id: HomePageSettingId, pub title: HomePageTitle }
pub struct HomePageSettingUpdated { pub home_page_setting_id: HomePageSettingId, pub changes: Vec<&'static str> }
pub struct HomePageSettingDeleted { pub home_page_setting_id: HomePageSettingId, pub deleted_by: UserId }
```

## Frontend Page

```rust
pub struct FrontendPageCreated { pub frontend_page_id: FrontendPageId, pub title: PageTitle, pub sub_title: PageSubTitle, pub is_dynamic: bool }
pub struct FrontendPageUpdated { pub frontend_page_id: FrontendPageId, pub changes: Vec<&'static str> }
pub struct FrontendPageDeleted { pub frontend_page_id: FrontendPageId, pub deleted_by: UserId }
```

## News Category

```rust
pub struct NewsCategoryCreated { pub news_category_id: NewsCategoryId, pub category_name: CategoryName }
pub struct NewsCategoryUpdated { pub news_category_id: NewsCategoryId, pub changes: Vec<&'static str> }
pub struct NewsCategoryDeleted { pub news_category_id: NewsCategoryId, pub deleted_by: UserId }
```
