# Wave 1 Communication Domain Audit Report (Phase 10)

**Scope:** `crates/domains/communication/`, `docs/specs/communication/`,
`docs/commands/communication.md`, `docs/events/communication.md`,
`docs/coverage.toml` (communication rows).

**Total findings:** 47

---

### FINDING 1

- **id:** DOMAIN-COM-001
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/communication/src/aggregate.rs:2068` and `docs/specs/communication/aggregates.md:663`
- **description:** The aggregate root for the chat-status presence record is named `ChatStatusRecord` in the Rust source but the spec calls the root type `ChatStatus`. The spec, prelude, and `ChatStatusRepository` (`repository.rs:578`) all reference the type by different names (`ChatStatusRecord` aggregate vs. `ChatStatus` enum vs. `ChatStatus` repository parameter). Consumers that consult only the spec will be unable to import the symbol the code exports.
- **expected:** `docs/specs/communication/aggregates.md:663` `**Root type:** \`ChatStatus\`` — the Rust aggregate must be `pub struct ChatStatus`.
- **evidence:**
  - `crates/domains/communication/src/aggregate.rs:2068` `pub struct ChatStatusRecord {`
  - `crates/domains/communication/src/repository.rs:578` `pub trait ChatStatusRepository: Send + Sync { async fn insert(&self, s: &ChatStatus) -> Result<()>; }` — uses `ChatStatus` as the aggregate reference.
  - `crates/domains/communication/src/value_objects.rs:801` `pub enum ChatStatus { ... }` — a separate status enum clashes with the aggregate name.

---

### FINDING 2

- **id:** DOMAIN-COM-002
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/communication/src/entities.rs:750` and `crates/domains/communication/src/value_objects.rs:2144`
- **description:** `NotificationSettingAudience` is defined twice with conflicting shapes. `entities.rs:750` declares it as a 4-variant enum `Roles | ClassSection | Users | All`, while `value_objects.rs:2144` declares it as `pub type NotificationSettingAudience = AudienceDescriptor;`. The aggregate (`aggregate.rs:1047`) imports the entities version under an alias; consumers importing from the prelude resolve to the `value_objects` alias. Two types with the same name but different memory layouts exist in one crate.
- **expected:** A single definition (either the enum from entities.rs or the alias from value_objects.rs) used everywhere.
- **evidence:**
  - `crates/domains/communication/src/entities.rs:749-765` `pub enum NotificationSettingAudience { Roles(Vec<RoleId>), ClassSection { ... }, Users(Vec<UserId>), All }`
  - `crates/domains/communication/src/value_objects.rs:2138-2144` `/// A type alias for the audience descriptor of a NotificationSetting... pub type NotificationSettingAudience = AudienceDescriptor;`
  - `crates/domains/communication/src/aggregate.rs:31-32` `use crate::entities::{ CustomSmsSettingParam as EntitiesCustomSmsSettingParam, NotificationSettingAudience as EntitiesNotificationSettingAudience, };`

---

### FINDING 3

- **id:** DOMAIN-COM-003
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/communication/src/entities.rs:724` and `crates/domains/communication/src/value_objects.rs:2116`
- **description:** `SmsTemplateVariable` is defined twice with conflicting shapes. `entities.rs:724` is a struct `{ name: String, description: String }`. `value_objects.rs:2116` is a wrapper `pub struct SmsTemplateVariable(pub Vec<TemplateVariable>)`. The two types have different memory layouts under the same name.
- **expected:** A single definition. Per `docs/specs/communication/entities.md:40-46` (`SmsTemplateVariable` is a list of `(name, description)` pairs), the value-objects wrapper is closest, but the entities.rs struct is what is imported by the aggregate.
- **evidence:**
  - `crates/domains/communication/src/entities.rs:723-730` `pub struct SmsTemplateVariable { pub name: String, pub description: String, }`
  - `crates/domains/communication/src/value_objects.rs:2114-2116` `pub struct SmsTemplateVariable(pub Vec<TemplateVariable>);`

---

### FINDING 4

- **id:** DOMAIN-COM-004
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/communication/src/entities.rs:814` and `crates/domains/communication/src/value_objects.rs:2152`
- **description:** `CustomSmsSettingParam` is defined twice with conflicting shapes. `entities.rs:814` is a struct `{ key: String, value: String }`. `value_objects.rs:2152` is `pub struct CustomSmsSettingParam(pub Vec<(String, String)>)`. Two incompatible types under the same name.
- **expected:** A single definition matching the spec at `docs/specs/communication/commands.md:659-669` (`Vec<(String, String)>`).
- **evidence:**
  - `crates/domains/communication/src/entities.rs:813-819` `pub struct CustomSmsSettingParam { pub key: String, pub value: String, }`
  - `crates/domains/communication/src/value_objects.rs:2150-2152` `pub struct CustomSmsSettingParam(pub Vec<(String, String)>);`

---

### FINDING 5

- **id:** DOMAIN-COM-005
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/communication/src/entities.rs:699` and `crates/domains/communication/src/value_objects.rs:2088`
- **description:** `NoticeAudience` is defined twice with the same `Vec<RoleId>` payload but in two different modules (`entities.rs` and `value_objects.rs`). Consumers importing `NoticeAudience` from `crate::value_objects` and from `crate::entities` resolve to different types.
- **expected:** A single definition of `NoticeAudience`.
- **evidence:**
  - `crates/domains/communication/src/entities.rs:699` `pub struct NoticeAudience(pub Vec<RoleId>);`
  - `crates/domains/communication/src/value_objects.rs:2088` `pub struct NoticeAudience(pub Vec<RoleId>);`

---

### FINDING 6

- **id:** DOMAIN-COM-006
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/communication/src/services.rs:1554-1582`
- **description:** The `BlockUser` service does not implement the spec's mandated idempotency. The spec at `docs/specs/communication/workflows.md:196-197` says "BlockUser is idempotent on (block_by, block_to). A duplicate is a no-op success." The service unconditionally mints a fresh `ChatBlockUserId` and emits a new `UserBlocked` event without consulting any existing block list.
- **expected:** A lookup-then-no-op-or-emit path that returns the existing block on duplicate.
- **evidence:**
  - `crates/domains/communication/src/services.rs:1554-1582` (block_user signature and body never reads existing blocks)
  - `docs/specs/communication/workflows.md:196-197` `BlockUser is idempotent on (block_by, block_to). A duplicate is a no-op success.`

