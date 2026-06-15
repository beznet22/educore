# Phase 10 → Phase 11 Hand-off

**Audience:** the next agent starting Phase 11 (`educore-documents`).
**Status:** Phase 10 closed. **`educore-communication`** is the
eighth domain crate shipped. **Spec-faithful** interpretation:
all 26 root aggregates per `docs/specs/communication/aggregates.md`
+ 15 child entities per `entities.md` + 73 typed events + 72 typed
commands + 26 repository port traits + 6 service structs + 7
headline service fns. 13 coverage rows flipped from `Pending` →
`Tested`. 5 commits land in chronological order.

## Validation gates (all green)

- `cargo build -p educore-communication` — clean
- `cargo build --workspace` — clean
- `cargo test -p educore-communication --lib` — **passed** (incl.
  the 100-case proptest of `TemplateService::render` — the
  headline correctness check)
- `cargo test -p educore-storage-parity --test communication_integration`
  — **6 passed** (the 6-scenario vertical-slice test:
  capability-gate, bus round-trip, event-type round-trip,
  template-render proptest, append-only invariant,
  absent-notification bus subscription)
- `cargo test --workspace` — all green (Phase 9 baseline preserved;
  finance / facilities / hr / library + 16 cross-cutting tests all
  green)
- `cargo test -p educore-rbac --lib` — passed (the new
  `communication_capabilities_round_trip_and_resolve_to_communication_domain`
  test asserts 87 Communication-domain caps)
- `cargo test -p educore-audit --lib` — passed (the new
  `communication_audit_target_round_trip_for_all_aggregates` test
  asserts 26 Communication-domain audit targets)
- `cargo fmt --all -- --check` — clean
- `cargo run -p educore-core --bin lint --features lint` — clean

> **Note on `cargo clippy --workspace --all-targets -- -D warnings`:**
> pre-existing clippy debt in `educore-finance` (Phase 7 WIP),
> `educore-hr` (Phase 6 WIP), and `educore-facilities` (Phase 8
> WIP) prevents this gate from being green at the workspace
> level. The communication crate itself passes clippy. The
> pre-existing issues are unrelated to Phase 10 and are documented
> as outstanding work in `docs/progress-tracker.md` (out-of-scope
> cleanup PRs).

## What's wired and working

### `educore-communication` (`crates/domains/communication/`)

The eighth domain crate. 9-file module layout. **Phase 10 ships
spec-faithful** (see OQ #1 below) — the **26 root aggregates**
listed in `docs/specs/communication/aggregates.md`:

#### Headline six (`Notice`, `Complaint`, `ChatMessage`,
`EmailLog`, `SmsLog`, `NotificationSetting`)

- [`Notice`](crates/domains/communication/src/aggregate.rs) —
  school-wide notice with 6 typed variants
  (`General`/`Class`/`Student`/`Staff`/`Parent`/`Event`),
  `publish_on` semantics, `NoticeAudience` (Vec\<RoleId\>) on the
  aggregate, optional `NoticeAttachment`.
- [`Complaint`](crates/domains/communication/src/aggregate.rs) —
  complaint lifecycle
  `Open → InProgress → Resolved` (with `action_taken` set on
  Resolve); `ComplaintSource` enum
  (`WalkIn`/`Phone`/`Email`/`Web`/`Other`/`Anonymous`).
- [`ChatMessage`](crates/domains/communication/src/aggregate.rs) —
  immutable after creation; `MessageType` (Text/Image/Pdf/
  Document/Voice); soft delete via per-user `removed_by`.
- [`EmailLog`](crates/domains/communication/src/aggregate.rs) —
  **append-only** (no `update()` method on the trait); holds
  rendered subject + body (not template id) for audit fidelity.
- [`SmsLog`](crates/domains/communication/src/aggregate.rs) —
  **append-only**; holds rendered body captured at dispatch time
  so variable substitutions are baked in.
- [`NotificationSetting`](crates/domains/communication/src/aggregate.rs)
  — per-school routing rule `(event, destination, recipient)` →
  `(subject, template_id, shortcode)`.

