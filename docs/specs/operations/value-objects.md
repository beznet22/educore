# Operations Domain — Value Objects

Value objects are immutable, validated at construction, and have no
identity. They are compared by value.

## Identifiers

All identifiers in the operations domain are typed and tenant-scoped
where appropriate. Jobs, system versions, version history, and OAuth
tokens are global (not tenant-scoped).

| Identifier                | Backing Type                | Notes                              |
| ------------------------- | --------------------------- | ---------------------------------- |
| `BackupId`                | `Id<Backup>`                | A backup record                    |
| `JobId`                   | `Id<Job>` (global)          | A pending job                      |
| `FailedJobId`             | `Id<FailedJob>` (global)    | A failed job                       |
| `SystemVersionId`         | `Id<SystemVersion>`         | A version metadata record          |
| `VersionHistoryId`        | `Id<VersionHistory>`        | A version bump record              |
| `UserLogId`               | `Id<UserLog>`               | A login event record               |
| `MaintenanceSettingId`    | `Id<MaintenanceSetting>`    | A maintenance mode config          |
| `SidebarId`               | `Id<Sidebar>`               | A sidebar layout projection        |
| `JobAttemptId`            | `Id<JobAttempt>` (global)   | A single job attempt               |
| `BackupScheduleId`        | `Id<BackupSchedule>`        | A backup schedule                  |
| `BackupRetentionId`       | `Id<BackupRetention>`       | A backup retention policy          |
| `AuditPartitionId`        | `Id<AuditPartition>`        | A time partition for the user log  |

## Backup

| Type           | Constraints                                                       |
| -------------- | ----------------------------------------------------------------- |
| `BackupFileName` | 1..255 chars, unique within `(school_id, file_name)`            |
| `BackupSourceLink` | URL or file-storage reference (1..255 chars)                  |
| `BackupFileType`  | `Database` (0), `File` (1), `Image` (2)                        |
| `BackupLangType`  | `i32` (consumer-defined)                                       |
| `BackupActiveStatus` | `bool`                                                      |

## Job

| Type             | Constraints                                                       |
| ---------------- | ----------------------------------------------------------------- |
| `JobQueue`       | 1..191 chars (e.g. `default`, `emails`, `webhooks`)               |
| `JobPayload`     | A serialized command envelope; engine-validated at dequeue       |
| `JobAttempts`    | `u8`, 0..=255                                                     |
| `JobReservedAt`  | Optional `Timestamp`                                              |
| `JobAvailableAt` | `Timestamp`                                                       |
| `JobCreatedAt`   | `Timestamp`                                                       |
| `JobStatus`      | `Pending`, `Reserved`, `Completed`, `Failed`                      |

## FailedJob

| Type               | Constraints                                                       |
| ------------------ | ----------------------------------------------------------------- |
| `FailedJobUuid`    | UUIDv4, unique                                                    |
| `FailedJobConnection` | 1..191 chars (e.g. `database`, `redis`, `sqs`)                 |
| `FailedJobQueue`   | 1..191 chars                                                      |
| `FailedJobPayload` | The original job payload                                          |
| `FailedJobException` | The captured exception text                                   |
| `FailedAt`         | `Timestamp`                                                       |

## SystemVersion

| Type            | Constraints                                                       |
| --------------- | ----------------------------------------------------------------- |
| `VersionName`   | A valid semantic version (e.g. `8.2.3`), unique                   |
| `VersionTitle`  | 1..255 chars                                                      |
| `VersionFeatures` | 1..255 chars (blurb)                                           |

## VersionHistory

| Type               | Constraints                                                       |
| ------------------ | ----------------------------------------------------------------- |
| `HistoryVersion`   | 1..191 chars (free-form; typically the same as `VersionName`)     |
| `HistoryReleaseDate` | 1..191 chars (consumer format string)                          |
| `HistoryUrl`       | URL or empty                                                      |
| `HistoryNotes`     | 1..191 chars                                                      |

## UserLog