---

### FINDING 7

- **id:** DOMAIN-COM-007
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/communication/src/services.rs:1043-1070`
- **description:** `configure_absent_notification` does not implement the spec's mandated idempotency. The spec at `docs/specs/communication/workflows.md:198-199` says "ConfigureAbsentNotification is idempotent on (school_id, time_from, time_to)." The service unconditionally mints a fresh `AbsentNotificationTimeSetupId` and emits a new `AbsentNotificationScheduled` event without checking for an existing window.
- **expected:** Lookup-then-no-op-or-emit semantics keyed on `(school_id, time_from, time_to)`.
- **evidence:**
  - `crates/domains/communication/src/services.rs:1043-1070` (no list-then-check path)
  - `docs/specs/communication/workflows.md:198-199` `ConfigureAbsentNotification is idempotent on (school_id, time_from, time_to).`

---

### FINDING 8

- **id:** DOMAIN-COM-008
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/communication/src/services.rs:195-228`
- **description:** `register_complaint` does not implement the spec's mandated idempotency. The spec at `docs/specs/communication/workflows.md:191-193` says "RegisterComplaint is idempotent on (complaint_type, date, phone). Re-issuing a complaint for the same phone on the same day returns the prior record." The service unconditionally mints a fresh `ComplaintId` and emits `ComplaintRegistered`.
- **expected:** Lookup-then-no-op-or-emit keyed on `(complaint_type_id, date, phone)`.
- **evidence:**
  - `crates/domains/communication/src/services.rs:195-228` (no lookup-then-check path)
  - `docs/specs/communication/workflows.md:191-193` `RegisterComplaint is idempotent on (complaint_type, date, phone). Re-issuing ... returns the prior record.`

---

### FINDING 9

- **id:** DOMAIN-COM-009
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/communication/src/services.rs:195-228`
- **description:** `register_complaint` does not enforce the spec's pre-condition "If source is not `Anonymous`, at least one of `complaint_by` or `phone` is set" (`docs/specs/communication/commands.md:113-115`). The service unconditionally creates the complaint.
- **expected:** A `Result`-returning validation that rejects `complaint_source != Anonymous && complaint_by.is_none() && phone.is_none()`.
- **evidence:**
  - `crates/domains/communication/src/services.rs:195-228` body has no source-vs-identity check.
  - `docs/specs/communication/commands.md:113-115` `Pre-conditions: If source is not \`Anonymous\`, at least one of \`complaint_by\` or \`phone\` is set.`

---

### FINDING 10

- **id:** DOMAIN-COM-010
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/communication/src/services.rs:1186-1224`
- **description:** `send_chat_message` does not enforce the spec's pre-condition "`to_id` is not blocked by `from_id`; `from_id` is not blocked by `to_id`" (`docs/specs/communication/commands.md:417-418`). The service unconditionally mints a new `ChatMessageId` and emits `ChatMessageSent` without consulting any block list.
- **expected:** A `Result`-returning block check before the message is created.
- **evidence:**
  - `crates/domains/communication/src/services.rs:1186-1224` (no block-list consultation)
  - `docs/specs/communication/commands.md:417-418` `Pre-conditions: \`to_id\` is not blocked by \`from_id\`; \`from_id\` is not blocked by \`to_id\`.`

---

### FINDING 11

- **id:** DOMAIN-COM-011
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/communication/src/services.rs:2164-2172` and `docs/specs/communication/services.md:11-12`
- **description:** `NotificationService::select_template` has a different signature from the spec. Spec: `pub fn select_template(event: &str, destination: Destination) -> Option<SmsTemplateId>`. Code: `pub fn select_template<'a>(event: &str, channel: Channel, candidates: &'a [SmsTemplate]) -> Option<&'a SmsTemplate>`.
- **expected:** The spec signature `(event, destination) -> Option<SmsTemplateId>`.
- **evidence:**
  - `crates/domains/communication/src/services.rs:2164-2172`
  - `docs/specs/communication/services.md:12` `pub fn select_template(event: &str, destination: Destination) -> Option<SmsTemplateId> { ... }`

---

### FINDING 12

- **id:** DOMAIN-COM-012
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/communication/src/services.rs:2186-2188` and `docs/specs/communication/services.md:14`
- **description:** `NotificationService::route` has a different signature from the spec. Spec: `pub fn route(setting: &NotificationSetting, recipient: &AudienceDescriptor) -> Vec<(UserId, Channel)>`. Code: `pub fn route(setting: &NotificationSetting) -> Destination`. The code merely returns `setting.destination` and discards the recipient filter.
- **expected:** `(setting, recipient) -> Vec<(UserId, Channel)>` per spec.
- **evidence:**
  - `crates/domains/communication/src/services.rs:2186-2188` `pub fn route(setting: &NotificationSetting) -> Destination { setting.destination }`
  - `docs/specs/communication/services.md:14` `pub fn route(setting: &NotificationSetting, recipient: &AudienceDescriptor) -> Vec<(UserId, Channel)> { ... }`

---

### FINDING 13

- **id:** DOMAIN-COM-013
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/communication/src/services.rs:2195-2197` and `docs/specs/communication/services.md:15`
- **description:** `NotificationService::next_window` has a different signature from the spec. Spec: `pub fn next_window(now: NaiveTime, window: &TimeWindow) -> Option<NaiveTime>`. Code: `pub fn next_window(setup: &AbsentNotificationTimeSetup) -> (TimeOfDay, TimeOfDay)`. The signature and return type are entirely different.
- **expected:** `(now: NaiveTime, window: &TimeWindow) -> Option<NaiveTime>` per spec.
- **evidence:**
  - `crates/domains/communication/src/services.rs:2195-2197`
  - `docs/specs/communication/services.md:15` `pub fn next_window(now: NaiveTime, window: &TimeWindow) -> Option<NaiveTime> { ... }`

