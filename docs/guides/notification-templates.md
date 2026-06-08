# Notification Templates Guide

## Goal

Define reusable notification templates that the engine can render
and dispatch via the notification port.

## Template

```rust
pub struct NotificationTemplate {
    pub template_id: NotificationTemplateId,
    pub tenant: TenantContext,
    pub name: String,
    pub channel: Channel,
    pub subject: Option<String>,         // email subject or push title
    pub body: String,                    // template body with placeholders
    pub variables: Vec<TemplateVariable>,
    pub locale: LanguageCode,
    pub version: u32,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub is_active: bool,
}
```

## Placeholders

Placeholders use Handlebars-style `{{name}}` syntax:

```text
Hello {{guardian.first_name}},

Your child {{student.full_name}} was absent on {{date}}.

Please contact the school if you have any questions.

Regards,
{{school.name}}
```

The engine compiles templates at template-creation time. Compilation
failures are reported before the template is saved.

## Variables

Variables are typed:

```rust
pub struct TemplateVariable {
    pub name: String,
    pub kind: TemplateVariableKind,
    pub required: bool,
    pub default: Option<TemplateValue>,
    pub description: String,
}

pub enum TemplateVariableKind {
    String,
    Integer,
    Decimal,
    Date,
    DateTime,
    Boolean,
    Enum(Vec<String>),
    PersonName,
    EmailAddress,
    PhoneNumber,
    Money,
}
```

When dispatching a notification, the engine validates that all
required variables are present and of the correct type.

## Example: Absence Notification

```rust
let template = engine.communication().create_template(CreateTemplateCommand {
    tenant,
    name: "Absence Notification".into(),
    channel: Channel::Email { from: None, reply_to: None },
    subject: Some("Absence: {{student.full_name}} on {{date}}".into()),
    body: r#"
        Hello {{guardian.first_name}},

        Your child {{student.full_name}} was absent on {{date}}.

        Class: {{student.class_name}}
        Section: {{student.section_name}}

        Please contact the school if you have any questions.

        Regards,
        {{school.name}}
    "#.into(),
    variables: vec![
        TemplateVariable { name: "guardian.first_name".into(), kind: PersonName, required: true, default: None, description: "Guardian first name".into() },
        TemplateVariable { name: "student.full_name".into(), kind: PersonName, required: true, default: None, description: "Student full name".into() },
        TemplateVariable { name: "date".into(), kind: Date, required: true, default: None, description: "Absence date".into() },
        TemplateVariable { name: "student.class_name".into(), kind: String, required: true, default: None, description: "Class name".into() },
        TemplateVariable { name: "student.section_name".into(), kind: String, required: true, default: None, description: "Section name".into() },
        TemplateVariable { name: "school.name".into(), kind: String, required: true, default: None, description: "School name".into() },
    ],
    locale: LanguageCode::En,
}).await?;
```

## Engine-Provided Variables

The engine automatically provides some variables to every template:

- `school.name`, `school.address`, `school.phone`, `school.email`,
  `school.logo_url`
- `tenant.school_id`, `tenant.user_id`
- `actor.full_name` (the user who triggered the notification)
- `correlation_id`, `causation_id`

Consumers may provide additional context-specific variables.

## Multi-Locale Templates

A template can be translated into multiple locales. The engine
selects the locale based on the recipient's preference.

```rust
let en_template = engine.communication().create_template(...).await?;
let es_template = engine.communication().create_template(
    CreateTemplateCommand { locale: LanguageCode::Es, ... }
).await?;

// Engine groups them by name
```

When dispatching, the engine looks up the recipient's preferred
locale and uses the matching template. If no match, it falls back to
the default locale.

## Versioning

Templates carry a `version` field. When a template is updated, a
new version is created. The engine keeps prior versions for
historical reference (e.g. "what was the welcome email in 2024?").

## Conditional Content

Templates may use Handlebars `{{#if}}` blocks:

```text
{{#if student.is_scholarship}}
You are receiving a 50% scholarship.
{{/if}}
```

The engine supports a subset of Handlebars:

- `{{variable}}` — variable substitution.
- `{{#if condition}}...{{/if}}` — conditional.
- `{{#each list}}...{{/each}}` — iteration.
- `{{helper arg1 arg2}}` — helpers (e.g. `{{format_date date "long"}}`).

Custom helpers are registered per school.

## HTML Emails

For email templates, the body may be HTML. The engine sanitizes the
output to prevent XSS:

```rust
body: "<p>Hello <strong>{{student.full_name}}</strong>,</p>...",
```

Auto-escaping is on by default. Use `{{{variable}}}` to bypass
escaping for trusted content.

## Attachment Templates

A template may attach files. The attachment is rendered with
variables:

```text
{{#each attachments}}
    {{this.filename}} ({{this.size}})
{{/each}}
```

## Testing

- A test of template compilation.
- A test of variable validation.
- A test of variable substitution.
- A test of conditional rendering.
- A test of multi-locale selection.
- A test of XSS sanitization.
- A test of attachment rendering.
- A snapshot test of the rendered output.
