# Settings Domain — Permissions

Permissions are capability strings. They are not roles. The RBAC
domain maps capabilities to roles.

## Naming

```text
<Domain>.<Aggregate>.<Action>
```

For the settings domain, the convention is `Settings.*`.

## Capabilities

### GeneralSettings

- `Settings.General.Read`
- `Settings.General.Update`
- `Settings.General.TwoFactor.Toggle`

### Theme

- `Settings.Theme.Create`
- `Settings.Theme.Read`
- `Settings.Theme.Update`
- `Settings.Theme.Activate`
- `Settings.Theme.Delete`
- `Settings.Theme.Replicate`
- `Settings.Theme.Select` — set the active theme for the school.

### Color

- `Settings.Color.Create` (system)
- `Settings.Color.Read`
- `Settings.Color.Update` (system)
- `Settings.Color.Delete` (system)

### ColorTheme

- `Settings.ColorTheme.Create`
- `Settings.ColorTheme.Read`
- `Settings.ColorTheme.Update`
- `Settings.ColorTheme.Delete`

### Language

- `Settings.Language.Add`
- `Settings.Language.Read`
- `Settings.Language.Update`
- `Settings.Language.Delete`
- `Settings.Language.Activate`
- `Settings.Language.Select`

### LanguagePhrase

- `Settings.LanguagePhrase.Add`
- `Settings.LanguagePhrase.Read`
- `Settings.LanguagePhrase.Update`
- `Settings.LanguagePhrase.Delete`
- `Settings.LanguagePhrase.Translate`

### BaseSetup

- `Settings.BaseGroup.Add`
- `Settings.BaseGroup.Read`
- `Settings.BaseGroup.Update`
- `Settings.BaseGroup.Delete`
- `Settings.BaseSetup.Add`
- `Settings.BaseSetup.Read`
- `Settings.BaseSetup.Update`
- `Settings.BaseSetup.Delete`

### DateFormat

- `Settings.DateFormat.Add`
- `Settings.DateFormat.Read`
- `Settings.DateFormat.Update`
- `Settings.DateFormat.Delete`
- `Settings.DateFormat.Select`

### TimeZone

- `Settings.TimeZone.Select`

### Session

- `Settings.Session.Select`

### Style

- `Settings.Style.Create`
- `Settings.Style.Read`
- `Settings.Style.Update`
- `Settings.Style.Activate`
- `Settings.Style.Delete`

### Background

- `Settings.Background.Create`
- `Settings.Background.Read`
- `Settings.Background.Update`
- `Settings.Background.Delete`

### Dashboard

- `Settings.Dashboard.Create`
- `Settings.Dashboard.Read`
- `Settings.Dashboard.Update`
- `Settings.Dashboard.Delete`

### CustomLink

- `Settings.CustomLink.Read`
- `Settings.CustomLink.Update`
- `Settings.CustomLink.Reset`

### BehaviorRecord

- `Settings.BehaviorRecord.Read`
- `Settings.BehaviorRecord.Update`

### SetupAdmin

- `Settings.SetupAdmin.Add`
- `Settings.SetupAdmin.Read`
- `Settings.SetupAdmin.Update`
- `Settings.SetupAdmin.Delete`

## Default Role Mapping

| Role             | Capabilities (highlights)                                          |
| ---------------- | ------------------------------------------------------------------ |
| SuperAdmin       | All                                                                |
| SchoolAdmin      | All within the school                                             |
| Teacher          | `Settings.General.Read`, `Settings.Theme.Read`                    |
| Student          | `Settings.General.Read`                                           |
| Parent           | `Settings.General.Read`                                           |
| Receptionist     | `Settings.General.Read`                                           |

The default mapping is configurable per school.

## Authorization Pattern

Capabilities are checked at the command boundary:

```rust
if !engine.rbac().has(actor_id, Capability::SettingsGeneralUpdate).await? {
    return Err(DomainError::forbidden("missing capability"));
}
```

## Read vs Write

Read capabilities are explicit. The engine does not assume
"Settings.General.Read" implies "Settings.General.Update".

## Tenant Isolation

Every capability check is paired with a tenant check. The actor
must be authenticated to the school that owns the settings row.