---

### FINDING 14

- **id:** DOMAIN-COM-014
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/communication/src/services.rs:2267-2272` and `docs/specs/communication/services.md:53`
- **description:** `ComplaintService::categorize` has a different signature from the spec. Spec: `pub fn categorize(cmd: &RegisterComplaintCommand) -> ComplaintTypeId`. Code: `pub fn categorize(complaint: &Complaint, types: &[ComplaintType]) -> String`. Different parameter shape and different return type.
- **expected:** `(cmd: &RegisterComplaintCommand) -> ComplaintTypeId` per spec.
- **evidence:**
  - `crates/domains/communication/src/services.rs:2267-2272`
  - `docs/specs/communication/services.md:53` `pub fn categorize(cmd: &RegisterComplaintCommand) -> ComplaintTypeId { ... }`

---

### FINDING 15

- **id:** DOMAIN-COM-015
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/communication/src/services.rs:2277-2279` and `docs/specs/communication/services.md:54`
- **description:** `ComplaintService::is_anonymous` has a different signature from the spec. Spec: `pub fn is_anonymous(source: ComplaintSource, by: Option<&PersonName>) -> bool`. Code: `pub fn is_anonymous(complaint: &Complaint) -> bool`. The spec parameters are source + name; the code passes the whole aggregate.
- **expected:** `(source: ComplaintSource, by: Option<&PersonName>) -> bool` per spec.
- **evidence:**
  - `crates/domains/communication/src/services.rs:2277-2279`
  - `docs/specs/communication/services.md:54` `pub fn is_anonymous(source: ComplaintSource, by: Option<&PersonName>) -> bool { ... }`

---

### FINDING 16

- **id:** DOMAIN-COM-016
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/communication/src/services.rs:2302-2314` and `docs/specs/communication/services.md:56`
- **description:** `ComplaintService::escalation_path` has a different signature and return type from the spec. Spec: `pub fn escalation_path(setting: &NotificationSetting, complaint_type: ComplaintTypeId) -> Vec<UserId>`. Code: `pub fn escalation_path(current: ComplaintStatus) -> Vec<ComplaintStatus>`. The spec routes a setting + type to a user list; the code returns a status path.
- **expected:** `(setting: &NotificationSetting, complaint_type: ComplaintTypeId) -> Vec<UserId>` per spec.
- **evidence:**
  - `crates/domains/communication/src/services.rs:2302-2314`
  - `docs/specs/communication/services.md:56` `pub fn escalation_path(setting: &NotificationSetting, complaint_type: ComplaintTypeId) -> Vec<UserId> { ... }`

---

### FINDING 17

- **id:** DOMAIN-COM-017
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/communication/src/services.rs:2220-2228` and `docs/specs/communication/services.md:35`
- **description:** `ChatService::resolve_conversation` has a different signature and return type from the spec. Spec: `pub fn resolve_conversation(from: UserId, to: UserId, existing: &[ChatConversation]) -> Option<ChatConversationId>`. Code: `pub fn resolve_conversation(a: UserId, b: UserId, conversations: &[ChatConversation]) -> Option<&ChatConversation>`. Returns a reference (not an owned id), forcing a lifetime-bound consumer.
- **expected:** `(from: UserId, to: UserId, existing: &[ChatConversation]) -> Option<ChatConversationId>` per spec.
- **evidence:**
  - `crates/domains/communication/src/services.rs:2220-2228`
  - `docs/specs/communication/services.md:35` `pub fn resolve_conversation(from: UserId, to: UserId, existing: &[ChatConversation]) -> Option<ChatConversationId> { ... }`

---

### FINDING 18

- **id:** DOMAIN-COM-018
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/communication/src/services.rs:2242-2250` and `docs/specs/communication/services.md:37`
- **description:** `ChatService::can_post` has a different signature from the spec. Spec: `pub fn can_post(group: &ChatGroup, user: &ChatGroupUser) -> bool`. Code: `pub fn can_post(group: &ChatGroup, user: UserId, membership: Option<&ChatGroupUser>) -> bool`. The spec takes a `&ChatGroupUser`; the code splits it into a `UserId` plus an `Option<&ChatGroupUser>` lookup. Also the code's logic ("not read-only ⇒ true") is inverted relative to the spec which says "Closed group only admins may post; ReadOnly group nobody may post".
- **expected:** `(group: &ChatGroup, user: &ChatGroupUser) -> bool` per spec.
- **evidence:**
  - `crates/domains/communication/src/services.rs:2242-2250` `if !group.read_only { return true; }` then `matches!(m.role, ChatGroupRole::Admin)`.
  - `docs/specs/communication/services.md:37` `pub fn can_post(group: &ChatGroup, user: &ChatGroupUser) -> bool { ... }`
  - `docs/specs/communication/services.md:44-45` `ChatService::can_post enforces the GroupType policy: in a Closed group only admins may post; in a ReadOnly group nobody may post.`

---

### FINDING 19

- **id:** DOMAIN-COM-019
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/communication/src/services.rs:2210` and `docs/specs/communication/services.md:34`
- **description:** `ChatService::is_blocked` has a different signature from the spec. Spec: `pub fn is_blocked(block_list: &[ChatBlockUser], between: (UserId, UserId)) -> bool`. Code: `pub fn is_blocked(from: UserId, blocks: &[ChatBlockUser]) -> bool`. The spec checks a `(from, to)` pair; the code only checks whether `from` has placed any block. The recipient-side block and the cross-block ("either side has blocked the other") are not detected.
- **expected:** `(block_list: &[ChatBlockUser], between: (UserId, UserId)) -> bool` per spec.
- **evidence:**
  - `crates/domains/communication/src/services.rs:2212-2214` `blocks.iter().any(|b| b.block_by == from && b.is_active())`
  - `docs/specs/communication/services.md:34` `pub fn is_blocked(block_list: &[ChatBlockUser], between: (UserId, UserId)) -> bool { ... }`
  - `docs/specs/communication/services.md:41-42` `ChatService::is_blocked returns true when either side has blocked the other, in which case the message is suppressed.`

