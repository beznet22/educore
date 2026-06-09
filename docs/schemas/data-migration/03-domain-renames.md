# 03 — Domain-Aware Table Renames (Phase 3)

## Goal

Rename every legacy `migrations/*.sql` table to the engine's
domain-aware name (`<domain>_<aggregate>`), applied to `devdb_v2`
during the ETL.

## The 310-row rename map

Grouped by legacy prefix family. The "Status" column says what
happens to the table in `devdb_v2`:

- **rename** — copy with the new name, drop the old.
- **archive** — move to a `legacy_<name>` table; engine doesn't
  read it.
- **drop** — Laravel meta; engine doesn't read it.
- **keep** — engine cross-cutting (no rename).

The "New name" column is the engine's `devdb_v2` table name. All
new names are unprefixed (engine-internal) or `<domain>_<aggregate>`.

### Engine cross-cutting tables (no rename, created in Phase 1)

| Legacy | New name | Status |
| --- | --- | --- |
| (none) | `outbox` | keep (Phase 1) |
| (none) | `audit_log` | keep (Phase 1) |
| (none) | `idempotency` | keep (Phase 1) |
| (none) | `event_log` | keep (Phase 1) |
| (none) | `schema_registry` | keep (Phase 1) |
| (none) | `system_user` | keep (Phase 1) |

### Platform domain (38 legacy tables, 38 renames)

| Legacy | New name |
| --- | --- |
| `sm_schools` | `platform_schools` |
| `users` | `platform_users` |
| `sm_sessions` (auth) | `platform_sessions` |
| `password_resets` | `platform_password_resets` |
| `sm_add_ons` | `platform_add_ons` |
| `sm_amount_transfers` | `platform_amount_transfers` |
| `sm_base_groups` | `platform_base_groups` |
| `sm_chart_of_accounts` | `platform_chart_of_accounts` |
| `sm_countries` | `platform_countries` |
| `continents` | `platform_continents` (fixes typo from `continets`) |
| `sm_courses` | `platform_courses` |
| `sm_course_categories` | `platform_course_categories` |
| `sm_currencies` | `platform_currencies` |
| `sm_custom_fields` | `platform_custom_fields` |
| `sm_custom_field_values` | `platform_custom_field_values` |
| `sm_expert_teachers` | `platform_expert_teachers` |
| `sm_frontend_persmissions` | `platform_frontend_permissions` (fixes typo) |
| `sm_header_menu_managers` | `platform_header_menu_managers` |
| `sm_instructions` | `platform_instructions` |
| `sm_modules` | `platform_modules` |
| `sm_module_links` | `platform_module_links` |
| `sm_module_student_parent_infos` | `platform_student_parent_menus` (drops brand `infix_`) |
| `sm_photo_galleries` | `platform_photo_galleries` |
| `sm_social_media_icons` | `platform_social_media_icons` |
| `sm_time_zones` | `platform_time_zones` |
| `sm_to_dos` | `platform_to_dos` |
| `sm_video_galleries` | `platform_video_galleries` |
| `sm_visitors` | `platform_visitors` |
| `infix_module_infos` | `platform_module_infos` (drops brand `infix_`) |
| `infix_module_managers` | `platform_module_managers` (drops brand `infix_`) |
| `notifications` (Laravel) | `platform_notifications` |
| `personal_access_tokens` | `platform_personal_access_tokens` |
| `tenant_id` (Laravel) | `platform_tenants` (consumer-side SaaS) |
| `tenants` (Laravel) | (drop; replaced by `platform_tenants`) |
| `tenant_domain` (Laravel) | `platform_tenant_domains` |
| `subscriptions` (Laravel) | `platform_subscriptions` |
| `subscription_items` (Laravel) | `platform_subscription_items` |
| `domains` (Laravel) | `platform_domains` |

### Academic domain (50 legacy tables, 50 renames)

