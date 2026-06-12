# SaaS Backend Deployment Guide

## Goal

A consumer turns the Educore library into a production SaaS: a
thin HTTP backend, a platform-admin control plane, an identity layer,
a sync engine for offline-first clients, and the desktop / mobile
clients themselves. Every school-domain concern flows through
Educore; every SaaS-infrastructure concern is owned by the
consumer.

This guide is the **canonical reference** for that split. It
complements the port contracts in `docs/ports/*.md` and the
consumer-facing API in `docs/library-docs.md`.

## The Library Boundary

Educore is a domain engine. It provides:

- 15 domain crates (`educore-academic`, `educore-finance`, ..., `educore-events-domain`).
- 4 shipped storage adapters (SurrealDB primary embedded,
  PostgreSQL, MySQL, SQLite) and 1 deferred
  (`educore-storage-mongodb`). SurrealDB is the primary adapter
  for new deployments because its embedded mode enables
  single-binary distribution.
- 6 port adapters (`educore-auth`, `educore-notify`,
  `educore-payment`, `educore-files`, `educore-event-bus`,
  `educore-integrations`).
- The `educore` umbrella that re-exports the above under short
  names (`educore::academic`, `educore::storage_mysql`, ...).
- A typed query layer (`#[derive(DomainQuery)]`).
- An event envelope and outbox primitives.
- A tracing instrumentation surface.

Educore does **not** provide:

- An HTTP server, an API gateway, or any RPC layer.
- A web UI, admin dashboard, or any presentation layer.
- A hosted identity provider (it provides the **port**; the
  consumer wires the provider).
- A billing system (it provides the **payment port**; the consumer
  wires Stripe / Paddle / etc.).
- An observability backend (it emits OTel-compatible spans and
  `tracing` events; the consumer ships them).
- Hosting, deployment, or container orchestration.
- A migration runner (migrations are owned by the consumer; see
  `docs/ports/storage.md#migrations`).

A consumer that wants any of the second list must **build it on
top of** the engine. The remainder of this guide shows how.

## Two Layers of Tenancy

A real deployment has two distinct tenancy layers. They are not the
same and must not be conflated.

| Layer | Identity | Lives in | Managed by |
| --- | --- | --- | --- |
| **Engine tenancy** | `SchoolId` (UUIDv7) | Every aggregate root in Educore | School admin |
| **Platform tenancy** | `TenantId` (consumer-defined) | Consumer's SaaS database | System / platform admin |

The engine never sees `TenantId`. It only enforces `SchoolId`. The
consumer maps `TenantId → many SchoolId(s)` (a SaaS workspace can
own one or more schools) and passes the active `SchoolId` to every
command via `TenantContext`.

```rust
pub struct TenantContext {
    pub school_id: SchoolId,        // engine-facing, mandatory
    pub user_id: UserId,            // the actor
    pub correlation_id: CorrelationId,
    pub causation_id: Option<CorrelationId>,
    // consumer-extensible fields can be added by the consumer
    // in their own wrapper, not in the engine.
}
```

`TenantContext` is a value type. It is constructed at the API
boundary and threaded through every command. Cross-school commands
are forbidden by the engine itself — the aggregate's `SchoolId`
must equal the `TenantContext::school_id` or the command returns
`DomainError::Forbidden`.

## Consumer Repository Layout

The consumer's repository is a **separate workspace** that depends
on the Educore crates. The Educore repo is consumed as a
path or git dependency; the consumer never edits Educore to ship
a SaaS feature.

