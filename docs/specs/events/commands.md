# Events Domain — Commands

Commands describe intent. They are validated, authorized, and dispatched
to the relevant aggregate. Every command produces zero or more events
that are recorded in the event log.

All commands carry a `TenantContext` (school + actor + correlation) and
are rejected if the actor lacks the required capability.

## CreateEvent

```rust
pub struct CreateEventCommand {
    pub tenant: TenantContext,
    pub event_title: EventTitle,
    pub for_whom: ForWhom,
    pub role_ids: Vec<RoleId>,
    pub url: Option<Url>,
    pub event_location: Option<EventLocation>,
    pub event_des: Option<EventDescription>,
    pub from_date: EventDate,
    pub to_date: EventDate,
    pub upload_image: Option<FileReference>,
}
```

**Capability:** `Event.Create`
**Pre-conditions:**
- `from_date ≤ to_date`.
- The actor has at least one of the role ids in `role_ids` (when
  `for_whom != All`).

**Effects:** Creates a `CalendarEvent` and emits `EventCreated`.

## UpdateEvent

```rust
pub struct UpdateEventCommand {
    pub tenant: TenantContext,
    pub event_id: CalendarEventId,
    pub event_title: Option<EventTitle>,
    pub for_whom: Option<ForWhom>,
    pub role_ids: Option<Vec<RoleId>>,
    pub url: Option<Url>,
    pub event_location: Option<EventLocation>,
    pub event_des: Option<EventDescription>,
    pub from_date: Option<EventDate>,
    pub to_date: Option<EventDate>,
    pub upload_image: Option<FileReference>,
}
```

**Capability:** `Event.Update`
**Pre-conditions:** Event exists and is not cancelled.
**Effects:** Emits `EventUpdated`.

## DeleteEvent

```rust
pub struct DeleteEventCommand {
    pub tenant: TenantContext,
    pub event_id: CalendarEventId,
}
```

**Capability:** `Event.Delete`
**Pre-conditions:** No notification fan-out has occurred, or the actor
has admin override.
**Effects:** Emits `EventDeleted`. Soft delete; audit record remains.

## CreateHoliday

```rust
pub struct CreateHolidayCommand {
    pub tenant: TenantContext,
    pub holiday_title: HolidayTitle,
    pub details: Option<HolidayDetails>,
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
    pub upload_image: Option<FileReference>,
}
```

**Capability:** `Holiday.Create`
**Pre-conditions:** `from_date ≤ to_date`.
**Effects:** Emits `HolidayCreated`.

## UpdateHoliday

```rust
pub struct UpdateHolidayCommand {
    pub tenant: TenantContext,
    pub holiday_id: HolidayId,
    pub holiday_title: Option<HolidayTitle>,
    pub details: Option<HolidayDetails>,
    pub from_date: Option<NaiveDate>,
    pub to_date: Option<NaiveDate>,
    pub upload_image: Option<FileReference>,
}
```

**Capability:** `Holiday.Update`
**Effects:** Emits `HolidayUpdated`.

## DeleteHoliday

```rust
pub struct DeleteHolidayCommand {
    pub tenant: TenantContext,
    pub holiday_id: HolidayId,
}
```

**Capability:** `Holiday.Delete`
**Effects:** Emits `HolidayDeleted`.

## ConfigureWeekends

```rust
pub struct ConfigureWeekendsCommand {
    pub tenant: TenantContext,
    pub entries: Vec<WeekendEntry>,
}

pub struct WeekendEntry {
    pub name: WeekendName,
    pub order: WeekendOrder,
    pub is_weekend: IsWeekend,
}
```

**Capability:** `Weekend.Configure`
**Pre-conditions:** All names are unique within the school.
**Effects:** Creates, updates, or removes the listed weekend entries
to match the supplied set. Emits `WeekendsConfigured`.

## CreateIncident

```rust
pub struct CreateIncidentCommand {
    pub tenant: TenantContext,
    pub title: IncidentTitle,
    pub point: IncidentPoint,
    pub description: IncidentDescription,
}
```

**Capability:** `Incident.Create`
**Effects:** Emits `IncidentReported`.

## UpdateIncident

```rust
pub struct UpdateIncidentCommand {
    pub tenant: TenantContext,
    pub incident_id: IncidentId,
    pub title: Option<IncidentTitle>,
    pub point: Option<IncidentPoint>,
    pub description: Option<IncidentDescription>,
}
```

**Capability:** `Incident.Update`
**Pre-conditions:** Incident is not `Resolved`.
**Effects:** Emits `IncidentUpdated`.

## AssignIncident

```rust
pub struct AssignIncidentCommand {
    pub tenant: TenantContext,
    pub incident_id: IncidentId,
    pub student_id: Option<StudentId>,
    pub user_id: Option<UserId>,
    pub record_id: Option<StudentRecordId>,
    pub point: IncidentPoint,
}
```

**Capability:** `Incident.Assign`
**Pre-conditions:** Exactly one of `student_id` or `user_id` is set.
**Effects:** Emits `IncidentAssigned`.

## ReassignIncident

```rust
pub struct ReassignIncidentCommand {
    pub tenant: TenantContext,
    pub assign_incident_id: AssignIncidentId,
    pub point: IncidentPoint,
}
```

**Capability:** `Incident.Reassign`
**Effects:** Emits `IncidentReassigned`.

## UnassignIncident

```rust
pub struct UnassignIncidentCommand {
    pub tenant: TenantContext,
    pub assign_incident_id: AssignIncidentId,
}
```

**Capability:** `Incident.Unassign`
**Effects:** Emits `IncidentUnassigned`.

## CommentOnIncident

```rust
pub struct CommentOnIncidentCommand {
    pub tenant: TenantContext,
    pub incident_id: IncidentId,
    pub comment: IncidentCommentBody,
}
```

**Capability:** `Incident.Comment`
**Effects:** Emits `IncidentCommented`.

## ResolveIncident

```rust
pub struct ResolveIncidentCommand {
    pub tenant: TenantContext,
    pub incident_id: IncidentId,
    pub resolution_note: Option<IncidentCommentBody>,
}
```

**Capability:** `Incident.Resolve`
**Pre-conditions:** Incident is not already `Resolved`.
**Effects:** Status transitions to `Resolved`; emits `IncidentResolved`.

## DeleteIncidentComment

```rust
pub struct DeleteIncidentCommentCommand {
    pub tenant: TenantContext,
    pub incident_comment_id: IncidentCommentId,
}
```

**Capability:** `IncidentComment.Delete`
**Effects:** Emits `IncidentCommentDeleted`.

## CreateCalendarSetting

```rust
pub struct CreateCalendarSettingCommand {
    pub tenant: TenantContext,
    pub menu_name: CalendarMenuName,
    pub status: CalendarStatus,
    pub font_color: FontColor,
    pub bg_color: BackgroundColor,
}
```

**Capability:** `CalendarSetting.Create`
**Effects:** Emits `CalendarSettingCreated`.

## UpdateCalendarSetting / EnableCalendarSetting / DisableCalendarSetting / DeleteCalendarSetting

```rust
pub struct UpdateCalendarSettingCommand { ... }
pub struct EnableCalendarSettingCommand { ... }
pub struct DisableCalendarSettingCommand { ... }
pub struct DeleteCalendarSettingCommand { ... }
```

**Capabilities:** `CalendarSetting.Update`, `CalendarSetting.Enable`,
`CalendarSetting.Disable`, `CalendarSetting.Delete`.