| Legacy | New name |
| --- | --- |
| `sm_academic_years` | `academic_academic_years` |
| `sm_sessions` (academic year) | `academic_sessions` |
| `sm_classes` | `academic_classes` |
| `sm_sections` | `academic_sections` |
| `sm_students` | `academic_students` |
| `sm_parents` | `academic_parents` |
| `sm_subjects` | `academic_subjects` |
| `sm_student_categories` | `academic_student_categories` |
| `sm_student_groups` | `academic_student_groups` |
| `sm_class_rooms` | `academic_class_rooms` |
| `sm_class_times` | `academic_class_times` |
| `sm_class_sections` | `academic_class_sections` |
| `sm_class_optional_subject` | `academic_class_optional_subject` |
| `sm_class_teachers` | `academic_class_teachers` |
| `sm_assign_subjects` | `academic_assign_subjects` |
| `sm_homeworks` | `academic_homeworks` |
| `sm_homework_students` | `academic_homework_students` |
| `sm_lessons` | `academic_lessons` |
| `sm_lesson_details` | `academic_lesson_details` |
| `sm_lesson_topics` | `academic_lesson_topics` |
| `sm_lesson_topic_details` | `academic_lesson_topic_details` |
| `sm_class_routines` | `academic_class_routines` |
| `sm_class_routine_updates` | `academic_class_routine_updates` |
| `sm_admission_queries` | `academic_admission_queries` |
| `sm_admission_query_followups` | `academic_admission_query_followups` |
| `sm_optional_subject_assigns` | `academic_optional_subject_assigns` |
| `sm_student_certificates` | `academic_student_certificates` |
| `sm_student_documents` | `academic_student_documents` |
| `sm_student_excel_formats` | `academic_student_excel_formats` |
| `sm_student_id_cards` | `academic_student_id_cards` |
| `sm_student_take_online_exams` | `academic_student_take_online_exams` |
| `sm_mark_sheets` | `academic_mark_sheets` |
| `sm_exam_setups` | `academic_exam_setups` |
| `sm_exam_routine_updates` | `academic_exam_routine_updates` |
| `sm_exam_questions` | `academic_exam_questions` |
| `sm_exam_question_assigns` | `academic_exam_question_assigns` |
| `sm_exam_question_options` | `academic_exam_question_options` |
| `sm_online_exam_questions` | `academic_online_exam_questions` |
| `sm_online_exam_question_assigns` | `academic_online_exam_question_assigns` |
| `sm_online_exam_question_options` | `academic_online_exam_question_options` |
| `sm_result_store` | `academic_result_store` |
| `sm_mark_stores` | `academic_mark_stores` |
| `sm_progress_card_rules` | `academic_progress_card_rules` |
| `sm_progress_card_template` | `academic_progress_card_template` |
| `sm_certificate_templates` | `academic_certificate_templates` |
| `sm_id_card_settings` | `academic_id_card_settings` |
| `sm_school_subjects` | `academic_school_subjects` |
| `sm_exam_signatures` | `academic_exam_signatures` |
| `sm_exam_policies` | `academic_exam_policies` |
| `sm_universities` | `academic_universities` (kept; was `un_*` confusion resolved) |
| `graduates` | `academic_graduates` |

### Assessment domain (43 legacy tables, 43 renames)

