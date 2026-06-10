# Documents Domain — Value Objects

Value objects are immutable, validated at construction, and have no
identity. They are compared by value.

## Identifiers

All identifiers in the documents domain are typed and tenant-scoped.
The generic `Id<S, T>` wrapper carries the `SchoolId` of the owning
school and the local id (`Uuid`).

| Identifier           | Backing Type             | Source Column                  |
| -------------------- | ------------------------ | ------------------------------ |
| `FormDownloadId`     | `Id<FormDownload>`       | `documents_form_downloads.id`         |
| `PostalDispatchId`   | `Id<PostalDispatch>`     | `documents_postal_dispatches.id`      |
| `PostalReceiveId`    | `Id<PostalReceive>`      | `documents_postal_receives.id`        |

## Names and Free Text

| Type                  | Constraints                                                       |
| --------------------- | ----------------------------------------------------------------- |
| `FormTitle`           | 1..191 chars                                                      |
| `FormDescription`     | 1..200 chars                                                      |
| `PostalTitle`         | 1..191 chars (used for both `to_title` and `from_title`)          |
| `PostalNote`          | 1..5000 chars                                                     |
| `PostalReferenceNo`   | 1..191 chars, unique within `(school_id, academic_id)`           |

## Addresses and Parties

| Type                  | Notes                                                              |
| --------------------- | ------------------------------------------------------------------ |
| `PostalAddress`       | 1..191 chars                                                       |
| `FromAddress`         | `PostalAddress` — the sender's address                             |
| `ToAddress`           | `PostalAddress` — the recipient's address                          |
| `FromTitle`           | `PostalTitle` — the sender's name/title                            |
| `ToTitle`             | `PostalTitle` — the recipient's name/title                         |

## Document Type

| Type                  | Values                                                              |
| --------------------- | ------------------------------------------------------------------- |
| `DocumentType`        | `Form`, `PostalDispatch`, `PostalReceive`                           |
| `DocumentVisibility`  | `Public`, `Staff`                                                   |

## Time and Schedule

| Type                  | Notes                                                              |
| --------------------- | ------------------------------------------------------------------ |
| `PublishDate`         | `NaiveDate`                                                        |
| `DispatchDate`        | `NaiveDate`                                                        |
| `ReceiveDate`         | `NaiveDate`                                                        |
| `AcademicYearId`      | From `educore-academic`                                            |
| `TenantContext`       | `(SchoolId, UserId, ...)` from `educore-platform`                 |

## URL and File

| Type                 | Notes                                                          |
| -------------------- | -------------------------------------------------------------- |
| `Url`                | Validated URL, max 2048 chars                                  |
| `FileReference`      | From `educore-platform`                                        |

## Visibility

| Type                  | Notes                                                              |
| --------------------- | ------------------------------------------------------------------ |
| `ShowPublic`          | `bool` — when `true`, the form is visible on the public site       |
| `ActiveStatus`        | `bool` — soft-delete flag; `false` means the row is archived        |

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
let title = FormTitle::new("Parent Consent Form 2026")?;
```

Parsing returns `Result<FormTitle, ValueError>`. There are no setters
that bypass validation.
