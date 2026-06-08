# CMS Domain — Workflows

Workflows orchestrate commands, queries, and policies to fulfill a
business goal. They are documented as ordered, conditional steps.

## Page Publishing Workflow

```text
1. Author drafts a page (CreatePage) with title, slug, body, settings.
2. Author updates the page (UpdatePage) on revisions.
3. Author publishes (PublishPage) when ready.
4. The public-site port adapter re-renders.
5. Author may archive (ArchivePage) to take the page down without
   deleting history.
6. Author or admin may delete (DeletePage) when superseded.
7. Default pages (is_default = true) cannot be deleted.
```

**Pre-conditions:**
- The `slug` is unique within the school.
- At most one page per school has `home_page = true`.

**Failure paths:**
- Duplicate slug → `ValidationError::UniqueViolation`.
- Two pages with `home_page = true` → `ConflictError::HomePageAlreadySet`.

## News Lifecycle Workflow

```text
1. Author creates a news entry (CreateNews) with title, category, body,
   publish date, image.
2. Author updates the entry (UpdateNews) on revisions.
3. Author publishes (PublishNews).
4. Public visitors read the news on the public site; view count
   increments (IncrementNewsView).
5. Visitors comment on the news (CommentOnNews):
   a. When auto_approve = true, the comment is approved immediately.
   b. When auto_approve = false, the comment is queued for moderation.
6. Moderator approves or hides the comment (ModerateNewsComment).
7. Author or admin may unpublish (UnpublishNews) or delete (DeleteNews).
```

**Edge cases:**
- A news with `is_comment = false` rejects `CommentOnNews`.
- A news with `is_global = true` is visible across all schools in a
  multi-tenant SaaS.

## Content Sharing Workflow

```text
1. SchoolAdmin creates a content share list (CreateContentShareList)
   with content ids, audience, share date, and valid_upto.
2. SchoolAdmin dispatches the list (DispatchContentShareList).
3. The recipient set is frozen at dispatch.
4. The communication domain may subscribe and dispatch notifications.
5. SchoolAdmin may cancel (CancelContentShareList) a not-yet-dispatched
   list.
6. After valid_upto, the list is no longer visible to recipients.
```

## Testimonial Curation Workflow

```text
1. Marketing or principal authors a testimonial (CreateTestimonial)
   with name, designation, institution, image, description, and
   star_rating.
2. The testimonial appears on the public site testimonial widget.
3. Author updates (UpdateTestimonial) on rotation.
4. Author deletes (DeleteTestimonial) when the testimonial is removed.
```

## News Comment Moderation Workflow

```text
1. Visitor submits a comment (CommentOnNews).
2. If auto_approve = false, the comment enters Pending status.
3. Moderator reviews the comment and either approves
   (ModerateNewsComment → Approve) or hides (ModerateNewsComment →
   Hide).
4. Approved comments are visible on the public site.
5. Hidden comments are retained for audit but not surfaced.
6. Moderator may delete (DeleteNewsComment) for legal or compliance
   reasons.
```

## Content Upload Workflow

### Teacher

```text
1. Teacher uploads content (CreateTeacherUploadContent) for a
   class-section.
2. The content is visible to the class-section students.
3. Teacher updates (UpdateTeacherUploadContent) on revisions.
4. Teacher deletes (DeleteTeacherUploadContent) when the content is
   no longer needed.
```

### Admin

```text
1. SchoolAdmin uploads content (CreateUploadContent) for a role,
   class, and section.
2. The content is visible to the matching role / class / section.
3. SchoolAdmin updates (UpdateUploadContent) on revisions.
4. SchoolAdmin deletes (DeleteUploadContent) when the content is
   no longer needed.
```

## Home Page Configuration Workflow

```text
1. Marketing or principal configures the home page
   (ConfigureHomePage) with title, long title, short description,
   link label, link URL, and image.
2. The home page is re-rendered with the new configuration.
3. The configuration may be updated (re-invoke ConfigureHomePage)
   or deleted (DeleteHomePageSetting).
```

## Home Slider Rotation Workflow

```text
1. Marketing creates a slider entry (CreateHomeSlider) with an image
   and optional link.
2. The slider rotates on the public site.
3. Marketing updates the slider (UpdateHomeSlider) on rotation.
4. Marketing deletes (DeleteHomeSlider) when the slider is removed.
```

## Speech Slider Workflow

```text
1. Marketing or principal authors a speech slider (CreateSpeechSlider).
2. The slider appears on the public site home page.
3. Author updates (UpdateSpeechSlider) and removes
   (DeleteSpeechSlider) on rotation.
```

## Course Page Workflow

```text
1. Marketing or principal creates a parent course page
   (CreateCoursePage with is_parent = true).
2. Marketing creates child course pages (CreateCoursePage with
   parent_id set) under the parent.
3. The course hierarchy is rendered on the public site.
4. Marketing updates (UpdateCoursePage) and deletes (DeleteCoursePage)
   on rotation.
5. Deleting a parent course page is rejected if child course pages
   reference it.
```

## About / Contact / News Page Workflow

```text
1. Marketing configures the about page (CreateAboutPage /
   UpdateAboutPage) with title, description, image, button text, and
   button URL.
2. Marketing configures the contact page (CreateContactPage /
   UpdateContactPage) with address, phone, email, map coordinates.
3. Marketing configures the news page (CreateNewsPage /
   UpdateNewsPage) with title, description, image, button text,
   button URL.
4. The pages are rendered on the public site.
5. Marketing deletes (DeleteAboutPage / DeleteContactPage /
   DeleteNewsPage) when the page is removed.
```

## Idempotency

- `CreatePage` is **not** idempotent on title. Two pages with the
  same title are distinct; the `slug` is the unique key.
- `CreateNews` is **not** idempotent.
- `CreateContent` is **not** idempotent.
- `DispatchContentShareList` is **not** idempotent: re-dispatch
  produces duplicate notifications.
- `CreateHomeSlider` is **not** idempotent; each slider is distinct.

## Audit Requirements

Every state-changing command writes a durable audit record with the
actor, the correlation id, and a hash of the payload. Public-site
renders are surfaced through port adapters, not domain events;
audit captures the publish, archive, and delete commands, not the
subsequent render.

Page bodies and news bodies are versioned through `PageRevision` and
`NewsRevision` entities to allow revert.
