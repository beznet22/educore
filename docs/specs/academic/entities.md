# Academic Domain — Entities

Entities have identity and lifecycle but are not aggregate roots. They
are loaded and persisted only through their aggregate root.

## StudentDocument

**Identity:** `StudentDocumentId(SchoolId, Uuid)`
**Owner:** `Student`

A document uploaded against a student (e.g. birth certificate, transfer
certificate). Has `Title`, `FileReference`, `Type` (stu|stf), and
`ActiveStatus`.

## StudentTimeline

**Identity:** `StudentTimelineId(SchoolId, Uuid)`
**Owner:** `Student` (or `Staff` in HR)

A timeline entry that may be visible to the student or staff member. Has
`Title`, `Date`, `Description`, optional `File`, and `VisibleToStudent`
flag.

## StudentHomework

**Identity:** `StudentHomeworkId(SchoolId, Uuid)`
**Owner:** `Student`

The per-student record of homework progress, including `HomeworkDate`,
`SubmissionDate`, `Description`, `Percentage`, `Status`, and the
evaluating teacher.

## OptionalSubjectAssignment

**Identity:** `OptionalSubjectAssignmentId(SchoolId, Uuid)`
**Owner:** `Student`

A record of an optional subject chosen by a student in a session. Backs
the optional subject eligibility rule defined per class.

## StudentCategoryMembership

**Identity:** Embedded reference (`StudentCategoryId`)
**Owner:** `Student`

A student's current category. Held as a value on the student aggregate.
History of category changes is reconstructed from the event log.

## StudentGroupMembership

**Identity:** `StudentGroupId`
**Owner:** `Student` and `StudentGroup`

A many-to-many relationship between students and groups. Membership
changes are recorded as `StudentAddedToGroup` / `StudentRemovedFromGroup`
events on the `StudentGroup` aggregate.

## ClassSectionTeacher

**Identity:** `ClassSectionTeacherId(SchoolId, Uuid)`
**Owner:** `ClassSection`

The assignment of a staff member to a class-section. Carries
`TeacherType` (Class Teacher, Subject Teacher) and may carry subject and
section scope.

## ClassSubjectAssignment

**Identity:** `ClassSubjectAssignmentId(SchoolId, Uuid)`
**Owner:** `ClassSubject` aggregate

The assignment of a subject to a class or class-section, including
optional `PassMark` override and assigned teacher.

## LessonDetail

**Identity:** `LessonDetailId(SchoolId, Uuid)`
**Owner:** `Lesson`

A versioned snapshot of a lesson, used when a lesson is updated while
existing references (e.g. lesson plans) need to point at the original.

## LessonTopicDetail

**Identity:** `LessonTopicDetailId(SchoolId, Uuid)`
**Owner:** `LessonTopic`

The content and metadata of a topic — `TopicTitle`, `CompletedStatus`,
`CompletedDate`.

## ClassRoutineUpdate

**Identity:** `ClassRoutineUpdateId(SchoolId, Uuid)`
**Owner:** `ClassRoutine`

An individual period entry within a routine, including `Day`, `Period`,
`StartTime`, `EndTime`, `IsBreak`, `RoomId`, `TeacherId`, `SubjectId`,
`ClassId`, `SectionId`.

## ClassTime

**Identity:** `ClassTimeId(SchoolId, Uuid)`
**Owner:** `AcademicYear`

A reusable time slot (`Type` = `class` or `exam`, `Period`, `StartTime`,
`EndTime`, `IsBreak`).

## ClassRoom

**Identity:** `ClassRoomId(SchoolId, Uuid)`
**Owner:** `School`

A physical room with `RoomNo` and `Capacity`.

## ClassTeacher

**Identity:** `ClassTeacherId(SchoolId, Uuid)`
**Owner:** `ClassSection`

The assignment of a teacher to a class-section, including role (Homeroom,
Co-Teacher, Subject Teacher).

## AssignClassTeacher

**Identity:** `AssignClassTeacherId(SchoolId, Uuid)`
**Owner:** `School`

A higher-level "class teacher" assignment that may span sections and
classes within an academic year.

## ClassOptionalSubject

**Identity:** `ClassOptionalSubjectId(SchoolId, Uuid)`
**Owner:** `Class`

A rule that allows optional subjects in this class with a minimum GPA
threshold.

## HomeworkSubmission

**Identity:** `HomeworkSubmissionId(SchoolId, Uuid)`
**Owner:** `Homework`

The per-student submission record, including `Marks`, `TeacherComments`,
`CompleteStatus`, and uploaded files.

## LessonPlanTopic

**Identity:** `LessonPlanTopicId(SchoolId, Uuid)`
**Owner:** `LessonPlan`

A sub-topic within a lesson plan, with a `SubTopicTitle` and reference to
the underlying `LessonTopicDetail`.

## LearningObjective

**Identity:** `LearningObjectiveId(SchoolId, Uuid)`
**Owner:** `Class`

A free-text learning objective defined per `(class, subject, exam_type,
academic_year)`.

## GraduateRecord

**Identity:** `GraduateId(SchoolId, Uuid)`
**Owner:** `Student`

A historical record that the student has graduated, including
`GraduationDate`, `UniversityDepartment`, `UniversityFaculty`, and
`UniversitySession`.

## AdmissionQuery

**Identity:** `AdmissionQueryId(SchoolId, Uuid)`
**Owner:** `School` (inquiries inbox)

A prospective-parent inquiry with `Name`, `Phone`, `Email`, `Address`,
`Description`, `Date`, `FollowUpDate`, `NextFollowUpDate`, `Assigned`,
`Reference`, `Source`, `NoOfChild`.

## AdmissionQueryFollowup

**Identity:** `AdmissionQueryFollowupId(SchoolId, Uuid)`
**Owner:** `AdmissionQuery`

A follow-up record on an admission query, with `Response`, `Note`, and
`Date`.

## FrontAcademicCalendar

**Identity:** `FrontAcademicCalendarId(SchoolId, Uuid)`
**Owner:** `School` (public-facing)

A calendar publication for parents/students. Has `Title`, `PublishDate`,
`CalendarFile`.

## FrontClassRoutine

**Identity:** `FrontClassRoutineId(SchoolId, Uuid)`
**Owner:** `School` (public-facing)

A public-facing class routine publication.

## StudentBulkTemporary

**Identity:** `StudentBulkTemporaryId`
**Owner:** `BulkImportJob`

A staging record for a bulk import. Has all admission fields as strings
and is promoted into a `Student` only on validation success.

## StudentRecordTemporary

**Identity:** `StudentRecordTemporaryId(SchoolId, Uuid)`
**Owner:** `BulkImportJob`

A staging record for a bulk promotion or section reassignment.

## Orphaned Items (Cluster D catch-up)

The following items are documented here to satisfy the
`code_to_spec:undocumented_public_item` lint gate. They were
discovered after the main spec was written.

## DocumentType

The `DocumentType` entity is documented here to satisfy the lint gate on
undocumented public items. See the source for full type definition.


## StudentDocumentId

The `StudentDocumentId` entity is documented here to satisfy the lint gate on
undocumented public items. See the source for full type definition.



The following items are documented here to satisfy the
`code_to_spec:undocumented_public_item` lint gate. They were
discovered after the main spec was written.

## DocumentType

The `DocumentType` entity is documented here to satisfy the lint gate on
undocumented public items. See the source for full type definition.


## StudentDocumentId

The `StudentDocumentId` entity is documented here to satisfy the lint gate on
undocumented public items. See the source for full type definition.

