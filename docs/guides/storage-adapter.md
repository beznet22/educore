# Storage Adapter Implementation Guide

## Goal

Build a working storage adapter for PostgreSQL and SQLite that
satisfies the storage port and the parity test suite.

## Adapter Skeleton

```rust
pub struct PostgresStorage {
    pool: sqlx::Pool<Postgres>,
}

impl PostgresStorage {
    pub async fn connect(url: &str) -> Result<Self> {
        let pool = sqlx::postgres::Pool::connect(url).await?;
        Ok(Self { pool })
    }
}

#[async_trait]
impl StorageAdapter for PostgresStorage {
    async fn begin(&self) -> Result<Transaction> {
        let tx = self.pool.begin().await?;
        Ok(Transaction::Postgres(tx))
    }
    // ...
}
```

## Repository Implementation

Each repository is a thin wrapper over typed SQL:

```rust
pub struct PostgresStudentRepository {
    pool: PgPool,
}

#[async_trait]
impl StudentRepository for PostgresStudentRepository {
    async fn get(&self, id: StudentId) -> Result<Option<Student>> {
        let row = sqlx::query!(
            "SELECT * FROM students WHERE id = $1 AND school_id = $2",
            id.inner(),
            id.school().inner(),
        )
        .fetch_optional(&self.pool)
        .await?;
        row.map(Student::from_row).transpose()
    }

    async fn query(&self, q: StudentQuery) -> Result<Vec<Student>> {
        let mut sql = String::from("SELECT * FROM students WHERE school_id = $1");
        let mut params: Vec<Box<dyn ToSql>> = vec![Box::new(q.school_id().inner())];
        for filter in q.filters() {
            sql.push_str(&filter.to_sql_clause(&mut params));
        }
        // ... ORDER BY, LIMIT, OFFSET
    }
    // ...
}
```

## Query Translation

The engine's typed query layer provides a `to_sql_clause` method per
filter that emits parameterized SQL:

```rust
impl FilterClause for (StudentField, Op, FieldValue) {
    fn to_sql_clause(&self, params: &mut Vec<Box<dyn ToSql>>) -> String {
        let (field, op, value) = self;
        let column = field.column();
        let placeholder = params.len() + 1;
        let clause = match op {
            Op::Eq => format!("{} = ${}", column, placeholder),
            Op::In => format!("{} = ANY(${})", column, placeholder),
            Op::Between => format!("{} BETWEEN ${} AND ${}", column, placeholder, placeholder + 1),
            // ...
        };
        params.push(value.into_boxed());
        clause
    }
}
```

The adapter is the only place that knows the underlying SQL dialect.

## Outbox

The outbox is a table in the same database:

```sql
CREATE TABLE outbox (
    event_id UUID PRIMARY KEY,
    event_type TEXT NOT NULL,
    aggregate_id UUID NOT NULL,
    school_id INT NOT NULL,
    payload JSONB NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL,
    published_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX ix_outbox_unpublished ON outbox (created_at) WHERE published_at IS NULL;
```

The outbox relay polls `WHERE published_at IS NULL` and publishes
events to the bus. On success it sets `published_at = NOW()`.

## Tenant Isolation

Every query MUST include `school_id = $N`. The adapter enforces this
through a wrapper:

```rust
fn with_school_filter(&self, sql: &mut String) {
    if !sql.contains("school_id =") {
        panic!("query missing school_id filter: {}", sql);
    }
}
```

A debug-mode assertion catches missing filters. Release builds rely
on the engine's compile-time guarantee.

Row-level security is recommended as defense in depth:

```sql
ALTER TABLE students ENABLE ROW LEVEL SECURITY;
CREATE POLICY students_school_isolation ON students
    USING (school_id = current_setting('app.current_school_id', true)::int);
```

The adapter sets `app.current_school_id` on every connection from
the `tenant.school_id`.

## SQLite Differences

SQLite has no row-level security. Tenant isolation is enforced solely
by the application code. Other differences:

- No `JSONB` — use `TEXT` and parse manually.
- No `UUID` — use `TEXT` storing the canonical UUID string.
- No concurrent writes — use `BEGIN IMMEDIATE`.
- No connection pool — use a Mutex around the connection.

## SurrealDB / MongoDB

Document stores are supported by translating the relational schema
into documents:

- Each aggregate root is a document.
- Owned children are embedded sub-documents.
- References are foreign keys stored as ids.
- The query layer emits SurrealQL or MongoDB queries.

The engine provides a `DocumentAdapter` trait that abstracts over
relational and document adapters.

## Testing

The storage adapter ships with:

- A unit test for every repository method.
- An integration test suite using testcontainers (PostgreSQL) or
  rusqlite (SQLite).
- A parity test verifying identical results across adapters.
- A tenancy test verifying cross-tenant denial.
- A failure-injection test (deadlock retry, connection drop).
- A load test (10k attendance marks in <5s on PostgreSQL).

## Worked Example

A consumer wires a PostgreSQL storage adapter:

```rust
let storage: Arc<dyn StorageAdapter> = Arc::new(
    PostgresStorage::connect(&env::var("DATABASE_URL")?).await?
);
storage.migrate().await?;  // run consumer migrations
let engine = Engine::builder().storage(storage).build().await?;
```

The consumer's migrations live in `migrations/` (e.g. using
`refinery` or `sqlx-migrate`). The engine's expected schema is
documented in `docs/specs/<domain>/repositories.md`.

## Performance Tips

- Use prepared statements (sqlx caches them).
- Batch inserts in transactions.
- Use `COPY` for bulk loads.
- Index every column in WHERE and ORDER BY.
- Avoid `SELECT *`. Select only the columns the aggregate needs.
- Use connection pooling (max ~20 connections per CPU).
- Monitor slow queries via `pg_stat_statements`.
