# 06 — Field-Level Data Flow (Phase 6)

## Goal

For the 15 most-critical legacy tables, the column-by-column
mapping from legacy column → engine column. Every column's
transform is specified: type change, rename, drop, nullability
change, FK target rename, default change, on-delete action change.

This is the most-mechanical phase. The ETL script reads from
`devdb.<legacy_table>` and writes to `devdb_v2.<engine_table>`
applying the transforms per row.

## The 15 priority tables

These are the 15 tables chosen for explicit field-level mapping.
The other ~295 tables follow the same pattern with their own
per-table field maps, which are produced by the consumer during
implementation (the engine does not maintain 295 field maps; the
pattern is canonical and the consumer applies it).

The 15 chosen are:

1. `sm_students` → `academic_students` (61 columns)
2. `sm_schools` → `platform_schools`
3. `users` (Laravel) → `platform_users`
4. `sm_academic_years` → `academic_academic_years`
5. `sm_classes` → `academic_classes`
6. `sm_sections` → `academic_sections`
7. `sm_subjects` → `academic_subjects`
8. `sm_staffs` → `hr_staffs` (50+ columns, many typos)
9. `sm_exams` → `assessment_exams`
10. `sm_marks_registers` + children → `assessment_marks_registers` + children
11. `sm_fees_assigns` + `sm_fees_payments` → `finance_fees_assigns` + `finance_fees_payments`
12. `fm_fees_invoices` → `finance_invoices`
13. `infix_roles` → `rbac_roles`
14. `infix_permission_assigns` → `rbac_permission_assigns`
15. `sm_general_settings` → `settings_general_settings` (the module_toggles drop)

## Convention notes applied to every table

- `id`: legacy `INT(10)/INT(11) UNSIGNED AUTO_INCREMENT` → engine
  `CHAR(36)` carrying a deterministic UUIDv7 derived from
  `legacy_id || table_namespace` (see `02-id-conversion.md`). The
  `id_v7_legacy BIGINT UNSIGNED NULL` column carries the original
  BIGINT for 90 days.
- `school_id`: legacy `INT(10) UNSIGNED DEFAULT 1` → engine
  `CHAR(36) NOT NULL`. The bootstrap-school UUIDv7 is generated
  by the seed.
- `version`, `etag`, `last_event_id`, `correlation_id`, `source`:
  NEW per `database-schema.md` § 5 and § 9.
- `active_status`: legacy already has it (most tables); engine
  retains verbatim.
- FK targets prefixed with `rbac_`, `platform_`, `academic_`, `hr_`,
  `assessment_`, `finance_`, `settings_`, etc. are the new engine
  aggregates. Their UUIDv7 PKs replace the legacy `INT(10)` PKs.
- `ON DELETE CASCADE` on parent `school_id` and `user_id` is
  flipped to `ON DELETE RESTRICT` per `database-schema.md` § 4.
- Money columns: `FLOAT(10,2)`, `DOUBLE(8,2)`, `VARCHAR(200)` are
  widened to `DECIMAL(12,2)` or `DECIMAL(14,2)`.
- PII columns: flagged for production encryption (consumer's
  deployment, not the migration itself).

## 1. `sm_students` → `academic_students`