| Legacy | New name |
| --- | --- |
| `sm_exams` | `assessment_exams` |
| `sm_exam_types` | `assessment_exam_types` |
| `sm_exam_subjects` | `assessment_exam_subjects` |
| `sm_exam_subject_assigns` | `assessment_exam_subject_assigns` |
| `sm_exam_schedules` | `assessment_exam_schedules` |
| `sm_exam_schedule_subjects` | `assessment_exam_schedule_subjects` |
| `sm_marks_registers` | `assessment_marks_registers` |
| `sm_marks_register_children` | `assessment_marks_register_children` |
| `sm_marks_send_sms` | `assessment_marks_send_sms` |
| `sm_exam_attendances` | `assessment_exam_attendances` |
| `sm_exam_registers` | `assessment_exam_registers` |
| `sm_exam_register_children` | `assessment_exam_register_children` |
| `sm_online_exams` | `assessment_online_exams` |
| `sm_online_exam_students` | `assessment_online_exam_students` |
| `sm_online_exam_questions` (dup) | (drop; see academic) |
| `sm_question_banks` | `assessment_question_banks` |
| `sm_question_bank_groups` | `assessment_question_bank_groups` |
| `sm_question_bank_subjects` | `assessment_question_bank_subjects` |
| `sm_question_options` | `assessment_question_options` |
| `sm_question_levels` | `assessment_question_levels` |
| `sm_online_exam_marks` | `assessment_online_exam_marks` |
| `sm_seat_plans` | `assessment_seat_plans` |
| `sm_seat_plan_children` | `assessment_seat_plan_children` |
| `sm_admit_cards` | `assessment_admit_cards` |
| `sm_admit_card_settings` | `assessment_admit_card_settings` |
| `sm_result_settings` | `assessment_result_settings` |
| `sm_result_publication_settings` | `assessment_result_publication_settings` |
| `sm_report_cards` | `assessment_report_cards` |
| `sm_report_card_settings` | `assessment_report_card_settings` |
| `sm_report_card_signatures` | `assessment_report_card_signatures` |
| `sm_marks_grades` | `assessment_marks_grades` |
| `sm_marks_grade_scales` | `assessment_marks_grade_scales` |
| `sm_merit_list_settings` | `assessment_merit_list_settings` |
| `sm_merit_list_settings_extras` | `assessment_merit_list_settings_extras` |
| `sm_merit_list_registrations` | `assessment_merit_list_registrations` |
| `sm_merit_list_registration_fields` | `assessment_merit_list_registration_fields` |
| `sm_merit_list_registration_payments` | `assessment_merit_list_registration_payments` |
| `sm_exam_merit_list_registrations` | `assessment_exam_merit_list_registrations` |
| `sm_online_exam_assigns` | `assessment_online_exam_assigns` |
| `sm_online_exam_titles` | `assessment_online_exam_titles` |
| `front_exam_routines` | `assessment_front_exam_routines` |
| `front_results` | `assessment_front_results` |

### Attendance domain (7 legacy tables, 7 renames)

| Legacy | New name |
| --- | --- |
| `sm_attendances` | `attendance_attendances` |
| `sm_staff_attendences` | `attendance_staff_attendances` (fixes typo from `attendences`) |
| `sm_subject_attendances` | `attendance_subject_attendances` |
| `sm_exam_attendances` (dup) | (drop; see assessment) |
| `sm_attendance_settings` | `attendance_attendance_settings` |
| `sm_attendance_devices` | `attendance_attendance_devices` |
| `sm_holidays` (dup) | (drop; see events) |

### Communication domain (23 legacy tables, 23 renames)

| Legacy | New name |
| --- | --- |
| `sm_notice_boards` | `communication_notice_boards` |
| `sm_notice_categories` | `communication_notice_categories` |
| `sm_send_messages` | `communication_send_messages` |
| `sm_message_credential` | `communication_message_credential` |
| `sm_email_sms_logs` | `communication_email_sms_logs` |
| `sm_event_communications` | `communication_event_communications` |
| `sm_complaints` | `communication_complaints` |
| `sm_complaint_categories` | `communication_complaint_categories` |
| `sm_complaint_settings` | `communication_complaint_settings` |
| `sm_complaint_comments` | `communication_complaint_comments` |
| `sm_chat_groups` | `communication_chat_groups` |
| `sm_chat_group_users` | `communication_chat_group_users` |
| `sm_chat_group_messages` | `communication_chat_group_messages` |
| `sm_chat_group_message_files` | `communication_chat_group_message_files` |
| `sm_chat_invitations` | `communication_chat_invitations` |
| `sm_chat_blocked_users` | `communication_chat_blocked_users` |
| `sm_sms_gateways` | `communication_sms_gateways` |
| `sm_sms_templates` | `communication_sms_templates` |
| `sm_email_templates` | `communication_email_templates` |
| `sm_notification_settings` | `communication_notification_settings` |
| `sm_notification_settings_for_users` | `communication_notification_settings_for_users` |
| `sm_groups` | `communication_groups` |
| `sm_group_users` | `communication_group_users` |

### Documents domain (3 legacy tables, 3 renames)

| Legacy | New name |
| --- | --- |
| `sm_forms` | `documents_forms` |
| `sm_form_fields` | `documents_form_fields` |
| `sm_form_field_values` | `documents_form_field_values` |

### Events domain (7 legacy tables, 7 renames)