---

### FINDING 20

- **id:** DOMAIN-COM-020
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/communication/src/commands.rs:373-390` and `docs/specs/communication/commands.md:99-109`
- **description:** `RegisterComplaintCommand` field type drift: spec uses `complaint_by: Option<PersonName>`; code uses `complaint_by: Option<UserId>`. The spec's `PersonName` cannot identify a system user; the code's `UserId` cannot capture a free-text anonymous complainant's display name.
- **expected:** `pub complaint_by: Option<PersonName>` per spec.
- **evidence:**
  - `crates/domains/communication/src/commands.rs:377` `pub complaint_by: Option<UserId>,`
  - `docs/specs/communication/commands.md:99-108` `pub struct RegisterComplaintCommand { pub tenant: TenantContext, pub complaint_by: Option<PersonName>, ... }`

---

### FINDING 21

- **id:** DOMAIN-COM-021
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/communication/src/commands.rs:523-538` and `docs/specs/communication/commands.md:178-186`
- **description:** `SendNotificationCommand` field type drift: spec uses `pub message: String`; code uses `pub message: NotificationMessage`. The spec treats the notification body as a free-form string; the code wraps it in a typed VO with separate validation rules.
- **expected:** `pub message: String` per spec.
- **evidence:**
  - `crates/domains/communication/src/commands.rs:531` `pub message: NotificationMessage,`
  - `docs/specs/communication/commands.md:178-186` `pub struct SendNotificationCommand { pub tenant: TenantContext, pub recipient_user_id: UserId, pub notification_type: NotificationType, pub message: String, ... }`

---

### FINDING 22

- **id:** DOMAIN-COM-022
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/communication/src/commands.rs:637-652` and `docs/specs/communication/commands.md:258-266`
- **description:** `CreateSmsTemplateCommand` field type drift on two fields. Spec: `purpose: TemplateKey, subject: String`. Code: `purpose: String, subject: EmailSubject`. The spec validates purpose via `TemplateKey` (1..100 chars); the code accepts any `String`.
- **expected:** `pub purpose: TemplateKey, pub subject: String` per spec.
- **evidence:**
  - `crates/domains/communication/src/commands.rs:643-645` `pub purpose: String, pub subject: EmailSubject,`
  - `docs/specs/communication/commands.md:258-266` `pub struct CreateSmsTemplateCommand { ... pub purpose: TemplateKey, pub subject: String, ... }`

---

### FINDING 23

- **id:** DOMAIN-COM-023
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/communication/src/commands.rs:837-856` and `docs/specs/communication/commands.md:660-670`
- **description:** `CreateCustomSmsSettingCommand` has three field-type drifts versus the spec. Spec: `gateway_name: String, set_auth: Option<SecretReference>, params: Vec<(String, String)>`. Code: `gateway_name: GatewayName, set_auth: Option<bool>, params: Vec<CustomSmsSettingParam>`. The spec encodes credentials via `SecretReference`; the code encodes it as a `bool`. The spec encodes params as raw tuples; the code wraps them in the conflicting duplicate struct (Finding DOMAIN-COM-004).
- **expected:** Per spec: `gateway_name: String, set_auth: Option<SecretReference>, params: Vec<(String, String)>`.
- **evidence:**
  - `crates/domains/communication/src/commands.rs:843-855`
  - `docs/specs/communication/commands.md:660-670` `pub struct CreateCustomSmsSettingCommand { ... pub gateway_name: String, pub set_auth: Option<SecretReference>, pub params: Vec<(String, String)>, }`

---

### FINDING 24

- **id:** DOMAIN-COM-024
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/communication/src/commands.rs:907-922` and `docs/specs/communication/commands.md:351-359`
- **description:** `CreateNotificationSettingCommand` field type drift on three fields. Spec: `recipient: AudienceDescriptor, subject: String, shortcode: Vec<TemplateVariable>`. Code: `recipient: NotificationSettingAudience, subject: EmailSubject, shortcode: String`. The spec's `shortcode` is a list of template variables; the code stores a single string.
- **expected:** Per spec.
- **evidence:**
  - `crates/domains/communication/src/commands.rs:915-921`
  - `docs/specs/communication/commands.md:351-359` `pub struct CreateNotificationSettingCommand { ... pub recipient: AudienceDescriptor, pub subject: String, pub shortcode: Vec<TemplateVariable>, }`

---

### FINDING 25

- **id:** DOMAIN-COM-025
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/communication/src/commands.rs:301-321` and `docs/specs/communication/commands.md:33-41`
- **description:** `UpdateNoticeCommand` field type drift on `publish_on`. Spec: `publish_on: Option<PublishOn>` (the typed wrapper VO). Code: `publish_on: Option<NaiveDate>` (raw `NaiveDate` with no clear/keep semantics).
- **expected:** `pub publish_on: Option<PublishOn>` per spec.
- **evidence:**
  - `crates/domains/communication/src/commands.rs:306` `pub publish_on: Option<NaiveDate>,`
  - `docs/specs/communication/commands.md:33-41` `pub struct UpdateNoticeCommand { ... pub publish_on: Option<PublishOn>, ... }`

---

### FINDING 26