| Legacy | Type | Engine | Type | Transform |
| --- | --- | --- | --- | --- |
| `id` | `bigint AUTO_INCREMENT` | `id` | `CHAR(36) PRIMARY KEY` | BIGINT → UUIDv7 |
| `admission_no` | `int(11) NULL` | `admission_number` | `VARCHAR(64) NULL` | INT → VARCHAR (preserves leading zeros) |
| `roll_no` | `int(11) NULL` | `roll_number` | `VARCHAR(32) NULL` | INT → VARCHAR |
| `first_name` | `varchar(200) NULL` | `first_name` | `VARCHAR(200) NOT NULL` | tighten NULL |
| `last_name` | `varchar(200) NULL` | `last_name` | `VARCHAR(200) NULL` | verbatim |
| `full_name` | `varchar(200) NULL` | `full_name` | `VARCHAR(200) NULL` | verbatim; computed on insert |
| `date_of_birth` | `date NULL` | `date_of_birth` | `DATE NULL` | verbatim |
| `caste` | `varchar(200) NULL` | `caste` | `VARCHAR(200) NULL` | verbatim |
| `email` | `varchar(200) NULL` | `email` | `VARCHAR(200) NULL` | PII; production encryption |
| `mobile` | `varchar(200) NULL` | `mobile` | `VARCHAR(32) NULL` | shrink; PII |
| `admission_date` | `date NULL` | `admission_date` | `DATE NULL` | verbatim |
| `student_photo` | `varchar(191) NULL` | `photo_storage_key` | `VARCHAR(191) NULL` | rename (object-storage key, not URL) |
| `age` | `varchar(200) NULL` | `age_at_admission` | `VARCHAR(16) NULL` | shrink; engine recomputes from DOB |
| `height` | `varchar(200) NULL` | `height_cm` | `DECIMAL(5,2) NULL` | widen to numeric |
| `weight` | `varchar(200) NULL` | `weight_kg` | `DECIMAL(5,2) NULL` | widen to numeric |
| `current_address` | `varchar(500) NULL` | `current_address` | `VARCHAR(500) NULL` | verbatim |
| `permanent_address` | `varchar(500) NULL` | `permanent_address` | `VARCHAR(500) NULL` | verbatim |
| `driver_id` | `varchar(200) NULL` (no FK) | `driver_id` | `CHAR(36) NULL` | new FK to `hr_drivers.id` |
| `national_id_no` | `varchar(200) NULL` | `national_id_number` | `VARCHAR(64) NULL` | PII |
| `local_id_no` | `varchar(200) NULL` | `local_id_number` | `VARCHAR(64) NULL` | PII |
| `bank_account_no` | `varchar(200) NULL` | `bank_account_number` | `VARCHAR(64) NULL` | PII |
| `bank_name` | `varchar(200) NULL` | `bank_name` | `VARCHAR(200) NULL` | verbatim |
| `previous_school_details` | `varchar(500) NULL` | `previous_school_details` | `VARCHAR(500) NULL` | verbatim |
| `aditional_notes` | `text NULL` | `additional_notes` | `TEXT NULL` | spelling fix |
| `ifsc_code` | `varchar(50) NULL` | `ifsc_code` | `VARCHAR(11) NULL` | tighten to standard |
| `document_title_1..4` | `varchar(200) NULL` | `document_1_title..4` | `VARCHAR(200) NULL` | rename |
| `document_file_1..4` | `varchar(200) NULL` | `document_1_key..4` | `VARCHAR(191) NULL` | rename; object-storage key |
| `active_status` | `tinyint(4) DEFAULT 1` | `active_status` | `TINYINT NOT NULL DEFAULT 1` | verbatim |
| `custom_field` | `text NULL` | `custom_fields` | `JSON NULL` | parse legacy `text` to JSON |
| `custom_field_form_name` | `string NULL` | `custom_field_form_name` | `VARCHAR(191) NULL` | verbatim |
| `created_at` / `updated_at` | `timestamp NULL` | `created_at` / `updated_at` | `TIMESTAMP NOT NULL` | tighten |
| `created_by` / `updated_by` | (not in file) | `created_by` / `updated_by` | `CHAR(36) NOT NULL` | NEW per § 2 |
| `bloodgroup_id` | `int FK sm_base_setups` | `blood_group_id` | `CHAR(36) FK settings_base_setups` | INT → UUIDv7 |
| `religion_id` | `int FK sm_base_setups` | `religion_id` | `CHAR(36) FK settings_base_setups` | INT → UUIDv7 |
| `route_list_id` | `int FK sm_routes` | `route_id` | `CHAR(36) FK facilities_routes` | INT → UUIDv7; rename |
| `dormitory_id` | `int FK sm_dormitory_lists` | `dormitory_id` | `CHAR(36) FK facilities_dormitories` | INT → UUIDv7; rename |
| `vechile_id` | `int FK sm_vehicles` | `vehicle_id` | `CHAR(36) FK facilities_vehicles` | typo fix; INT → UUIDv7 |
| `room_id` | `int FK sm_room_lists` | `room_id` | `CHAR(36) FK facilities_rooms` | INT → UUIDv7; rename |
| `student_category_id` | `int FK sm_student_categories` | `category_id` | `CHAR(36) FK academic_student_categories` | INT → UUIDv7; rename |
| `student_group_id` | `int FK sm_student_groups` | `group_id` | `CHAR(36) FK academic_student_groups` | INT → UUIDv7; rename |
| `class_id` | `int FK sm_classes` | `class_id` | `CHAR(36) FK academic_classes` | INT → UUIDv7 |
| `section_id` | `int FK sm_sections` | `section_id` | `CHAR(36) FK academic_sections` | INT → UUIDv7 |
| `session_id` | `int FK sm_academic_years` | `academic_id` | `CHAR(36) FK academic_academic_years` | INT → UUIDv7; rename |
| `parent_id` | `int FK sm_parents` | `guardian_id` | `CHAR(36) FK academic_parents` | INT → UUIDv7; rename |
| `user_id` | `int FK users (CASCADE)` | `user_id` | `CHAR(36) FK platform_users (RESTRICT)` | INT → UUIDv7; CASCADE → RESTRICT |
| `role_id` | `int FK infix_roles (CASCADE)` | `role_id` | `CHAR(36) FK rbac_roles (RESTRICT)` | INT → UUIDv7; CASCADE → RESTRICT |
| `gender_id` | `int FK sm_base_setups` | `gender_id` | `CHAR(36) FK settings_base_setups` | INT → UUIDv7 |
| `school_id` | `int DEFAULT 1` | `school_id` | `CHAR(36) NOT NULL` | INT → UUIDv7 |
| `academic_id` | `int FK sm_academic_years` | `academic_id` | `CHAR(36) FK academic_academic_years` | INT → UUIDv7 |
| (NEW) | — | `version` | `BIGINT NOT NULL DEFAULT 1` | NEW per § 5, § 9 |
| (NEW) | — | `etag` | `CHAR(32) NOT NULL` | NEW per § 5, § 9 |
| (NEW) | — | `last_event_id` | `CHAR(36) NULL` | NEW per § 5 |
| (NEW) | — | `correlation_id` | `CHAR(36) NULL` | NEW per § 5 |
| (NEW) | — | `source` | `VARCHAR(16) NULL` | NEW per § 5 |
| (NEW) | — | `id_v7_legacy` | `BIGINT UNSIGNED NULL` | NEW (transitional; 90 days) |

## 2. `sm_schools` → `platform_schools`

