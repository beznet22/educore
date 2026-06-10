# Settings Domain — Value Objects

Value objects are immutable, validated at construction, and have no
identity. They are compared by value.

## Identifiers

All identifiers in the settings domain are typed and tenant-scoped.

| Identifier                   | Backing Type                  | Notes                              |
| ---------------------------- | ----------------------------- | ---------------------------------- |
| `GeneralSettingsId`          | `Id<GeneralSettings>`         | The settings row id                |
| `LanguageId`                 | `Id<Language>`                | A language entry                   |
| `LanguagePhraseId`           | `Id<LanguagePhrase>`          | A phrase key                       |
| `BaseSetupId`                | `Id<BaseSetup>`               | A lookup value                     |
| `BaseGroupId`                | `Id<BaseGroup>`               | A lookup grouping                  |
| `DateFormatId`               | `Id<DateFormat>`              | A date format pattern              |
| `StyleId`                    | `Id<Style>`                   | A style profile                    |
| `BackgroundSettingId`        | `Id<BackgroundSetting>`       | A background preset                |
| `DashboardSettingId`         | `Id<DashboardSetting>`        | A dashboard card binding           |
| `CustomLinkId`               | `Id<CustomLink>`              | The custom link bundle             |
| `ColorThemeId`               | `Id<ColorTheme>` (global)     | A color binding in a theme         |
| `ThemeId`                    | `Id<Theme>`                   | A theme                            |
| `ColorId`                    | `Id<Color>` (global)          | A color entry                      |
| `BehaviorRecordSettingId`    | `Id<BehaviorRecordSetting>`   | The behavior record config         |
| `SetupAdminId`               | `Id<SetupAdmin>`              | A purpose/complaint/source         |

## General Settings

