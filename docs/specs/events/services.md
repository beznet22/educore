# Events Domain — Services

Domain services encapsulate business logic that does not fit cleanly in
a single aggregate. They are stateless, sync, and pure (no I/O).

## CalendarService

```rust
pub struct CalendarService;

impl CalendarService {
    pub fn in_range(event: &CalendarEvent, date: NaiveDate) -> bool { ... }
    pub fn audience_resolves_to(audience: &ForWhom, role_ids: &[RoleId], actor: &ActorRoles) -> bool { ... }
    pub fn overlaps(a: &EventDateRange, b: &EventDateRange) -> bool { ... }
    pub fn visible_to(event: &CalendarEvent, actor: &ActorRoles) -> bool { ... }
}
```

`CalendarService::visible_to` is the canonical "can this actor see
this event" predicate. It checks `for_whom` against the actor's
roles and the `role_ids` list.

## HolidayService

```rust
pub struct HolidayService;

impl HolidayService {
    pub fn contains(holiday: &Holiday, date: NaiveDate) -> bool { ... }
    pub fn overlaps(a: &Holiday, b: &Holiday) -> bool { ... }
    pub fn is_instructional(weekends: &[Weekend], holidays: &[Holiday], date: NaiveDate) -> bool { ... }
    pub fn instructional_days_in(weekends: &[Weekend], holidays: &[Holiday], from: NaiveDate, to: NaiveDate) -> u32 { ... }
}
```

`HolidayService::is_instructional` is the canonical answer to "is
school in session on date X?". It returns `false` when the date is a
weekend day or a holiday.

## IncidentService

```rust
pub struct IncidentService;

impl IncidentService {
    pub fn next_status(current: IncidentStatus, action: IncidentAction) -> IncidentStatus { ... }
    pub fn total_points(assignments: &[AssignIncident]) -> i32 { ... }
    pub fn participants(assignments: &[AssignIncident]) -> Vec<UserReference> { ... }
    pub fn is_resolved(incident: &Incident) -> bool { ... }
}
```

`IncidentService::next_status` is the canonical state machine:
`Open → InProgress → Resolved`. Re-issuing `Resolved` is a no-op.

## WeekendService

```rust
pub struct WeekendService;

impl WeekendService {
    pub fn reconcile(current: &[Weekend], proposed: &[Weekend]) -> Vec<WeekendChange> { ... }
    pub fn is_weekend(weekends: &[Weekend], date: NaiveDate) -> bool { ... }
    pub fn ordered(weekends: &[Weekend]) -> Vec<&Weekend> { ... }
}
```

`WeekendService::reconcile` is the canonical diff for
`ConfigureWeekends`. The output is a list of `WeekendChange` actions
(create, update, delete) that the command processor applies.

## CalendarSettingService

```rust
pub struct CalendarSettingService;

impl CalendarSettingService {
    pub fn validate_color(c: &CssColor) -> Result<(), ValidationError> { ... }
    pub fn visible(settings: &[CalendarSetting], name: &CalendarMenuName) -> bool { ... }
}
```

## Specification: ActiveIncidents

```rust
pub struct ActiveIncidents;

impl Specification<Incident> for ActiveIncidents {
    fn is_satisfied_by(&self, i: &Incident) -> bool { ... }
}
```

A specification that filters incidents whose status is not
`Resolved`. Composed with date-range or assignee filters in queries.

## Specification: EventsInMonth

```rust
pub struct EventsInMonth {
    pub year: i32,
    pub month: u32,
}

impl Specification<CalendarEvent> for EventsInMonth {
    fn is_satisfied_by(&self, e: &CalendarEvent) -> bool { ... }
}
```

A specification that matches calendar events whose date range overlaps
the given month.

## Policy: IncidentPointLimit

```rust
pub struct IncidentPointLimit {
    pub max_points_per_incident: IncidentPoint,
}

impl Policy<AssignIncidentCommand> for IncidentPointLimit {
    type Outcome = Allow | Deny { reason: &'static str };
    fn check(&self, ctx: &Context, cmd: &AssignIncidentCommand) -> Outcome { ... }
}
```

A school policy that caps the points attributed to a single
assignee. Consumers configure the cap.

## Cross-Domain Coordinator

A thin coordinator lives in the engine facade and orchestrates
multi-domain flows (e.g. incident notification = events + communication).
It is **not** a service; it composes command calls:

```rust
pub struct EventsCoordinator<'a> {
    engine: &'a Engine,
}

impl<'a> EventsCoordinator<'a> {
    pub async fn report_and_notify(&self, cmd: CreateIncidentCommand) -> Result<Incident, DomainError> {
        let incident = self.engine.events().create_incident(cmd).await?;
        // Subscribers (communication domain) handle the notification
        // fan-out in response to the IncidentReported event.
        Ok(incident)
    }
}
```

Domain services are pure. Cross-domain coordination happens through
events and command composition, never through service-to-service calls.
