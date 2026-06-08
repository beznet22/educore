# CMS Domain — Permissions

Permissions are capability strings. They are not roles. The RBAC domain
maps capabilities to roles.

## Naming

```text
<Domain>.<Aggregate>.<Action>
```

## Capabilities

### CMS (Cross-Cutting)

- `CMS.Read`

### Page

- `Page.Create`
- `Page.Update`
- `Page.Publish`
- `Page.Archive`
- `Page.Delete`
- `Page.Read`

### News

- `News.Create`
- `News.Update`
- `News.Publish`
- `News.Unpublish`
- `News.Delete`
- `News.Read`

### News Comment

- `NewsComment.Create`
- `NewsComment.Moderate`
- `NewsComment.Delete`
- `NewsComment.Read`

### News Category

- `NewsCategory.Create`
- `NewsCategory.Update`
- `NewsCategory.Delete`
- `NewsCategory.Read`

### News Page

- `NewsPage.Create`
- `NewsPage.Update`
- `NewsPage.Delete`
- `NewsPage.Read`

### Notice Board

- `NoticeBoard.Create`
- `NoticeBoard.Update`
- `NoticeBoard.Publish`
- `NoticeBoard.Unpublish`
- `NoticeBoard.Delete`
- `NoticeBoard.Read`

### Testimonial

- `Testimonial.Create`
- `Testimonial.Update`
- `Testimonial.Delete`
- `Testimonial.Read`

### Home Slider

- `HomeSlider.Create`
- `HomeSlider.Update`
- `HomeSlider.Delete`
- `HomeSlider.Read`

### Speech Slider

- `SpeechSlider.Create`
- `SpeechSlider.Update`
- `SpeechSlider.Delete`
- `SpeechSlider.Read`

### Content

- `Content.Create`
- `Content.Update`
- `Content.Delete`
- `Content.Read`

### Content Type

- `ContentType.Create`
- `ContentType.Update`
- `ContentType.Delete`
- `ContentType.Read`

### Content Share List

- `ContentShareList.Create`
- `ContentShareList.Dispatch`
- `ContentShareList.Cancel`
- `ContentShareList.Delete`
- `ContentShareList.Read`

### Teacher Upload Content

- `TeacherUploadContent.Create`
- `TeacherUploadContent.Update`
- `TeacherUploadContent.Delete`
- `TeacherUploadContent.Read`

### Upload Content

- `UploadContent.Create`
- `UploadContent.Update`
- `UploadContent.Delete`
- `UploadContent.Read`

### About Page

- `AboutPage.Create`
- `AboutPage.Update`
- `AboutPage.Delete`
- `AboutPage.Read`

### Contact Page

- `ContactPage.Create`
- `ContactPage.Update`
- `ContactPage.Delete`
- `ContactPage.Read`

### Course Page

- `CoursePage.Create`
- `CoursePage.Update`
- `CoursePage.Delete`
- `CoursePage.Read`

### Home Page Setting

- `HomePageSetting.Configure`
- `HomePageSetting.Read`
- `HomePageSetting.Delete`

### Frontend Page

- `FrontendPage.Create`
- `FrontendPage.Update`
- `FrontendPage.Delete`
- `FrontendPage.Read`

## Default Role Mapping

The platform's default role catalog binds the following:

| Role        | Capabilities (highlights)                                                       |
| ----------- | ------------------------------------------------------------------------------- |
| SuperAdmin  | All                                                                             |
| SchoolAdmin | All within the school                                                          |
| Teacher     | Page.Read, News.Create, News.Update, News.Read, NewsComment.Moderate, Content.Create, Content.Read, TeacherUploadContent.*, HomePageSetting.Read, CoursePage.Read |
| Student     | News.Read, Content.Read (when available), TeacherUploadContent.Read            |
| Parent      | News.Read, Content.Read (when available)                                        |
| Marketing   | Page.*, Testimonial.*, HomeSlider.*, SpeechSlider.*, HomePageSetting.*        |
| Public      | Page.Read (when `status = published`), News.Read (when `active_status = 1`)    |

The default mapping is a starting point and is configurable per school.

## Authorization Pattern

Capabilities are checked at the command boundary. The engine never
trusts the caller to assert their own role.

```rust
if !engine.rbac().has(actor_id, Capability::PagePublish).await? {
    return Err(DomainError::forbidden("missing capability"));
}
```

Some commands have a secondary ownership check: a teacher creating
content in their own class-section is allowed; a teacher moderating
a news comment requires `NewsComment.Moderate`.

## Read vs Write

Read capabilities are explicit. The engine does not assume that
`Page.Read` implies `Page.Create`. A consumer may grant only
read-only access to a parent or auditor. The `Public` role is a
special read-only role granted to anonymous visitors; it is paired
with status checks at the application boundary (a page in `Draft`
status is not visible to the public role).

## Tenant Isolation

Every capability check is paired with a tenant check. The actor must
be authenticated to the school that owns the target aggregate. There
is no cross-tenant capability elevation. The exception is
`is_global = true` news, which is visible across all schools; this
visibility is enforced by the public-site port adapter, not the
domain layer.