```text
consumer-repo/                       <-- the consumer's own workspace
├── Cargo.toml                       <-- depends on `educore = "0.1"`
├── backend/                         <-- the SaaS HTTP API
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs                  <-- axum / actix-web / hyper entry
│       ├── api/                     <-- HTTP route handlers
│       ├── auth/                    <-- JWT middleware, session cache
│       ├── tenant/                  <-- platform-tenant onboarding
│       ├── billing/                 <-- Stripe webhooks, plan limits
│       ├── sync/                    <-- sync-engine HTTP endpoints
│       └── admin/                   <-- platform-admin endpoints
├── control-plane/                   <-- platform-admin web app (separate binary)
│   ├── Cargo.toml
│   └── src/main.rs                  <-- Tauri / Yew / Leptos / server-rendered
├── sync-engine/                     <-- offline ⇄ central sync worker
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs                  <-- cron / queue worker
│       ├── outbox.rs                <-- reads local outbox
│       ├── relay.rs                 <-- posts to central
│       └── conflict.rs              <-- resolution policies
├── client/                          <-- offline-first desktop / mobile
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs                  <-- Tauri entry, embeds educore
│       ├── outbox.rs                <-- local pending events
│       └── sync.rs                  <-- invokes sync-engine
├── ops/
│   ├── observability/               <-- Grafana dashboards, Sentry, OTel
│   ├── deploy/                      <-- k8s manifests, Terraform
│   └── ci/                          <-- GitHub Actions, container build
└── .env.example                     <-- documented in this repo's `.env.example`
```

This layout is a recommendation, not a requirement. A consumer may
collapse two crates into one (e.g. a small SaaS that runs the
control plane inside the backend) or split further (e.g. a
`client-core` shared between desktop and mobile).

## The Thin Backend

The backend is **a thin shell** around `Engine::builder()`. It
does not contain school-domain logic — every domain operation is a
single call into the engine.

