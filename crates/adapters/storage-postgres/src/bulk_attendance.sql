-- attendance_student_attendances — bulk-marking target table.
-- The wire form is decoupled from the existing six engine
-- cross-cutting tables; per the spec, UUIDs are stored as
-- BYTEA (16 bytes big-endian), dates and timestamps as
-- TEXT (ISO 8601), and counters as INTEGER.
--
-- The PRIMARY KEY is (school_id, id) so the row's primary
-- key is unique within a school. The UNIQUE constraint on
-- (school_id, student_id, attendance_date) is the
-- per-student-per-day uniqueness invariant; a duplicate
-- insert surfaces as DomainError::Conflict.

CREATE TABLE IF NOT EXISTS attendance_student_attendances (
    school_id            BYTEA      NOT NULL,
    id                   BYTEA      NOT NULL,
    student_id           BYTEA      NOT NULL,
    student_record_id    BYTEA      NOT NULL,
    class_id             BYTEA      NOT NULL,
    section_id           BYTEA      NOT NULL,
    attendance_date      TEXT       NOT NULL,
    attendance_type      TEXT       NOT NULL,
    in_time              TEXT           NULL,
    out_time             TEXT           NULL,
    notes                TEXT           NULL,
    is_absent            INTEGER    NOT NULL DEFAULT 0,
    marked_by            BYTEA      NOT NULL,
    marked_at            TEXT       NOT NULL,
    marked_from          TEXT       NOT NULL,
    version              INTEGER    NOT NULL DEFAULT 1,
    etag                 TEXT       NOT NULL,
    created_at           TEXT       NOT NULL,
    updated_at           TEXT       NOT NULL,
    created_by           BYTEA      NOT NULL,
    updated_by           BYTEA      NOT NULL,
    active_status        INTEGER    NOT NULL DEFAULT 1,
    last_event_id        BYTEA          NULL,
    correlation_id       BYTEA      NOT NULL,
    PRIMARY KEY (school_id, id),
    UNIQUE (school_id, student_id, attendance_date)
);