| Legacy | Type | Engine | Type | Transform |
| --- | --- | --- | --- | --- |
| `id` | `bigint AUTO_INCREMENT` | `id` | `CHAR(36) PRIMARY KEY` | BIGINT → UUIDv7 |
| `school_name` | `varchar(200) NULL` | `name` | `VARCHAR(200) NOT NULL` | rename; tighten |
| `created_by` | `tinyint(4) DEFAULT 1` | `created_by` | `CHAR(36) NOT NULL` | TINYINT → UUIDv7 (bootstrap user) |
| `updated_by` | `tinyint(4) DEFAULT 1` | `updated_by` | `CHAR(36) NOT NULL` | TINYINT → UUIDv7 |
| `email` | `varchar(200) NULL` | `email` | `VARCHAR(200) NULL` | PII |
| `domain` | `varchar(191) DEFAULT 'school'` | `domain` | `VARCHAR(191) NOT NULL DEFAULT 'school'` | verbatim; add UQ |
| `address` | `text NULL` | `address` | `TEXT NULL` | widen (legacy limit was spurious) |
| `phone` | `varchar(20) NULL` | `phone` | `VARCHAR(32) NULL` | PII |
| `school_code` | `varchar(200) NULL` | `code` | `VARCHAR(64) NULL` | shrink; add UQ |
| `is_email_verified` | `boolean DEFAULT 0` | `is_email_verified` | `BOOLEAN NOT NULL DEFAULT FALSE` | verbatim |
| `starting_date` | `date NULL` | `subscription_start` | `DATE NULL` | rename |
| `ending_date` | `date NULL` | `subscription_end` | `DATE NULL` | rename |
| `package_id` | `int(11) NULL` | `package_id` | `CHAR(36) NULL` | INT → UUIDv7 of `platform_packages` |
| `plan_type` | `varchar(200) NULL` | `plan_type` | `VARCHAR(64) NULL` (CHECK enum) | shrink |
| `region` | `int(11) NULL` | `region_id` | `CHAR(36) NULL` | INT → UUIDv7 of `platform_regions` |
| `contact_type` | `enum('yearly','monthly','once')` | `billing_cycle` | `VARCHAR(16) NULL` (CHECK enum) | rename |
| `active_status` | `tinyint(4) DEFAULT 1` | `active_status` | `TINYINT NOT NULL DEFAULT 1` | verbatim |
| `is_enabled` | `varchar(20) DEFAULT 'yes'` | `login_enabled` | `BOOLEAN NOT NULL DEFAULT TRUE` | rename; tighten |
| `created_at` / `updated_at` | `timestamp NULL` | `created_at` / `updated_at` | `TIMESTAMP NOT NULL` | tighten |
| (NEW) | — | `version`, `etag`, `last_event_id`, `correlation_id`, `source`, `id_v7_legacy` | per § 5, § 9 | NEW |

Note: `platform_schools` is the **root tenant table**. It has no
`school_id` self-reference; the row-level-security policy is
`USING (id = current_setting('app.school_id')::uuid)`. Every other
table's `school_id` FKs point at `platform_schools.id`.

## 3. `users` (Laravel) → `platform_users`

| Legacy | Type | Engine | Type | Transform |
| --- | --- | --- | --- | --- |
| `id` | `bigint AUTO_INCREMENT` | `id` | `CHAR(36) PRIMARY KEY` | BIGINT → UUIDv7 |
| `full_name` | `varchar(192) NULL` | `full_name` | `VARCHAR(200) NOT NULL` | tighten |
| `username` | `varchar(192) NULL` | `username` | `VARCHAR(64) NULL` | shrink |
| `phone_number` | `varchar(191) NULL` | `phone_number` | `VARCHAR(32) NULL` | PII |
| `email` | `varchar(192) NULL` | `email` | `VARCHAR(200) NULL` | PII |
| `password` | `varchar(100) NULL` | `password_hash` | `VARCHAR(255) NOT NULL` (Argon2id) | rename; rehash on first login |
| `usertype` | `varchar(210) NULL` | (dropped) | — | deprecated |
| `active_status` | `tinyint(4) DEFAULT 1` | `active_status` | `TINYINT NOT NULL DEFAULT 1` | verbatim |
| `random_code` | `text NULL` | `password_reset_code_hash` | `VARCHAR(255) NULL` | engine stores hash only |
| `notificationToken` | `text NULL` | `notification_token` | `VARCHAR(255) NULL` | shrink |
| `remember_token` | `varchar(100) NULL` | `remember_token` | `VARCHAR(100) NULL` | verbatim |
| `created_at` / `updated_at` | `timestamp NULL` | `created_at` / `updated_at` | `TIMESTAMP NOT NULL` | tighten |
| `language` | `varchar NULL DEFAULT 'en'` | `locale` | `VARCHAR(8) NOT NULL DEFAULT 'en'` | rename; shrink |
| `style_id` | `int(11) DEFAULT 1` | `ui_style_id` | `CHAR(36) NULL` | INT → UUIDv7 |
| `rtl_ltl` | `int(11) DEFAULT 2` | `text_direction` | `VARCHAR(3) NOT NULL DEFAULT 'ltr'` | rename; tighten |
| `selected_session` | `int(11) DEFAULT 1` | `selected_academic_id` | `CHAR(36) NULL` | INT → UUIDv7; rename |
| `created_by` / `updated_by` | `int(11) DEFAULT 1` | `created_by` / `updated_by` | `CHAR(36) NOT NULL` | INT → UUIDv7 |
| `access_status` | `int(11) DEFAULT 1` | `access_status` | `TINYINT NOT NULL DEFAULT 1` | shrink |
| `school_id` | `int(10) UNSIGNED DEFAULT 1` | `school_id` | `CHAR(36) NOT NULL` | INT → UUIDv7 |
| `role_id` | `int(10) UNSIGNED NULL FK infix_roles (CASCADE)` | `role_id` | `CHAR(36) NOT NULL FK rbac_roles (RESTRICT)` | INT → UUIDv7; CASCADE → RESTRICT; tighten |
| `is_administrator` | `enum('yes','no') DEFAULT 'no'` | `is_administrator` | `BOOLEAN NOT NULL DEFAULT FALSE` | enum → boolean |
| `is_registered` | `tinyint(4) DEFAULT 0` | `is_registered` | `BOOLEAN NOT NULL DEFAULT FALSE` | verbatim |
| `device_token` | `text NULL` | `device_token` | `VARCHAR(255) NULL` | shrink |
| `stripe_id` / `card_brand` / `card_last_four` | (dropped) | — | — | engine doesn't store payment-provider IDs |
| `verified` | (dropped) | — | — | replaced by `is_email_verified` |
| `trial_ends_at` | `timestamp NULL` | `trial_ends_at` | `TIMESTAMP NULL` | verbatim |
| `wallet_balance` | `double(8,2) DEFAULT 0.00` | (dropped) | — | engine canonical: `finance_wallet` aggregate |
| (NEW) | — | `version`, `etag`, `last_event_id`, `correlation_id`, `source`, `id_v7_legacy` | per § 5, § 9 | NEW |