```rust
// backend/src/main.rs
use educore::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Storage — the central MySQL / Postgres adapter.
    let storage: Arc<dyn StorageAdapter> = Arc::new(
        MysqlStorage::builder()
            .url(env::var("DATABASE_URL")?)
            .max_connections(env::var("EDUCORE_STORAGE_MAX_CONNECTIONS")?
                .parse().unwrap_or(20))
            .build()
            .await?,
    );

    // 2. Authentication — JWT + OAuth2 + SAML via educore-auth.
    let auth: Arc<dyn AuthProvider> = Arc::new(
        JwtAuthProvider::builder()
            .signing_key(env::var("JWT_SECRET")?)
            .issuer(env::var("JWT_ISSUER")?)
            .audience(env::var("JWT_AUDIENCE")?)
            .access_ttl(Duration::from_secs(
                env::var("JWT_ACCESS_TTL_SECS")?.parse()?))
            .refresh_ttl(Duration::from_secs(
                env::var("JWT_REFRESH_TTL_SECS")?.parse()?))
            .build(),
    );

    // 3. Notification — SMTP / SMS / push via educore-notify.
    let notify: Arc<dyn NotificationProvider> = Arc::new(
        EmailNotifier::from_env()?,        // reads SMTP_* from .env
    );

    // 4. Payment — Stripe / PayPal / cash via educore-payment.
    let payment: Arc<dyn PaymentProvider> = Arc::new(
        StripePaymentProvider::from_env()?, // reads STRIPE_* from .env
    );

    // 5. File storage — S3 / local via educore-files.
    let files: Arc<dyn FileStorage> = Arc::new(
        S3FileStorage::from_env()?,         // reads S3_* from .env
    );

    // 6. Event bus — in-process default; NATS / Redis / Kafka in prod.
    let bus: Arc<dyn EventBus> = Arc::new(
        NatsBus::from_env()?,               // reads NATS_URL from .env
    );

    // 7. Clock, id-generator, audit sink — engine infra.
    let clock = Arc::new(SystemClock::new());
    let id_gen = Arc::new(UuidV7Generator::new());
    let audit = Arc::new(OtelAuditSink::from_env()?);

    // 8. Build the engine.
    let engine = Engine::builder()
        .storage(storage)
        .auth(auth)
        .notify(notify)
        .payment(payment)
        .files(files)
        .event_bus(bus)
        .clock(clock)
        .id_gen(id_gen)
        .audit_sink(audit)
        .build()
        .await?;

    // 9. Hand the engine to the HTTP layer.
    let app = api::router(engine);
    let listener = tokio::net::TcpListener::bind(
        env::var("EDUCORE_BIND").unwrap_or_else(|_| "0.0.0.0:8080".into())
    ).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

The backend's HTTP handlers are **thin dispatchers**. They do not
contain business logic; they extract the `TenantContext` from the
authenticated session, call an engine command, and translate
`DomainError` into an HTTP status code.

```rust
// backend/src/api/students.rs
async fn admit(
    State(engine): State<Engine>,
    Extension(session): Extension<Session>,
    Json(cmd): Json<AdmitStudentCommand>,
) -> Result<Json<Student>, ApiError> {
    let tenant = TenantContext::new(session.active_school_id(), session.user_id());
    let student = engine.students()
        .with_tenant(&tenant)
        .admit(cmd)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(student))
}
```

`ApiError::from` maps `DomainError` to HTTP:

| `DomainError` variant | HTTP status |
| --- | --- |
| `Validation { field, reason }` | 400 |
| `NotFound { entity, id }` | 404 |
| `Conflict { entity, reason }` | 409 |
| `Forbidden { reason }` | 403 |
| `Infrastructure(_)` | 500 (and logged) |

### Auth middleware

The backend extracts the bearer token, calls `engine.auth()
.validate(&token).await`, and produces a `Session`. The session's
`active_school_id` becomes the `TenantContext::school_id`. The
`Session` is added to the request as an `axum::Extension`.

```rust
// backend/src/auth/middleware.rs
async fn auth_middleware(
    State(engine): State<Engine>,
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let token = req.bearer_token()
        .ok_or(ApiError::Unauthorized)?;
    let session = engine.auth()
        .validate(&token)
        .await
        .map_err(|_| ApiError::Unauthorized)?;
    req.extensions_mut().insert(session);
    Ok(next.run(req).await)
}
```

### Route groups

The backend organises its routes by capability, not by domain. The
capability check is the **last** check before the engine call.

```rust
// backend/src/api/mod.rs
pub fn router(engine: Engine) -> Router {
    Router::new()
        .route("/v1/students", post(admit).layer(capability("students.admit")))
        .route("/v1/students", get(list).layer(capability("students.read")))
        .route("/v1/students/:id", get(get).layer(capability("students.read")))
        .route("/v1/attendance", post(mark).layer(capability("attendance.mark")))
        // ... ~80 routes, one per command + one per query
        .route_layer(middleware::from_fn_with_state(
            engine.clone(), auth_middleware))
        .with_state(engine)
}
```

## Identity Provider via `educore-auth`

The consumer has three realistic options for the identity layer.
All three go through the `educore-auth` port, so the engine code
does not change between them.

### Option A — JWT only (consumer-issued)

```text
┌──────────┐  POST /auth/login   ┌──────────────┐
│  Client  │ ──────────────────► │  Backend     │
│          │  { username, pwd }  │              │
│          │ ◄────────────────── │  JwtAuth     │
│          │  { access, refresh }│  (argon2 + JWT)
└──────────┘                     └──────────────┘
```

- `LocalPasswordAuthProvider` hashes with argon2id
  (`ARGON2_MEMORY_KIB`, `ARGON2_TIME_COST` from `.env`).
- `JwtAuthProvider` signs with `JWT_SECRET`, rotates on refresh.
- Refresh tokens are stored in a revocation list (consumer's
  choice: Redis, in-memory, or a table in the central DB).

### Option B — External IdP (OAuth2 / OIDC)

```text
┌──────────┐  GET /auth/oauth/google   ┌──────────┐  redirect  ┌──────────┐
│  Client  │ ────────────────────────►│ Backend  │ ─────────► │  Google  │
│          │ ◄────────────────────── │ (proxy)  │ ◄───────── │  OIDC    │
│          │  redirect_uri + code     └──────────┘   id_token └──────────┘
│          │  POST /auth/oauth/callback
│          │  { code }
│          │ ◄──────────────────────
│          │  { access, refresh }     ┌──────────────┐
│          │                          │  Oauth2Auth  │
└──────────┘                          │  (educore- │
                                      │   auth)      │
                                      └──────────────┘
