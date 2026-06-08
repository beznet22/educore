# Communication Domain — Permissions

Permissions are capability strings. They are not roles. The RBAC domain
maps capabilities to roles.

## Naming

```text
<Domain>.<Aggregate>.<Action>
```

## Capabilities

### Communication (Cross-Cutting)

- `Communication.Read`

### Notice

- `Notice.Create`
- `Notice.Update`
- `Notice.Publish`
- `Notice.Unpublish`
- `Notice.Delete`
- `Notice.Read`

### Complaint

- `Complaint.Create`
- `Complaint.Update`
- `Complaint.Assign`
- `Complaint.Resolve`
- `Complaint.Note`
- `Complaint.Read`
- `ComplaintType.Create`
- `ComplaintType.Update`
- `ComplaintType.Delete`

### Notification

- `Notification.Send`
- `Notification.Read`
- `Notification.Withdraw`
- `Notification.Read.All` (admin override across all users)

### Email & SMS Logs

- `EmailLog.Create`
- `EmailLog.Read`
- `SmsLog.Create`
- `SmsLog.Read`

### Template

- `Template.Create`
- `Template.Update`
- `Template.Enable`
- `Template.Disable`
- `Template.Delete`
- `Template.Read`

### Email Setting

- `EmailSetting.Configure`
- `EmailSetting.Activate`
- `EmailSetting.Read`
- `EmailSetting.Delete`

### SMS Gateway

- `SmsGateway.Configure`
- `SmsGateway.Activate`
- `SmsGateway.Read`
- `SmsGateway.Delete`
- `CustomSmsSetting.Create`
- `CustomSmsSetting.Update`
- `CustomSmsSetting.Delete`

### Notification Setting

- `NotificationSetting.Create`
- `NotificationSetting.Update`
- `NotificationSetting.Delete`
- `NotificationSetting.Read`

### Absent Notification

- `AbsentNotification.Configure`
- `AbsentNotification.Enable`
- `AbsentNotification.Disable`
- `AbsentNotification.Delete`
- `AbsentNotification.Read`

### Chat — One-to-One

- `Chat.Send`
- `Chat.Read`
- `Chat.Delete` (per-user)
- `Chat.Block`
- `Chat.Unblock`
- `Chat.Invite`
- `Chat.Accept`
- `Chat.Reject`
- `Chat.SetStatus`

### Chat — Group

- `ChatGroup.Create`
- `ChatGroup.Update`
- `ChatGroup.Delete`
- `ChatGroup.AddUser`
- `ChatGroup.SetRole`
- `ChatGroup.RemoveUser`
- `ChatGroup.Moderate`

### Send Message (Bulk)

- `SendMessage.Create`
- `SendMessage.Dispatch`
- `SendMessage.Cancel`
- `SendMessage.Read`

### Contact Message

- `ContactMessage.Create`
- `ContactMessage.View`
- `ContactMessage.Reply`
- `ContactMessage.Delete`

### Speech Slider

- `SpeechSlider.Create`
- `SpeechSlider.Update`
- `SpeechSlider.Delete`
- `SpeechSlider.Read`

### Phone Call Log

- `PhoneCallLog.Create`
- `PhoneCallLog.Update`
- `PhoneCallLog.Read`

## Default Role Mapping

The platform's default role catalog binds the following:

| Role        | Capabilities (highlights)                                                       |
| ----------- | ------------------------------------------------------------------------------- |
| SuperAdmin  | All                                                                             |
| SchoolAdmin | All within the school                                                          |
| Teacher     | Notice.Read, Complaint.Create, Complaint.Read, Notification.Read, Chat.*, SendMessage.Create, SendMessage.Dispatch, PhoneCallLog.*, Template.Read |
| Student     | Chat.*, Notice.Read, Notification.Read, Complaint.Create                       |
| Parent      | Chat.*, Notice.Read, Notification.Read, Complaint.Create                       |
| Reception   | Complaint.*, ContactMessage.*, PhoneCallLog.*                                  |
| Marketing   | SpeechSlider.*, Notice.Create, Notice.Publish, SendMessage.*                   |

The default mapping is a starting point and is configurable per school.

## Authorization Pattern

Capabilities are checked at the command boundary. The engine never
trusts the caller to assert their own role.

```rust
if !engine.rbac().has(actor_id, Capability::NoticePublish).await? {
    return Err(DomainError::forbidden("missing capability"));
}
```

Some commands have a secondary ownership check: a student reading a
chat message is only allowed to read a message addressed to them; a
user blocking is the `block_by` party.

## Read vs Write

Read capabilities are explicit. The engine does not assume that
`Notice.Read` implies `Notice.Create`. A consumer may grant only
read-only access to a parent or auditor.

## Tenant Isolation

Every capability check is paired with a tenant check. The actor must
be authenticated to the school that owns the target aggregate. There
is no cross-tenant capability elevation. The `Communication` domain
never accepts commands across schools.