## 4. `sm_academic_years` → `academic_academic_years`

| Legacy | Type | Engine | Type | Transform |
| --- | --- | --- | --- | --- |
| `id` | `bigint AUTO_INCREMENT` | `id` | `CHAR(36) PRIMARY KEY` | BIGINT → UUIDv7 |
| `year` | `varchar(200) NOT NULL` | `year` | `VARCHAR(16) NOT NULL` | shrink (`2024` / `2024-2025`) |
| `title` | `varchar(200) NOT NULL` | `title` | `VARCHAR(200) NOT NULL` | verbatim |
| `starting_date` | `date NOT NULL` | `start_date` | `DATE NOT NULL` | rename |
| `ending_date` | `date NOT NULL` | `end_date` | `DATE NOT NULL` | rename |
| `copy_with_academic_year` | `varchar(191) NULL` | (dropped) | — | replaced by `CopyAcademicYear` command |
| `active_status` | `tinyint(4) DEFAULT 1` | `active_status` | `TINYINT NOT NULL DEFAULT 1` | verbatim |
| `created_at` / `updated_at` | `varchar(191) NULL` (legacy bug) | `created_at` / `updated_at` | `TIMESTAMP NOT NULL` | parse `YYYY-MM-DD` on backfill; reject malformed |
| `created_by` / `updated_by` | `int(10) UNSIGNED DEFAULT 1` | `created_by` / `updated_by` | `CHAR(36) NOT NULL` | INT → UUIDv7 |
| `school_id` | `int(10) UNSIGNED DEFAULT 1` | `school_id` | `CHAR(36) NOT NULL` | INT → UUIDv7; CASCADE → RESTRICT |
| (NEW) | — | `version`, `etag`, `last_event_id`, `correlation_id`, `source`, `id_v7_legacy` | per § 5, § 9 | NEW |

## 5. `sm_classes` → `academic_classes`

| Legacy | Type | Engine | Type | Transform |
| --- | --- | --- | --- | --- |
| `id` | `bigint AUTO_INCREMENT` | `id` | `CHAR(36) PRIMARY KEY` | BIGINT → UUIDv7 |
| `class_name` | `varchar(200) NOT NULL` | `name` | `VARCHAR(200) NOT NULL` | rename |
| `pass_mark` | `float NULL` | `pass_mark` | `DECIMAL(5,2) NULL` | float → DECIMAL |
| `active_status` | `tinyint(4) DEFAULT 1` | `active_status` | `TINYINT NOT NULL DEFAULT 1` | verbatim |
| `created_at` / `updated_at` | `timestamp NULL` | `created_at` / `updated_at` | `TIMESTAMP NOT NULL` | tighten |
| `created_by` / `updated_by` | `int(10) UNSIGNED DEFAULT 1` | `created_by` / `updated_by` | `CHAR(36) NOT NULL` | INT → UUIDv7 |
| `school_id` | `int(10) UNSIGNED DEFAULT 1` | `school_id` | `CHAR(36) NOT NULL` | INT → UUIDv7; CASCADE → RESTRICT |
| `academic_id` | `int(10) UNSIGNED NULL` | `academic_id` | `CHAR(36) NULL` | INT → UUIDv7 |
| `parent_id` | `int(11) NULL` | (dropped) | — | not modelled in engine |
| (NEW) | — | `version`, `etag`, `last_event_id`, `correlation_id`, `source`, `id_v7_legacy` | per § 5, § 9 | NEW |

## 6. `sm_sections` → `academic_sections`

