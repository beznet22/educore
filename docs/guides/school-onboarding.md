# School Onboarding Guide

## Goal

Take a new school from a bare database to a working configuration
ready for daily operations.

## Steps

1. **Create the school**: a super-admin or platform operator
   provisions a new school via `CreateSchoolCommand`.
2. **Configure general settings**: school name, address, contact,
   logo, theme.
3. **Set up users**: school admin, principal, accountant, librarian,
   transport officer.
4. **Set up classes and sections**: Class 1 through Class 12 with
   sections A, B, C, etc.
5. **Set up subjects**: Mathematics, English, Science, etc.
6. **Set up academic year**: 2026-2027, June 1 to May 31.
7. **Assign class teachers and subject teachers**.
8. **Create class sections** for the new academic year.
9. **Set up fees groups and types**: Tuition, Transport, Exam, etc.
10. **Configure the fees master** for each class.
11. **Set up the bank account** for fee collection.
12. **Set up communication channels**: SMS gateway, email server.
13. **Set up the timetable** (class times, periods).
14. **Set up class routines**.
15. **Admit students** (one per `AdmitStudentCommand` or bulk import).
16. **Set up the library** (books, categories).
17. **Set up transport** (routes, vehicles, drivers).
18. **Set up dormitories** (rooms, allocations) — if applicable.
19. **Configure notifications** (absence notifications, fee
    reminders).
20. **Configure exam schedule** for the term.
21. **Train staff** on the system.

## Day 1 Workflow

After onboarding, daily operations begin:

```text
1. Students arrive.
2. Teachers mark attendance.
3. Absent students' guardians receive SMS/email.
4. Teachers enter marks for the day's lessons.
5. Admin records any fee payments.
6. Library issues/returns books.
7. Bus routes complete their trips.
8. Day ends; backups run; logs are rotated.
```

## Configuration as Code

Schools are configured declaratively. The consumer can provide a
"YAML school config" that the engine ingests:

```yaml
school:
  name: "Springfield Elementary"
  address: "..."
  phone: "+1-555-0100"
  email: "admin@springfield.edu"

academic_year:
  title: "2026-2027"
  start: 2026-06-01
  end: 2027-05-31

classes:
  - name: "Grade 1"
    sections: [A, B, C]
  - name: "Grade 2"
    sections: [A, B, C]
  ...

subjects:
  - code: "MATH"
    name: "Mathematics"
    type: Theory
  - code: "ENG"
    name: "English"
    type: Theory
  ...

fees_groups:
  - name: "Tuition"
    types:
      - name: "Tuition Q1"
        amount: 500
      - name: "Tuition Q2"
        amount: 500
      ...

bank:
  name: "Springfield National Bank"
  account_number: "..."
  ifsc: "..."
```

The engine processes this and issues the appropriate commands.

## Bulk Student Import

Schools with many existing students need a bulk import:

```rust
let importer = BulkStudentImporter::new(engine.clone());
let report = importer.import_from_csv("students.csv").await?;
println!("Imported {}, failed {}", report.imported, report.failed);
```

The CSV format is documented in `docs/specs/academic/tables.md`.
The importer validates each row and reports errors per row. The
consumer can fix errors and re-run.

## Worked Example

A consumer onboards a new school programmatically:

```rust
async fn onboard_school(engine: &Engine, super_admin: &Session) -> Result<SchoolId> {
    // 1. Create the school
    let school = engine.platform().create_school(CreateSchoolCommand {
        actor: super_admin,
        name: "Springfield Elementary".into(),
        address: ...,
        ...,
    }).await?;

    // 2. Become the school admin
    let admin_session = engine.auth().issue_session(school.school_admin_id)?;

    // 3. Configure
    engine.settings().update_general_settings(...).await?;
    engine.academic().create_class(...).await?;
    engine.academic().create_section(...).await?;
    engine.academic().create_class_section(...).await?;
    engine.academic().create_subject(...).await?;
    engine.academic().create_academic_year(...).await?;
    engine.fees().create_group(...).await?;
    engine.fees().create_type(...).await?;
    engine.finance().open_bank_account(...).await?;
    engine.hr().register_staff(...).await?;

    Ok(school.school_id)
}
```

## Audit

Every onboarding step is audited. A new school's onboarding trail
is a complete record of its initial configuration.

## Testing

- A test of the full onboarding flow against a fresh database.
- A test of bulk student import with errors.
- A test of YAML config ingestion.
- A test of duplicate school name rejection.
- A test of the day-1 workflow simulation.
