# Documents Domain — Repositories

Repositories are ports (Rust traits). Adapters implement them. The
default adapter targets PostgreSQL; an SQLite adapter is provided for
embedded deployments.

## FormDownloadRepository

```rust
#[async_trait]
pub trait FormDownloadRepository: Send + Sync {
    async fn get(&self, id: FormDownloadId) -> Result<Option<FormDownload>>;
    async fn list(&self, school: SchoolId, q: FormDownloadQuery) -> Result<Vec<FormDownload>>;
    async fn list_public(&self, school: SchoolId) -> Result<Vec<FormDownload>>;
    async fn insert(&self, f: &FormDownload) -> Result<()>;
    async fn update(&self, f: &FormDownload) -> Result<()>;
    async fn delete(&self, id: FormDownloadId) -> Result<()>;
    async fn by_publish_date(&self, school: SchoolId, from: NaiveDate, to: NaiveDate) -> Result<Vec<FormDownload>>;
    async fn count(&self, school: SchoolId, q: FormDownloadQuery) -> Result<u64>;
    async fn page(&self, school: SchoolId, q: FormDownloadQuery, offset: u32, limit: u32) -> Result<Page<FormDownload>>;
}
```

## PostalDispatchRepository

```rust
#[async_trait]
pub trait PostalDispatchRepository: Send + Sync {
    async fn get(&self, id: PostalDispatchId) -> Result<Option<PostalDispatch>>;
    async fn list(&self, school: SchoolId, q: PostalDispatchQuery) -> Result<Vec<PostalDispatch>>;
    async fn insert(&self, d: &PostalDispatch) -> Result<()>;
    async fn update(&self, d: &PostalDispatch) -> Result<()>;
    async fn delete(&self, id: PostalDispatchId) -> Result<()>;
    async fn find_by_reference(&self, school: SchoolId, reference: &PostalReferenceNo) -> Result<Vec<PostalDispatch>>;
    async fn between(&self, school: SchoolId, from: NaiveDate, to: NaiveDate) -> Result<Vec<PostalDispatch>>;
    async fn by_academic_year(&self, school: SchoolId, year: AcademicYearId) -> Result<Vec<PostalDispatch>>;
}
```

## PostalReceiveRepository

```rust
#[async_trait]
pub trait PostalReceiveRepository: Send + Sync {
    async fn get(&self, id: PostalReceiveId) -> Result<Option<PostalReceive>>;
    async fn list(&self, school: SchoolId, q: PostalReceiveQuery) -> Result<Vec<PostalReceive>>;
    async fn insert(&self, r: &PostalReceive) -> Result<()>;
    async fn update(&self, r: &PostalReceive) -> Result<()>;
    async fn delete(&self, id: PostalReceiveId) -> Result<()>;
    async fn find_by_reference(&self, school: SchoolId, reference: &PostalReferenceNo) -> Result<Vec<PostalReceive>>;
    async fn between(&self, school: SchoolId, from: NaiveDate, to: NaiveDate) -> Result<Vec<PostalReceive>>;
    async fn by_academic_year(&self, school: SchoolId, year: AcademicYearId) -> Result<Vec<PostalReceive>>;
}
```

## Indexes (recommended)

The default PostgreSQL adapter documents the following indexes; consumers
should declare them in their migrations:

```sql
CREATE INDEX ix_form_downloads_school_id_publish ON documents_form_downloads (school_id, publish_date);
CREATE INDEX ix_form_downloads_school_id_public ON documents_form_downloads (school_id, show_public);
CREATE INDEX ix_postal_dispatches_school_id_date ON documents_postal_dispatches (school_id, date);
CREATE INDEX ix_postal_dispatches_school_id_reference ON documents_postal_dispatches (school_id, reference_no);
CREATE INDEX ix_postal_dispatches_school_id_academic ON documents_postal_dispatches (school_id, academic_id);
CREATE INDEX ix_postal_receives_school_id_date ON documents_postal_receives (school_id, date);
CREATE INDEX ix_postal_receives_school_id_reference ON documents_postal_receives (school_id, reference_no);
CREATE INDEX ix_postal_receives_school_id_academic ON documents_postal_receives (school_id, academic_id);
```

The `school_id` predicate is mandatory for tenant isolation.