| Legacy | Type | Engine | Type | Transform |
| --- | --- | --- | --- | --- |
| `id` | `bigint AUTO_INCREMENT` | `id` | `CHAR(36) PRIMARY KEY` | BIGINT → UUIDv7 |
| `parent_id` | `int(11) NULL` | (dropped) | — | not modelled |
| `section_name` | `varchar(200) NOT NULL` | `name` | `VARCHAR(200) NOT NULL` | rename |
| `active_status` | `tinyint(4) DEFAULT 1` | `active_status` | `TINYINT NOT NULL DEFAULT 1` | verbatim |
| `created_at` / `updated_at` | `timestamp NULL` | `created_at` / `updated_at` | `TIMESTAMP NOT NULL` | tighten |
| `created_by` / `updated_by` | `int(10) UNSIGNED DEFAULT 1` | `created_by` / `updated_by` | `CHAR(36) NOT NULL` | INT → UUIDv7 |
| `school_id` | `int(10) UNSIGNED DEFAULT 1` | `school_id` | `CHAR(36) NOT NULL` | INT → UUIDv7; CASCADE → RESTRICT |
| `un_academic_id` | `int(10) UNSIGNED NULL` | (dropped) | — | legacy "university" track not in MVP |
| `academic_id` | `int(10) UNSIGNED DEFAULT 1` | `academic_id` | `CHAR(36) NULL` | INT → UUIDv7; CASCADE → RESTRICT |
| (NEW) | — | `version`, `etag`, `last_event_id`, `correlation_id`, `source`, `id_v7_legacy` | per § 5, § 9 | NEW |

## 7. `sm_subjects` → `academic_subjects`

| Legacy | Type | Engine | Type | Transform |
| --- | --- | --- | --- | --- |
| `id` | `bigint AUTO_INCREMENT` | `id` | `CHAR(36) PRIMARY KEY` | BIGINT → UUIDv7 |
| `subject_name` | `varchar(255) NOT NULL` | `name` | `VARCHAR(200) NOT NULL` | rename; tighten |
| `subject_code` | `varchar(255) NULL` | `code` | `VARCHAR(32) NULL` | rename; shrink |
| `pass_mark` | `float NULL` | `pass_mark` | `DECIMAL(5,2) NULL` | float → DECIMAL |
| `subject_type` | `enum('T','P') NOT NULL` | `subject_type` | `VARCHAR(16) NOT NULL` (CHECK enum) | rename; widen |
| `active_status` | `tinyint(4) DEFAULT 1` | `active_status` | `TINYINT NOT NULL DEFAULT 1` | verbatim |
| `created_at` / `updated_at` | `timestamp NULL` | `created_at` / `updated_at` | `TIMESTAMP NOT NULL` | tighten |
| `created_by` / `updated_by` | `int(10) UNSIGNED DEFAULT 1` | `created_by` / `updated_by` | `CHAR(36) NOT NULL` | INT → UUIDv7 |
| `school_id` | `int(10) UNSIGNED DEFAULT 1` | `school_id` | `CHAR(36) NOT NULL` | INT → UUIDv7; CASCADE → RESTRICT |
| `academic_id` | `int(10) UNSIGNED DEFAULT 1` | `academic_id` | `CHAR(36) NULL` | INT → UUIDv7; CASCADE → RESTRICT |
| `parent_id` | `int(11) NULL` | (dropped) | — | not modelled |
| (NEW) | — | `version`, `etag`, `last_event_id`, `correlation_id`, `source`, `id_v7_legacy` | per § 5, § 9 | NEW |

## 8. `sm_staffs` → `hr_staffs` (50+ columns, many typos fixed)

The `sm_staffs` table has 50+ columns including several typos. The
full field map is in the legacy schema inventory; the key
transforms:

| Legacy | Type | Engine | Type | Transform |
| --- | --- | --- | --- | --- |
| `id` | `bigint AUTO_INCREMENT` | `id` | `CHAR(36) PRIMARY KEY` | BIGINT → UUIDv7 |
| `staff_no` | `int(11) NULL` | `staff_number` | `VARCHAR(32) NULL` | INT → VARCHAR |
| `first_name`, `last_name`, `full_name` | `varchar(100) NULL` | `first_name`, `last_name`, `full_name` | `VARCHAR(200) NOT NULL / NULL` | widen; tighten |
| `fathers_name`, `mothers_name` | `varchar(100) NULL` | `father_name`, `mother_name` | `VARCHAR(200) NULL` | spelling; widen |
| `date_of_birth`, `date_of_joining` | `date DEFAULT '2024-11-04'` | `date_of_birth`, `date_of_joining` | `DATE NULL` | drop bogus default |
| `email`, `mobile`, `emergency_mobile` | `varchar(50) NULL` | same | `VARCHAR(200/32) NULL` | widen; shrink; PII |
| `marital_status` (typo: `merital_status`) | `varchar(30) NULL` | `marital_status` | `VARCHAR(16) NULL` (CHECK enum) | spelling; tighten |
| `staff_photo` | `string NULL` | `photo_storage_key` | `VARCHAR(191) NULL` | rename |
| `experience` | `varchar(200) NULL` | `experience_years` | `DECIMAL(4,1) NULL` | widen |
| `epf_no` | `varchar(20) NULL` | `epf_number` | `VARCHAR(32) NULL` | PII |
| `basic_salary` | `varchar(200) NULL` | `basic_salary` | `DECIMAL(14,2) NULL` | widen to numeric |
| `contract_type` | `varchar(200) NULL` | `contract_type` | `VARCHAR(32) NULL` (CHECK enum) | tighten |
| `casual_leave`, `medical_leave`, `metarnity_leave` (typo) | `varchar(15) NULL` | `casual_leave_quota`, `medical_leave_quota`, `maternity_leave_quota` | `DECIMAL(4,1) NULL` | widen; typo fix |
| `bank_account_name`, `bank_account_no`, `bank_name`, `bank_brach` (typo) | `varchar` | same | widen; spelling | |
| `facebook_url`, `twiteer_url` (typo), `linkedin_url`, `instragram_url` (typo) | `varchar(100) NULL` | same | `VARCHAR(255) NULL` | widen; typo fix |
| `driving_license` | `varchar(255) NULL` | `driving_license_number` | `VARCHAR(64) NULL` | PII |
| `driving_license_ex_date` | `date NULL` | `driving_license_expiry` | `DATE NULL` | rename |
| `custom_field` | `text NULL` | `custom_fields` | `JSON NULL` | parse |
| `designation_id` | `int FK sm_designations` | `designation_id` | `CHAR(36) FK hr_designations` | INT → UUIDv7 |
| `department_id` | `int FK sm_human_departments` | `department_id` | `CHAR(36) FK hr_departments` | INT → UUIDv7 |
| `user_id` | `int FK users (CASCADE)` | `user_id` | `CHAR(36) FK platform_users (RESTRICT)` | CASCADE → RESTRICT |
| `role_id` | `int FK infix_roles (SET NULL)` | `role_id` | `CHAR(36) NOT NULL FK rbac_roles (RESTRICT)` | INT → UUIDv7; SET NULL → RESTRICT; tighten |
| `is_saas` | `int DEFAULT 0` | `is_saas_staff` | `BOOLEAN NOT NULL DEFAULT FALSE` | rename; tighten |
| (NEW) | — | `version`, `etag`, `last_event_id`, `correlation_id`, `source`, `id_v7_legacy` | per § 5, § 9 | NEW |

