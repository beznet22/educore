# Documents Domain — Services

Domain services encapsulate business logic that does not fit cleanly in
a single aggregate. They are stateless, sync, and pure (no I/O).

## FormService

```rust
pub struct FormService;

impl FormService {
    pub fn validate_content(link: Option<&Url>, file: Option<&FileReference>) -> Result<(), ValidationError> { ... }
    pub fn is_public(form: &FormDownload) -> bool { ... }
    pub fn is_deliverable(form: &FormDownload) -> bool { ... }
    pub fn matches_publish_date(form: &FormDownload, date: NaiveDate) -> bool { ... }
}
```

`FormService::validate_content` enforces the "at least one of link or
file" rule.

## PostalService

```rust
pub struct PostalService;

impl PostalService {
    pub fn reference_unique(reference: &PostalReferenceNo, existing: &[PostalReferenceNo]) -> bool { ... }
    pub fn pair_by_reference(dispatches: &[PostalDispatch], receives: &[PostalReceive]) -> Vec<PostalPair> { ... }
    pub fn within_year(dispatches: &[PostalDispatch], receives: &[PostalReceive], year: AcademicYearId) -> Vec<PostalReference> { ... }
    pub fn format_address(addr: &PostalAddress) -> String { ... }
}
```

`PostalService::pair_by_reference` is the canonical pairing algorithm:
it returns the list of `(dispatch, receive)` tuples that share a
reference number. The pair is a derived value, not a stored relation.

## Specification: PublicForms

```rust
pub struct PublicForms;

impl Specification<FormDownload> for PublicForms {
    fn is_satisfied_by(&self, f: &FormDownload) -> bool { ... }
}
```

A specification that filters forms with `show_public = true`.

## Specification: ActiveForms

```rust
pub struct ActiveForms;

impl Specification<FormDownload> for ActiveForms {
    fn is_satisfied_by(&self, f: &FormDownload) -> bool { ... }
}
```

A specification that filters forms with `active_status = true`.

## Specification: DispatchesInDateRange

```rust
pub struct DispatchesInDateRange {
    pub from: NaiveDate,
    pub to: NaiveDate,
}

impl Specification<PostalDispatch> for DispatchesInDateRange {
    fn is_satisfied_by(&self, d: &PostalDispatch) -> bool { ... }
}
```

A specification that filters dispatches whose `date` falls within the
given range. Composed with other filters in queries.

## Specification: ReceivesInDateRange

```rust
pub struct ReceivesInDateRange {
    pub from: NaiveDate,
    pub to: NaiveDate,
}

impl Specification<PostalReceive> for ReceivesInDateRange {
    fn is_satisfied_by(&self, r: &PostalReceive) -> bool { ... }
}
```

A specification that filters receives whose `date` falls within the
given range.

## Cross-Domain Coordinator

A thin coordinator lives in the engine facade and orchestrates
multi-domain flows (e.g. publish-form-to-site = documents + CMS). It
is **not** a service; it composes command calls:

```rust
pub struct DocumentsCoordinator<'a> {
    engine: &'a Engine,
}

impl<'a> DocumentsCoordinator<'a> {
    pub async fn upload_form(&self, cmd: UploadFormCommand) -> Result<FormDownload, DomainError> {
        let form = self.engine.documents().upload_form(cmd).await?;
        // Subscribers (CMS domain) handle the public-site
        // publication in response to the FormUploaded event.
        Ok(form)
    }
}
```

Domain services are pure. Cross-domain coordination happens through
events and command composition, never through service-to-service calls.
