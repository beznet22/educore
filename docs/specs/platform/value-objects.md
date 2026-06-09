# Platform Domain — Value Objects

Value objects are immutable, validated at construction, and have no
identity. They are compared by value.

## Identifiers

All identifiers in the platform domain are typed and tenant-scoped.
Schools are globally unique and have no parent tenant.

| Identifier                    | Backing Type             | Notes                                |
| ----------------------------- | ------------------------ | ------------------------------------ |
| `SchoolId`                    | `Id<School>` (global)    | A school or organization             |
| `UserId`                      | `Id<User>`               | An actor                             |
| `OtpCodeId`                   | `Id<OtpCode>`            | An OTP row                           |
| `CourseId`                    | `Id<Course>`             | An online course                     |
| `CourseCategoryId`            | `Id<CourseCategory>`     | A course grouping                    |
| `CoursePageId`                | `Id<CoursePage>`         | A course landing page                |
| `CustomFieldId`               | `Id<CustomField>`        | A field definition                   |
| `CustomFieldValueId`          | `Id<CustomFieldValue>`   | A field value                        |
| `ChartOfAccountId`            | `Id<ChartOfAccount>`     | An accounting head                   |
| `BaseGroupId`                 | `Id<BaseGroup>`          | A base-setup group                   |
| `BaseSetupId`                 | `Id<BaseSetup>`          | A base-setup value                   |
| `ModuleId`                    | `Id<Module>`             | A top-level functional area          |
| `ModuleLinkId`                | `Id<ModuleLink>`         | A menu item                          |
| `AddOnId`                     | `Id<AddOn>` (global)     | A registered add-on                  |
| `ModuleManagerId`             | `Id<ModuleManager>`      | A module manager                     |
| `ModuleStudentParentInfoId`   | `Id<ModuleStudentParentInfo>` | A student/parent menu config    |
| `TimeZoneId`                  | `Id<TimeZone>` (global)  | A timezone entry                     |
| `CountryId`                   | `Id<Country>`            | A country                            |
| `ContinentId`                 | `Id<Continent>`          | A continent                          |
| `CurrencyId`                  | `Id<Currency>`           | A currency                           |
| `LanguageId`                  | `Id<Language>`           | A language                           |
| `SocialMediaIconId`           | `Id<SocialMediaIcon>`    | A social link                        |
| `HeaderMenuManagerId`         | `Id<HeaderMenuManager>`  | A header menu item                   |
| `PhotoGalleryId`              | `Id<PhotoGallery>`       | A photo gallery                      |
| `VideoGalleryId`              | `Id<VideoGallery>`       | A video gallery                      |
| `VisitorId`                   | `Id<Visitor>`            | A visitor log entry                  |
| `ToDoId`                      | `Id<ToDo>`               | A to-do                              |
| `InstructionId`               | `Id<Instruction>`        | An instruction                       |
| `ExpertTeacherId`             | `Id<ExpertTeacher>`      | A featured staff member              |
| `FrontendPermissionId`        | `Id<FrontendPermission>` | A public-facing permission           |
| `AmountTransferId`            | `Id<AmountTransfer>`     | A fund transfer                      |
| `PluginId`                    | `Id<Plugin>`             | A front-office plugin                |
| `CommentId`                   | `Id<Comment>`            | A comment                            |
| `CommentTagId`                | `Id<CommentTag>`         | A comment tag                        |
| `PersonalAccessTokenId`       | `Id<PersonalAccessToken>` (global) | A PAT                          |
| `VideoUploadId`               | `Id<VideoUpload>`        | A class-section video                |
| `ModuleInfoId`                | `Id<ModuleInfo>`         | A module info projection             |

## Names & Identifiers

| Type                | Constraints                                                       |
| ------------------- | ----------------------------------------------------------------- |
| `SchoolName`        | 1..200 chars                                                      |
| `SchoolCode`        | 1..200 chars, unique across the platform                          |
| `Domain`            | 1..191 chars, unique, must be a valid domain label                |
| `EmailAddress`      | RFC 5322, length cap 200                                          |
| `PhoneNumber`       | E.164 preferred; national formats accepted                        |
| `Address`           | 1..500 chars                                                      |
| `PersonName`        | 1..200 chars, unicode letters and basic punctuation               |
| `FullName`          | Computed from `PersonName` parts                                  |
| `Username`          | 1..192 chars, unique within `(school_id, lower(username))`        |
| `RoleId`            | From `smsengine-rbac`                                               |

## School Status

| Type                | Values                                                                 |
| ------------------- | ---------------------------------------------------------------------- |
| `SchoolActiveStatus`| `Approved`, `Pending`                                                 |
| `LoginEnabled`      | `Yes`, `No`                                                           |
| `ContactType`       | `Yearly`, `Monthly`, `Once`                                           |
| `PlanType`          | Free-form string (consumer-defined)                                    |
| `Region`            | `i32` referencing a continent/country                                  |