## 9. `sm_exams` → `assessment_exams`

| Legacy | Type | Engine | Type | Transform |
| --- | --- | --- | --- | --- |
| `id` | `bigint AUTO_INCREMENT` | `id` | `CHAR(36) PRIMARY KEY` | BIGINT → UUIDv7 |
| `parent_id` | `int DEFAULT 0` | `parent_id` | `CHAR(36) NULL` | INT → UUIDv7; `0` → `NULL` |
| `exam_mark` | `float NULL` | `total_marks` | `DECIMAL(6,2) NULL` | rename; widen |
| `pass_mark` | `float NULL` | `pass_marks` | `DECIMAL(6,2) NULL` | rename; widen |
| `exam_type_id`, `class_id`, `section_id`, `subject_id` | `int FK ...` | same | `CHAR(36) FK ...` | INT → UUIDv7 |
| `created_by` / `updated_by` | `int DEFAULT 1` | same | `CHAR(36) NOT NULL` | INT → UUIDv7 |
| `school_id` | `int DEFAULT 1` | `school_id` | `CHAR(36) NOT NULL` | INT → UUIDv7 |
| `academic_id` | `int DEFAULT 1` | `academic_id` | `CHAR(36) NULL` | INT → UUIDv7 |
| (NEW) | — | engine invariants | per § 5, § 9 | NEW |

## 10. `sm_marks_registers` + children

`sm_marks_registers` → `assessment_marks_registers`:

| Legacy | Type | Engine | Type | Transform |
| --- | --- | --- | --- | --- |
| `id` | `bigint AUTO_INCREMENT` | `id` | `CHAR(36) PRIMARY KEY` | BIGINT → UUIDv7 |
| `student_id`, `exam_id`, `class_id`, `section_id` | `int FK ...` | same | `CHAR(36) FK ...` | INT → UUIDv7 |
| `school_id`, `academic_id` | `int DEFAULT 1` | same | `CHAR(36) NOT NULL` / `NULL` | INT → UUIDv7 |
| `created_by` / `updated_by` | `int DEFAULT 1` | same | `CHAR(36) NOT NULL` | INT → UUIDv7 |
| (NEW) | — | engine invariants | per § 5, § 9 | NEW |

`sm_marks_register_children` → `assessment_marks_register_children`:

| Legacy | Type | Engine | Type | Transform |
| --- | --- | --- | --- | --- |
| `id` | `bigint AUTO_INCREMENT` | `id` | `CHAR(36) PRIMARY KEY` | BIGINT → UUIDv7 |
| `marks` | `int(11) NULL` | `marks_obtained` | `DECIMAL(6,2) NULL` | rename; widen |
| `abs` | `int(11) DEFAULT 0` | `is_absent` | `BOOLEAN NOT NULL DEFAULT FALSE` | rename; tighten |
| `gpa_point` | `float NULL` | `gpa_point` | `DECIMAL(4,2) NULL` | widen |
| `gpa_grade` | `varchar(55) NULL` | `gpa_grade` | `VARCHAR(8) NULL` | shrink |
| `marks_register_id`, `subject_id` | `int FK ... (CASCADE)` | same | `CHAR(36) NOT NULL FK ... (RESTRICT)` | INT → UUIDv7; CASCADE → RESTRICT; tighten |
| (NEW) | — | engine invariants | per § 5, § 9 | NEW |

## 11. `sm_fees_assigns` + `sm_fees_payments`

`sm_fees_assigns` → `finance_fees_assigns`:

| Legacy | Type | Engine | Type | Transform |
| --- | --- | --- | --- | --- |
| `id` | `bigint AUTO_INCREMENT` | `id` | `CHAR(36) PRIMARY KEY` | BIGINT → UUIDv7 |
| `fees_amount`, `applied_discount` | `float(10,2) NULL` | same | `DECIMAL(12,2) NULL` | widen |
| `fees_master_id`, `fees_discount_id`, `student_id`, `class_id`, `section_id` | `int FK ...` | same | `CHAR(36) FK ...` | INT → UUIDv7 |
| `record_id` | `int` (no FK) | `student_record_id` | `CHAR(36) FK academic_student_records` | rename; new FK |
| `school_id`, `academic_id` | `int DEFAULT 1` | same | `CHAR(36) NOT NULL` / `NULL` | INT → UUIDv7 |
| (NEW) | — | engine invariants | per § 5, § 9 | NEW |