- **id:** DOMAIN-COM-026
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/communication/src/commands.rs:1504-1517` and `docs/specs/communication/commands.md:575-582`
- **description:** `ReceiveContactMessageCommand` field type drift on `subject`. Spec: `subject: String`. Code: `subject: EmailSubject`. The spec treats the contact-form subject as a free-form string; the code enforces email-subject validation (1..=200 chars).
- **expected:** `pub subject: String` per spec.
- **evidence:**
  - `crates/domains/communication/src/commands.rs:1514` `pub subject: EmailSubject,`
  - `docs/specs/communication/commands.md:574-582` `pub struct ReceiveContactMessageCommand { ... pub subject: String, ... }`

---

### FINDING 27

- **id:** DOMAIN-COM-027
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/communication/src/events.rs:116-122` and `docs/specs/communication/events.md:46`
- **description:** `NoticeUpdated.changes` type drift: spec `pub changes: Vec<&'static str>`, code `pub changes: Vec<String>`. The spec keeps the change-list as a static string slice; the code forces a heap allocation per change.
- **expected:** `pub changes: Vec<&'static str>` per spec.
- **evidence:**
  - `crates/domains/communication/src/events.rs:118` `pub changes: Vec<String>,`
  - `docs/specs/communication/events.md:46` `pub struct NoticeUpdated { pub notice_id: NoticeId, pub changes: Vec<&'static str> }`

---

### FINDING 28

- **id:** DOMAIN-COM-028
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/communication/src/events.rs:1009-1023` and `docs/specs/communication/events.md:120`
- **description:** `SmsTemplateUpdated.changes` type drift: spec `Vec<&'static str>`, code `Vec<String>`.
- **expected:** `Vec<&'static str>` per spec.
- **evidence:**
  - `crates/domains/communication/src/events.rs:1011` `pub changes: Vec<String>,`
  - `docs/specs/communication/events.md:120` `pub struct SmsTemplateUpdated { pub sms_template_id: SmsTemplateId, pub changes: Vec<&'static str> }`

---

### FINDING 29

- **id:** DOMAIN-COM-029
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/communication/src/events.rs:3261-3268` and `docs/specs/communication/events.md:302`
- **description:** `SpeechSliderCreated.name` type drift: spec `name: PersonName`, code `name: String`. The spec wraps the leader's name in a 1..=200 char validated VO; the code stores it as a raw string.
- **expected:** `pub name: PersonName` per spec.
- **evidence:**
  - `crates/domains/communication/src/events.rs:3263` `pub name: String,`
  - `docs/specs/communication/events.md:302` `pub struct SpeechSliderCreated { pub speech_slider_id: SpeechSliderId, pub name: PersonName, pub designation: String }`

---

### FINDING 30

- **id:** DOMAIN-COM-030
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/communication/src/events.rs:3309-3316` and `docs/specs/communication/events.md:303`
- **description:** `SpeechSliderUpdated.changes` type drift: spec `Vec<&'static str>`, code `Vec<String>`.
- **expected:** `Vec<&'static str>` per spec.
- **evidence:**
  - `crates/domains/communication/src/events.rs:3312` `pub changes: Vec<String>,`
  - `docs/specs/communication/events.md:303` `pub struct SpeechSliderUpdated { pub speech_slider_id: SpeechSliderId, pub changes: Vec<&'static str> }`

---

### FINDING 31

- **id:** DOMAIN-COM-031
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/communication/src/events.rs:3107-3116` and `docs/specs/communication/events.md:287-293`
- **description:** `ContactMessageReceived` field type drift: spec declares `email: Option<EmailAddress>` and `phone: Option<PhoneNumber>`; code declares them as non-optional `email: EmailAddress` and `phone: PhoneNumber`. The spec allows anonymous contact-form submissions (no email, no phone); the code rejects them at compile-time.
- **expected:** `pub email: Option<EmailAddress>, pub phone: Option<PhoneNumber>` per spec.
- **evidence:**
  - `crates/domains/communication/src/events.rs:3110-3111` `pub email: EmailAddress, pub phone: PhoneNumber,`
  - `docs/specs/communication/events.md:287-293` `pub struct ContactMessageReceived { pub contact_message_id: ContactMessageId, pub name: PersonName, pub email: Option<EmailAddress>, pub phone: Option<PhoneNumber>, pub subject: String, }`

---

### FINDING 32

- **id:** DOMAIN-COM-032
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/communication/src/events.rs:2905-2913` and `docs/specs/communication/events.md:258`
- **description:** `ChatInvitationClassified` field name drift: spec uses `pub type: ChatInvitationTypeEnum` (Rust keyword `type`); code renames the field to `invitation_type`. The spec accepts the awkward `type` name as the canonical identifier; the code's rename is a benign ergonomic change but is a wire-format-level divergence from the spec.
- **expected:** Spec field name `type: ChatInvitationTypeEnum` per `docs/specs/communication/events.md:258`.
- **evidence:**
  - `crates/domains/communication/src/events.rs:2909` `pub invitation_type: ChatInvitationTypeEnum,`
  - `docs/specs/communication/events.md:258` `pub struct ChatInvitationClassified { pub chat_invitation_type_id: ChatInvitationTypeId, pub invitation_id: ChatInvitationId, pub type: ChatInvitationTypeEnum }`

---

### FINDING 33

- **id:** DOMAIN-COM-033
- **area:** domain-crates
- **severity:** Critical
- **location:** `docs/commands/communication.md:13-69` and `docs/specs/communication/aggregates.md:111-113,653,741,447-448`
- **description:** The commands catalog at `docs/commands/communication.md` omits commands that exist in `crates/domains/communication/src/commands.rs` and are mandated by the spec aggregate definitions: `CreateComplaintType`, `UpdateComplaintType`, `DeleteComplaintType`, `ClassifyChatInvitation`, `MarkContactMessageViewed`, `OpenChatConversation`, `CloseChatConversation`, and `DeleteChatMessage`. Consumers relying on the catalog as a quick-reference index will be unaware of these commands.
- **expected:** Rows for each of the 8 missing commands in `docs/commands/communication.md`, with capability, description, emitted events, and idempotency column populated.
- **evidence:**
  - `crates/domains/communication/src/commands.rs:471-513, 1406-1421, 1526-1535, 1035-1060, 1107-1117` (the 8 commands exist in code).
  - `docs/specs/communication/aggregates.md:111-113, 653, 741, 447-448` (spec mandates them as aggregate commands).
  - `docs/commands/communication.md` lines 13-69 do not list any of these 8 commands.