## User

| Type                 | Constraints                                                        |
| -------------------- | ------------------------------------------------------------------ |
| `UserType`           | `SuperAdmin`, `SchoolAdmin`, `Teacher`, `Student`, `Parent`, `Accountant`, `Librarian`, `Receptionist`, `Staff`, `Driver`, `Customer` |
| `UserActiveStatus`   | `Active`, `Inactive`                                               |
| `IsAdministrator`    | `Yes`, `No`                                                        |
| `IsRegistered`       | `bool`                                                             |
| `AccessStatus`       | `i32` (consumer-defined; 1 = allowed by default)                  |
| `LanguagePreference` | `LanguageCode` (default `en`)                                      |
| `StylePreference`    | `i32` (references a settings style id)                             |
| `RtlPreference`      | `RtlLtl` (1 = RTL, 2 = LTR)                                        |
| `SelectedSessionId`  | `AcademicYearId` (from the academic domain)                        |
| `RandomCode`         | Hashed random string used in the password reset flow               |
| `NotificationToken`  | `String` (FCM/APNs token; opaque)                                  |
| `DeviceToken`        | `String` (mobile push token; opaque)                               |
| `RememberToken`      | Hashed remember-me cookie value                                    |
| `WalletBalance`      | `Decimal` (non-negative)                                           |
| `Verified`           | `bool` (email verified)                                            |
| `TrialEndsAt`        | Optional `Timestamp`                                               |
| `StripeId`           | Optional opaque id from the payment port                           |
| `CardBrand`          | Optional brand label                                               |
| `CardLastFour`       | Optional last-four-digits string                                   |

## OTP

| Type              | Constraints                                                       |
| ----------------- | ----------------------------------------------------------------- |
| `OtpCode`         | 4..10 digits, hashed before storage                               |
| `OtpExpiry`       | `Timestamp`                                                       |
| `OtpChannel`      | `Sms`, `Email`                                                    |
| `OtpDeliveryMode` | `Sms`, `Email`, `Authenticator`                                   |

## Course

| Type            | Constraints                                                       |
| --------------- | ----------------------------------------------------------------- |
| `CourseTitle`   | 1..191 chars, unique within `(school_id, title)`                  |
| `CourseStatus`  | `Active`, `Inactive`                                              |
| `CourseImage`   | `FileReference`                                                   |
| `CourseOverview`| `String` (1..65000 chars)                                         |
| `CourseOutline` | `String` (1..65000 chars)                                         |
| `Prerequisites` | `String` (1..65000 chars)                                         |
| `Resources`     | `String` (1..65000 chars)                                         |
| `Stats`         | `String` (1..65000 chars)                                         |

## Custom Field

| Type                    | Constraints                                                   |
| ----------------------- | ------------------------------------------------------------- |
| `FormName`              | 1..191 chars (e.g. `student_registration`)                    |
| `FieldLabel`            | 1..191 chars                                                  |
| `FieldType`             | `Text`, `Number`, `Select`, `Date`, `File`, `Textarea`, `Radio`, `Checkbox` |
| `LengthRange`           | `String` `"min..max"` parsed as `(u32, u32)`                  |
| `ValueRange`            | `String` `"min..max"` parsed as `(f64, f64)`                  |
| `NameValueList`         | `String` comma-separated                                      |
| `Width`                 | `String` (CSS width)                                          |
| `IsRequired`            | `bool`                                                        |
| `EntityType`            | `Student`, `Staff`, `Admission`, `Course`                     |
| `FieldValue`            | `String` (1..65000 chars)                                     |

## Chart of Account

| Type           | Constraints                                                       |
| -------------- | ----------------------------------------------------------------- |
| `AccountHead`  | 1..200 chars, unique within `(school_id, lower(head))`            |
| `AccountType`  | `Expense` (`E`) or `Income` (`I`)                                 |

## Base Setup

| Type            | Constraints                                                       |
| --------------- | ----------------------------------------------------------------- |
| `BaseGroupName` | 1..200 chars, unique within `(school_id, name)`                   |
| `BaseSetupName` | 1..255 chars                                                      |

## Module

| Type             | Constraints                                                       |
| ---------------- | ----------------------------------------------------------------- |
| `ModuleName`     | 1..191 chars, unique within `(school_id, name)`                   |
| `ModuleOrder`    | `i32`, non-negative                                              |
| `ModuleRoute`    | 1..191 chars                                                      |
| `ParentRoute`    | Optional, 1..191 chars                                           |
| `LangName`       | 1..191 chars (i18n key)                                           |
| `IconClass`      | 1..191 chars                                                      |
| `ModuleInfoType` | `1` (module), `2` (module link), `3` (module link crud)           |

## AddOn / Module Manager