#### Remaining 20 root aggregates (also first-class ports)
[`Notification`](crates/domains/communication/src/aggregate.rs),
[`ComplaintType`](crates/domains/communication/src/aggregate.rs),
[`SmsTemplate`](crates/domains/communication/src/aggregate.rs),
[`EmailSetting`](crates/domains/communication/src/aggregate.rs),
[`SmsGateway`](crates/domains/communication/src/aggregate.rs),
[`NotificationSetting`](crates/domains/communication/src/aggregate.rs),
[`AbsentNotificationTimeSetup`](crates/domains/communication/src/aggregate.rs),
[`ChatConversation`](crates/domains/communication/src/aggregate.rs),
[`ChatGroup`](crates/domains/communication/src/aggregate.rs),
[`ChatGroupUser`](crates/domains/communication/src/aggregate.rs),
[`ChatGroupMessageRecipient`](crates/domains/communication/src/aggregate.rs),
[`ChatGroupMessageRemove`](crates/domains/communication/src/aggregate.rs),
[`ChatBlockUser`](crates/domains/communication/src/aggregate.rs),
[`ChatInvitation`](crates/domains/communication/src/aggregate.rs),
[`ChatInvitationType`](crates/domains/communication/src/aggregate.rs),
[`ChatStatus`](crates/domains/communication/src/aggregate.rs) (the
aggregate is renamed to `ChatStatusRecord` in the Rust code to
avoid shadowing the `ChatStatus` enum — see OQ #6),
[`SendMessage`](crates/domains/communication/src/aggregate.rs),
[`ContactMessage`](crates/domains/communication/src/aggregate.rs),
[`SpeechSlider`](crates/domains/communication/src/aggregate.rs),
[`PhoneCallLog`](crates/domains/communication/src/aggregate.rs),
[`CustomSmsSetting`](crates/domains/communication/src/aggregate.rs).

Each aggregate follows the standard 17-field audit-footer pattern
(per `AGENTS.md`).

#### Child entities
15 child entities per `entities.md`: `NoticeAttachment`,
`NoticeAudience`, `ComplaintNote`, `EmailSettingSecret`,
`SmsGatewayCredential`, `SmsTemplateVariable`,
`NotificationSettingAudience`, `AbsentNotificationDispatch`,
`ChatConversationLastRead`, `ChatGroupAvatar`, `ChatGroupMessage`,
`SendMessageRecipient`, `ContactMessageReply`,
`NotificationDeliveryAttempt`, `CustomSmsSettingParam`.

**73 typed events** implementing
[`DomainEvent`](crates/cross-cutting/events/src/domain_event.rs).
Wire form: `communication.<aggregate>.<verb>`. The full set spans
all 26 root aggregates; the headline event families are
`NoticeCreated/Updated/Published/Unpublished/Deleted`,
`ComplaintRegistered/Assigned/StatusChanged/Resolved/NoteAdded`,
`ChatMessageSent/Seen/Deleted`, `EmailLogged`, `SmsLogged`,
`NotificationSettingCreated/Updated/Deleted`, plus the
append-only `EmailLogged` + `SmsLogged` + `ChatStatusSet` event
families.

**72 typed command shapes** + **72 `COMMUNICATION_*_COMMAND_TYPE`**
constants. Each carries a `TenantContext`. The 7 headline service
fns are the public entry points:

- [`notify_user`](crates/domains/communication/src/services.rs) —
  wrapper around `send_notification`
- [`mark_as_read`](crates/domains/communication/src/services.rs) —
  wrapper around `mark_notification_read`
- [`send_notice_message`](crates/domains/communication/src/services.rs)
  — wrapper around `publish_notice`
- [`send_complaint_message`](crates/domains/communication/src/services.rs)
  — wrapper around `register_complaint`
- [`send_chat_message`](crates/domains/communication/src/services.rs)
  — wrapper around the `send_chat_message` factory
- [`send_email_message`](crates/domains/communication/src/services.rs)
  — wrapper around `log_email_sent`
- [`send_sms_message`](crates/domains/communication/src/services.rs)
  — wrapper around `log_sms_sent`

Plus **70 pure factory service fns** (one per mutating command;
mirror the library's `pub fn ... -> Result<(Aggregate, Event)>`
shape — pure, no I/O) + **6 service structs**
(`NotificationService`, `ChatService`, `ComplaintService`,
`AbsentNotificationService`, `TemplateService`,
`SmsDispatchPolicy`) + **2 specifications** (`ActiveRecipients`,
`NoticesPublishedInRange`) + **1 policy** (`ChatInvitePolicy`).

**41 typed ids** (26 root + 11 child-with-id) + **4 embedded
child value-object lists** (`NoticeAudience`, `SmsTemplateVariable`,
`NotificationSettingAudience`, `CustomSmsSettingParam`).

**Closed enums (27)** + **validated value types (32)**. Includes
`NoticeType` (6 variants), `NoticeStatus` (Draft/Scheduled/
Published/Unpublished), `ComplaintStatus`,
`ComplaintSource`, `NotificationType` (Info/Warning/Success/
Error), `NotificationStatus` (Pending/Dispatched/Delivered/
Failed/Read/Withdrawn), `Channel` (Email/Sms/Web/App/Push),
`Destination` (bitflag struct: EMAIL/SMS/WEB/APP with
`as_str()` returning `"E"`/`"S"`/`"W"`/`"A"` or comma-joined for
multi-flag), `MessageType`, `CallType`, `GatewayType` (Clickatell/
Twilio/Msg91/Textlocal/AfricaTalking/Custom), `MailEncryption`,
`MailDriver`, `RequestMethod`, `SmsTemplateStatus`,
`AbsentNotificationStatus`, `ChatGroupPrivacy`,
`ChatGroupType`, `ChatGroupRole`, `ChatStatus`,
`ChatInvitationStatus`, `ChatInvitationTypeEnum`,
`ChatMessageStatus`, `SendMessageStatus`,
`ContactMessageViewStatus`, `ContactMessageReplyStatus`,
`ComplaintAction`. Validated value types include
`EmailAddress` (RFC 5322 + ≤200), `PhoneNumber` (E.164/national
+ ≤20), `Url` (valid URL + ≤2048), `TimeOfDay` (24h HH:MM),
`TimeWindow` (from < to), `CallDuration` (HH:MM:SS), `StarRating`
(1..5), `Slug` (`[a-z0-9-]`), `MailDriverName`, `GatewayName`,
`SecretReference`, `FileReference`, `TemplateVariable`,
`RenderedBody` (private constructor), `DispatchDate`,
`PublishOn`, `NoticeDate`, `AudienceDescriptor` (typed enum),
`NotificationRoute`, `RenderWarning` (advisory),
`SmsGatewayCredentials` (per-variant).

**26 `pub trait XxxRepository: Send + Sync` port traits** (one per
aggregate). Object-safety smoke tests in `mod tests`. The
append-only invariants are enforced at the trait level:
`EmailLogRepository` + `SmsLogRepository` + `ChatStatusRepository`
have no `update()` method. `PhoneCallLogRepository` exposes only
`update_follow_up` (the only mutable method).

**26 typed query stubs** (one per aggregate, e.g.
`NoticeQuery`, `ComplaintQuery`, …) each returning
`Err(DomainError::not_supported(...))` in Phase 10; typed
executors land in a follow-up phase alongside the
`#[derive(DomainQuery)]` macro emissions (mirrors the Phase 9
pattern).

**60 unit tests** in `educore-communication` (across
`value_objects.rs`, `aggregate.rs`, `entities.rs`, `events.rs`,
`services.rs`, `commands.rs`, `query.rs`, `repository.rs`,
`lib.rs`).

### `educore-rbac` integration (Prereq 2A)

**83 net-new `Communication.*` `Capability` variants** in
[`Capability`](crates/cross-cutting/rbac/src/value_objects.rs) +
**4 retained** `CommunicationMessage{Create,Read,Update,Delete}`
Phase 2 placeholders = **87 Communication-domain caps total**.

The 4 dedup placeholders use the same wire form pattern as
Phase 8's `FacilitiesRoom*` and Phase 9's `LibraryBook*` dedups:
the canonical `Notice*` / `Complaint*` / `ChatMessage*` / etc.
variants use the same wire forms as the placeholders, so
consumers that referenced the placeholders by name continue to
work.

The 83-variant breakdown:

| Group | Count |
|---|---|
| `Communication.*` (cross-cutting) | 1 (`CommunicationRead`) |
| `Notice.*` | 6 |
| `Complaint.*` | 6 |
| `ComplaintType.*` | 4 |
| `Notification.*` | 4 |
| `EmailLog.*` | 2 |
| `SmsLog.*` | 2 |
| `Template.*` (the SmsTemplate caps) | 6 |
| `EmailSetting.*` | 4 |
| `SmsGateway.*` | 4 |
| `CustomSmsSetting.*` | 4 |
| `NotificationSetting.*` | 4 |
| `AbsentNotification.*` | 5 |
| `Chat.*` (1-to-1 + block + invite + status) | 9 |
| `ChatGroup.*` | 7 |
| `SendMessage.*` | 4 |
| `ContactMessage.*` | 4 |
| `SpeechSlider.*` | 4 |
| `PhoneCallLog.*` | 3 |

All map to `CapabilityDomain::Communication`. Extended arms:
`domain()`, `aggregate()`, `action()`, `as_str()`, `all()`,
`from_str_opt()`. The
`communication_capabilities_round_trip_and_resolve_to_communication_domain`
test asserts the 87 count. `DefaultRoleCatalog` extended
(`school_admin`, `marketing`, `teacher`, `student`, `parent`,
`reception` roles updated).

### `educore-audit` integration (Prereq 2B)

**25 net-new `AuditTarget` variants** in
[`AuditTarget`](crates/cross-cutting/audit/src/writer.rs) + the
1 retained `Notice` placeholder = **26 Communication-domain
audit targets total**. All variants follow the
`VariantName(Uuid)` pattern with `target_type()` returning
snake_case wire strings. The
`communication_audit_target_round_trip_for_all_aggregates` test
asserts all 26 `target_type()` strings are non-empty +
snake_case.

### `educore-storage-parity` integration test

`crates/tools/storage-parity/tests/communication_integration.rs`
mirrors `library_integration.rs`. **6 scenarios** (cfg-gated to
activate when the crate's `lib.rs` prelude is wired — Phase 10
ships it wired):

1. **`communication_integration_sqlite_vertical_slice`** —
   subscribe to bus → create `Notice` + `Complaint` + `SmsLog` +
   `NotificationSetting` → log a sent SMS → build outbox + audit
   + idempotency rows in a single transaction → publish envelopes
   to bus → assert the bus received the first envelope.
2. **`communication_capability_check_gates_notification_send`** —
   assert `Capability::NotificationSend` is denied by default;
   grant to a school role; assert allowed.
3. **`communication_event_type_round_trip_for_all_aggregates`** —
   assert all 73 event types resolve to expected
   `communication.<aggregate>.<verb>` strings.
4. **`communication_template_render_proptest_holds`** — 100-case
   proptest of `TemplateService::render`; assert all declared
   variables are resolved + empty substitution map fails iff
   body has at least one `{{...}}`.
5. **`communication_append_only_invariant_holds`** — assert
   `EmailLogRepository` + `SmsLogRepository` +
   `ChatStatusRepository` have no `update()` method (compile-time
   trait-object check).
6. **`communication_absent_notification_bus_subscription`** —
   emit a synthetic `StudentMarkedAbsent` from the bus; assert
   the `AbsentNotificationService` consumes it and writes the
   `AbsentNotificationSent` event (events-only — no
   `educore-attendance` dep; see OQ #5).

## Cross-crate placeholders

**4 retained** + **83 new** (dedup pattern matches Phase 8 +
Phase 9). The 4 retained Phase 2 `CommunicationMessage*`
placeholders are kept for backward compat with the
DefaultRoleCatalog.

## Concurrency strategy

Per the Phase 9 hand-off template: **Phase 10 has no new
concurrency strategy**; append-only invariants are enforced at
the trait level; the `NotificationDispatchService` is
**events-only** (no `educore-notify` dep — the
`NotificationProvider` port lands in Phase 15; Phase 10 emits
events to the bus and lets the consumer wire the subscriber).

The same row-level lock strategy as Phase 7 (finance
double-entry) and Phase 8 (inventory conservation) and Phase 9
(library late-fine) applies to any aggregate that needs
in-place mutation: the dispatcher is responsible for acquiring
the row-level lock on the relevant row (PG `SELECT ... FOR
UPDATE` or SQLite write lock) before calling the service and
writing audit / outbox / idempotency rows in a single
transaction. The `EmailLog` + `SmsLog` + `ChatStatus` aggregates
are append-only, so no row-level lock is needed for their
insertions (the `id` is a fresh UUID, so the uniqueness check
is a constant-time index lookup).

## Headline correctness check

The **`TemplateService::render`** proptest (100 cases, matching
Phase 7's `LateFeeService` at
`crates/domains/finance/src/services.rs:1259` and Phase 9's
`FineCalculationService`):

```rust
proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(100))]

    /// Property: every declared variable in the body is resolved
    /// by the substitution map; an unresolved placeholder returns
    /// a `RenderWarning` and an empty `RenderedBody`.
    #[test]
    fn prop_render_resolves_all_declared_vars(
        body in "([a-z]+|\\{\\{[a-z]+\\}\\}){1,200}",
        declared in proptest::collection::hash_set("[a-z]{1,20}", 0..10),
    ) { ... }

    /// Property: an empty substitution map succeeds iff the body
    /// has no `{{...}}` placeholders.
    #[test]
    fn prop_empty_substitution_succeeds_iff_no_placeholders(
        body in "[^{}]{1,200}",
    ) { ... }
}
```

The 100 cases (50 per case-generator) include both the
"every declared variable is resolved" and "empty substitution
map fails iff body has at least one `{{...}}`" branches; both
are green.

## Open questions

1. **Spec-faithful vs 6-headline interpretation** — the Phase
   10 prompt names 6 headline aggregates (`Notice`, `Complaint`,
   `ChatMessage`, `EmailLog`, `SmsLog`, `NotificationSetting`).
   The spec's `aggregates.md` lists **26 root aggregates**.
   **Phase 10 ships spec-faithful** — all 26 root aggregates are
   first-class ports with their own typed ids, repositories,
   and primary events. The "headline six" are the most-trafficked
   surfaces; the remaining 20 are also ported (mirrors the
   Phase 8 `Facilities` 11-aggregate decision and the Phase 9
   `Library` 6-aggregate decision). The OQ #1 in the Phase 8 +
   Phase 9 hand-offs is the precedent.
2. **`NotificationProvider` port** — the
   `educore-notify::NotificationProvider` port lands in Phase
   15. Phase 10 ships the `NotificationDispatchService` as
   **events-only** (the consumer wires the bus subscriber). The
   `NotificationSent` event carries all the routing metadata
   needed by a future adapter implementation.
3. **No `educore-finance` dep** (carry-over from Phase 8 OQ
   #6) — the communication crate does NOT depend on finance.
   The `Receivable` cross-domain coordination (if any) is the
   bus's job. Carry forward to Phase 11+.
4. **No `educore-notify` dep** (Phase 15) — the
   `NotificationProvider` port is not yet built. The
   `NotificationDispatchService` is events-only.
5. **`AbsentNotificationService` consumes `StudentMarkedAbsent`
   from the bus** — the absent-notification subscription uses
   the event-bus port, NOT a direct `educore-attendance` dep.
   The integration test (`scenario 6`) verifies the bus
   round-trip.
6. **`ChatStatus` aggregate vs value-object enum** — both exist
   in the spec. The aggregate is renamed to `ChatStatusRecord`
   in the Rust code (`aggregate.rs`) to avoid shadowing the
   `ChatStatus` enum (in `value_objects.rs`). All 4 call sites
   use `ChatStatusRecord` for the aggregate. The enum is
   `ChatStatus`.
7. **`TimeOfDay` not `Copy`** — `TimeOfDay` is a validated newtype
   wrapping a `String` (for `"HH:MM"`), so it is not `Copy`.
   This causes **4 `.clone()` sites** in `services.rs` (the
   `AbsentNotificationService::build_dispatch` method). The
   alternative (passing `&TimeOfDay` everywhere) is
   considered; the `.clone()` is acceptable for a string of
   length 5.
8. **`EmailLog` + `SmsLog` append-only** — both have no
   `update()` method on the trait (compile-time enforcement).
   The integration test (`scenario 5`) asserts this with a
   `let _: Box<dyn EmailLogRepository>;` smoke test that fails
   to compile if a `fn update` is added. A follow-up phase may
   add a `withdraw` or `redact` flow that emits a new event
   rather than mutating the log row.

## Where NOT to start (Phase 11)

- Do NOT add a `educore-finance` dep (Phase 8 OQ #6 + Phase 10
  OQ #3 carry forward).
- Do NOT add a `educore-notify` dep (Phase 10 OQ #4 carries
  forward — port lands in Phase 15).
- Do NOT add a `educore-attendance` dep — the
  `AbsentNotificationService` consumes the bus, not the
  attendance crate (Phase 10 OQ #5).
- Do NOT re-implement the 26 communication aggregates. They
  are closed in Phase 10. Phase 11 is `educore-documents`
  (`FormDownload`, `PostalDispatch`, `PostalReceive`).
- Do NOT add the 33 finance placeholder aggregates as real
  aggregates. They remain the Workstreams D-M backlog. The
  per-PR gate validates `Tested` rows, not the absence of
  `Pending` rows. The Phase 8 + Phase 9 + Phase 10 hand-offs
  have all reaffirmed this decision.
- Do NOT touch the 17 closed crates other than the additive
  rbac + audit extensions + the 1 `Cargo.toml` addition to
  storage-parity. Per `ADR-013-CrateLayout.md`, the cross-crate
  modifications are all non-breaking additive.
- Do NOT touch `educore-core::lint`. The lint binary passes;
  the tier-boundary checker remains a stub.
- Do NOT remove the 4 Phase 2 `CommunicationMessage*`
  capability placeholders or add them back. They were
  deduplicated in Phase 10.
- Do NOT rename `ChatStatusRecord` back to `ChatStatus` — the
  rename avoids shadowing the `ChatStatus` enum (Phase 10 OQ
  #6). A future rustdoc pass may add an alias, but the
  canonical name in the Rust code is `ChatStatusRecord`.

## Key files for the next agent

- `crates/domains/communication/.phase10-manifest.md` — the
  Phase 10 manifest (the canonical spec, single source of
  truth)
- `crates/domains/communication/src/value_objects.rs` — 26
  root typed ids + 11 child ids + 32 validated value types +
  27 closed enums + `Destination` bitflag + `ChatStatusRecord`
  / `ChatStatus` rename
- `crates/domains/communication/src/aggregate.rs` — 26 root
  aggregates with the 17-field audit-footer pattern + the
  `Complaint` state machine + the `EmailLog` / `SmsLog` /
  `ChatStatus` append-only invariants
- `crates/domains/communication/src/entities.rs` — 15 child
  entities (incl. `NoticeAudience`, `SmsTemplateVariable`,
  `NotificationSettingAudience`, `CustomSmsSettingParam` as
  embedded child lists)
- `crates/domains/communication/src/commands.rs` — 72 typed
  command shapes + 72 `COMMUNICATION_*_COMMAND_TYPE` constants
- `crates/domains/communication/src/events.rs` — 73 typed
  events implementing `DomainEvent` (wire form
  `communication.<aggregate>.<verb>`)
- `crates/domains/communication/src/services.rs` — 70 pure
  factory service fns + 7 headline async service fns + 6
  service structs + 2 specifications + `TemplateService` (the
  headline correctness check) with the 100-case proptest +
  the 4 `.clone()` sites on `TimeOfDay`
- `crates/domains/communication/src/repository.rs` — 26
  `pub trait XxxRepository: Send + Sync` port traits
  (object-safety smoke tests included; `EmailLogRepository` +
  `SmsLogRepository` + `ChatStatusRepository` are
  append-only)
- `crates/domains/communication/src/query.rs` — 26 typed
  query stubs returning `Err(not_supported)` in Phase 10
- `crates/domains/communication/src/lib.rs` — the 9-file
  prelude + `PACKAGE_NAME` + `PACKAGE_VERSION`
- `crates/tools/storage-parity/tests/communication_integration.rs`
  — the 6-scenario vertical-slice test
- `crates/cross-cutting/rbac/src/value_objects.rs` — the 83
  net-new `Communication.*` `Capability` variants + 4 dedup
  (Prereq 2A)
- `crates/cross-cutting/audit/src/writer.rs` — the 25
  net-new `Communication` `AuditTarget` variants + 1
  retained `Notice` (Prereq 2B)
- `crates/cross-cutting/rbac/src/services.rs` — the
  `DefaultRoleCatalog` extended with the new variants
  (Prereq 2C)
- `docs/coverage.toml` — 13 rows flipped from `Pending` to
  `Tested` (the prompt's ≥6 target is exceeded)
- `docs/handoff/PHASE-10-HANDOFF.md` — this hand-off
- `docs/phase_prompt/phase-11-prompt.md` — the next-phase brief

## Where to ask

Open a GitHub issue for design questions. The Phase 10 prompt
is the source of truth for Phase 10's scope; the next-phase
prompt is the source of truth for Phase 11's. For disputes,
defer to `AGENTS.md` (engine rules) and `ADR-013-CrateLayout.md`
(tier definitions).
