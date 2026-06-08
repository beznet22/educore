# CMS Domain — Repositories

Repositories are ports (Rust traits). Adapters implement them. The
default adapter targets PostgreSQL; an SQLite adapter is provided for
embedded deployments.

## PageRepository

```rust
#[async_trait]
pub trait PageRepository: Send + Sync {
    async fn get(&self, id: PageId) -> Result<Option<Page>>;
    async fn find_by_slug(&self, school: SchoolId, slug: &Slug) -> Result<Option<Page>>;
    async fn find_home(&self, school: SchoolId) -> Result<Option<Page>>;
    async fn list(&self, school: SchoolId, q: PageQuery) -> Result<Vec<Page>>;
    async fn list_published(&self, school: SchoolId) -> Result<Vec<Page>>;
    async fn insert(&self, p: &Page) -> Result<()>;
    async fn update(&self, p: &Page) -> Result<()>;
    async fn delete(&self, id: PageId) -> Result<()>;
    async fn count(&self, school: SchoolId, q: PageQuery) -> Result<u64>;
    async fn page(&self, school: SchoolId, q: PageQuery, offset: u32, limit: u32) -> Result<Page<Page>>;
}
```

## NewsRepository

```rust
#[async_trait]
pub trait NewsRepository: Send + Sync {
    async fn get(&self, id: NewsId) -> Result<Option<News>>;
    async fn list(&self, school: SchoolId, q: NewsQuery) -> Result<Vec<News>>;
    async fn list_active(&self, school: SchoolId) -> Result<Vec<News>>;
    async fn list_global(&self) -> Result<Vec<News>>;
    async fn list_by_category(&self, school: SchoolId, category: NewsCategoryId) -> Result<Vec<News>>;
    async fn list_published_between(&self, school: SchoolId, from: NaiveDate, to: NaiveDate) -> Result<Vec<News>>;
    async fn increment_view(&self, id: NewsId) -> Result<()>;
    async fn insert(&self, n: &News) -> Result<()>;
    async fn update(&self, n: &News) -> Result<()>;
    async fn delete(&self, id: NewsId) -> Result<()>;
}
```

## NewsCommentRepository

```rust
#[async_trait]
pub trait NewsCommentRepository: Send + Sync {
    async fn get(&self, id: NewsCommentId) -> Result<Option<NewsComment>>;
    async fn list_for_news(&self, news: NewsId) -> Result<Vec<NewsComment>>;
    async fn list_pending(&self, school: SchoolId) -> Result<Vec<NewsComment>>;
    async fn insert(&self, c: &NewsComment) -> Result<()>;
    async fn update(&self, c: &NewsComment) -> Result<()>;
    async fn delete(&self, id: NewsCommentId) -> Result<()>;
}
```

## NewsCategoryRepository

```rust
#[async_trait]
pub trait NewsCategoryRepository: Send + Sync {
    async fn get(&self, id: NewsCategoryId) -> Result<Option<NewsCategory>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<NewsCategory>>;
    async fn find_by_name(&self, school: SchoolId, name: &str) -> Result<Option<NewsCategory>>;
    async fn insert(&self, c: &NewsCategory) -> Result<()>;
    async fn update(&self, c: &NewsCategory) -> Result<()>;
    async fn delete(&self, id: NewsCategoryId) -> Result<()>;
}
```

## NewsPageRepository

```rust
#[async_trait]
pub trait NewsPageRepository: Send + Sync {
    async fn get(&self, id: NewsPageId) -> Result<Option<NewsPage>>;
    async fn find_active(&self, school: SchoolId) -> Result<Option<NewsPage>>;
    async fn insert(&self, p: &NewsPage) -> Result<()>;
    async fn update(&self, p: &NewsPage) -> Result<()>;
    async fn delete(&self, id: NewsPageId) -> Result<()>;
}
```

## NoticeBoardRepository