`sm_fees_payments` → `finance_fees_payments`:

| Legacy | Type | Engine | Type | Transform |
| --- | --- | --- | --- | --- |
| `id` | `bigint AUTO_INCREMENT` | `id` | `CHAR(36) PRIMARY KEY` | BIGINT → UUIDv7 |
| `discount_amount`, `fine`, `amount` | `double/float` | `discount_amount`, `fine_amount`, `paid_amount` | `DECIMAL(12,2) NULL` | rename; widen |
| `payment_date` | `date NULL` | `payment_date` | `DATE NULL` | verbatim |
| `payment_mode` | `varchar(100) NULL` | `payment_mode` | `VARCHAR(32) NULL` (CHECK enum) | tighten |
| `slip` | `string NULL` | `slip_storage_key` | `VARCHAR(191) NULL` | rename |
| `assign_id`, `bank_id`, `fees_discount_id`, `fees_type_id`, `student_id` | `int FK ...` | same | `CHAR(36) FK ...` | INT → UUIDv7 |
| `direct_fees_installment_assign_id`, `installment_payment_id` | `int` (no FK) | renamed | `CHAR(36) FK ...` | new FKs |
| `school_id`, `academic_id` | `int DEFAULT 1` | same | `CHAR(36) NOT NULL` / `NULL` | INT → UUIDv7 |
| (NEW) | — | engine invariants | per § 5, § 9 | NEW |

## 12. `fm_fees_invoices` → `finance_invoices`

The engine dump shows two related tables: the **config** table
`fees_invoices` (only `prefix` + `start_form`, no real invoice
rows) and the **invoice** table `fm_fees_invoices` (real invoice
rows). Both are migrated; the config table becomes
`finance_invoice_settings`, the invoice table becomes
`finance_invoices`.

`fm_fees_invoices` → `finance_invoices`:

| Legacy | Type | Engine | Type | Transform |
| --- | --- | --- | --- | --- |
| `id` | `bigint AUTO_INCREMENT` | `id` | `CHAR(36) PRIMARY KEY` | BIGINT → UUIDv7 |
| `invoice_number` | `varchar(191)` | `invoice_number` | `VARCHAR(64) NOT NULL` | tighten |
| `amount`, `paid_amount`, `due_amount`, `waiver_amount`, `fine_amount` | `double/float` | same | `DECIMAL(12,2) NULL` | widen |
| `issue_date`, `due_date`, `paid_date` | `date NULL` | same | `DATE NULL` | verbatim |
| `payment_status` | `varchar(20)` | `payment_status` | `VARCHAR(16) NOT NULL` (CHECK enum) | tighten |
| `class_id`, `student_id`, `assign_id`, `fees_master_id`, `fees_type_id` | `int FK ...` | same | `CHAR(36) FK ...` | INT → UUIDv7 |
| `bank_id` | `int FK sm_bank_accounts` | `bank_account_id` | `CHAR(36) FK finance_bank_accounts` | INT → UUIDv7; rename |
| `school_id`, `academic_id`, `created_by`, `updated_by` | `int DEFAULT 1` | same | `CHAR(36) NOT NULL` | INT → UUIDv7 |
| (NEW) | — | engine invariants | per § 5, § 9 | NEW |

## 13. `infix_roles` → `rbac_roles`

| Legacy | Type | Engine | Type | Transform |
| --- | --- | --- | --- | --- |
| `id` | `bigint AUTO_INCREMENT` | `id` | `CHAR(36) PRIMARY KEY` | BIGINT → UUIDv7 |
| `name` | `varchar(191) NOT NULL` | `name` | `VARCHAR(64) NOT NULL` | tighten |
| `type` | `int NOT NULL` (1=System, 2=Custom) | `role_type` | `VARCHAR(16) NOT NULL` (CHECK enum) | tighten |
| `school_id` | `int DEFAULT 1` | `school_id` | `CHAR(36) NOT NULL` | INT → UUIDv7 |
| `is_saas` | `int DEFAULT 0` | `is_replicated` | `BOOLEAN NOT NULL DEFAULT FALSE` | rename; tighten |
| `active_status` | `int DEFAULT 1` | `active_status` | `TINYINT NOT NULL DEFAULT 1` | tighten |
| `created_by` / `updated_by` / `created_at` / `updated_at` | `int` / `timestamp` | same | `CHAR(36) NOT NULL` / `TIMESTAMP NOT NULL` | type changes |
| (NEW) | — | engine invariants | per § 5, § 9 | NEW |

The shadow `InfixRole` aggregate is **deleted** (see `05-brand-removal.md`).
The `is_saas` flag is renamed to `is_replicated` and carries the
same semantics: `TRUE` means this role is replicated across
sibling schools in a SaaS deployment.

## 14. `infix_permission_assigns` → `rbac_permission_assigns`