---

### FINDING 34

- **id:** DOMAIN-COM-034
- **area:** domain-crates
- **severity:** Critical
- **location:** `docs/specs/communication/commands.md:1-674` (full file)
- **description:** The spec at `docs/specs/communication/commands.md` is missing full Rust struct definitions for: `CreateComplaintTypeCommand`, `UpdateComplaintTypeCommand`, `DeleteComplaintTypeCommand`, `ClassifyChatInvitationCommand`, `MarkContactMessageViewedCommand`, `OpenChatConversationCommand`, `CloseChatConversationCommand`, and `DeleteChatMessageCommand`. The aggregate spec (`docs/specs/communication/aggregates.md`) mandates these commands as part of the public surface, and the code defines and uses them, but no spec-level definition documents their fields or capability requirements.
- **expected:** A full code block per command with fields, capability, pre-conditions, and effects.
- **evidence:**
  - `docs/specs/communication/commands.md:1-674` searches return zero matches for `CreateComplaintTypeCommand`, `UpdateComplaintTypeCommand`, `DeleteComplaintTypeCommand`, `ClassifyChatInvitationCommand`, `MarkContactMessageViewedCommand`, `OpenChatConversationCommand`, `CloseChatConversationCommand`, `DeleteChatMessageCommand`.
  - `crates/domains/communication/src/commands.rs:471, 487, 505, 1406, 1526, 1035, 1051, 1107` (the 8 commands exist with full fields in code).
  - `docs/specs/communication/aggregates.md:111-113, 653, 741, 447-448` (spec mandates them).

---

### FINDING 35

- **id:** DOMAIN-COM-035
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/communication/src/aggregate.rs:76,119`
- **description:** `Notice` carries a `notice_type: NoticeType` field and a default value `NoticeType::General` that is not present in the spec. `docs/specs/communication/aggregates.md:5-54` lists only `title`, `body`, `notice_date`, `publish_on`, `audience`, `attachment` as Notice fields. The `NoticeType` value object is also absent from `docs/specs/communication/value-objects.md`.
- **expected:** Either remove the field, or add a spec entry documenting the field and the enum.
- **evidence:**
  - `crates/domains/communication/src/aggregate.rs:76` `pub notice_type: NoticeType,`
  - `crates/domains/communication/src/aggregate.rs:119` `notice_type: NoticeType::General,`
  - `crates/domains/communication/src/value_objects.rs:260-280` defines `NoticeType` (General, Class, Student, Staff, Parent, Event).
  - `docs/specs/communication/aggregates.md:5-54` does not mention `notice_type`.
  - `docs/specs/communication/value-objects.md:1-162` does not mention `NoticeType`.

---

### FINDING 36

- **id:** DOMAIN-COM-036
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/communication/src/aggregate.rs:2683` and `crates/domains/communication/src/aggregate.rs:20`
- **description:** `aggregate.rs:2683` declares `fn _unused_imports(_: StudentId, _: BTreeMap<String, String>) {}` as a dead-code anchor to silence the `unused_imports` lint on `StudentId` and `BTreeMap`. The lint is allowed at module level (`#![allow(unused_imports)]` at line 17). The function is unreachable and adds no behavior; both imports are used elsewhere in the file via aggregate fields. The anchor itself is evidence that the lint allow is wider than necessary.
- **expected:** A focused lint allow at the import sites, or removal of the dead-code anchor.
- **evidence:**
  - `crates/domains/communication/src/aggregate.rs:2683` `fn _unused_imports(_: StudentId, _: BTreeMap<String, String>) {}`
  - `crates/domains/communication/src/aggregate.rs:17` `#![allow(unused_imports)]`

---

### FINDING 37

- **id:** DOMAIN-COM-037
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/communication/src/aggregate.rs:2210`
- **description:** `SendMessage::dispatch` performs a `usize -> u32` truncation via `as` cast: `let count = self.audience.len() as u32;`. The engine's code standards forbid `as` casts on numerics (`AGENTS.md` "Numeric conversions use TryFrom/TryInto; `as` on numerics is forbidden").
- **expected:** `u32::try_from(self.audience.len()).map_err(|_| DomainError::validation(...))?` or equivalent.
- **evidence:**
  - `crates/domains/communication/src/aggregate.rs:2210` `let count = self.audience.len() as u32;`
  - `AGENTS.md` "Numeric conversions use TryFrom/TryInto; `as` on numerics is forbidden."

---

### FINDING 38

- **id:** DOMAIN-COM-038
- **area:** domain-crates
- **severity:** Critical
- **location:** `crates/domains/communication/src/services.rs:138` and `crates/domains/communication/src/services.rs:1813` and `crates/domains/communication/src/services.rs:1197`
- **description:** Production-path uses of `.unwrap_or(...)` in service factory functions: `publish_notice` (`cmd.publish_at.unwrap_or(now)`), `unpublish_notice` (`cmd.reason.unwrap_or_default()`), and `send_chat_message` (`cmd.conversation_id.unwrap_or_else(|| ...)`). The engine's code standards forbid `unwrap()`/`expect()` in production paths.
- **expected:** Idiomatic `Option::unwrap_or` is allowed by clippy::unwrap_used only when gated by `#![allow(...)]`, but `AGENTS.md` and `docs/code-standards.md` say "unwrap, expect, panic! are forbidden in production paths" — the gating should be removed and replaced with explicit handling.
- **evidence:**
  - `crates/domains/communication/src/services.rs:138` `let published_at = cmd.publish_at.unwrap_or(now);`
  - `crates/domains/communication/src/services.rs:1813` `cmd.reason.unwrap_or_default(),`
  - `crates/domains/communication/src/services.rs:1197` `.unwrap_or_else(|| ChatConversationId::new(school, event_id_to_uuid(ids.next_event_id())));`
  - `crates/domains/communication/src/services.rs:28` `#![allow(unused_imports)]` (no `unwrap_used` allow at module level).