| Legacy | New name |
| --- | --- |
| `sm_events` | `events_calendar_events` |
| `sm_holidays` | `events_holidays` |
| `sm_weekends` | `events_weekends` |
| `sm_calendar_settings` | `events_calendar_settings` |
| `sm_event_types` | `events_event_types` |
| `sm_event_audiences` | `events_event_audiences` |
| `sm_event_invitations` | `events_event_invitations` |

### Facilities domain (15 legacy tables, 15 renames)

| Legacy | New name |
| --- | --- |
| `sm_vehicles` | `facilities_vehicles` |
| `sm_routes` | `facilities_routes` |
| `sm_route_stops` | `facilities_route_stops` |
| `sm_assign_vehicles` | `facilities_assign_vehicles` |
| `sm_dormitory_lists` | `facilities_dormitories` (drops `lists_` per engine convention) |
| `sm_room_types` | `facilities_room_types` |
| `sm_room_lists` | `facilities_rooms` |
| `sm_room_assignments` | `facilities_room_assignments` |
| `sm_items` | `facilities_items` |
| `sm_item_categories` | `facilities_item_categories` |
| `sm_item_stores` | `facilities_item_stores` |
| `sm_item_issues` | `facilities_item_issues` |
| `sm_item_receives` | `facilities_item_receives` |
| `sm_item_sells` | `facilities_item_sells` |
| `sm_suppliers` | `facilities_suppliers` |

### Finance domain (47 legacy tables, 47 renames)

| Legacy | New name |
| --- | --- |
| `fm_fees_groups` | `finance_fees_groups` |
| `fm_fees_types` | `finance_fees_types` |
| `fm_fees_masters` | `finance_fees_masters` |
| `fm_fees_masters_children` | `finance_fees_masters_children` |
| `fm_fees_assigns` | `finance_fees_assigns` |
| `fm_fees_assign_children` | `finance_fees_assign_children` |
| `fm_fees_transactions` | `finance_fees_transactions` |
| `fm_fees_transcation_children` | `finance_fees_transcation_children` (fixes typo from `transcation`) |
| `fm_fees_invoices` | `finance_invoices` |
| `fm_fees_invoice_chields` | `finance_invoice_children` (fixes typo from `chields`) |
| `fm_fees_invoice_payments` | `finance_invoice_payments` |
| `fm_fees_weavers` | `finance_fees_weavers` (kept; engine is unsure about this aggregate) |
| `fees_invoices` (Laravel) | `finance_invoice_settings` |
| `fees_payments` (Laravel) | `finance_payment_settings` |
| `fees_discounts` | `finance_fees_discounts` |
| `fees_discount_assigns` | `finance_fees_discount_assigns` |
| `fees_installments` | `finance_fees_installments` |
| `fees_installment_children` | `finance_fees_installment_children` |
| `fees_installment_assigns` | `finance_fees_installment_assigns` |
| `direct_fees_installments` | `finance_direct_fees_installments` |
| `direct_fees_installment_assigns` | `finance_direct_fees_installment_assigns` |
| `direct_fees_installment_child_payments` | `finance_direct_fees_installment_child_payments` |
| `due_fees_login_prevents` | `finance_due_fees_login_prevents` |
| `bank_accounts` | `finance_bank_accounts` |
| `bank_statements` | `finance_bank_statements` |
| `bank_statement_lines` | `finance_bank_statement_lines` |
| `expenses` | `finance_expenses` |
| `expense_categories` | `finance_expense_categories` |
| `expense_sectors` | `finance_expense_sectors` |
| `incomes` | `finance_incomes` |
| `income_categories` | `finance_income_categories` |
| `add_incomes` | `finance_add_incomes` |
| `add_expenses` | `finance_add_expenses` |
| `payrolls` | `finance_payrolls` |
| `payroll_earn_deducs` | `finance_payroll_earn_deducs` |
| `payroll_payments` | `finance_payroll_payments` |
| `wallets` | `finance_wallets` |
| `wallet_transactions` | `finance_wallet_transactions` |
| `wallet_transaction_reports` | `finance_wallet_transaction_reports` |
| `donors` | `finance_donors` |
| `donation_requests` | `finance_donation_requests` |
| `fees_reports` | `finance_fees_reports` |
| `transaction_heads` | `finance_transaction_heads` |
| `transactions` | `finance_transactions` |
| `accounts` | `finance_accounts` |
| `transfer_ledgers` | `finance_transfer_ledgers` |

