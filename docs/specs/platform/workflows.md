# Platform Domain — Workflows

Workflows orchestrate commands, queries, and policies to fulfill a
business goal. They are documented as ordered, conditional steps.

## School Onboarding Workflow

```text
1. Operator issues CreateSchoolCommand.
2. Platform creates the School row, emits SchoolCreated.
3. RBAC subscribes: seeds SuperAdmin + school admin role catalog
   and creates the school admin user with the SuperAdmin role.
4. Settings subscribes: seeds the default GeneralSettings row.
5. Academic subscribes: seeds the first AcademicYear.
6. Operations subscribes: records the school in the audit log.
7. The school admin is returned a one-time login URL.
```

**Pre-conditions:**

- The operator holds `Platform.School.Create` (a system-level
  capability).

**Failure paths:**

- Duplicate `school_code` or `domain` →
  `ValidationError::UniqueViolation`.
- Subscriber failure rolls back the school creation.

## User Registration Workflow

```text
1. SchoolAdmin issues RegisterUserCommand.
2. Platform validates email/phone/username uniqueness.
3. Platform creates the User row and binds the role.
4. Platform emits UserRegistered.
5. RBAC subscribes: materializes the role binding.
6. Communication subscribes: sends a welcome email.
7. The user is returned an activation link (if
   is_registered=false).
```

## OTP Verification Workflow

```text
1. User requests an OTP (forgot password or 2FA).
2. Platform issues IssueOtpCommand.
3. The OTP is created and delivered via the configured channel.
4. The user enters the OTP; Platform issues VerifyOtpCommand.
5. The OTP is verified and marked consumed.
6. The user is granted a session.
7. If the OTP is not entered before expired_time, a nightly job
   emits OtpExpired and the OTP is rejected on late entry.
```

**Edge cases:**

- An expired OTP is rejected with `ConflictError::OtpExpired`.
- A consumed OTP is rejected with `ConflictError::OtpAlreadyUsed`.
- An OTP for an inactive user is rejected with
  `NotFoundError::UserInactive`.

## Course Management Workflow

```text
1. SchoolAdmin creates a CourseCategory (CreateCourseCategory).
2. SchoolAdmin creates a Course (CreateCourse).
3. SchoolAdmin creates a CoursePage (CreateCoursePage) for
   detailed landing content.
4. SchoolAdmin publishes the course (PublishCourse).
5. The course is now visible on the public site.
6. Users can enroll; the enrollment is recorded as a
   CourseEnrollment entity.
```

## Module Installation Workflow

```text
1. SchoolAdmin selects a module.
2. SchoolAdmin issues EnableModuleCommand.
3. Platform emits ModuleEnabled.
4. RBAC subscribes: the module's ModuleLink rows are added to
   every role's menu.
5. Settings subscribes: the module's dashboard card is added.
6. The module is now usable.
7. To remove, SchoolAdmin issues DisableModuleCommand; the
   reverse subscribers run.
```

## AddOn Installation Workflow

```text
1. SchoolAdmin issues InstallAddOnCommand with the AddOn id.
2. Platform checks the school's plan allows the add-on.
3. Platform creates an AddOnInstallation and emits AddOnInstalled.
4. The add-on's integration port hook is registered.
5. SchoolAdmin issues EnablePlugin to surface the add-on's
   widgets on the public site.
6. To remove, SchoolAdmin issues UninstallAddOnCommand.
```

## Custom Field Configuration Workflow

```text
1. SchoolAdmin defines a form name (e.g. `student_registration`).
2. SchoolAdmin creates a CustomField (CreateCustomField).
3. SchoolAdmin marks the field as required or optional.
4. The field is now rendered on the relevant form.
5. When a user submits the form, SetCustomFieldValue writes the
   value.
6. To remove the field, SchoolAdmin issues DeleteCustomField;
   the value rows are cascade-deleted.
```

## Header Menu Configuration Workflow

```text
1. SchoolAdmin creates header menu items (CreateHeaderMenuItem).
2. SchoolAdmin sets parents and positions.
3. SchoolAdmin reorders if needed (ReorderHeaderMenu).
4. The menu is rendered on the public site.
5. SchoolAdmin toggles show/is_newtab flags.
```

## Visitor Log Workflow

```text
1. Receptionist records the visitor (RecordVisitor).
2. Receptionist prints a badge with the visitor id.
3. When the visitor leaves, Receptionist updates the record
   with the out time.
4. A daily report lists all visitors for the day.
```

## ToDo Workflow

```text
1. Staff creates a to-do (CreateToDo).
2. The to-do is assigned (ToDoAssignee).
3. Staff works on the to-do.
4. Staff marks the to-do complete (MarkToDoComplete).
5. The to-do is archived.
```

## Amount Transfer Workflow

```text
1. Accountant issues CreateAmountTransfer with from/to methods
   and amount.
2. Platform records the transfer.
3. The payment port is called to move the funds.
4. The transfer is reconciled against the chart of accounts.
5. A reversal is a new transfer that offsets the original.
```

## Personal Access Token Workflow

```text
1. Developer issues IssuePersonalAccessTokenCommand.
2. Platform returns the plaintext token exactly once.
3. The developer uses the token as a Bearer credential.
4. On each request, the engine hashes the token, looks up the
   PersonalAccessToken row, and checks abilities + expiry.
5. Developer issues RevokePersonalAccessTokenCommand to retire
   the token.
```

## Comment Moderation Workflow

```text
1. User creates a comment (CreateComment).
2. The comment is rendered publicly (if applicable).
3. A moderator may flag the comment (FlagComment) if it
   violates policy.
4. A moderator may delete the comment (DeleteComment).
5. A daily report lists flagged comments for review.
```

## User Deactivation Workflow

```text
1. SchoolAdmin issues DeactivateUserCommand.
2. Platform sets active_status=0 and emits UserDeactivated.
3. RBAC subscribes: all sessions for the user are invalidated.
4. Operations subscribes: writes a UserLogged audit entry.
5. The user is blocked from logging in.
6. The user may be reactivated (ReactivateUserCommand) within
   a grace period; after that, the user record is preserved but
   the active_status=0 is permanent.
```

## Idempotency

- `RegisterUser` is idempotent on `(school_id, lower(email))`. A
  duplicate returns the existing user.
- `EnableModule` is idempotent on `(school_id, module_id)`. A
  duplicate is a no-op success.
- `IssueOtp` is idempotent on `(user_id, channel)` for a short
  window (typically 60 seconds). A duplicate issues a fresh
  code and invalidates the previous one.
- `VerifyOtp` is not idempotent — each call consumes the OTP.