```rust
#[async_trait]
pub trait NoticeBoardRepository: Send + Sync {
    async fn get(&self, id: NoticeBoardId) -> Result<Option<NoticeBoard>>;
    async fn list(&self, school: SchoolId, q: NoticeBoardQuery) -> Result<Vec<NoticeBoard>>;
    async fn list_published(&self, school: SchoolId) -> Result<Vec<NoticeBoard>>;
    async fn list_between(&self, school: SchoolId, from: NaiveDate, to: NaiveDate) -> Result<Vec<NoticeBoard>>;
    async fn insert(&self, n: &NoticeBoard) -> Result<()>;
    async fn update(&self, n: &NoticeBoard) -> Result<()>;
    async fn delete(&self, id: NoticeBoardId) -> Result<()>;
}
```

## TestimonialRepository

```rust
#[async_trait]
pub trait TestimonialRepository: Send + Sync {
    async fn get(&self, id: TestimonialId) -> Result<Option<Testimonial>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<Testimonial>>;
    async fn insert(&self, t: &Testimonial) -> Result<()>;
    async fn update(&self, t: &Testimonial) -> Result<()>;
    async fn delete(&self, id: TestimonialId) -> Result<()>;
}
```

## HomeSliderRepository

```rust
#[async_trait]
pub trait HomeSliderRepository: Send + Sync {
    async fn get(&self, id: HomeSliderId) -> Result<Option<HomeSlider>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<HomeSlider>>;
    async fn insert(&self, s: &HomeSlider) -> Result<()>;
    async fn update(&self, s: &HomeSlider) -> Result<()>;
    async fn delete(&self, id: HomeSliderId) -> Result<()>;
}
```

## SpeechSliderRepository

```rust
#[async_trait]
pub trait SpeechSliderRepository: Send + Sync {
    async fn get(&self, id: SpeechSliderId) -> Result<Option<SpeechSlider>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<SpeechSlider>>;
    async fn insert(&self, s: &SpeechSlider) -> Result<()>;
    async fn update(&self, s: &SpeechSlider) -> Result<()>;
    async fn delete(&self, id: SpeechSliderId) -> Result<()>;
}
```

## ContentRepository

```rust
#[async_trait]
pub trait ContentRepository: Send + Sync {
    async fn get(&self, id: ContentId) -> Result<Option<Content>>;
    async fn list(&self, school: SchoolId, q: ContentQuery) -> Result<Vec<Content>>;
    async fn list_by_type(&self, school: SchoolId, type_id: ContentTypeId) -> Result<Vec<Content>>;
    async fn list_for_role(&self, school: SchoolId, role: RoleId) -> Result<Vec<Content>>;
    async fn list_for_class(&self, school: SchoolId, class: ClassId, section: Option<SectionId>) -> Result<Vec<Content>>;
    async fn insert(&self, c: &Content) -> Result<()>;
    async fn update(&self, c: &Content) -> Result<()>;
    async fn delete(&self, id: ContentId) -> Result<()>;
}
```

## ContentTypeRepository

```rust
#[async_trait]
pub trait ContentTypeRepository: Send + Sync {
    async fn get(&self, id: ContentTypeId) -> Result<Option<ContentType>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<ContentType>>;
    async fn find_by_name(&self, school: SchoolId, name: &str) -> Result<Option<ContentType>>;
    async fn insert(&self, t: &ContentType) -> Result<()>;
    async fn update(&self, t: &ContentType) -> Result<()>;
    async fn delete(&self, id: ContentTypeId) -> Result<()>;
}
```

## ContentShareListRepository

```rust
#[async_trait]
pub trait ContentShareListRepository: Send + Sync {
    async fn get(&self, id: ContentShareListId) -> Result<Option<ContentShareList>>;
    async fn list(&self, school: SchoolId, q: ContentShareListQuery) -> Result<Vec<ContentShareList>>;
    async fn list_active(&self, school: SchoolId, on: NaiveDate) -> Result<Vec<ContentShareList>>;
    async fn insert(&self, l: &ContentShareList) -> Result<()>;
    async fn update(&self, l: &ContentShareList) -> Result<()>;
    async fn delete(&self, id: ContentShareListId) -> Result<()>;
}
```

## TeacherUploadContentRepository