```

- The consumer implements `Oauth2AuthProvider` per
  `docs/ports/authentication.md`.
- `OAUTH2_GOOGLE_CLIENT_ID`, `OAUTH2_GOOGLE_CLIENT_SECRET`,
  `OAUTH2_GOOGLE_REDIRECT_URI` come from `.env`.
- The IdP's `id_token` is verified; the consumer's `JwtAuthProvider`
  issues the local session token.

### Option C — Enterprise IdP (SAML)

- `SamlAuthProvider` reads `SAML_IDP_METADATA_URL`,
  `SAML_SP_ENTITY_ID`, `SAML_SP_ACS_URL` from `.env`.
- The SP is the backend itself (or a thin shim that forwards to
  the engine).

### Multi-factor

`educore-auth` supports TOTP, SMS, email, WebAuthn, and backup
codes. The consumer decides which command surfaces require MFA
based on configuration; the engine restricts sensitive commands
when `session.mfa_satisfied == false`.

### Capability check at the handler

The engine exposes a capability check helper. The handler calls it
**before** dispatching the command:

```rust
engine.rbac()
    .require(&session, Capability::StudentsAdmit)
    .await?;
```

This is in addition to the layer-based check. The layer check is
fast (cache lookup); the engine call is authoritative.

## The Control Plane (Platform Admin)

The control plane is **a second consumer** of the engine, with
elevated capabilities. It uses the same `educore` library but is
configured for cross-tenant operations.

```text
┌────────────────────────────────────────────────────────────┐
│                  Platform Admin (browser)                  │
│                                                            │
│  List all schools across all tenants                       │
│  Suspend / unsuspend a school                              │
│  View cross-tenant analytics                               │
│  Manage subscriptions                                      │
│  Inspect system health                                     │
└──────────────────────┬─────────────────────────────────────┘
                       │ HTTPS
                       ▼
┌────────────────────────────────────────────────────────────┐
│            control-plane/ (separate binary)                │
│                                                            │
│  Server-rendered (askama / maud) or SPA (Yew / Leptos)     │
│  Talks to backend's admin endpoints                        │
└──────────────────────┬─────────────────────────────────────┘
                       │ admin tokens
                       ▼