| Type                | Constraints                                                       |
| ------------------- | ----------------------------------------------------------------- |
| `IpAddress`         | A valid IPv4 or IPv6 address or empty                            |
| `UserAgent`         | 1..191 chars                                                      |
| `LoginOutcome`      | `Success`, `Failure`                                              |
| `LoginFailureReason`| `InvalidCredentials`, `InactiveUser`, `Locked`, `MaintenanceMode` |

## Maintenance

| Type                | Constraints                                                       |
| ------------------- | ----------------------------------------------------------------- |
| `MaintenanceTitle`  | 1..191 chars (default: `"We will be back soon!"`)                 |
| `MaintenanceSubTitle` | 1..191 chars (default: `"Sorry for the inconvenience..."`)      |
| `MaintenanceImage`  | `FileReference?`                                                  |
| `MaintenanceApplicableFor` | Free-form string (e.g. `all`, `student,parent`)            |
| `MaintenanceMode`   | `bool`                                                            |

## Sidebar

| Type                | Constraints                                                       |
| ------------------- | ----------------------------------------------------------------- |
| `SidebarPosition`   | `i32` (sort order)                                                |
| `SidebarSectionId`  | `i32` (a section id, consumer-defined)                            |
| `SidebarLevel`      | `Parent` (1), `Child` (2), `SubChild` (3)                         |
| `SidebarParent`     | `i32` (parent sidebar id) or 0                                    |
| `SidebarParentRoute` | `i32` (parent route id) or 0                                    |
| `SidebarIgnore`     | `i32` (0=Show, 1=Hide, 2=Disabled)                                 |
| `SidebarIsSaas`     | `bool`                                                            |
| `SidebarActiveStatus` | `bool`                                                          |

## Migration

| Type                | Constraints                                                       |
| ------------------- | ----------------------------------------------------------------- |
| `MigrationName`     | 1..191 chars                                                      |
| `MigrationBatch`    | `i32` (the batch number)                                          |

## OAuth

| Type                 | Constraints                                                       |
| -------------------- | ----------------------------------------------------------------- |
| `OAuthClientId`      | 1..191 chars (the unique client id)                               |
| `OAuthClientName`    | 1..191 chars                                                      |
| `OAuthClientSecret`  | 1..200 chars (hashed in storage)                                  |
| `OAuthRedirectUri`   | 1..1000 chars                                                     |
| `OAuthScopes`        | 1..1000 chars (space-separated)                                   |
| `OAuthAccessTokenId` | 1..191 chars (the token id)                                       |
| `OAuthExpiresAt`     | Optional `Timestamp`                                              |
| `OAuthRevoked`       | `bool`                                                            |
| `OAuthProvider`      | 1..191 chars (the provider name)                                  |

## Password Reset

| Type                | Constraints                                                       |
| ------------------- | ----------------------------------------------------------------- |
| `PasswordResetEmail`| 1..100 chars (RFC 5322)                                           |
| `PasswordResetToken`| 1..191 chars (hashed)                                            |

## School Identity Bindings

| Type            | Notes                                                       |
| --------------- | ------------------------------------------------------------ |
| `SchoolId`      | From `smsengine-platform`                                     |
| `TenantContext` | `(SchoolId, UserId, ...)` from `smsengine-platform`           |
| `UserId`        | From `smsengine-platform`                                     |
| `RoleId`        | From `smsengine-rbac`                                         |
| `AcademicYearId`| From `smsengine-academic`                                     |

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
let name = BackupFileName::new("backup-2026-06-08.sql")?;
let ip = IpAddress::new("192.0.2.1")?;
let version = VersionName::new("8.2.4")?;
```

Parsing returns `Result<T, ValueError>`. There are no setters that
bypass validation.

## Type-Safe Wrappers

```rust
pub struct BackupFileName(String);

impl BackupFileName {
    pub fn new(s: &str) -> Result<Self, ValueError> {
        if s.is_empty() || s.len() > 255 {
            return Err(ValueError::InvalidBackupFileName);
        }
        Ok(Self(s.to_string()))
    }
}
```

`BackupFileName` is the type carried in the storage row. It cannot
be constructed from an empty or oversized string.