---

### FINDING 39

- **id:** DOMAIN-COM-039
- **area:** domain-crates
- **severity:** High
- **location:** `crates/tools/storage-parity/tests/communication_integration.rs:150`
- **description:** The integration test file `communication_integration.rs` defines 6 scenarios under `mod full_prelude_scenarios` gated by `#[cfg(any())]`. The `cfg(any())` with no conditions never matches, so none of the 6 scenarios (vertical slice, capability check, event-type round trip, append-only invariant, notification dispatch, bulk send) actually run. Only `communication_package_metadata_is_set` and `communication_full_prelude_scenarios_compile_only_when_wired` execute, both of which are trivial assertions. The `coverage.toml` rows for the 12 aggregates all carry `status = "Tested"` despite this gap.
- **expected:** Either flip the gate to `#[cfg(all())]` (so the 6 scenarios compile and run) or downgrade the coverage rows to `status = "NotTested"` / `status = "Stub"`.
- **evidence:**
  - `crates/tools/storage-parity/tests/communication_integration.rs:150` `#[cfg(any())]`
  - `crates/tools/storage-parity/tests/communication_integration.rs:117` `assert!(PACKAGE_NAME == "educore-communication");`
  - `docs/coverage.toml:1372-1480` (12 rows with `status = "Tested"` referencing `communication_integration.rs`).

---

### FINDING 40

- **id:** DOMAIN-COM-040
- **area:** domain-crates
- **severity:** High
- **location:** `crates/domains/communication/src/aggregate.rs:2683` (and the `tests/` directory is absent)
- **description:** The crate-local integration-test directory `crates/domains/communication/tests/` does not exist. Per `AGENTS.md`'s "Module Layout (per domain)" pattern and the per-domain convention used by other crates (e.g. `crates/domains/academic/tests/`), the communication crate should host its own `tests/` directory. The current setup forces all domain tests into `crates/tools/storage-parity/tests/`, which then gates them with `#[cfg(any())]` (see Finding DOMAIN-COM-039).
- **expected:** A populated `crates/domains/communication/tests/` directory containing end-to-end scenarios for the 26 aggregates, the 73 events, and the 70+ service factory functions.
- **evidence:**
  - `crates/domains/communication/` contents (no `tests/` subdir).
  - `crates/domains/communication/Cargo.toml` has no `[[test]]` entries.

---

### FINDING 41

- **id:** DOMAIN-COM-041
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/communication/src/lib.rs:118-137` (prelude block) and `crates/domains/communication/src/lib.rs:54` (comment)
- **description:** Self-inconsistent comment vs. prelude block: `lib.rs:54` says "26 headline aggregate roots (from crate::aggregate)" but the prelude block (lines 50-57) only re-exports 25 distinct types from `crate::aggregate::*` (the second-to-last line ends with `Notification, NotificationSetting, PhoneCallLog, SendMessage, SmsGateway, SmsLog, SmsTemplate, SpeechSlider,` — 25 names visible before the trailing comma; line 56 has `};`). The 26th aggregate (`ChatStatusRecord`) is missing from the prelude re-export. Consumers cannot access `ChatStatusRecord` via `educore::communication::*`.
- **expected:** Prelude re-exports all 26 aggregates including `ChatStatusRecord` (or rename to `ChatStatus` per Finding DOMAIN-COM-001).
- **evidence:**
  - `crates/domains/communication/src/lib.rs:50-57` re-export list (notice absence of `ChatStatusRecord`).
  - `crates/domains/communication/src/aggregate.rs:2068` defines `pub struct ChatStatusRecord`.

---

### FINDING 42

- **id:** DOMAIN-COM-042
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/communication/src/lib.rs:15-22`
- **description:** Module visibility is inconsistent with the manifest. The manifest at `crates/domains/communication/.phase10-manifest.md:695-703` lists modules as `pub mod value_objects; mod aggregate; pub mod commands; mod entities; mod errors; pub mod events; pub mod query; mod repository; pub mod services;`. The actual `lib.rs:15-22` declares `mod aggregate; mod entities; mod errors; mod repository;` (4 modules private), but `lib.rs` re-exports the contents of those private modules from the prelude. Consumers cannot `use educore_communication::aggregate::Notice` directly even though `Notice` is reachable via the prelude.
- **expected:** Per the manifest, all 9 modules are `pub mod`. The current private visibility contradicts the manifest.
- **evidence:**
  - `crates/domains/communication/src/lib.rs:15-22` `mod aggregate; pub mod commands; mod entities; mod errors; pub mod events; pub mod query; mod repository; pub mod services; pub mod value_objects;`
  - `crates/domains/communication/.phase10-manifest.md:695-703` lists `pub mod value_objects; mod aggregate; pub mod commands; mod entities; mod errors; pub mod events; pub mod query; mod repository; pub mod services;` — note `mod aggregate;`, `mod entities;`, `mod errors;`, `mod repository;` all `mod` not `pub mod`.

---

### FINDING 43

- **id:** DOMAIN-COM-043
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/communication/src/lib.rs:111-116` and `crates/domains/communication/src/lib.rs:118-137`
- **description:** The prelude block (lines 118-137) re-exports 70 service functions under the "70 pure factory service fns" comment, but the count is actually 72. The same block also lists "7 headline service fns" (lines 112-116) but only re-exports 6 of them (`mark_as_read`, `notify_user`, `send_chat_message`, `send_complaint_message`, `send_email_message`, `send_notice_message`, `send_sms_message` = 7 names, but the comment says 7). The actual count from `services.rs` is 72 sync + 7 async = 79 functions. The "70 pure factory service functions" comment is misleading.
- **expected:** Comments accurately reflect the function count (72 pure factory fns + 7 headline async fns).
- **evidence:**
  - `crates/domains/communication/src/lib.rs:117` `// 70 pure factory service fns (re-export all from crate::services)`
  - `grep -c "^pub fn " crates/domains/communication/src/services.rs` = 72 sync fns.
  - `grep -c "pub async fn " crates/domains/communication/src/services.rs` = 7 async fns.