┌────────────────────────────────────────────────────────────┐
│            backend/  /admin/* routes                       │
│                                                            │
│  capability_required = "platform.*"                        │
│  uses engine.platform() commands                           │
└────────────────────────────────────────────────────────────┘
```

### Engine-level support for platform admins

The engine ships a `platform` domain crate
(`educore-platform`) that contains:

- `CreateSchoolCommand`, `SuspendSchoolCommand`,
  `UnsuspendSchoolCommand`, `ArchiveSchoolCommand`.
- `InviteUserCommand`, `AssignRoleCommand`, `RevokeRoleCommand`.
- `PlatformQuery` (list schools across tenants, with pagination).
- `School` and `User` aggregates, both rooted at the
  `Platform` aggregate (consumer-managed `TenantId` lives in a
  `PlatformMetadata` extension on the engine side; the engine
  does not enforce `TenantId`, but the `Platform` aggregate can
  carry one for consumer-side isolation).

A platform admin is just a `Session` whose capabilities include
the `platform.*` namespace. The engine enforces that these
capabilities are **not** granted to non-admin sessions.

```rust
// control-plane/src/main.rs (sketch)
let engine = Engine::builder()
    .storage(central_mysql.clone())
    .auth(jwt_provider.clone())
    .rbac(RbacConfig {
        super_admin_role: "platform.owner",
        capabilities: &[
            "platform.tenant.create",
            "platform.tenant.suspend",
            "platform.tenant.archive",
            "platform.user.invite",
            "platform.billing.override",
            "platform.analytics.read_all",
            "platform.system.health",
        ],
    })
    .build()
    .await?;

let schools = engine.platform()
    .query_schools(PlatformQuery::all()
        .with_status(SchoolStatus::Active)
        .page(0, 50))
    .await?;

engine.platform()
    .suspend_school(SuspendSchoolCommand {
        school_id,
        reason: "non-payment".into(),
        effective_at: clock.now(),
        actor: admin_user_id,
    })
    .await?;
```

### The `platform.*` capability namespace

The engine's RBAC port treats capabilities as typed strings. The
consumer decides the naming; the recommended namespace is
`platform.<action>`. The engine never interprets a `platform.*`
capability beyond "the bearer has elevated privileges"; the
**command** enforces the rest.

## The Sync Engine (Offline ⇄ Central)

The sync engine is the **only** place that talks both to the
local SQLite outbox and to the central API. The engine provides
the outbox primitives (`docs/ports/storage.md#outbox`); the
consumer writes the relay.

```text
┌─────────────────────────────────────────────┐
│  client/ (Tauri, offline-first)             │
│                                             │
│  ┌─────────────┐    ┌──────────────┐        │
│  │  educore  │    │  local       │        │
│  │  (embedded) │    │  SQLite      │        │
│  └──────┬──────┘    └──────┬───────┘        │
│         │ outbox writes   │                 │
│         ▼                  ▼                 │
│  ┌──────────────────────────────────┐       │
│  │  local outbox table              │       │
│  │  (event_id, type, payload, ts)   │       │
│  └──────────────┬───────────────────┘       │
└─────────────────┼───────────────────────────┘
                  │ HTTPS
                  ▼
┌─────────────────────────────────────────────┐
│  sync-engine/ (worker process)              │
│                                             │
│  1. read pending events from local outbox   │
│  2. POST /v1/sync { events: [...] }          │
│  3. on 200: delete from outbox              │
│  4. on 4xx: mark conflict, alert user       │
│  5. on 5xx/network: retry with backoff      │
└─────────────────┬───────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────┐
│  backend/  /v1/sync                         │
│                                             │
│  for each event:                            │
│    educore.handle_synced_event(event)     │
│  (re-runs the command in central MySQL,     │
│   same domain rules, same idempotency,      │
│   same audit trail)                         │
└─────────────────────────────────────────────┘
```

### The contract: events, not tables

The sync contract is **events**, not SQL. The engine guarantees:

- The same `event_id` is a no-op on replay (idempotency).
- Conflicts surface as `DomainError::Conflict` rather than silent
  corruption.
- Every state change that committed locally is replayed in
  central in the same order it was committed locally (per
  `correlation_id`).

```rust
// sync-engine/src/main.rs (sketch)
loop {
    let pending = read_pending_events(&local_db, BATCH_SIZE).await?;
    if pending.is_empty() {
        tokio::time::sleep(RETRY_INTERVAL).await;
        continue;
    }

    let response = http_client
        .post(format!("{}/v1/sync", central_url))
        .bearer_auth(&device_token)
        .json(&SyncRequest { events: pending.clone() })
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            let body: SyncResponse = resp.json().await?;
            mark_synced(&local_db, &body.accepted).await?;
            mark_conflicts(&local_db, &body.conflicts).await?;
        }
        Ok(resp) if resp.status().is_client_error() => {
            // 4xx: hard failure, surface to user
            alert_user(pending).await?;
        }
        _ => {
            // 5xx / network: backoff
            tokio::time::sleep(BACKOFF).await;
        }
    }
}
```

### Conflict resolution

Conflicts are **not** resolved by the sync engine; they are
returned to the client. The engine's command layer is the
authoritative resolver. Common patterns:

- **Last-write-wins** (per field, with vector clocks).
- **Manual merge** (the user reconciles on next online session).
- **Domain rule** (e.g. "if a student was admitted twice with
  different DOBs, the central admission wins" — enforced by the
  command's `Conflict` variant).

The sync engine's job is to **deliver** events, not interpret
them. Domain semantics stay in the engine.

## Clients (Offline-First)

A client embeds the engine. The engine is a library; it can be
linked into a Tauri desktop app, a mobile app (iOS / Android via
UniFFI or direct FFI), or even a CLI.

```toml
# client/Cargo.toml
# In development, point at the local engine workspace; in production,
# depend on the published crate:
#   educore = "0.1"
[dependencies]
educore        = { path = "../engine/crates/educore" }
educore-core   = { path = "../engine/crates/core" }
educore-storage-sqlite = { path = "../engine/crates/storage-sqlite" }
educore-auth   = { path = "../engine/crates/auth" }
# ... only the domains the client actually uses
educore-academic   = { path = "../engine/crates/academic" }
educore-attendance = { path = "../engine/crates/attendance" }

[target.'cfg(target_os = "android")'.dependencies]
# android-specific UniFFI bindings
```

```rust
// client/src/main.rs (Tauri sketch)
use educore::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let local_db = dirs::data_local_dir()
        .unwrap()
        .join("educore/client.db");
    tokio::fs::create_dir_all(local_db.parent().unwrap()).await?;

    let storage: Arc<dyn StorageAdapter> = Arc::new(
        SqliteStorage::open(&local_db)?
    );

    let auth: Arc<dyn AuthProvider> = Arc::new(
        JwtAuthProvider::from_env()?       // client-side, for offline tokens
    );

    let bus: Arc<dyn EventBus> = Arc::new(InProcessBus::new());

    let engine = Engine::builder()
        .storage(storage)
        .auth(auth)
        .event_bus(bus)
        .clock(Arc::new(SystemClock::new()))
        .id_gen(Arc::new(UuidV7Generator::new()))
        .build()
        .await?;

    tauri::Builder::default()
        .manage(engine)
        .invoke_handler(tauri::generate_handler![
            // Tauri commands wrap engine calls
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    Ok(())
}
```

The client **never** talks SQL directly. It talks the engine's
command and query surface, exactly like the backend. This is what
makes the same code work online (against central MySQL) and
offline (against local SQLite).

## Observability

The engine emits a `tracing` span for every command:

```text
INFO command.start command=admit_student school_id=… actor=… correlation_id=…
INFO command.end   command=admit_student school_id=… duration_ms=42 outcome=ok
```

It also emits a `tracing::info!` line for every domain event:

```text
INFO domain_event event_type=StudentAdmitted event_id=… school_id=… aggregate_id=…
```

The consumer configures the `tracing` subscriber to ship these
spans and events to its observability backend. Two common
patterns:

### OTel collector

```rust
// backend/src/main.rs
use opentelemetry::global;
use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::layer::SubscriberExt;

let tracer = opentelemetry_otlp::new_pipeline()
    .tracing()
    .with_exporter(opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(env::var("OTEL_EXPORTER_OTLP_ENDPOINT")?))
    .install_batch(opentelemetry::runtime::Tokio)?;

let subscriber = tracing_subscriber::registry()
    .with(tracing_subscriber::EnvFilter::from_default_env())
    .with(tracing_opentelemetry::layer().with_tracer(tracer))
    .with(tracing_subscriber::fmt::layer().json());

tracing::subscriber::set_global_default(subscriber)?;
```

The `OTEL_EXPORTER_OTLP_ENDPOINT` env var points to the consumer's
collector (Honeycomb, Tempo, Jaeger, Datadog APM, etc.).

### Audit sink

The engine writes an `AuditSink` for every state change. The
consumer implements `AuditSink` against its preferred store (the
same central DB, a separate audit DB, a SIEM like Splunk, etc.):

```rust
pub struct OtelAuditSink;

impl AuditSink for OtelAuditSink {
    async fn record(&self, entry: AuditEntry) -> Result<()> {
        tracing::info!(
            target: "educore.audit",
            audit_id = %entry.audit_id,
            school_id = %entry.school_id,
            actor = %entry.actor,
            action = %entry.action,
            entity = %entry.entity,
            entity_id = %entry.entity_id,
            correlation_id = %entry.correlation_id,
            "audit"
        );
        Ok(())
    }
}
```

The `tracing` subscriber above captures this and ships it.

### Metrics

Domain events are the natural metric surface:

```sql
-- example: count of admissions per school per day
SELECT school_id, date_trunc('day', occurred_at), count(*)
FROM outbox
WHERE event_type = 'StudentAdmitted'
GROUP BY 1, 2;
```

A consumer that wants Prometheus / OpenMetrics exports a worker
that tails the outbox and updates a counter per event type.

## Billing Integration

The engine's `educore-payment` port handles the money movement.
The consumer wires Stripe (or Paddle, LemonSqueezy, etc.) and
turns Stripe webhooks into engine events.

```rust
// backend/src/billing/webhook.rs
async fn stripe_webhook(
    State(engine): State<Engine>,
    Extension(verify): Extension<StripeSignature>,
    body: Bytes,
) -> Result<(), ApiError> {
    let event = verify.verify(&body)?;      // consumer's Stripe SDK

    match event {
        StripeEvent::InvoicePaid(inv) => {
            engine.finance().record_external_payment(
                RecordExternalPaymentCommand {
                    tenant: platform_tenant_to_school(inv.account_id),
                    stripe_invoice_id: inv.id,
                    amount: inv.amount_paid.into(),
                    currency: inv.currency.into(),
                    paid_at: inv.paid_at,
                    idempotency_key: idempotency_key_from_stripe(inv.id),
                }
            ).await?;
        }
        StripeEvent::CustomerSubscriptionDeleted(sub) => {
            engine.platform().suspend_school(
                SuspendSchoolCommand {
                    school_id: school_id_from_stripe_customer(sub.customer),
                    reason: "subscription cancelled".into(),
                    effective_at: clock.now(),
                    actor: system_actor(),
                }
            ).await?;
        }
        _ => {}
    }
    Ok(())
}
```

The engine never reads a Stripe webhook directly. The consumer
**translates** the webhook into an engine command, which keeps the
billing boundary clean and the engine pure.

### Plan limits

Plan limits (e.g. "max 500 students on the Starter plan") are
enforced by the backend, not the engine. The engine returns
`DomainError::QuotaExceeded` from notification and payment
ports; the consumer mirrors that for its own quotas.

```rust
// backend/src/billing/limits.rs
fn enforce_plan_limit(plan: &Plan, school: &School) -> Result<()> {
    if school.student_count() > plan.max_students {
        return Err(ApiError::Forbidden(
            format!("plan {} allows {} students", plan.name, plan.max_students)
        ));
    }
    Ok(())
}
```

## Deployment Topology

### Single-region (smallest viable)

```text
                          ┌──────────────────┐
                          │   Tauri client   │
                          │   (offline)      │
                          └────────┬─────────┘
                                   │ HTTPS
                                   ▼
                          ┌──────────────────┐
                          │   Backend        │
                          │   (axum, 1 node) │
                          └────────┬─────────┘
                                   │
                          ┌────────▼─────────┐
                          │  MySQL (single)  │
                          └──────────────────┘
```

### Multi-region (production)

```text
                          ┌──────────────────┐
                          │   Tauri client   │
                          └────────┬─────────┘
                                   │ HTTPS, near region
                                   ▼
                          ┌──────────────────┐
                          │   Edge LB        │
                          └────────┬─────────┘
                                   │
                ┌──────────────────┼──────────────────┐
                ▼                  ▼                  ▼
       ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
       │  region-a    │    │  region-b    │    │  region-c    │
       │  Backend x N │    │  Backend x N │    │  Backend x N │
       └──────┬───────┘    └──────┬───────┘    └──────┬───────┘
              │                  │                  │
       ┌──────▼───────┐    ┌──────▼───────┐    ┌──────▼───────┐
       │  MySQL       │    │  MySQL       │    │  MySQL       │
       │  (primary)   │◄──►│  (replica)   │    │  (replica)   │
       └──────┬───────┘    └──────┬───────┘    └──────┬───────┘
              │                  │                  │
              └──────────────────┼──────────────────┘
                                 │
                        ┌────────▼─────────┐
                        │  Event bus       │
                        │  (NATS JetStream)│
                        └──────────────────┘
```

The engine is region-agnostic. The consumer chooses:

- **Active-passive** for the central store (simpler).
- **Active-active** with cross-region replication (harder; needs
  per-region outbox → central event bus → per-region write fan-out).
- **Per-tenant region pinning** (each school stays in its
  nearest region; cross-region reads go through a thin proxy).

The outbox pattern (engine) + the event bus (consumer) is the
primitive that makes all three viable.

## Best Practices

1. **Never bypass the engine.** If you find yourself writing
   domain logic in the backend, stop. The engine owns the
   domain; the consumer owns the wires.

2. **The outbox is the sync contract.** The sync engine reads
   outbox events and posts them; the central engine re-runs the
   same command. Don't replicate by reading tables; the engine's
   commands are the source of truth.

3. **`SchoolId` is structural, not optional.** Every aggregate
   has it. The engine refuses to dispatch a command whose target
   aggregate's `SchoolId` doesn't match the `TenantContext`'s.

4. **Audit everything.** The engine emits `AuditEntry` for every
   state change. The consumer's `AuditSink` ships it. No direct
   DB writes from outside the engine.

5. **Capabilities, not roles.** A "principal" or "school admin"
   is a `Session` whose capability set includes the relevant
   command capabilities. The engine never consults a `role` field
   at runtime.

6. **Trace everything.** Every command gets a `correlation_id`
   at the API boundary. The engine threads it through every log,
   event, audit entry, and downstream call. Consumers can rebuild
   a full causal chain from the outbox + audit log.

7. **Treat the engine as a black box at the API boundary.** The
   engine's internal `educore_*` module paths are not part of
   its public API. Consumers depend on `educore::*` and never
   on the internal crate names.

8. **The control plane is a separate binary.** Even if it shares
   the workspace with the backend, it is deployed and scaled
   independently. The blast radius of a control-plane bug should
   not include the school API.

9. **The sync engine is a separate worker.** Not a library
   inside the client, not a thread inside the backend. A
   standalone process that can be scaled, restarted, and audited
   on its own.

10. **`.env` belongs to the consumer, not the engine.** The
    engine ships `.env.example` as a tracked template; each
    consumer's workspace has its own `.env` (gitignored) with
    real values.

11. **The sync engine is a build feature, not a hard dependency.** Consumers who
    don't need offline-first behavior should not enable the `sync` feature
    on the umbrella crate. The in-process sync coordinator is only
    compiled in when the feature is on. The wire protocol is identical
    either way — the consumer's `sync-engine` worker binary, when they
    build one, talks to the same `SyncAdapter` port.

## Reference Map

| Concern | Where it lives | Doc |
| --- | --- | --- |
| Engine construction | `Engine::builder()` | `docs/library-docs.md` |
| Command dispatch | `engine.<domain>().<command>(cmd)` | `docs/commands/<domain>.md` |
| Query surface | `engine.<domain>().query().<chain>` | `docs/query_layer.md` |
| Storage adapter | `educore-storage-{postgres,mysql,sqlite}` | `docs/ports/storage.md` |
| Authentication | `educore-auth` | `docs/ports/authentication.md` |
| Notification | `educore-notify` | `docs/ports/notifications.md` |
| Payment | `educore-payment` | `docs/ports/payments.md` |
| File storage | `educore-files` | `docs/ports/file-storage.md` |
| Event bus | `educore-event-bus` | `docs/ports/event-bus.md` |
| Integrations | `educore-integrations` | `docs/ports/integrations.md` |
| Sync port       | `educore-sync` (gated by `sync` feature) | `docs/ports/sync.md` |
| Outbox pattern | Engine | `docs/ports/storage.md#outbox` |
| Offline mode | Engine + consumer sync-engine | `docs/guides/offline-sync.md` |
| Audit | Engine + consumer `AuditSink` | `docs/guides/audit-trail.md` |
| Multi-tenancy | Engine (`SchoolId`) + consumer (`TenantId`) | `docs/guides/multi-tenancy.md` |
| RBAC / capabilities | Engine RBAC port | `docs/guides/capability-rbac.md` |
| Idempotency | Every command | `docs/guides/idempotent-commands.md` |
| CI / CD | Consumer | `docs/guides/ci-cd.md` |
| Testing | Engine + consumer | `docs/guides/test-strategy.md` |