### HR domain (14 legacy tables, 14 renames)

| Legacy | New name |
| --- | --- |
| `sm_human_departments` | `hr_departments` |
| `sm_designations` | `hr_designations` |
| `sm_staffs` | `hr_staffs` |
| `sm_staff_education` | `hr_staff_education` |
| `sm_staff_experience` | `hr_staff_experience` |
| `sm_staff_certifications` | `hr_staff_certifications` |
| `sm_staff_social_networks` | `hr_staff_social_networks` |
| `sm_staff_documents` | `hr_staff_documents` |
| `sm_leave_types` | `hr_leave_types` |
| `sm_leave_defines` | `hr_leave_defines` |
| `sm_leave_requests` | `hr_leave_requests` |
| `sm_leave_deduction_infos` | `hr_leave_deduction_infos` |
| `sm_hr_payroll_generates` | `hr_payroll_generates` |
| `sm_hr_payroll_earn_deducs` | `hr_payroll_earn_deducs` |

### Library domain (4 legacy tables, 4 renames)

| Legacy | New name |
| --- | --- |
| `sm_book_categories` | `library_book_categories` |
| `sm_books` | `library_books` |
| `sm_library_members` | `library_members` |
| `sm_book_issues` | `library_book_issues` |

### CMS domain (20 legacy tables, 20 renames)