| Legacy | Type | Engine | Type | Transform |
| --- | --- | --- | --- | --- |
| `id` | `bigint AUTO_INCREMENT` | `id` | `CHAR(36) PRIMARY KEY` | BIGINT → UUIDv7 |
| `role_id` | `int FK infix_roles` | `role_id` | `CHAR(36) FK rbac_roles` | INT → UUIDv7; CASCADE → RESTRICT |
| `permission_id` | `int FK permissions` | `permission_id` | `CHAR(36) FK rbac_permissions` | INT → UUIDv7; CASCADE → RESTRICT |
| `school_id` | `int DEFAULT 1` | `school_id` | `CHAR(36) NOT NULL` | INT → UUIDv7 |
| `is_saas` | `tinyint NOT NULL DEFAULT 0` | `is_replicated` | `BOOLEAN NOT NULL DEFAULT FALSE` | rename; tighten |
| `menu_status`, `active_status` | `int` | same | `TINYINT NOT NULL DEFAULT 1` | tighten |
| `module_info` | `text NULL` | (dropped) | — | not modelled; engine uses `permission_id.module` |
| `created_by` / `updated_by` | `int DEFAULT 1` | same | `CHAR(36) NOT NULL` | INT → UUIDv7 |
| (NEW) | — | engine invariants | per § 5, § 9 | NEW |

The `module_info` text field on the legacy `InfixPermissionAssign`
is dropped. The engine's `Permission` aggregate carries the module
as a structured field, and the assignment uses `permission_id` (not
a free-text module_info).

## 15. `sm_general_settings` → `settings_general_settings`

The 35 flat-int `module_toggles` columns are dropped entirely (see
`05-brand-removal.md`). The remaining columns:

| Legacy | Type | Engine | Type | Transform |
| --- | --- | --- | --- | --- |
| `id` | `bigint AUTO_INCREMENT` | `id` | `CHAR(36) PRIMARY KEY` | BIGINT → UUIDv7 |
| `school_id` | `int DEFAULT 1` | `school_id` | `CHAR(36) NOT NULL` | INT → UUIDv7 |
| `site_title` | `varchar(255) NULL` | `site_title` | `VARCHAR(200) NULL` | tighten |
| `site_logo` | `varchar(255) NULL` | `site_logo_storage_key` | `VARCHAR(191) NULL` | rename; object-storage key |
| `favicon` | `varchar(255) NULL` | `favicon_storage_key` | `VARCHAR(191) NULL` | rename |
| `language` | `varchar(30) NULL DEFAULT 'en'` | `default_locale` | `VARCHAR(8) NOT NULL DEFAULT 'en'` | rename; tighten |
| `date_format` | `varchar(30) NULL` | `date_format` | `VARCHAR(32) NULL` | tighten |
| `time_zone` | `varchar(30) NULL` | `time_zone` | `VARCHAR(64) NULL` | widen |
| `currency` | `varchar(30) NULL` | `default_currency` | `VARCHAR(8) NOT NULL DEFAULT 'USD'` | rename; tighten |
| `currency_symbol` | `varchar(30) NULL` | `currency_symbol` | `VARCHAR(8) NOT NULL` | tighten |
| `ss_page_load` | `int(11) DEFAULT 3` | (dropped) | — | legacy bootstrap hint |
| `is_custom_saas` | `int(11) NOT NULL DEFAULT 0` | `is_custom_saas` | `BOOLEAN NOT NULL DEFAULT FALSE` | tighten |
| `un_academic_id` | `int(10) UNSIGNED DEFAULT 1` | (dropped) | — | legacy "university" track |
| `InfixBiometrics` | `int(11) DEFAULT 0` | (renamed to `biometrics_enabled` then dropped; consumer uses `platform_packages.modules`) | | |
| (all 35 module_toggles columns) | `int(11) DEFAULT 0` | (dropped) | — | replaced by `platform_packages.modules` JSON |
| (NEW) | — | engine invariants | per § 5, § 9 | NEW |

The `module_toggles` are replaced by `platform_packages.modules JSON`
which is a `["Lesson", "Chat", ...]` array. The migration backfills
the array by reading the flat-int columns:

```sql
UPDATE platform_packages pp
JOIN sm_general_settings gs ON gs.school_id = pp.school_id
SET pp.modules = JSON_ARRAYAGG(name) FROM (
  SELECT 'Lesson' AS name WHERE gs.Lesson = 1
  UNION ALL SELECT 'Chat' WHERE gs.Chat = 1
  -- ... 35 modules
) AS enabled;
```

## What the ETL script does

The consumer's ETL script (consumer-side, not engine-side) runs:

```sql
-- For each (legacy_table, engine_table) pair:
-- 1. SELECT from legacy with the type-driven column transforms
-- 2. Apply the deterministic UUIDv7 derivation
-- 3. Backfill engine-invariant columns (version=1, etag=blake3(id||school_id), etc.)
-- 4. INSERT into engine table
-- 5. Verify row count and sample integrity

-- Per-table 1:1 in most cases; some legacy tables merge into one engine table
-- (e.g. legacy `users` + `platform_users_history` may merge).
```

The ETL is idempotent: re-running with the same `(legacy_id,
table_namespace)` produces the same UUIDv7.

## Exit criteria

- The 15 priority tables are migrated with row counts matching the
  legacy DB.
- A spot check of 5 random rows per table shows the new column
  values match the legacy values modulo the documented transforms
  (e.g. `first_name` trimmed from `varchar(200) NULL` to `NOT NULL`
  is empty string in legacy, becomes `''` in engine; `email` from
  `varchar(200)` becomes `varchar(200)` unchanged; etc.).
- The engine's repository trait can be implemented against the new
  schema (the storage adapter's `to_typed_*` methods succeed).
- The `id_v7_legacy` column is backfilled for every row.
- The FK constraints are reissued.
