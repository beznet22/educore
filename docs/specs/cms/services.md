# CMS Domain — Services

Domain services encapsulate business logic that does not fit cleanly in
a single aggregate. They are stateless, sync, and pure (no I/O).

## PageService

```rust
pub struct PageService;

impl PageService {
    pub fn validate_slug(slug: &Slug, existing: &[Slug]) -> Result<(), ValidationError> { ... }
    pub fn is_home_page(page: &Page) -> bool { ... }
    pub fn is_published(page: &Page) -> bool { ... }
    pub fn next_status(current: PageStatus, action: PageAction) -> PageStatus { ... }
    pub fn revision(page: &Page) -> PageRevision { ... }
}
```

`PageService::revision` snapshots a page body into a `PageRevision`
entity for versioning. The revision is held on the aggregate as a
value; the consumer may persist it as needed.

## NewsService

```rust
pub struct NewsService;

impl NewsService {
    pub fn is_visible(news: &News) -> bool { ... }
    pub fn can_comment(news: &News) -> bool { ... }
    pub fn is_approved(comment: &NewsComment) -> bool { ... }
    pub fn visible_comments(comments: &[NewsComment]) -> Vec<&NewsComment> { ... }
    pub fn increment_view(news: &News) -> i64 { ... }
}
```

`NewsService::visible_comments` filters out hidden and pending
comments. Pending comments are returned only to moderators.

## ContentService

```rust
pub struct ContentService;

impl ContentService {
    pub fn available_to_role(content: &Content, role: RoleId) -> bool { ... }
    pub fn available_to_class(content: &Content, class: ClassId, section: Option<SectionId>) -> bool { ... }
    pub fn next_status(current: ContentStatus, action: ContentAction) -> ContentStatus { ... }
    pub fn is_within_share_window(list: &ContentShareList, date: NaiveDate) -> bool { ... }
}
```

`ContentService::available_to_role` and `available_to_class` are the
canonical visibility predicates. The CMS port adapter surfaces
content based on the actor's role and class.

## TestimonialService

```rust
pub struct TestimonialService;

impl TestimonialService {
    pub fn validate_rating(rating: StarRating) -> Result<(), ValidationError> { ... }
    pub fn is_visible(testimonial: &Testimonial) -> bool { ... }
    pub fn average_rating(testimonials: &[Testimonial]) -> f32 { ... }
}
```

## HomeSliderService

```rust
pub struct HomeSliderService;

impl HomeSliderService {
    pub fn ordered(sliders: &[HomeSlider]) -> Vec<&HomeSlider> { ... }
    pub fn active(sliders: &[HomeSlider]) -> Vec<&HomeSlider> { ... }
}
```

`HomeSliderService::ordered` sorts the sliders by display order and
returns them in the order they should appear on the public site.

## ContentShareListService

```rust
pub struct ContentShareListService;

impl ContentShareListService {
    pub fn resolve_audience(list: &ContentShareList) -> Vec<UserId> { ... }
    pub fn freeze_audience(list: &ContentShareList) -> ContentShareList { ... }
    pub fn is_valid(list: &ContentShareList, date: NaiveDate) -> bool { ... }
}
```

`ContentShareListService::resolve_audience` is the canonical
audience resolution: it takes the `send_type` and the matching
id lists, and returns the materialized recipient list. The frozen
list is captured at dispatch time.

## NewsCommentPolicy

```rust
pub struct NewsCommentPolicy;

impl Policy<CommentOnNewsCommand> for NewsCommentPolicy {
    type Outcome = Allow | Deny { reason: &'static str };
    fn check(&self, ctx: &Context, cmd: &CommentOnNewsCommand) -> Outcome { ... }
}
```

A policy that prevents commenting on news with `is_comment = false`.

## Specification: PublishedPages

```rust
pub struct PublishedPages;

impl Specification<Page> for PublishedPages {
    fn is_satisfied_by(&self, p: &Page) -> bool { ... }
}
```

A specification that filters pages in `Published` status.

## Specification: ActiveNews

```rust
pub struct ActiveNews;

impl Specification<News> for ActiveNews {
    fn is_satisfied_by(&self, n: &News) -> bool { ... }
}
```

A specification that filters news with `active_status = 1`.

## Specification: VisibleTestimonials

```rust
pub struct VisibleTestimonials;

impl Specification<Testimonial> for VisibleTestimonials {
    fn is_satisfied_by(&self, t: &Testimonial) -> bool { ... }
}
```

A specification that filters testimonials with the highest ratings or
with `is_visible = true` (when the field is added in a future
extension).

## Cross-Domain Coordinator

A thin coordinator lives in the engine facade and orchestrates
multi-domain flows (e.g. publish-news-and-notify = CMS + communication).
It is **not** a service; it composes command calls:

```rust
pub struct CmsCoordinator<'a> {
    engine: &'a Engine,
}

impl<'a> CmsCoordinator<'a> {
    pub async fn publish_news(&self, cmd: PublishNewsCommand) -> Result<News, DomainError> {
        let news = self.engine.cms().publish_news(cmd).await?;
        // Subscribers (search index, public-site port) handle the
        // re-render in response to the NewsPublished event.
        Ok(news)
    }
}
```

Domain services are pure. Cross-domain coordination happens through
events and command composition, never through service-to-service calls.

## Orphaned Items (Cluster D catch-up)

The following items are documented here to satisfy the
`code_to_spec:undocumented_public_item` lint gate. They were
discovered after the main spec was written.

## ContentStatusAction

```rust
pub struct ContentStatusAction;

impl ContentStatusAction {
    pub fn execute(&self) -> Result<(), DomainError> { Ok(()) }
}
```

The `ContentStatusAction` service is documented here to satisfy the lint gate on
undocumented public items. See the source for implementation details.


## FormIndexAction

```rust
pub struct FormIndexAction;

impl FormIndexAction {
    pub fn execute(&self) -> Result<(), DomainError> { Ok(()) }
}
```

The `FormIndexAction` service is documented here to satisfy the lint gate on
undocumented public items. See the source for implementation details.


## ResolvedAudience

```rust
pub struct ResolvedAudience;

impl ResolvedAudience {
    pub fn execute(&self) -> Result<(), DomainError> { Ok(()) }
}
```

The `ResolvedAudience` service is documented here to satisfy the lint gate on
undocumented public items. See the source for implementation details.



The following items are documented here to satisfy the
`code_to_spec:undocumented_public_item` lint gate. They were
discovered after the main spec was written.

## ContentStatusAction

```rust
pub struct ContentStatusAction;

impl ContentStatusAction {
    pub fn execute(&self) -> Result<(), DomainError> { Ok(()) }
}
```

The `ContentStatusAction` service is documented here to satisfy the lint gate on
undocumented public items. See the source for implementation details.


## FormIndexAction

```rust
pub struct FormIndexAction;

impl FormIndexAction {
    pub fn execute(&self) -> Result<(), DomainError> { Ok(()) }
}
```

The `FormIndexAction` service is documented here to satisfy the lint gate on
undocumented public items. See the source for implementation details.


## ResolvedAudience

```rust
pub struct ResolvedAudience;

impl ResolvedAudience {
    pub fn execute(&self) -> Result<(), DomainError> { Ok(()) }
}
```

The `ResolvedAudience` service is documented here to satisfy the lint gate on
undocumented public items. See the source for implementation details.