| Type                 | Constraints                                                       |
| -------------------- | ----------------------------------------------------------------- |
| `SiteTitle`          | 1..191 chars                                                      |
| `SchoolName`         | 1..191 chars                                                      |
| `SchoolCode`         | 1..191 chars (matches platform's SchoolCode)                      |
| `Address`            | 1..191 chars                                                      |
| `PhoneNumber`        | E.164 preferred                                                   |
| `EmailAddress`       | RFC 5322                                                          |
| `FileSize`           | `u64` bytes, 1024..1073741824 (1KB..1GB)                          |
| `CurrencyCode`       | ISO 4217, 3 chars                                                 |
| `CurrencySymbol`     | 1..191 chars                                                      |
| `CurrencyFormat`     | `SymbolAmount`, `AmountSymbol`, or consumer-defined               |
| `PromotionSetting`   | `i32` (consumer-defined promotion policy)                         |
| `LogoFile`           | `FileReference?`                                                  |
| `FaviconFile`        | `FileReference?`                                                  |
| `SystemVersion`      | Semantic version string                                           |
| `CopyrightText`      | 1..1000 chars                                                     |
| `ApiUrl`             | `i32` (consumer-defined)                                          |
| `WebsiteUrl`         | URL                                                               |
| `PhoneNumberPrivacy` | `i32` (1 = masked, 2 = visible)                                   |
| `SessionYear`        | `String` (display string)                                        |
| `RtlLtl`             | `RtlLtl` (1 = RTL, 2 = LTR)                                       |
| `SystemPurchaseCode` | Opaque string                                                     |
| `SystemActivatedDate`| `NaiveDate`                                                       |
| `LastUpdate`         | `NaiveDate`                                                       |
| `EnvatoUser`         | 1..191 chars                                                      |
| `EnvatoItemId`       | 1..191 chars                                                      |
| `SystemDomain`       | 1..191 chars                                                      |
| `WeekStartId`        | `i32` (0 = Sunday, 1 = Monday, etc.)                              |
| `TimeZoneId`         | From platform                                                     |
| `AttendanceLayout`   | `i32` (1 = compact, 2 = expanded)                                 |
| `SessionId`          | `AcademicYearId`                                                  |
| `LanguageId`         | From platform                                                     |
| `DateFormatId`       | From settings                                                     |
| `SsPageLoad`         | `i32` (page-load count to seed)                                   |
| `SubTopicEnable`     | `bool`                                                            |
| `SoftwareVersion`    | 1..100 chars                                                      |
| `EmailDriver`        | String identifier for the email port adapter                      |
| `FcmKey`             | Opaque string                                                     |
| `MultipleRoll`       | `bool`                                                            |
| `ResultType`         | 1..191 chars (consumer-defined)                                   |
| `DirectFeesAssign`   | `bool`                                                            |
| `WithGuardian`       | `bool`                                                            |
| `PreloaderStatus`    | `bool`                                                            |
| `PreloaderStyle`     | `i32` (style id)                                                  |
| `PreloaderType`      | `i32` (1 = spinner, 2 = image)                                    |
| `PreloaderImage`     | `FileReference?`                                                  |
| `DueFeesLogin`       | `bool`                                                            |
| `TwoFactor`          | `bool`                                                            |
| `ActiveTheme`        | 1..191 chars (theme name)                                         |
| `QueueConnection`    | String identifier for the queue port adapter                      |
| `IsCustomSaas`       | `i32`                                                             |
| `IsComment`          | `bool`                                                            |
| `AutoApprove`        | `bool`                                                            |
| `BlogSearch`         | `bool`                                                            |
| `RecentBlog`         | `bool`                                                            |
| `AcademicId`         | `AcademicYearId?`                                                 |
| `UnAcademicId`       | `AcademicYearId` (default 1)                                      |
| `BehaviorRecords`    | `bool`                                                            |

### Module Toggles

A large set of boolean feature flags carried on `GeneralSettings`:

| Type                | Notes                                                       |
| ------------------- | ------------------------------------------------------------ |
| `LessonEnabled`     | `bool`                                                       |
| `ChatEnabled`       | `bool`                                                       |
| `FeesCollectionEnabled` | `bool`                                                  |
| `IncomeHeadId`      | `i32` (default 0)                                            |
| `BiometricsEnabled` | `bool`                                                       |
| `ResultReportsEnabled` | `bool`                                                   |
| `TemplateSettingsEnabled` | `bool`                                                |
| `MenuManageEnabled` | `bool`                                                       |
| `RolePermissionEnabled` | `bool`                                                   |
| `RazorPayEnabled`   | `bool`                                                       |
| `SaasEnabled`       | `bool`                                                       |
| `StudentAbsentNotificationEnabled` | `bool`                                  |
| `ParentRegistrationEnabled` | `bool`                                               |
| `ZoomEnabled`       | `bool`                                                       |
| `BbbEnabled`        | `bool`                                                       |
| `VideoWatchEnabled` | `bool`                                                       |
| `JitsiEnabled`      | `bool`                                                       |
| `OnlineExamEnabled` | `bool`                                                       |
| `SaasRolePermissionEnabled` | `bool`                                              |
| `BulkPrintEnabled`  | `bool`                                                       |
| `HimalayaSmsEnabled`| `bool`                                                       |
| `XenditPaymentEnabled` | `bool`                                                    |
| `WalletEnabled`     | `bool`                                                       |
| `LmsEnabled`        | `bool`                                                       |
| `ExamPlanEnabled`   | `bool`                                                       |
| `UniversityEnabled` | `bool`                                                       |
| `GmeetEnabled`      | `bool`                                                       |
| `KhaltiPaymentEnabled` | `bool`                                                   |
| `RaudhahpayEnabled` | `bool`                                                       |
| `AppSliderEnabled`  | `bool`                                                       |
| `DownloadCenterEnabled` | `bool`                                                  |
| `AiContentEnabled`  | `bool`                                                       |
| `WhatsappSupportEnabled` | `bool`                                                  |
| `InAppLiveClassEnabled` | `bool`                                                   |
| `FeesStatus`        | `i32` (1 = enabled)                                          |
| `LmsCheckout`       | `bool`                                                       |

## Language

| Type                | Constraints                                                       |
| ------------------- | ----------------------------------------------------------------- |
| `LanguageCode`      | ISO 639-1, 2..191 chars                                           |
| `LanguageName`      | 1..191 chars                                                      |
| `LanguageNative`    | 1..191 chars                                                      |
| `LanguageUniversal` | 1..191 chars                                                      |
| `RtlFlag`           | `bool`                                                            |
| `LanguageStatus`    | `Active`, `Inactive`                                              |

## Language Phrase

| Type             | Constraints                                                       |
| ---------------- | ----------------------------------------------------------------- |
| `PhraseModule`   | 1..65000 chars (the module the phrase belongs to)                 |
| `DefaultPhrase`  | 1..65000 chars (the source-of-truth translation)                  |
| `Translation`    | 1..65000 chars per locale                                         |
| `LocaleCode`     | ISO 639-1                                                         |

## Base Setup

| Type             | Constraints                                                       |
| ---------------- | ----------------------------------------------------------------- |
| `BaseGroupName`  | 1..200 chars                                                      |
| `BaseSetupName`  | 1..255 chars                                                      |
| `BaseGroupOrder` | `i32` (display order)                                             |

## Date Format

| Type              | Constraints                                                       |
| ----------------- | ----------------------------------------------------------------- |
| `DateFormatPattern` | A valid `strftime` pattern (e.g. `%Y-%m-%d`)                    |
| `DateFormatPreview` | A human-readable example (e.g. `YYYY-MM-DD`)                   |
| `DateFormatActive`  | `bool`                                                         |

## Style

| Type                | Constraints                                                       |
| ------------------- | ----------------------------------------------------------------- |
| `StyleName`         | 1..255 chars                                                      |
| `StylePath`         | 1..255 chars (CSS file path)                                      |
| `ColorHex`          | `#RRGGBB` (7 chars) or `#RRGGBBAA` (9 chars)                      |
| `FontFamily`        | 1..255 chars (CSS font-family)                                    |
| `StyleActive`       | `bool`                                                            |
| `StyleDefault`      | `bool`                                                            |

## Background

| Type                | Constraints                                                       |
| ------------------- | ----------------------------------------------------------------- |
| `BackgroundTitle`   | 1..255 chars                                                      |
| `BackgroundType`    | `Image`, `Color`                                                  |
| `BackgroundImage`   | `FileReference?`                                                  |
| `BackgroundColor`   | `ColorHex` (when `type=Color`)                                    |
| `BackgroundDefault` | `bool`                                                            |

## Dashboard

| Type                | Constraints                                                       |
| ------------------- | ----------------------------------------------------------------- |
| `DashboardSectionId`| `i32`                                                             |
| `DashboardActive`   | `bool`                                                            |

## Custom Link

| Type                | Constraints                                                       |
| ------------------- | ----------------------------------------------------------------- |
| `LinkLabel`         | 1..255 chars                                                      |
| `LinkHref`          | URL or empty                                                      |
| `SocialUrl`         | URL or empty                                                      |

## Theme

| Type                | Constraints                                                       |
| ------------------- | ----------------------------------------------------------------- |
| `ThemeTitle`        | 1..191 chars                                                      |
| `ThemePath`         | 1..255 chars (CSS file path)                                      |
| `ColorMode`         | `Gradient`, `Solid`                                               |
| `BoxShadow`         | `bool`                                                            |
| `BackgroundType`    | `Image`, `Color`                                                  |
| `ThemeDefault`      | `bool`                                                            |
| `ThemeSystem`       | `bool`                                                            |

## Color

| Type           | Constraints                                                       |
| -------------- | ----------------------------------------------------------------- |
| `ColorName`    | 1..191 chars                                                      |
| `ColorValue`   | `ColorHex`                                                        |
| `IsColor`      | `bool`                                                            |
| `ColorStatus`  | `bool`                                                            |
| `LawnGreen`    | `ColorHex` (preview default)                                      |

## Behavior Record

| Type                 | Constraints                                                       |
| -------------------- | ----------------------------------------------------------------- |
| `BehaviorFlag`       | `i32` in `0..=2` (`Off`, `On`, `Inherit`)                         |

## Setup Admin

| Type              | Constraints                                                       |
| ----------------- | ----------------------------------------------------------------- |
| `SetupAdminType`  | `Purpose` (1), `ComplaintType` (2), `Source` (3), `Reference` (4) |
| `SetupAdminName`  | 1..191 chars                                                      |
| `SetupAdminDescription` | 1..65000 chars                                              |

## School Identity Bindings

| Type            | Notes                                                       |
| --------------- | ------------------------------------------------------------ |
| `SchoolId`      | From `educore-platform`                                     |
| `TenantContext` | `(SchoolId, UserId, ...)` from `educore-platform`           |
| `RoleId`        | From `educore-rbac`                                         |
| `LanguageCode`  | From `educore-platform`                                     |
| `CurrencyCode`  | From `educore-platform`                                     |
| `TimeZoneId`    | From `educore-platform`                                     |
| `AcademicYearId` | From `educore-academic`                                     |

## Validation Rules

All value objects implement `Validate` and refuse construction when
validation fails:

```rust
pub trait Validate {
    fn validate(&self) -> Result<(), ValueError>;
}
```

Construction is the only entry point:

```rust
let pattern = DateFormatPattern::new("%Y-%m-%d")?;
let hex = ColorHex::new("#FF00AA")?;
let url = LinkHref::new("https://example.com/contact")?;
```

Parsing returns `Result<T, ValueError>`. There are no setters that
bypass validation.

## Type-Safe Wrappers

```rust
pub struct ColorHex(String);

impl ColorHex {
    pub fn new(s: &str) -> Result<Self, ValueError> {
        if !is_valid_hex(s) {
            return Err(ValueError::InvalidHex);
        }
        Ok(Self(s.to_string()))
    }
    pub fn as_str(&self) -> &str { &self.0 }
}
```

`ColorHex` is the type carried in every color field. It cannot be
constructed from a non-hex string.