| Type             | Constraints                                                       |
| ---------------- | ----------------------------------------------------------------- |
| `PackageName`    | 1..200 chars, unique                                              |
| `Version`        | Semantic version string                                           |
| `UpdateUrl`      | URL                                                                |
| `PurchaseCode`   | Opaque string                                                     |
| `Checksum`       | SHA-256 hex                                                       |
| `InstalledDomain`| 1..200 chars                                                      |
| `AddonUrl`       | URL                                                                |
| `ActivatedDate`  | `NaiveDate`                                                       |
| `LangType`       | `i32`                                                              |

## Locale & Lookups

| Type              | Constraints                                                       |
| ----------------- | ----------------------------------------------------------------- |
| `CountryCode`     | ISO 3166-1 alpha-2, 2 chars                                       |
| `CountryName`     | 1..191 chars                                                      |
| `CountryNative`   | 1..191 chars                                                      |
| `PhoneCode`       | 1..191 chars (the country phone prefix)                           |
| `ContinentCode`   | 1..191 chars                                                      |
| `ContinentName`   | 1..191 chars                                                      |
| `CurrencyCode`    | ISO 4217, 3 chars                                                 |
| `CurrencyName`    | 1..191 chars                                                      |
| `CurrencySymbol`  | 1..191 chars                                                      |
| `CurrencyType`    | `1` (custom), `2` (system)                                        |
| `CurrencyPosition`| `1` (prefix with space), `2` (suffix with space)                  |
| `DecimalDigit`    | `u8` in `0..=8`                                                   |
| `DecimalSeparator`| 1 char                                                            |
| `ThousandSeparator` | 1..191 chars (can be empty)                                    |
| `SpaceBetween`    | `bool`                                                            |
| `LanguageCode`    | ISO 639-1, 2..191 chars                                           |
| `LanguageName`    | 1..191 chars                                                      |
| `LanguageNative`  | 1..191 chars                                                      |
| `LanguageUniversal` | 1..191 chars                                                   |
| `RtlFlag`         | `bool`                                                            |
| `TimeZoneCode`    | 1..191 chars                                                      |
| `TimeZoneIana`    | 1..191 chars (IANA tz identifier)                                 |

## Visitor

| Type                | Constraints                                                       |
| ------------------- | ----------------------------------------------------------------- |
| `VisitorName`       | 1..255 chars                                                      |
| `VisitorId`         | 1..255 chars (printed badge id)                                   |
| `NoOfPerson`        | `i32`, positive                                                   |
| `Purpose`           | 1..255 chars                                                      |
| `InTime`            | `String` "HH:MM"                                                  |
| `OutTime`           | `String` "HH:MM"                                                  |
| `VisitorFile`       | `FileReference?`                                                  |

## ToDo

| Type             | Constraints                                                       |
| ---------------- | ----------------------------------------------------------------- |
| `ToDoTitle`      | 1..191 chars                                                      |
| `ToDoStatus`     | `Complete` (`C`), `NotComplete` (`N`), `Pending` (`P`)            |
| `ToDoDate`       | `NaiveDate`                                                       |

## Amount Transfer

| Type                | Constraints                                                       |
| ------------------- | ----------------------------------------------------------------- |
| `Amount`            | `Decimal`, positive                                              |
| `TransferPurpose`   | 1..191 chars                                                      |
| `PaymentMethod`     | `i32` (references a base setup)                                  |
| `BankName`          | `i32` (references a base setup)                                  |
| `TransferDate`      | `NaiveDate`                                                       |

## Frontend Permission

| Type                   | Constraints                                                |
| ---------------------- | ---------------------------------------------------------- |
| `FrontendPermissionName` | 1..255 chars                                             |
| `IsPublished`          | `bool`                                                     |

## Personal Access Token

| Type                | Constraints                                                       |
| ------------------- | ----------------------------------------------------------------- |
| `TokenName`         | 1..191 chars                                                      |
| `TokenHash`         | SHA-256 hex (64 chars)                                            |
| `TokenableType`     | Aggregate type name (e.g. `User`)                                 |
| `Abilities`         | `BTreeSet<Capability>` (parsed at issuance)                       |
| `LastUsedAt`        | Optional `Timestamp`                                              |
| `ExpiresAt`         | Optional `Timestamp`                                              |

## School Identity Bindings

| Type            | Notes                                                       |
| --------------- | ------------------------------------------------------------ |
| `SchoolId`      | Globally unique; carries no parent tenant                   |
| `TenantContext` | `(SchoolId, UserId, CorrelationId, ...)` per `smsengine-core` |
| `Capability`    | From `smsengine-rbac`                                         |

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
let email = EmailAddress::parse("ada@example.com")?;
let otp = OtpCode::new("123456")?;
let code = CurrencyCode::new("USD")?;
```

Parsing returns `Result<T, ValueError>`. There are no setters that
bypass validation.
