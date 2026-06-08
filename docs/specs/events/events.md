# Events Domain — Events

Domain events describe facts that have already happened. They are
immutable, append-only records used for cross-domain integration, audit,
and event sourcing.

All events implement:

```rust
pub trait DomainEvent: Serialize + DeserializeOwned + Send + Sync {
    const TYPE: &'static str;
    fn aggregate_id(&self) -> Uuid;
    fn school_id(&self) -> SchoolId;
    fn occurred_at(&self) -> Timestamp;
}
```

The event envelope wraps the event with metadata:

```rust
pub struct EventEnvelope<E> {
    pub event_id: EventId,
    pub event_type: &'static str,
    pub school_id: SchoolId,
    pub aggregate_id: Uuid,
    pub aggregate_type: &'static str,
    pub actor_id: UserId,
    pub correlation_id: CorrelationId,
    pub causation_id: Option<EventId>,
    pub occurred_at: Timestamp,
    pub payload: E,
}
```

## Calendar Event Lifecycle

```rust
pub struct EventCreated {
    pub event_id: CalendarEventId,
    pub event_title: EventTitle,
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
    pub for_whom: ForWhom,
    pub academic_id: AcademicYearId,
}

pub struct EventUpdated { pub event_id: CalendarEventId, pub changes: Vec<&'static str> }
pub struct EventDeleted { pub event_id: CalendarEventId }
```

**Subscribers:**
- `communication` may dispatch notifications to the audience
  (subscribes to `EventCreated`).
- The calendar UI port (consumer adapter) re-renders.

## Holiday Lifecycle

```rust
pub struct HolidayCreated {
    pub holiday_id: HolidayId,
    pub holiday_title: HolidayTitle,
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
    pub academic_id: AcademicYearId,
}

pub struct HolidayUpdated { pub holiday_id: HolidayId, pub changes: Vec<&'static str> }
pub struct HolidayDeleted { pub holiday_id: HolidayId }
```

**Subscribers:**
- `attendance` may mark the days as non-instructional and exempt
  students from attendance requirements.

## Weekend Lifecycle

```rust
pub struct WeekendCreated {
    pub weekend_id: WeekendId,
    pub name: WeekendName,
    pub order: WeekendOrder,
    pub is_weekend: bool,
}

pub struct WeekendUpdated { pub weekend_id: WeekendId, pub changes: Vec<&'static str> }
pub struct WeekendsConfigured { pub school_id: SchoolId, pub weekend_count: u32 }
pub struct WeekendDeleted { pub weekend_id: WeekendId }
```

## Incident Lifecycle

```rust
pub struct IncidentReported {
    pub incident_id: IncidentId,
    pub title: IncidentTitle,
    pub point: IncidentPoint,
    pub reported_by: UserId,
    pub reported_at: Timestamp,
}

pub struct IncidentUpdated { pub incident_id: IncidentId, pub changes: Vec<&'static str> }
pub struct IncidentResolved {
    pub incident_id: IncidentId,
    pub resolved_by: UserId,
    pub resolved_at: Timestamp,
}

pub struct IncidentDeleted { pub incident_id: IncidentId }
```

**Subscribers:**
- HR or discipline policy may record a behavior note against
  assigned students or staff.

## Incident Assignment

```rust
pub struct IncidentAssigned {
    pub assign_incident_id: AssignIncidentId,
    pub incident_id: IncidentId,
    pub student_id: Option<StudentId>,
    pub user_id: Option<UserId>,
    pub point: IncidentPoint,
    pub added_by: UserId,
}

pub struct IncidentReassigned {
    pub assign_incident_id: AssignIncidentId,
    pub incident_id: IncidentId,
    pub from_point: IncidentPoint,
    pub to_point: IncidentPoint,
}

pub struct IncidentUnassigned {
    pub assign_incident_id: AssignIncidentId,
    pub incident_id: IncidentId,
    pub removed_by: UserId,
}
```

## Incident Comments

```rust
pub struct IncidentCommented {
    pub incident_comment_id: IncidentCommentId,
    pub incident_id: IncidentId,
    pub user_id: UserId,
    pub comment: IncidentCommentBody,
    pub commented_at: Timestamp,
}

pub struct IncidentCommentDeleted {
    pub incident_comment_id: IncidentCommentId,
    pub incident_id: IncidentId,
    pub deleted_by: UserId,
}
```

## Calendar Setting

```rust
pub struct CalendarSettingCreated {
    pub calendar_setting_id: CalendarSettingId,
    pub menu_name: CalendarMenuName,
    pub status: CalendarStatus,
}

pub struct CalendarSettingUpdated { pub calendar_setting_id: CalendarSettingId, pub changes: Vec<&'static str> }
pub struct CalendarSettingEnabled { pub calendar_setting_id: CalendarSettingId }
pub struct CalendarSettingDisabled { pub calendar_setting_id: CalendarSettingId }
pub struct CalendarSettingDeleted { pub calendar_setting_id: CalendarSettingId }
```
