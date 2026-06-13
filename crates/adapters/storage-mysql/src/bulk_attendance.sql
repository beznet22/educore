-- attendance_student_attendances — bulk-marking target table.
-- The wire form is decoupled from the existing six engine
-- cross-cutting tables; per the spec, UUIDs are stored as
-- VARBINARY(255) (16 bytes big-endian, with a 255-byte
-- upper bound that MySQL 8+ uses for storage sizing), dates
-- and timestamps as TEXT (ISO 8601), and counters as INT.
--
-- The PRIMARY KEY is (school_id, id) so the row's primary
-- key is unique within a school. The UNIQUE KEY on
-- (school_id, student_id, attendance_date) is the
-- per-student-per-day uniqueness invariant; a duplicate
-- insert surfaces as DomainError::Conflict.

CREATE TABLE IF NOT EXISTS `attendance_student_attendances` (
    `school_id`         VARBINARY(255) NOT NULL,
    `id`                VARBINARY(255) NOT NULL,
    `student_id`        VARBINARY(255) NOT NULL,
    `student_record_id` VARBINARY(255) NOT NULL,
    `class_id`          VARBINARY(255) NOT NULL,
    `section_id`        VARBINARY(255) NOT NULL,
    `attendance_date`   TEXT           NOT NULL,
    `attendance_type`   TEXT           NOT NULL,
    `in_time`           TEXT               NULL,
    `out_time`          TEXT               NULL,
    `notes`             TEXT               NULL,
    `is_absent`         INT            NOT NULL DEFAULT 0,
    `marked_by`         VARBINARY(255) NOT NULL,
    `marked_at`         TEXT           NOT NULL,
    `marked_from`       TEXT           NOT NULL,
    `version`           INT            NOT NULL DEFAULT 1,
    `etag`              TEXT           NOT NULL,
    `created_at`        TEXT           NOT NULL,
    `updated_at`        TEXT           NOT NULL,
    `created_by`        VARBINARY(255) NOT NULL,
    `updated_by`        VARBINARY(255) NOT NULL,
    `active_status`     INT            NOT NULL DEFAULT 1,
    `last_event_id`     VARBINARY(255)     NULL,
    `correlation_id`    VARBINARY(255) NOT NULL,
    PRIMARY KEY (`school_id`, `id`),
    UNIQUE KEY `ux_attendance_student_attendances_school_student_date` (
        `school_id`, `student_id`, `attendance_date`
    )
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
