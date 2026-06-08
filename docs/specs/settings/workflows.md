# Settings Domain — Workflows

Workflows orchestrate commands, queries, and policies to fulfill a
business goal. They are documented as ordered, conditional steps.

## Initial School Setup Workflow

```text
1. Platform emits SchoolCreated.
2. Settings subscribes and seeds:
   - GeneralSettings (with the school's name, default language,
     default currency, default date format).
   - BehaviorRecordSetting (all flags off).
   - DashboardSetting (one row per dashboard section, bound to
     SuperAdmin).
   - Default Theme (the engine's bundled `default` theme).
3. SchoolAdmin is returned to the onboarding wizard.
4. SchoolAdmin updates the school's branding (logo, favicon,
   copyright) via UpdateGeneralSettings.
5. SchoolAdmin picks a theme (SelectActiveTheme).
6. SchoolAdmin picks a date format (SelectDateFormat).
7. SchoolAdmin picks a time zone (SelectTimeZone).
8. SchoolAdmin picks the active academic session (SelectSession).
```

## Language Management Workflow

```text
1. SchoolAdmin adds a new language (AddLanguage).
2. SchoolAdmin activates the language (ActivateLanguage).
3. SchoolAdmin selects the language (SelectLanguage) as the
   school's default.
4. Translators begin adding translations
   (TranslateLanguagePhrase) for existing phrase keys.
5. To remove a language, SchoolAdmin issues DeleteLanguage; the
   translations are cascade-deleted.
```

**Edge cases:**

- A language that is the active default cannot be deleted; the
  school must first select a different default.
- A language with active translations cannot be deleted; the
  engine rejects the command with `ConflictError::LanguageInUse`.

## Theme Configuration Workflow

```text
1. SchoolAdmin creates a new theme (CreateTheme) or replicates
   an existing one (ReplicateTheme).
2. SchoolAdmin binds colors to the theme (CreateColorTheme).
3. SchoolAdmin sets the theme as active (SelectActiveTheme).
4. The consumer re-renders with the new theme.
5. To remove a theme, SchoolAdmin issues DeleteTheme; cascading
   color bindings are deleted.
```

**Edge cases:**

- A default theme cannot be deleted; the engine rejects with
  `ConflictError::ThemeIsDefault`.
- A system theme cannot be deleted.

## Base Setup Management Workflow

```text
1. SchoolAdmin creates a base group (AddBaseGroup) e.g. "Gender".
2. SchoolAdmin adds values (AddBaseSetup) e.g. "Male", "Female".
3. The values appear in dropdowns throughout the engine.
4. To remove a value, SchoolAdmin issues DeleteBaseSetup; the
   engine rejects if any aggregate references it.
```

## Custom Link Configuration Workflow

```text
1. SchoolAdmin issues UpdateCustomLinks with up to 16 link
   pairs and 5 social URLs.
2. The footer/sidebar re-renders.
3. To clear, SchoolAdmin issues ResetCustomLinks.
```

## Date Format Configuration Workflow

```text
1. SchoolAdmin adds a date format (AddDateFormat) with a
   strftime pattern and a human-readable preview.
2. SchoolAdmin selects the format (SelectDateFormat).
3. The engine renders all dates in the new format.
```

## Dashboard Configuration Workflow

```text
1. SchoolAdmin issues CreateDashboardSetting per dashboard
   card, bound to a role.
2. The dashboard re-renders.
3. To remove a card from a role, SchoolAdmin issues
   DeleteDashboardSetting.
```

## Two-Factor Configuration Workflow

```text
1. SchoolAdmin issues EnableTwoFactor on GeneralSettings.
2. The settings domain emits TwoFactorToggled.
3. The RBAC domain subscribes and refreshes its
   TwoFactorSetting view.
4. The platform domain's authentication flow begins to require
   the second factor for the affected roles.
5. To disable, SchoolAdmin issues DisableTwoFactor.
```

## Idempotency

- `UpdateGeneralSettings` is idempotent on `school_id`. A
  duplicate applies the patch on top of the current state.
- `AddLanguage` is idempotent on `(school_id, code)`. A duplicate
  returns the existing language.
- `AddBaseSetup` is idempotent on
  `(base_group_id, base_setup_name)`.
- `SelectActiveTheme` is idempotent on `(school_id, theme_id)`.
- `EnableTwoFactor` is idempotent on `school_id`.