| Legacy | New name |
| --- | --- |
| `sm_pages` | `cms_pages` |
| `infixedu__pages` | (drop; replaced by `cms_pages`) |
| `infixedu__settings` | (drop; legacy was a synonym for `cms_pages.settings`) |
| `sm_news` | `cms_news` |
| `sm_news_categories` | `cms_news_categories` |
| `sm_news_comments` | `cms_news_comments` |
| `sm_news_pages` | `cms_news_pages` |
| `sm_testimonials` | `cms_testimonials` |
| `sm_teacher_upload_contents` | `cms_teacher_upload_contents` |
| `sm_upload_contents` | `cms_upload_contents` |
| `sm_about_pages` | `cms_about_pages` |
| `sm_contact_pages` | `cms_contact_pages` |
| `sm_course_pages` | `cms_course_pages` |
| `sm_home_page_settings` | `cms_home_page_settings` |
| `sm_content_types` | `cms_content_types` |
| `sm_content_upload_categories` | `cms_content_upload_categories` |
| `front_academic_calendars` | `cms_front_academic_calendars` (kept as engine has a publishing concept) |
| `front_class_routines` | `cms_front_class_routines` |
| `check_classes` | `cms_class_publish_sentinel` (kept as engine's "is published" flag) |

### RBAC domain (10 legacy tables, 10 renames)

| Legacy | New name |
| --- | --- |
| `infix_roles` | `rbac_roles` |
| `infix_permission_assigns` | `rbac_permission_assigns` |
| `permissions` | `rbac_permissions` |
| `roles` | `rbac_role_prototypes` (Laravel Spatie; the engine's `rbac_roles` is canonical) |
| `sm_module_permissions` | `rbac_module_permissions` |
| `sm_module_permission_assigns` | `rbac_module_permission_assigns` |
| `sm_role_permissions` | `rbac_role_permissions` |
| `sidebars` | `rbac_sidebars` |
| `sidebar_childs` | `rbac_sidebar_children` |
| `assign_permissions` | (drop; replaced by `rbac_permission_assigns`) |

### Settings domain (14 legacy tables, 14 renames)

| Legacy | New name |
| --- | --- |
| `sm_general_settings` | `settings_general_settings` |
| `sm_background_settings` | `settings_background_settings` |
| `sm_base_setups` | `settings_base_setups` |
| `sm_custom_links` | `settings_custom_links` |
| `sm_dashboard_settings` | `settings_dashboard_settings` |
| `sm_date_formats` | `settings_date_formats` |
| `sm_languages` | `settings_languages` |
| `sm_language_phrases` | `settings_language_phrases` |
| `sm_setup_admins` | `settings_setup_admins` |
| `sm_styles` | `settings_styles` |
| `themes` | `settings_themes` |
| `theme_settings` | `settings_theme_settings` |
| `transcations` (typo) | (drop; legacy table was empty per `0009_finance.sql`) |
| `time_zones` | (drop; `platform_time_zones` is the engine's table) |

### Operations domain (15 legacy tables, 15 renames)

| Legacy | New name |
| --- | --- |
| `sm_backups` | `operations_backups` |
| `sm_system_versions` | `operations_system_versions` |
| `sm_user_logs` | `operations_user_logs` |
| `version_histories` | `operations_version_histories` |
| `todo` | `operations_todos` |
| `todos` | `operations_todo_list` |
| `notification_settings` (Laravel) | `operations_notification_settings` |
| `jobs` (Laravel) | (drop; Laravel meta) |
| `job_batches` (Laravel) | (drop; Laravel meta) |
| `failed_jobs` (Laravel) | (drop; Laravel meta) |
| `migrations` (Laravel) | `operations_migration_history` (rename; keep) |
| `mfa_codes` (Laravel) | `operations_mfa_codes` |
| `audit_logins` (Laravel) | `operations_audit_logins` |
| `lockouts` (Laravel) | `operations_lockouts` |
| `password_reset_tokens` (Laravel) | `operations_password_reset_tokens` (alternative naming; engine keeps `platform_password_resets`) |

### Consumer-side tables added by the engine (not in legacy)

These are created fresh; the engine provides the domain but not the
SaaS layer.

| New name | Purpose |
| --- | --- |
| `platform_tenants` | consumer SaaS workspace (1..* `platform_schools`) |
| `platform_subscriptions` | Stripe sync (consumer-side) |
| `platform_packages` | plan catalog (consumer-side) |
| `platform_regions` | multi-region routing (consumer-side) |
| `platform_invitations` | onboarding (consumer-side) |
| `platform_pricing_tiers` | billing tiers (consumer-side) |
| `platform_invoices` | consumer's billing engine output (consumer-side) |

## Aggregate count

| Status | Count |
| --- | --- |
| rename | ~290 |
| archive | ~7 (`front_*`, `un_*`, `check_classes`, `infixedu__*`) |
| drop | ~10 (Laravel meta) |
| keep | 6 (engine cross-cutting) |
| add (consumer-side) | 7 |
| **total** | **320** |

The slight overflow above 310 is because the consumer-side tables
are net additions. The legacy 310 are mapped 1:1 (or merged /
dropped per the rationale above).

## The renumbering

The 15 legacy `migrations/00XX_<domain>.sql` files are split into 15
new engine files plus the `0000_engine_core.sql`:

```text
migrations/
├── 0000_engine_core.sql                       <-- engine cross-cutting (Phase 1)
├── 0001_engine_academic.sql
├── 0002_engine_assessment.sql
├── 0003_engine_attendance.sql
├── 0004_engine_communication.sql
├── 0005_engine_documents.sql
├── 0006_engine_events.sql
├── 0007_engine_facilities.sql
├── 0008_engine_finance.sql
├── 0009_engine_hr.sql
├── 0010_engine_library.sql
├── 0011_engine_cms.sql
├── 0012_engine_rbac.sql
├── 0013_engine_settings.sql
├── 0014_engine_operations.sql
├── 0015_engine_platform.sql
└── 0016_engine_consumer_saas.sql             <-- consumer-side tables
```

The legacy `migrations/0001_academic.sql` through
`migrations/0015_settings.sql` are kept for analysis but are NOT
applied to `devdb_v2`. The ETL script reads from them as a reference
for the data flow but writes to the new files.

## Exit criteria

- All 290 renames applied to `devdb_v2`.
- 7 archives in `legacy_<name>` tables in `devdb_v2`.
- 10 drops executed (Laravel meta).
- 6 engine cross-cutting tables present (Phase 1).
- 7 consumer-side tables created.
- All FKs reissued against the new table names.
- The legacy `devdb` is unchanged (read-only during the migration).
- A spot check of 5 random renames per domain shows the new table
  has the expected row count and a sample row matches the legacy
  data modulo the type-driven normalisation.
