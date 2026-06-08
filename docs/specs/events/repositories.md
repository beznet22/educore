# Events Domain — Repositories

Repositories are ports (Rust traits). Adapters implement them. The
default adapter targets PostgreSQL; an SQLite adapter is provided for
embedded deployments.

## CalendarEventRepository

```rust
#[async_trait]
pub trait CalendarEventRepository: Send + Sync {
    async fn get(&self, id: CalendarEventId) -> Result<Option<CalendarEvent>>;
    async fn list(&self, school: SchoolId, q: CalendarEventQuery) -> Result<Vec<CalendarEvent>>;
    async fn insert(&self, e: &CalendarEvent) -> Result<()>;
    async fn update(&self, e: &CalendarEvent) -> Result<()>;
    async fn delete(&self, id: CalendarEventId) -> Result<()>;
    async fn between(&self, school: SchoolId, from: NaiveDate, to: NaiveDate) -> Result<Vec<CalendarEvent>>;
    async fn for_audience(&self, school: SchoolId, for_whom: ForWhom, role_id: RoleId) -> Result<Vec<CalendarEvent>>;
    async fn count(&self, school: SchoolId, q: CalendarEventQuery) -> Result<u64>;
    async fn page(&self, school: SchoolId, q: CalendarEventQuery, offset: u32, limit: u32) -> Result<Page<CalendarEvent>>;
}
```

## HolidayRepository

```rust
#[async_trait]
pub trait HolidayRepository: Send + Sync {
    async fn get(&self, id: HolidayId) -> Result<Option<Holiday>>;
    async fn list(&self, school: SchoolId, q: HolidayQuery) -> Result<Vec<Holiday>>;
    async fn insert(&self, h: &Holiday) -> Result<()>;
    async fn update(&self, h: &Holiday) -> Result<()>;
    async fn delete(&self, id: HolidayId) -> Result<()>;
    async fn between(&self, school: SchoolId, from: NaiveDate, to: NaiveDate) -> Result<Vec<Holiday>>;
    async fn in_year(&self, school: SchoolId, year: AcademicYearId) -> Result<Vec<Holiday>>;
}
```

## WeekendRepository

```rust
#[async_trait]
pub trait WeekendRepository: Send + Sync {
    async fn get(&self, id: WeekendId) -> Result<Option<Weekend>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<Weekend>>;
    async fn find_by_name(&self, school: SchoolId, name: &WeekendName) -> Result<Option<Weekend>>;
    async fn insert(&self, w: &Weekend) -> Result<()>;
    async fn update(&self, w: &Weekend) -> Result<()>;
    async fn delete(&self, id: WeekendId) -> Result<()>;
}
```

## IncidentRepository

```rust
#[async_trait]
pub trait IncidentRepository: Send + Sync {
    async fn get(&self, id: IncidentId) -> Result<Option<Incident>>;
    async fn list(&self, school: SchoolId, q: IncidentQuery) -> Result<Vec<Incident>>;
    async fn insert(&self, i: &Incident) -> Result<()>;
    async fn update(&self, i: &Incident) -> Result<()>;
    async fn resolve(&self, id: IncidentId) -> Result<()>;
    async fn delete(&self, id: IncidentId) -> Result<()>;
    async fn open(&self, school: SchoolId) -> Result<Vec<Incident>>;
    async fn in_progress(&self, school: SchoolId) -> Result<Vec<Incident>>;
    async fn by_student(&self, school: SchoolId, student: StudentId) -> Result<Vec<Incident>>;
    async fn by_user(&self, school: SchoolId, user: UserId) -> Result<Vec<Incident>>;
    async fn between(&self, school: SchoolId, from: NaiveDate, to: NaiveDate) -> Result<Vec<Incident>>;
}
```

## AssignIncidentRepository

```rust
#[async_trait]
pub trait AssignIncidentRepository: Send + Sync {
    async fn get(&self, id: AssignIncidentId) -> Result<Option<AssignIncident>>;
    async fn list_for_incident(&self, incident: IncidentId) -> Result<Vec<AssignIncident>>;
    async fn list_for_student(&self, school: SchoolId, student: StudentId) -> Result<Vec<AssignIncident>>;
    async fn list_for_user(&self, school: SchoolId, user: UserId) -> Result<Vec<AssignIncident>>;
    async fn insert(&self, a: &AssignIncident) -> Result<()>;
    async fn update(&self, a: &AssignIncident) -> Result<()>;
    async fn delete(&self, id: AssignIncidentId) -> Result<()>;
}
```

## IncidentCommentRepository

```rust
#[async_trait]
pub trait IncidentCommentRepository: Send + Sync {
    async fn get(&self, id: IncidentCommentId) -> Result<Option<IncidentComment>>;
    async fn list_for_incident(&self, incident: IncidentId) -> Result<Vec<IncidentComment>>;
    async fn insert(&self, c: &IncidentComment) -> Result<()>;
    async fn delete(&self, id: IncidentCommentId) -> Result<()>;
}
```

## CalendarSettingRepository

```rust
#[async_trait]
pub trait CalendarSettingRepository: Send + Sync {
    async fn get(&self, id: CalendarSettingId) -> Result<Option<CalendarSetting>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<CalendarSetting>>;
    async fn find_by_name(&self, school: SchoolId, name: &CalendarMenuName) -> Result<Option<CalendarSetting>>;
    async fn list_enabled(&self, school: SchoolId) -> Result<Vec<CalendarSetting>>;
    async fn insert(&self, s: &CalendarSetting) -> Result<()>;
    async fn update(&self, s: &CalendarSetting) -> Result<()>;
    async fn delete(&self, id: CalendarSettingId) -> Result<()>;
}
```

## Indexes (recommended)

The default PostgreSQL adapter documents the following indexes; consumers
should declare them in their migrations:

```sql
CREATE INDEX ix_events_school_id_from_to ON sm_events (school_id, from_date, to_date);
CREATE INDEX ix_events_school_id_audience ON sm_events (school_id, for_whom);
CREATE INDEX ix_events_school_id_academic ON sm_events (school_id, academic_id);
CREATE INDEX ix_holidays_school_id_from_to ON sm_holidays (school_id, from_date, to_date);
CREATE INDEX ix_holidays_school_id_academic ON sm_holidays (school_id, academic_id);
CREATE INDEX ix_weekends_school_id_name ON sm_weekends (school_id, name);
CREATE INDEX ix_incidents_school_id_status ON incidents (school_id);
CREATE INDEX ix_assign_incidents_school_id_incident ON assign_incidents (school_id, incident_id);
CREATE INDEX ix_assign_incidents_school_id_student ON assign_incidents (school_id, student_id);
CREATE INDEX ix_assign_incident_comments_incident ON assign_incident_comments (incident_id);
CREATE INDEX ix_calendar_settings_school_id_name ON sm_calendar_settings (school_id, menu_name);
```

The `school_id` predicate is mandatory for tenant isolation.
