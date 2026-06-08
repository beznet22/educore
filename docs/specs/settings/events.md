# Settings Domain — Events

Domain events describe facts that have already happened. They are
immutable, append-only records used for cross-domain integration,
audit, and event sourcing.

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

## General Settings Lifecycle

### GeneralSettingsUpdated

```rust
pub struct GeneralSettingsUpdated {
    pub school_id: SchoolId,
    pub changed_fields: Vec<&'static str>,
}
```

**Subscribers:**
- `rbac` — refreshes the school's branding in audit emails.
- `audit` — records the change.

### ActiveThemeChanged

```rust
pub struct ActiveThemeChanged {
    pub school_id: SchoolId,
    pub from_theme: Option<ActiveTheme>,
    pub to_theme: ActiveTheme,
}
```

### LanguageChanged

```rust
pub struct LanguageChanged {
    pub school_id: SchoolId,
    pub from_language_id: Option<LanguageId>,
    pub to_language_id: LanguageId,
}
```

### DateFormatChanged

```rust
pub struct DateFormatChanged {
    pub school_id: SchoolId,
    pub from_date_format_id: Option<DateFormatId>,
    pub to_date_format_id: DateFormatId,
}
```

### TimeZoneChanged

```rust
pub struct TimeZoneChanged {
    pub school_id: SchoolId,
    pub from_time_zone_id: Option<TimeZoneId>,
    pub to_time_zone_id: TimeZoneId,
}
```

### SessionChanged

```rust
pub struct SessionChanged {
    pub school_id: SchoolId,
    pub from_session_id: Option<AcademicYearId>,
    pub to_session_id: AcademicYearId,
}
```

### TwoFactorToggled

```rust
pub struct TwoFactorToggled {
    pub school_id: SchoolId,
    pub enabled: bool,
}
```

**Subscribers:**
- `rbac` — refreshes its `TwoFactorSetting` view.
- `platform` — refreshes its authentication flow.

## Language Lifecycle

### LanguageAdded

```rust
pub struct LanguageAdded {
    pub language_id: LanguageId,
    pub code: LanguageCode,
    pub name: LanguageName,
    pub rtl: bool,
}
```

### LanguageUpdated / LanguageDeleted

```rust
pub struct LanguageUpdated {
    pub language_id: LanguageId,
    pub changed_fields: Vec<&'static str>,
}

pub struct LanguageDeleted {
    pub language_id: LanguageId,
    pub prior_code: LanguageCode,
}
```

### LanguageActivated / LanguageDeactivated

```rust
pub struct LanguageActivated {
    pub language_id: LanguageId,
}

pub struct LanguageDeactivated {
    pub language_id: LanguageId,
}
```

## Language Phrase Lifecycle

### LanguagePhraseAdded

```rust
pub struct LanguagePhraseAdded {
    pub phrase_id: LanguagePhraseId,
    pub modules: PhraseModule,
    pub default_phrases: DefaultPhrase,
}
```

### LanguagePhraseUpdated / LanguagePhraseDeleted

```rust
pub struct LanguagePhraseUpdated {
    pub phrase_id: LanguagePhraseId,
    pub changed_fields: Vec<&'static str>,
}

pub struct LanguagePhraseDeleted {
    pub phrase_id: LanguagePhraseId,
}
```

### LanguagePhraseTranslated

```rust
pub struct LanguagePhraseTranslated {
    pub phrase_id: LanguagePhraseId,
    pub locale: LocaleCode,
    pub translation: Translation,
}
```

## Base Setup Lifecycle

- `BaseGroupAdded { id, name }`
- `BaseGroupUpdated { id, changed_fields }`
- `BaseGroupDeleted { id }`
- `BaseSetupAdded { id, name, base_group_id }`
- `BaseSetupUpdated { id, changed_fields }`
- `BaseSetupDeleted { id }`

## Date Format Lifecycle

- `DateFormatAdded { id, format, normal_view }`
- `DateFormatUpdated { id, changed_fields }`
- `DateFormatDeleted { id }`

## Style Lifecycle

- `StyleCreated { id, style_name }`
- `StyleUpdated { id, changed_fields }`
- `StyleActivated { id, style_name, previous_id? }`
- `StyleDeleted { id, prior_style_name }`

## Background Lifecycle

- `BackgroundSettingCreated { id, title, type }`
- `BackgroundSettingUpdated { id, changed_fields }`
- `BackgroundSettingDeleted { id }`

## Dashboard Lifecycle

- `DashboardSettingCreated { id, dashboard_sec_id, role_id }`
- `DashboardSettingUpdated { id, changed_fields }`
- `DashboardSettingDeleted { id }`

## Custom Link Lifecycle

- `CustomLinksUpdated { id, link_count, social_count }`
- `CustomLinksReset { id }`

## Theme Lifecycle

### ThemeCreated

```rust
pub struct ThemeCreated {
    pub theme_id: ThemeId,
    pub title: ThemeTitle,
    pub color_mode: ColorMode,
    pub background_type: BackgroundType,
}
```

### ThemeUpdated / ThemeActivated / ThemeDeleted

```rust
pub struct ThemeUpdated {
    pub theme_id: ThemeId,
    pub changed_fields: Vec<&'static str>,
}

pub struct ThemeActivated {
    pub theme_id: ThemeId,
    pub from_active: Option<ThemeId>,
}

pub struct ThemeDeleted {
    pub theme_id: ThemeId,
    pub prior_title: ThemeTitle,
}
```

### ThemeReplicated

```rust
pub struct ThemeReplicated {
    pub source_theme_id: ThemeId,
    pub new_theme_id: ThemeId,
    pub copied_color_themes: u32,
}
```

## Color Lifecycle

- `ColorCreated { id, name, default_value }`
- `ColorUpdated { id, changed_fields }`
- `ColorDeleted { id, prior_name }`

## Color Theme Lifecycle

- `ColorThemeCreated { id, color_id, theme_id, value }`
- `ColorThemeUpdated { id, changed_fields }`
- `ColorThemeDeleted { id }`

## Behavior Record Lifecycle

### BehaviorRecordSettingUpdated

```rust
pub struct BehaviorRecordSettingUpdated {
    pub school_id: SchoolId,
    pub changed_fields: Vec<&'static str>,
}
```

## Setup Admin Lifecycle

- `SetupAdminAdded { id, type, name }`
- `SetupAdminUpdated { id, changed_fields }`
- `SetupAdminDeleted { id, prior_name }`
