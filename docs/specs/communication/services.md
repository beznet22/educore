# Communication Domain — Services

Domain services encapsulate business logic that does not fit cleanly in
a single aggregate. They are stateless, sync, and pure (no I/O).

## NotificationService

```rust
pub struct NotificationService;

impl NotificationService {
    pub fn select_template(event: &str, destination: Destination) -> Option<SmsTemplateId> { ... }
    pub fn render(template: &SmsTemplate, variables: &BTreeMap<String, String>) -> Result<RenderedBody, RenderError> { ... }
    pub fn route(setting: &NotificationSetting, recipient: &AudienceDescriptor) -> Vec<(UserId, Channel)> { ... }
    pub fn next_window(now: NaiveTime, window: &TimeWindow) -> Option<NaiveTime> { ... }
}
```

`NotificationService::render` is the canonical template renderer: it
parses the body for `{{name}}` placeholders, validates that every
declared variable is supplied, and produces a `RenderedBody`. Rendering
fails when a variable is missing or has the wrong type.

`NotificationService::route` resolves a `NotificationSetting` to a list
of `(UserId, Channel)` pairs. Recipients in roles the user does not
hold are filtered out.

## ChatService

```rust
pub struct ChatService;

impl ChatService {
    pub fn is_blocked(block_list: &[ChatBlockUser], between: (UserId, UserId)) -> bool { ... }
    pub fn resolve_conversation(from: UserId, to: UserId, existing: &[ChatConversation]) -> Option<ChatConversationId> { ... }
    pub fn fan_out_group_recipients(group: &ChatGroup, members: &[ChatGroupUser]) -> Vec<UserId> { ... }
    pub fn can_post(group: &ChatGroup, user: &ChatGroupUser) -> bool { ... }
}
```

`ChatService::is_blocked` returns `true` when either side has blocked
the other, in which case the message is suppressed.

`ChatService::can_post` enforces the `GroupType` policy: in a `Closed`
group only admins may post; in a `ReadOnly` group nobody may post.

## ComplaintService

```rust
pub struct ComplaintService;

impl ComplaintService {
    pub fn categorize(cmd: &RegisterComplaintCommand) -> ComplaintTypeId { ... }
    pub fn is_anonymous(source: ComplaintSource, by: Option<&PersonName>) -> bool { ... }
    pub fn next_status(current: ComplaintStatus, action: ComplaintAction) -> ComplaintStatus { ... }
    pub fn escalation_path(setting: &NotificationSetting, complaint_type: ComplaintTypeId) -> Vec<UserId> { ... }
}
```

`ComplaintService::next_status` is the canonical state machine: `Open →
InProgress → Resolved`. A re-issue of `Resolved` is a no-op.

`ComplaintService::escalation_path` is optional; the default returns
empty. Consumers may implement it to route unresolved complaints to a
head teacher after N days.

## AbsentNotificationService

```rust
pub struct AbsentNotificationService;

impl AbsentNotificationService {
    pub fn in_window(now: NaiveTime, window: &TimeWindow) -> bool { ... }
    pub fn should_dispatch(setting: &NotificationSetting, event: &str) -> bool { ... }
    pub fn build_dispatch(setting: &NotificationSetting, student: &Student) -> AbsentNotificationDispatch { ... }
    pub fn render(setting: &NotificationSetting, template: &SmsTemplate, student: &Student) -> Result<RenderedBody, RenderError> { ... }
}
```

This service is the canonical implementation of the absence
notification workflow. It subscribes to `StudentMarkedAbsent` and
decides whether to dispatch now, queue, or skip.

## TemplateService

```rust
pub struct TemplateService;

impl TemplateService {
    pub fn validate_body(body: &str, variables: &[TemplateVariable]) -> Result<(), ValidationError> { ... }
    pub fn declared(body: &str) -> Vec<String> { ... }
    pub fn substitute(body: &str, vars: &BTreeMap<String, String>) -> Result<String, RenderError> { ... }
    pub fn lint(body: &str) -> Vec<RenderWarning> { ... }
}
```

`TemplateService::lint` reports warnings for unused variables,
mismatched braces, or HTML in an SMS template. These are advisory.

## SmsDispatchPolicy

```rust
pub struct SmsDispatchPolicy;

impl Policy<DispatchSendMessageCommand> for SmsDispatchPolicy {
    type Outcome = Allow | Deny { reason: &'static str };
    fn check(&self, ctx: &Context, cmd: &DispatchSendMessageCommand) -> Outcome { ... }
}
```

A school policy that may cap the number of SMS recipients per job
(e.g. `MaxRecipientsPerJob(500)`). Consumers configure the cap.

## Specification: ActiveRecipients

```rust
pub struct ActiveRecipients;

impl Specification<UserId> for ActiveRecipients {
    fn is_satisfied_by(&self, u: &UserId) -> bool { ... }
}
```

Filters users whose status is `Active` and whose role is current.

## Specification: NoticesPublishedInRange

```rust
pub struct NoticesPublishedInRange {
    pub from: NaiveDate,
    pub to: NaiveDate,
}

impl Specification<Notice> for NoticesPublishedInRange {
    fn is_satisfied_by(&self, n: &Notice) -> bool { ... }
}
```

Used by reporting queries to scope by date.

## ChatInvitePolicy

```rust
pub struct ChatInvitePolicy;

impl Policy<SendChatInvitationCommand> for ChatInvitePolicy {
    type Outcome = Allow | Deny { reason: &'static str };
    fn check(&self, ctx: &Context, cmd: &SendChatInvitationCommand) -> Outcome { ... }
}
```

A policy that prevents inviting users who have explicitly blocked the
actor or have not enabled chat.

## Cross-Domain Coordinator

A thin coordinator lives in the engine facade and orchestrates
multi-domain flows (e.g. absence notification = attendance + communication).
It is **not** a service; it composes command calls:

```rust
pub struct CommunicationCoordinator<'a> {
    engine: &'a Engine,
}

impl<'a> CommunicationCoordinator<'a> {
    pub async fn publish_notice(&self, cmd: PublishNoticeCommand) -> Result<Notice, DomainError> {
        let notice = self.engine.communication().publish_notice(cmd).await?;
        // Subscribers (notification adapter) handle the fan-out in
        // response to the NoticePublished event.
        Ok(notice)
    }
}
```

Domain services are pure. Cross-domain coordination happens through
events and command composition, never through service-to-service calls.
