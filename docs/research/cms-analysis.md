# CMS Domain — Business Analysis

## Purpose

The CMS (Content Management System) domain owns the
school's public-facing website content. The CMS
operates in a separate context from the operational
data (academic, finance, etc.) but is owned by the
same school.

This document describes how a school's public website
content is managed in real schools, with the edge
cases that real schools hit.

## Key Concepts

- **Page** — a content page on the school's website
  (About, Admissions, Academics, etc.).
- **NewsArticle** — a news post (announcement, event
  coverage, achievement).
- **EventPage** — a public-facing event listing.
- **Gallery** — a photo or video gallery.
- **Menu** — the navigation structure of the
  website.
- **Media** — uploaded files (images, videos,
  documents).
- **Slider** — a homepage image slider.
- **Testimonial** — a parent or student testimonial.
- **FAQ** — a frequently-asked question and answer.

## Real-World Scenarios

### Page Management

A school website has a set of static pages:
- About Us
- Admissions
- Academics
- Facilities
- Faculty
- Contact Us

The school's admin creates these pages in the CMS.
Each page has:
- A title.
- A slug (URL-friendly identifier).
- A body (rich text or HTML).
- A status (`Draft`, `Published`, `Archived`).
- A publication date.
- A meta description (for SEO).
- An optional featured image.

The engine's `Page` aggregate captures this. The
consumer's web frontend renders the page from the
engine's query result.

### News and Announcements

The school publishes news regularly:
- "Annual Day celebration on December 15."
- "School wins inter-school debate competition."
- "Holiday notice: school closed on Monday."

A `NewsArticle` has the same fields as a `Page`,
plus:
- A category (announcement, achievement, event).
- An optional expiry date (auto-archive after).
- An optional featured flag (for the homepage).

### Event Pages

The school has a public calendar of upcoming events:
- Open house.
- Sports day.
- Annual day.
- Parent-teacher meeting.

A `EventPage` has:
- A title.
- A description.
- A start date / time.
- An end date / time.
- A location.
- A category.
- An optional image.

The engine's `EventPage` is **public-facing**. It
is different from the operational `Event` aggregate
in the events domain, which tracks internal school
events (staff meetings, holidays, etc.).

### Photo and Video Galleries

The school uploads photos from events and videos of
performances. A `Gallery` has:
- A title.
- A date.
- A category (sports, cultural, academic).
- A list of media items.

The media storage is a port-driven concern (S3, GCS,
local filesystem). The engine's `Media` aggregate
records the metadata; the actual files are in the
consumer's storage.

### Menu Management

The website has a navigation menu. The admin
configures the menu:
- Main menu (top of the page).
- Footer menu.
- Sidebar menu (for sub-pages).

The engine's `Menu` aggregate captures the menu
structure. The menu items are references to
`Page`s or external URLs.

### Homepage Slider

The homepage has a hero slider with 3-5 images.
The engine's `Slider` aggregate captures the
slider configuration. The slider is rendered by
the consumer's web frontend.

### Testimonials

A school's website shows testimonials from parents
and students. The engine's `Testimonial` aggregate
captures the testimonial:
- The author (name, role, photo).
- The text.
- An optional rating.
- A status (`Pending`, `Approved`, `Rejected`).

The testimonials are moderated before publication.

### FAQ

A school's website has a list of frequently-asked
questions. The engine's `FAQ` aggregate captures
the question and answer.

## Business Rules

1. A `Page`'s slug is unique within the school.
2. A `Page` in `Draft` status is not visible on the
   public website.
3. A `Page` cannot be deleted if it has child pages
   or menu items; it is archived instead.
4. A `NewsArticle`'s expiry date, if set, archives
   the article automatically at midnight on that
   date.
5. A `Gallery` cannot be published if it has no
   media items.
6. A `Menu` item cannot link to a `Page` that does
   not exist.
7. A `Testimonial` in `Pending` status is not
   visible; the admin must approve it.
8. A `Media` item is private to the school; the
   engine does not expose it across schools.

## Edge Cases

### Page with Same Slug in Different Sections

A school has "About Us" as a top-level page and
"About Our Founder" as a sub-page. The slugs are
`/about` and `/about/founder`. The engine
supports nested pages with parent-child
relationships.

### Page Translated to Multiple Languages

A school serves a multilingual community. The
admin translates the page to two languages. The
engine's `Page` aggregate supports per-language
content; the slug is per-language. The
consumer's frontend chooses the language based
on the user's preference.

### Media Upload Failure

An admin uploads a 50MB image. The upload fails
(network timeout). The engine's `Media`
aggregate records the failure with a retry
mechanism. The admin re-uploads.

### Bulk Page Import

A school migrates from another CMS. The admin
uploads a 200-page export. The engine's bulk
import command is all-or-nothing; a single
validation failure aborts the import. The admin
fixes the export and re-imports.

### Page Linked from Multiple Menus

A page is linked from both the main menu and
the footer. The engine allows multiple
references; deleting the page is blocked until
all references are removed.

### Image with Sensitive Content

An admin accidentally uploads an image with
student faces. The admin deletes the image. The
engine's `Media` aggregate soft-deletes (the
image is no longer public but the record is
retained for audit).

### Public Page with Draft Content

An editor publishes a page with a typo. The
admin reverts to draft, fixes the typo, and
re-publishes. The engine's audit log captures
every state transition.

### Old Event Page Still Indexed

A past event page is still indexed by Google.
The school wants to remove it. The engine's
`Page` archive removes the page from public
view but retains a 404 redirect for SEO.

### Photo Gallery Performance

A gallery with 500 photos loads slowly. The
engine's `Gallery` aggregate supports pagination
and lazy loading; the consumer's frontend
implements the rendering.

## Notes for SMSengine Implementation

- The **cms** crate depends on
  `smscore-platform` for `SchoolId` and
  `CustomField`. It does not depend on the
  operational domains (academic, finance, etc.).
- The CMS is **public-facing**. The engine's
  read-side queries for the CMS may be served
  by a public read-only API; no authentication
  is required for published content.
- The CMS's media is **storage-port driven**.
  The engine's `Media` aggregate records the
  metadata; the actual files are in the
  consumer's storage (S3, GCS, local).
- The CMS's pages are **versioned**. The
  engine's `Page` aggregate supports revision
  history; a page can be reverted to a
  previous version.
- The CMS's slugs are **per-school
  unique**. A `School` has its own slug
  namespace.
- The CMS's menus are **rendered by the
  consumer**. The engine's `Menu` aggregate
  is the configuration; the frontend
  consumes it.
- The CMS's testimonials are **moderated**.
  The engine's status workflow is
  `Pending → Approved | Rejected`. The
  `Approved` testimonials are public; the
  `Pending` and `Rejected` are admin-only.
- The CMS's news is **optionally
  scheduled**. The engine supports a
  scheduled publication date; a background
  job publishes at the scheduled time.
- The CMS's galleries support **multiple
  media types**: image, video, document.
  The engine's `Media` aggregate has a
  `media_type` field.
- The CMS's events are **separate from
  the operational `Event` aggregate** in
  the events domain. The CMS's events are
  public-facing; the events domain's events
  are internal school events.