```rust
#[async_trait]
pub trait TeacherUploadContentRepository: Send + Sync {
    async fn get(&self, id: TeacherUploadContentId) -> Result<Option<TeacherUploadContent>>;
    async fn list(&self, school: SchoolId, q: TeacherUploadContentQuery) -> Result<Vec<TeacherUploadContent>>;
    async fn list_for_class(&self, class: ClassId, section: SectionId) -> Result<Vec<TeacherUploadContent>>;
    async fn list_for_teacher(&self, teacher: UserId) -> Result<Vec<TeacherUploadContent>>;
    async fn insert(&self, c: &TeacherUploadContent) -> Result<()>;
    async fn update(&self, c: &TeacherUploadContent) -> Result<()>;
    async fn delete(&self, id: TeacherUploadContentId) -> Result<()>;
}
```

## UploadContentRepository

```rust
#[async_trait]
pub trait UploadContentRepository: Send + Sync {
    async fn get(&self, id: UploadContentId) -> Result<Option<UploadContent>>;
    async fn list(&self, school: SchoolId, q: UploadContentQuery) -> Result<Vec<UploadContent>>;
    async fn list_for_role(&self, school: SchoolId, role: RoleId) -> Result<Vec<UploadContent>>;
    async fn list_for_class(&self, school: SchoolId, class: ClassId, section: Option<SectionId>) -> Result<Vec<UploadContent>>;
    async fn insert(&self, c: &UploadContent) -> Result<()>;
    async fn update(&self, c: &UploadContent) -> Result<()>;
    async fn delete(&self, id: UploadContentId) -> Result<()>;
}
```

## AboutPageRepository / ContactPageRepository / CoursePageRepository / HomePageSettingRepository / FrontendPageRepository

Each follows the same pattern: `get`, `find_active` (where the
aggregate is unique per school), `insert`, `update`, `delete`, plus
domain-specific queries.

## Indexes (recommended)

The default PostgreSQL adapter documents the following indexes; consumers
should declare them in their migrations:

```sql
CREATE INDEX ix_pages_school_id_status ON infixedu__pages (school_id, status);
CREATE INDEX ix_pages_school_id_slug ON infixedu__pages (school_id, slug);
CREATE INDEX ix_pages_school_id_home ON infixedu__pages (school_id, home_page) WHERE home_page = 1;
CREATE INDEX ix_news_school_id_category ON sm_news (school_id, category_id);
CREATE INDEX ix_news_school_id_publish ON sm_news (school_id, publish_date);
CREATE INDEX ix_news_school_id_active ON sm_news (school_id, active_status);
CREATE INDEX ix_news_school_id_global ON sm_news (is_global) WHERE is_global = 1;
CREATE INDEX ix_news_comments_news_id ON sm_news_comments (news_id);
CREATE INDEX ix_news_comments_user_id ON sm_news_comments (user_id);
CREATE INDEX ix_news_categories_school_id_name ON sm_news_categories (school_id, category_name);
CREATE INDEX ix_notice_boards_school_id_publish ON sm_notice_boards (school_id, publish_on);
CREATE INDEX ix_notice_boards_school_id_published ON sm_notice_boards (school_id, is_published);
CREATE INDEX ix_testimonials_school_id ON sm_testimonials (school_id);
CREATE INDEX ix_home_sliders_school_id ON home_sliders (school_id);
CREATE INDEX ix_speech_sliders_school_id ON speech_sliders (school_id);
CREATE INDEX ix_contents_school_id_type ON contents (school_id, content_type_id);
CREATE INDEX ix_contents_school_id_academic ON contents (school_id, academic_id);
CREATE INDEX ix_content_share_lists_school_id ON content_share_lists (school_id, share_date);
CREATE INDEX ix_teacher_upload_contents_school_id_class ON sm_teacher_upload_contents (school_id, class, section);
CREATE INDEX ix_teacher_upload_contents_school_id_academic ON sm_teacher_upload_contents (school_id, academic_id);
CREATE INDEX ix_upload_contents_school_id ON sm_upload_contents (school_id, content_type);
CREATE INDEX ix_course_pages_school_id_parent ON sm_course_pages (school_id, is_parent);
CREATE INDEX ix_pages_school_id_sub_title ON sm_pages (school_id, sub_title);
```

The `school_id` predicate is mandatory for tenant isolation.