---

### FINDING 44

- **id:** DOMAIN-COM-044
- **area:** domain-crates
- **severity:** Low
- **location:** `crates/domains/communication/src/services.rs:1135-1162`
- **description:** `open_chat_conversation` mints a new `ChatConversationId` on every invocation regardless of whether an open conversation between the same two users already exists. The spec at `docs/specs/communication/commands.md:420-423` says "If `conversation_id` is null and a prior conversation exists between `from_id` and `to_id`, the existing conversation is reused". The service unconditionally creates a new conversation aggregate.
- **expected:** A lookup-then-create-or-reuse path that searches existing conversations for `(from_id, to_id)`.
- **evidence:**
  - `crates/domains/communication/src/services.rs:1135-1162` (no lookup path).
  - `docs/specs/communication/commands.md:420-423` `Effects: Creates a ChatMessage and emits ChatMessageSent. If conversation_id is null and a prior conversation exists between from_id and to_id, the existing conversation is reused; otherwise a new ChatConversation is implicitly opened.`

---

### FINDING 45

- **id:** DOMAIN-COM-045
- **area:** domain-crates
- **severity:** High
- **location:** `docs/specs/communication/value-objects.md:41-162` (full table) and `crates/domains/communication/src/value_objects.rs:959-982`
- **description:** The "complaint workflow value object" at `value_objects.rs:959` is `ComplaintAction` (an enum with `Open`, `InProgress`, `Resolve` variants). The spec table at `docs/specs/communication/value-objects.md:41-162` does NOT list `ComplaintAction` as a value object. The spec's only mention of `ComplaintAction` is at `docs/specs/communication/services.md:55` as a parameter to `ComplaintService::next_status`. The Rust source places it in `value_objects.rs` but the spec categorizes it as a service-input type, not a value object.
- **expected:** Either move `ComplaintAction` to `services.rs` (matching the spec's placement in `services.md`) or add a row to `docs/specs/communication/value-objects.md` documenting the VO.
- **evidence:**
  - `crates/domains/communication/src/value_objects.rs:959-982` `pub enum ComplaintAction { Open, InProgress, Resolve }`
  - `docs/specs/communication/value-objects.md:41-162` searches return zero matches for `ComplaintAction`.
  - `docs/specs/communication/services.md:55` `pub fn next_status(current: ComplaintStatus, action: ComplaintAction) -> ComplaintStatus { ... }` — the only spec mention.

---

### FINDING 46

- **id:** DOMAIN-COM-046
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/communication/src/entities.rs:42-44`
- **description:** `#![allow(missing_docs)]` at the top of `entities.rs` (and similarly in `aggregate.rs`, `commands.rs`, `events.rs`, `query.rs`, `repository.rs`, `services.rs`, `value_objects.rs`) suppresses `deny(missing_docs)` for the entire module. `lib.rs:10` has `#![deny(missing_docs)]` for the crate, but the inner modules all opt out. Consumers browsing `educore-communication::entities::NoticeAttachment` see no rustdoc; the per-module lint allow is at odds with the crate-level deny.
- **expected:** Either remove the module-level `allow(missing_docs)` (so the crate-level deny fires and forces docs) or replace the crate-level deny with `warn(missing_docs)` and document the policy explicitly.
- **evidence:**
  - `crates/domains/communication/src/lib.rs:10` `#![deny(missing_docs)]`
  - `crates/domains/communication/src/entities.rs:42` `#![allow(missing_docs)]`
  - `crates/domains/communication/src/aggregate.rs:16` `#![allow(missing_docs)]`
  - `crates/domains/communication/src/commands.rs:18` `#![allow(missing_docs)]`
  - `crates/domains/communication/src/events.rs:16` `#![allow(missing_docs)]`
  - `crates/domains/communication/src/query.rs:14` `#![allow(missing_docs)]`
  - `crates/domains/communication/src/repository.rs:30` `#![allow(missing_docs)]`
  - `crates/domains/communication/src/services.rs:28` `#![allow(missing_docs)]`
  - `crates/domains/communication/src/value_objects.rs:27` `#![allow(missing_docs)]`

---

### FINDING 47

- **id:** DOMAIN-COM-047
- **area:** domain-crates
- **severity:** Medium
- **location:** `crates/domains/communication/src/entities.rs` (entire file, 833 lines) and `docs/specs/communication/tables.md:7-33` (25 rows)
- **description:** None of the 25 tables in `docs/specs/communication/tables.md` has a corresponding `#[derive(DomainQuery)]` struct in `entities.rs`. The spec at `docs/specs/communication/tables.md:7-33` lists 25 tables (`communication_notice_boards`, `communication_complaints`, etc.), but `entities.rs` defines child entities (with their own aggregate-scoped fields) rather than row-level `DomainQuery` derive structs. The `DomainQuery` macro is referenced as future work in `query.rs:7-9` and is not yet shipped; until the macro lands, no table has a typed query AST consumer.
- **expected:** A `DomainQuery`-derived struct per table row, plus the macro itself.
- **evidence:**
  - `crates/domains/communication/src/entities.rs` (no `#[derive(DomainQuery)]` anywhere).
  - `crates/domains/communication/src/query.rs:7-9` `Phase 10 ships the 26 typed query stubs ... The typed executors land in a follow-up phase alongside the #[derive(DomainQuery)] macro emissions`
  - `crates/domains/communication/src/query.rs:59` `"NoticeQuery::execute is a Phase 10 stub; real executor lands with the DomainQuery macro"`
  - `docs/specs/communication/tables.md:7-33` 25 table rows without a corresponding derive struct.

---

### END FINDINGS

Total Findings: 47
