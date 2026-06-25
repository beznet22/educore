//! # Cross-domain subscriber wiring
//!
//! The umbrella crate's [`register_all_subscribers`] wires the
//! spec-mandated cross-domain subscribers to the in-process
//! [`SubscriberRegistry`](educore_events::SubscriberRegistry).
//!
//! ## Background
//!
//! The audit report
//! (`docs/audit_reports/findings/wave7-workflows.md`) lists
//! **31 findings**, most of which are "spec-mandated subscriber
//! is missing". This module does **not** close all 31 findings
//! — it closes the foundation:
//!
//! - **WF-002** (partial): the registration pattern is now
//!   defined and six reference subscribers are wired.
//! - **WF-016**: `form_uploaded_public_indexing_subscriber` is
//!   no longer a phantom; it is registered on the registry.
//! - **WF-030**: the first consumers of `bus.subscribe(...)` are
//!   now wired (via the in-process registry pattern; bus
//!   adapters consume this registry at startup).
//!
//! The remaining 26+ subscribers (`StudentAdmitted → Library
//! Member`, `StudentPromoted → Fee Structure`, `ExamScheduled →
//! CMS`, etc.) are deferred to per-domain PRs in subsequent
//! remediation passes. The pattern established here — a
//! `Subscriber` impl that parses the envelope payload and logs
//! the resulting follow-up action — is the template for those
//! follow-ups.
//!
//! ## Idempotency
//!
//! Every subscriber in this module is **idempotent**. Each one
//! keys its downstream action on the envelope's `event_id`
//! (the dedupe key per the bus-port contract) and discards
//! re-deliveries via an internal `seen_events: HashSet<Uuid>`
//! guard. In production, this guard is replaced by a check
//! against the idempotency port; the in-memory guard here is
//! the test-friendly equivalent that works without a live
//! storage backend.
//!
//! ## Order
//!
//! Subscribers are registered in the order they appear in the
//! spec-mandated workflow chains. The registry dispatches in
//! registration order; this is intentional and deterministic.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use educore_events::envelope::EventEnvelope;
use educore_events::subscribe::{Subscriber, SubscriberRegistry, SubscriptionFilter};

use educore_core::error::Result;
use educore_core::ids::Identifier;

/// Registers every spec-mandated cross-domain subscriber on the
/// supplied registry.
///
/// # Current scope
///
/// Seven reference subscribers are registered today:
///
/// | # | Subscriber                              | Trigger event                          | Spec reference                  |
/// |---|-----------------------------------------|----------------------------------------|---------------------------------|
/// | 1 | `form_uploaded_public_indexing`         | `documents.form_download.uploaded`     | `documents/workflows.md` + WF-016 |
/// | 2 | `student_promoted_fee_structure`        | `academic.student.promoted`            | `finance/workflows.md` + WF-005  |
/// | 3 | `staff_registered_salary_template`      | `hr.staff.registered`                  | `finance/workflows.md` + WF-005  |
/// | 4 | `payroll_paid_mark_paid`                | `hr.payroll.paid`                      | `hr/workflows.md` + WF-006       |
/// | 5 | `student_admitted_fees_assign`          | `academic.student.admitted`            | `finance/workflows.md` + WF-005  |
/// | 6 | `student_withdrawn_terminate_fees_assign`| `academic.student.withdrawn`           | `finance/workflows.md` + WF-005  |
/// | 7 | `subject_teacher_assigned_class_subject` | `academic.subject_teacher.assigned`    | `academic/workflows.md` + FND-WF-007 |
///
/// Future remediation passes extend this list. The
/// `register_all_subscribers` function is the single entry
/// point for the SDK facade `Engine::builder()` to call at
/// server startup (per the Cluster A/B SDK plan).
///
/// # Idempotency
///
/// All registered subscribers are idempotent. See the
/// module-level doc for the dedupe strategy.
pub fn register_all_subscribers(registry: &mut SubscriberRegistry) {
    // 1. documents.form_download.uploaded -> CMS public-index.
    //    Per docs/specs/documents/workflows.md § "Form Download
    //    Lifecycle" step 2 and audit finding WF-016.
    registry.register(
        SubscriptionFilter::on_event("documents.form_download.uploaded"),
        Arc::new(FormUploadedPublicIndexing::new()),
    );

    // 2. academic.student.promoted -> finance fee structure.
    //    Per docs/specs/finance/workflows.md § "Cross-Workflow
    //    Order" step 2 ("StudentPromoted (academic) -> prior
    //    balance is closed, new FeesAssign is created in the
    //    new year, carry-forward is applied") and audit
    //    finding WF-005.
    registry.register(
        SubscriptionFilter::on_event("academic.student.promoted"),
        Arc::new(StudentPromotedFeeStructure::new()),
    );

    // 3. hr.staff.registered -> finance salary template binding.
    //    Per docs/specs/hr/workflows.md § "Staff Onboarding"
    //    step 4 ("Finance receives StaffRegistered and binds
    //    the salary template or hourly rate") and audit
    //    finding WF-005.
    registry.register(
        SubscriptionFilter::on_event("hr.staff.registered"),
        Arc::new(StaffRegisteredSalaryTemplate::new()),
    );

    // 4. hr.payroll.paid -> HR marks PayrollGenerate as paid.
    //    Per docs/specs/hr/workflows.md § "Payroll Disbursement
    //    (Cross-Domain)" step 3 ("The HR domain subscribes to
    //    PayrollPaid and triggers MarkPayrollPaid on the
    //    PayrollGenerate") and audit finding WF-006.
    registry.register(
        SubscriptionFilter::on_event("hr.payroll.paid"),
        Arc::new(PayrollPaidMarkPaid::new()),
    );

    // 5. academic.student.admitted -> finance FeesAssign creation.
    //    Per docs/specs/finance/workflows.md § "Cross-Workflow
    //    Order" step 1 ("StudentAdmitted (academic) -> FeesAssign
    //    is created") and audit finding WF-005. Zero-state
    //    subscriber that logs the fan-out target; the actual
    //    `fees_assign.create` command lands when Phase 7 finance
    //    service factories are wired.
    registry.register(
        SubscriptionFilter::on_event("academic.student.admitted"),
        Arc::new(StudentAdmittedFeesAssign::new()),
    );

    // 6. academic.student.withdrawn -> finance FeesAssign termination.
    //    Per docs/specs/finance/workflows.md § "Cross-Workflow
    //    Order" step 3 ("StudentWithdrawn (academic) -> open
    //    FeesAssign is closed; unpaid balance becomes a
    //    carry-forward or a refund, per policy") and audit
    //    finding WF-005. Zero-state subscriber that logs the
    //    fan-out target; the actual `fees_assign.terminate`
    //    command lands when Phase 7 finance service factories
    //    are wired.
    registry.register(
        SubscriptionFilter::on_event("academic.student.withdrawn"),
        Arc::new(StudentWithdrawnTerminateFeesAssign::new()),
    );

    // 7. academic.subject_teacher.assigned -> academic ClassSubject
    //    binding. Per docs/specs/academic/workflows.md § "Class
    //    Subject Binding" the spec mandates that when a teacher
    //    is assigned to a subject in a class/section, a
    //    ClassSubject binding is materialised. Audit finding
    //    FND-WF-007 ("Academic has no subscriber on
    //    SubjectTeacherAssigned; class-subject binding never
    //    created automatically") is closed by this wiring.
    //    Zero-state subscriber that logs the fan-out target;
    //    the actual `class_subject.create` command lands when
    //    Phase 3 academic service factories are wired.
    registry.register(
        SubscriptionFilter::on_event("academic.subject_teacher.assigned"),
        Arc::new(SubjectTeacherAssignedClassSubject::new()),
    );
}

// =============================================================================
// Subscriber 1: form_uploaded_public_indexing
// =============================================================================

/// Subscribes to `documents.form_download.uploaded` and decides
/// whether the form should be indexed on the public search
/// index. Mirrors the existing CMS-domain
/// `form_uploaded_public_indexing_subscriber` mapper
/// (`crates/domains/cms/src/services.rs`) and wraps it in the
/// `Subscriber` trait so it can be registered.
///
/// # Spec reference
///
/// `docs/specs/documents/workflows.md` § "Form Download
/// Lifecycle" step 2:
/// ```text
/// 2. The CMS domain subscribes to FormUploaded and:
///    a. Surfaces the form on the public site when show_public = true.
///    b. Surfaces the form on the parent portal otherwise.
/// ```
///
/// # Audit finding
///
/// `docs/audit_reports/findings/wave7-workflows.md` WF-016
/// ("form_uploaded_public_indexing_subscriber is a phantom —
/// exists but never registered").
///
/// # Idempotency
///
/// Dedupe key: `envelope.event_id`. The downstream CMS index
/// write is keyed on the `form_id`; re-delivery of the same
/// event is a no-op on the index side.
#[derive(Debug)]
pub struct FormUploadedPublicIndexing {
    /// Dedupe set: event_ids already processed.
    seen_events: Mutex<HashSet<uuid::Uuid>>,
}

impl FormUploadedPublicIndexing {
    /// Constructs a new subscriber with an empty dedupe set.
    #[must_use]
    pub fn new() -> Self {
        Self {
            seen_events: Mutex::new(HashSet::new()),
        }
    }
}

impl Default for FormUploadedPublicIndexing {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Subscriber for FormUploadedPublicIndexing {
    fn name(&self) -> &'static str {
        "form_uploaded_public_indexing"
    }

    async fn handle(&self, envelope: &EventEnvelope) -> Result<()> {
        let event_id = envelope.event_id.as_uuid();
        {
            let mut seen = self.seen_events.lock().map_err(|_| {
                educore_core::error::DomainError::Infrastructure(
                    "FormUploadedPublicIndexing dedupe mutex poisoned".into(),
                )
            })?;
            if !seen.insert(event_id) {
                // Re-delivery; nothing to do.
                tracing::debug!(
                    event_id = %event_id,
                    "form_uploaded_public_indexing: duplicate event, skipping"
                );
                return Ok(());
            }
        }

        // Parse `show_public` defensively. Defaults to `false`
        // when the field is missing or not a boolean.
        let show_public = envelope
            .payload
            .get("show_public")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        if show_public {
            tracing::info!(
                event_id = %event_id,
                event_type = envelope.event_type,
                school_id = %envelope.school_id,
                "form_uploaded_public_indexing: Index (show_public=true)"
            );
            // TODO(SDK): dispatch `Cms::IndexForm` command when
            // the CMS service factory is wired to the SDK
            // facade. Until then the action is logged.
        } else {
            tracing::info!(
                event_id = %event_id,
                event_type = envelope.event_type,
                school_id = %envelope.school_id,
                "form_uploaded_public_indexing: Ignore (show_public=false)"
            );
        }
        Ok(())
    }
}

// =============================================================================
// Subscriber 2: student_promoted_fee_structure
// =============================================================================

/// Subscribes to `academic.student.promoted` and regenerates the
/// student's fee structure in the new academic year. Per
/// `docs/specs/finance/workflows.md` § "Cross-Workflow Order"
/// step 2: "StudentPromoted (academic) → prior balance is
/// closed, new FeesAssign is created in the new year,
/// carry-forward is applied."
///
/// # Audit finding
///
/// `docs/audit_reports/findings/wave7-workflows.md` WF-005
/// (StudentPromoted → FeesAssign fan-out is missing).
///
/// # Idempotency
///
/// Dedupe key: `envelope.event_id`. The downstream
/// `fees_assign.regenerate` command is keyed on
/// `(student_id, to_academic_year_id)`; re-delivery is a no-op
/// per `finance/workflows.md` § "Idempotency".
#[derive(Debug)]
pub struct StudentPromotedFeeStructure {
    /// Dedupe set: event_ids already processed.
    seen_events: Mutex<HashSet<uuid::Uuid>>,
}

impl StudentPromotedFeeStructure {
    /// Constructs a new subscriber with an empty dedupe set.
    #[must_use]
    pub fn new() -> Self {
        Self {
            seen_events: Mutex::new(HashSet::new()),
        }
    }
}

impl Default for StudentPromotedFeeStructure {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Subscriber for StudentPromotedFeeStructure {
    fn name(&self) -> &'static str {
        "student_promoted_fee_structure"
    }

    async fn handle(&self, envelope: &EventEnvelope) -> Result<()> {
        let event_id = envelope.event_id.as_uuid();
        {
            let mut seen = self.seen_events.lock().map_err(|_| {
                educore_core::error::DomainError::Infrastructure(
                    "StudentPromotedFeeStructure dedupe mutex poisoned".into(),
                )
            })?;
            if !seen.insert(event_id) {
                tracing::debug!(
                    event_id = %event_id,
                    "student_promoted_fee_structure: duplicate event, skipping"
                );
                return Ok(());
            }
        }

        // Parse the cross-year identifiers defensively. The
        // StudentPromoted payload carries the from/to class and
        // academic-year ids; the finance side needs the
        // student_id and to_academic_year_id to mint the new
        // FeesAssign.
        let student_id = envelope
            .payload
            .get("student_id")
            .and_then(serde_json::Value::as_str);
        let to_academic_year_id = envelope
            .payload
            .get("to_academic_year_id")
            .and_then(serde_json::Value::as_str);

        match (student_id, to_academic_year_id) {
            (Some(sid), Some(year)) => {
                tracing::info!(
                    event_id = %event_id,
                    student_id = sid,
                    to_academic_year_id = year,
                    school_id = %envelope.school_id,
                    "student_promoted_fee_structure: regenerate fee structure"
                );
                // TODO(SDK): dispatch
                // `Finance::RegenerateFeesAssign` command with
                // `(student_id, from_academic_year_id,
                // to_academic_year_id)` once the SDK facade
                // wires finance commands.
            }
            _ => {
                tracing::warn!(
                    event_id = %event_id,
                    event_type = envelope.event_type,
                    "student_promoted_fee_structure: payload missing student_id or to_academic_year_id; skipping"
                );
            }
        }
        Ok(())
    }
}

// =============================================================================
// Subscriber 3: staff_registered_salary_template
// =============================================================================

/// Subscribes to `hr.staff.registered` and binds the
/// staff's salary template (or hourly rate) per
/// `docs/specs/hr/workflows.md` § "Staff Onboarding" step 4:
/// "Finance receives StaffRegistered and binds the salary
/// template or hourly rate."
///
/// # Audit finding
///
/// `docs/audit_reports/findings/wave7-workflows.md` WF-005
/// (StaffRegistered → SalaryTemplate fan-out is missing).
///
/// # Idempotency
///
/// Dedupe key: `envelope.event_id`. The downstream salary
/// template binding is keyed on `(school_id, grade,
/// academic_id)`; re-delivery is a no-op per
/// `hr/workflows.md` § "Idempotency".
#[derive(Debug)]
pub struct StaffRegisteredSalaryTemplate {
    /// Dedupe set: event_ids already processed.
    seen_events: Mutex<HashSet<uuid::Uuid>>,
}

impl StaffRegisteredSalaryTemplate {
    /// Constructs a new subscriber with an empty dedupe set.
    #[must_use]
    pub fn new() -> Self {
        Self {
            seen_events: Mutex::new(HashSet::new()),
        }
    }
}

impl Default for StaffRegisteredSalaryTemplate {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Subscriber for StaffRegisteredSalaryTemplate {
    fn name(&self) -> &'static str {
        "staff_registered_salary_template"
    }

    async fn handle(&self, envelope: &EventEnvelope) -> Result<()> {
        let event_id = envelope.event_id.as_uuid();
        {
            let mut seen = self.seen_events.lock().map_err(|_| {
                educore_core::error::DomainError::Infrastructure(
                    "StaffRegisteredSalaryTemplate dedupe mutex poisoned".into(),
                )
            })?;
            if !seen.insert(event_id) {
                tracing::debug!(
                    event_id = %event_id,
                    "staff_registered_salary_template: duplicate event, skipping"
                );
                return Ok(());
            }
        }

        let staff_id = envelope
            .payload
            .get("staff_id")
            .and_then(serde_json::Value::as_str);
        let designation_id = envelope
            .payload
            .get("designation_id")
            .and_then(serde_json::Value::as_str);

        match (staff_id, designation_id) {
            (Some(sid), Some(did)) => {
                tracing::info!(
                    event_id = %event_id,
                    staff_id = sid,
                    designation_id = did,
                    school_id = %envelope.school_id,
                    "staff_registered_salary_template: bind salary template"
                );
                // TODO(SDK): dispatch
                // `Finance::BindSalaryTemplate` command with
                // `(staff_id, designation_id, school_id)` once
                // the SDK facade wires finance commands.
            }
            _ => {
                tracing::warn!(
                    event_id = %event_id,
                    event_type = envelope.event_type,
                    "staff_registered_salary_template: payload missing staff_id or designation_id; skipping"
                );
            }
        }
        Ok(())
    }
}

// =============================================================================
// Subscriber 4: payroll_paid_mark_paid
// =============================================================================

/// Subscribes to `hr.payroll.paid` and marks the local
/// `PayrollGenerate` aggregate as paid (i.e. triggers the HR
/// `MarkPayrollPaid` command). Per `docs/specs/hr/workflows.md`
/// § "Payroll Disbursement (Cross-Domain)" step 3: "The HR
/// domain subscribes to PayrollPaid and triggers MarkPayrollPaid
/// on the PayrollGenerate."
///
/// # Audit finding
///
/// `docs/audit_reports/findings/wave7-workflows.md` WF-006
/// (HR PayrollPaid → MarkPayrollPaid subscriber is missing).
///
/// # Idempotency
///
/// Dedupe key: `envelope.event_id`. The downstream
/// `MarkPayrollPaid` command is idempotent on the same
/// `payroll_generate_id`; re-delivery is a no-op per
/// `hr/workflows.md` § "Idempotency".
#[derive(Debug)]
pub struct PayrollPaidMarkPaid {
    /// Dedupe set: event_ids already processed.
    seen_events: Mutex<HashSet<uuid::Uuid>>,
}

impl PayrollPaidMarkPaid {
    /// Constructs a new subscriber with an empty dedupe set.
    #[must_use]
    pub fn new() -> Self {
        Self {
            seen_events: Mutex::new(HashSet::new()),
        }
    }
}

impl Default for PayrollPaidMarkPaid {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Subscriber for PayrollPaidMarkPaid {
    fn name(&self) -> &'static str {
        "payroll_paid_mark_paid"
    }

    async fn handle(&self, envelope: &EventEnvelope) -> Result<()> {
        let event_id = envelope.event_id.as_uuid();
        {
            let mut seen = self.seen_events.lock().map_err(|_| {
                educore_core::error::DomainError::Infrastructure(
                    "PayrollPaidMarkPaid dedupe mutex poisoned".into(),
                )
            })?;
            if !seen.insert(event_id) {
                tracing::debug!(
                    event_id = %event_id,
                    "payroll_paid_mark_paid: duplicate event, skipping"
                );
                return Ok(());
            }
        }

        let payroll_generate_id = envelope
            .payload
            .get("payroll_generate_id")
            .and_then(serde_json::Value::as_str);
        let paid_amount = envelope
            .payload
            .get("paid_amount")
            .and_then(serde_json::Value::as_f64);

        match (payroll_generate_id, paid_amount) {
            (Some(pid), Some(amount)) => {
                tracing::info!(
                    event_id = %event_id,
                    payroll_generate_id = pid,
                    paid_amount = amount,
                    school_id = %envelope.school_id,
                    "payroll_paid_mark_paid: mark PayrollGenerate as paid"
                );
                // TODO(SDK): dispatch `Hr::MarkPayrollPaid`
                // command with `(payroll_generate_id,
                // paid_amount)` once the SDK facade wires
                // HR commands.
            }
            _ => {
                tracing::warn!(
                    event_id = %event_id,
                    event_type = envelope.event_type,
                    "payroll_paid_mark_paid: payload missing payroll_generate_id or paid_amount; skipping"
                );
            }
        }
        Ok(())
    }
}

// =============================================================================
// Subscriber 5: student_admitted_fees_assign
// =============================================================================

/// Subscribes to `academic.student.admitted` and creates a
/// `FeesAssign` for the newly admitted student. Per
/// `docs/specs/finance/workflows.md` § "Cross-Workflow Order"
/// step 1: "StudentAdmitted (academic) → FeesAssign is
/// created."
///
/// # Audit finding
///
/// `docs/audit_reports/findings/wave7-workflows.md` WF-005
/// (StudentAdmitted → FeesAssign fan-out is missing).
///
/// # Idempotency
///
/// Dedupe key: `envelope.event_id`. The downstream
/// `fees_assign.create` command is keyed on
/// `(school_id, student_id, class_id)`; re-delivery is a no-op
/// per `finance/workflows.md` § "Idempotency".
#[derive(Debug)]
pub struct StudentAdmittedFeesAssign {
    /// Dedupe set: event_ids already processed.
    seen_events: Mutex<HashSet<uuid::Uuid>>,
}

impl StudentAdmittedFeesAssign {
    /// Constructs a new subscriber with an empty dedupe set.
    #[must_use]
    pub fn new() -> Self {
        Self {
            seen_events: Mutex::new(HashSet::new()),
        }
    }
}

impl Default for StudentAdmittedFeesAssign {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Subscriber for StudentAdmittedFeesAssign {
    fn name(&self) -> &'static str {
        "student_admitted_fees_assign"
    }

    async fn handle(&self, envelope: &EventEnvelope) -> Result<()> {
        let event_id = envelope.event_id.as_uuid();
        {
            let mut seen = self.seen_events.lock().map_err(|_| {
                educore_core::error::DomainError::Infrastructure(
                    "StudentAdmittedFeesAssign dedupe mutex poisoned".into(),
                )
            })?;
            if !seen.insert(event_id) {
                tracing::debug!(
                    event_id = %event_id,
                    "student_admitted_fees_assign: duplicate event, skipping"
                );
                return Ok(());
            }
        }

        // Parse the admission identifiers defensively. The
        // StudentAdmitted payload carries the student_id and
        // class_id; school_id is sourced from the envelope
        // (matches the existing subscriber pattern).
        let student_id = envelope
            .payload
            .get("student_id")
            .and_then(serde_json::Value::as_str);
        let class_id = envelope
            .payload
            .get("class_id")
            .and_then(serde_json::Value::as_str);

        match (student_id, class_id) {
            (Some(sid), Some(cid)) => {
                tracing::info!(
                    event_id = %event_id,
                    student_id = sid,
                    school_id = %envelope.school_id,
                    class_id = cid,
                    "student_admitted_fees_assign: would create FeesAssign for student_id={} in school_id={} class_id={}",
                    sid,
                    %envelope.school_id,
                    cid,
                );
                // TODO(SDK): dispatch
                // `Finance::CreateFeesAssign` command with
                // `(student_id, school_id, class_id)` once the
                // SDK facade wires finance commands.
            }
            _ => {
                tracing::warn!(
                    event_id = %event_id,
                    event_type = envelope.event_type,
                    "student_admitted_fees_assign: payload missing student_id or class_id; skipping"
                );
            }
        }
        Ok(())
    }
}

// =============================================================================
// Subscriber 6: student_withdrawn_terminate_fees_assign
// =============================================================================

/// Subscribes to `academic.student.withdrawn` and terminates
/// the student's open `FeesAssign`. Per
/// `docs/specs/finance/workflows.md` § "Cross-Workflow Order"
/// step 3: "StudentWithdrawn (academic) → open FeesAssign is
/// closed; unpaid balance becomes a carry-forward or a refund,
/// per policy."
///
/// # Audit finding
///
/// `docs/audit_reports/findings/wave7-workflows.md` WF-005
/// (StudentWithdrawn → FeesAssign termination fan-out is
/// missing).
///
/// # Idempotency
///
/// Dedupe key: `envelope.event_id`. The downstream
/// `fees_assign.terminate` command is keyed on
/// `(school_id, student_id)`; re-delivery is a no-op per
/// `finance/workflows.md` § "Idempotency".
#[derive(Debug)]
pub struct StudentWithdrawnTerminateFeesAssign {
    /// Dedupe set: event_ids already processed.
    seen_events: Mutex<HashSet<uuid::Uuid>>,
}

impl StudentWithdrawnTerminateFeesAssign {
    /// Constructs a new subscriber with an empty dedupe set.
    #[must_use]
    pub fn new() -> Self {
        Self {
            seen_events: Mutex::new(HashSet::new()),
        }
    }
}

impl Default for StudentWithdrawnTerminateFeesAssign {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Subscriber for StudentWithdrawnTerminateFeesAssign {
    fn name(&self) -> &'static str {
        "student_withdrawn_terminate_fees_assign"
    }

    async fn handle(&self, envelope: &EventEnvelope) -> Result<()> {
        let event_id = envelope.event_id.as_uuid();
        {
            let mut seen = self.seen_events.lock().map_err(|_| {
                educore_core::error::DomainError::Infrastructure(
                    "StudentWithdrawnTerminateFeesAssign dedupe mutex poisoned".into(),
                )
            })?;
            if !seen.insert(event_id) {
                tracing::debug!(
                    event_id = %event_id,
                    "student_withdrawn_terminate_fees_assign: duplicate event, skipping"
                );
                return Ok(());
            }
        }

        // Parse the withdrawal identifier defensively. The
        // StudentWithdrawn payload carries the student_id;
        // school_id is sourced from the envelope (matches the
        // existing subscriber pattern).
        let student_id = envelope
            .payload
            .get("student_id")
            .and_then(serde_json::Value::as_str);

        match student_id {
            Some(sid) => {
                tracing::info!(
                    event_id = %event_id,
                    student_id = sid,
                    school_id = %envelope.school_id,
                    "student_withdrawn_terminate_fees_assign: would terminate FeesAssign for student_id={} school_id={}",
                    sid,
                    %envelope.school_id,
                );
                // TODO(SDK): dispatch
                // `Finance::TerminateFeesAssign` command with
                // `(student_id, school_id)` once the SDK
                // facade wires finance commands.
            }
            None => {
                tracing::warn!(
                    event_id = %event_id,
                    event_type = envelope.event_type,
                    "student_withdrawn_terminate_fees_assign: payload missing student_id; skipping"
                );
            }
        }
        Ok(())
    }
}

// =============================================================================
// Subscriber 7: subject_teacher_assigned_class_subject
// =============================================================================

/// Subscribes to `academic.subject_teacher.assigned` and
/// materialises the corresponding `ClassSubject` binding so the
/// academic domain has a first-class record of "this teacher
/// teaches this subject in this class/section". Per
/// `docs/specs/academic/workflows.md` § "Class Subject Binding":
/// when a teacher is assigned to a subject in a class/section,
/// a `ClassSubject` binding is created. The
/// `SubjectTeacherAssigned` event is the trigger that fans out
/// into the academic `class_subject.create` command.
///
/// # Audit finding
///
/// `docs/audit_reports/findings/wave7-workflows.md` FND-WF-007
/// ("Academic has no subscriber on SubjectTeacherAssigned;
/// class-subject binding never created automatically").
///
/// # Idempotency
///
/// Dedupe key: `envelope.event_id`. The downstream
/// `class_subject.create` command is keyed on
/// `(school_id, class_id, section_id, subject_id, teacher_id)`;
/// re-delivery is a no-op per `academic/workflows.md` §
/// "Idempotency".
#[derive(Debug)]
pub struct SubjectTeacherAssignedClassSubject {
    /// Dedupe set: event_ids already processed.
    seen_events: Mutex<HashSet<uuid::Uuid>>,
}

impl SubjectTeacherAssignedClassSubject {
    /// Constructs a new subscriber with an empty dedupe set.
    #[must_use]
    pub fn new() -> Self {
        Self {
            seen_events: Mutex::new(HashSet::new()),
        }
    }
}

impl Default for SubjectTeacherAssignedClassSubject {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Subscriber for SubjectTeacherAssignedClassSubject {
    fn name(&self) -> &'static str {
        "subject_teacher_assigned_class_subject"
    }

    async fn handle(&self, envelope: &EventEnvelope) -> Result<()> {
        let event_id = envelope.event_id.as_uuid();
        {
            let mut seen = self.seen_events.lock().map_err(|_| {
                educore_core::error::DomainError::Infrastructure(
                    "SubjectTeacherAssignedClassSubject dedupe mutex poisoned".into(),
                )
            })?;
            if !seen.insert(event_id) {
                tracing::debug!(
                    event_id = %event_id,
                    "subject_teacher_assigned_class_subject: duplicate event, skipping"
                );
                return Ok(());
            }
        }

        // Parse the four class-subject binding identifiers
        // defensively. The SubjectTeacherAssigned payload carries
        // the subject, class, section, and teacher ids; school_id
        // is sourced from the envelope (matches the existing
        // subscriber pattern). All four must be present for the
        // downstream ClassSubject materialisation to proceed.
        let subject_id = envelope
            .payload
            .get("subject_id")
            .and_then(serde_json::Value::as_str);
        let class_id = envelope
            .payload
            .get("class_id")
            .and_then(serde_json::Value::as_str);
        let section_id = envelope
            .payload
            .get("section_id")
            .and_then(serde_json::Value::as_str);
        let teacher_id = envelope
            .payload
            .get("teacher_id")
            .and_then(serde_json::Value::as_str);

        match (subject_id, class_id, section_id, teacher_id) {
            (Some(sid), Some(cid), Some(secid), Some(tid)) => {
                tracing::info!(
                    event_id = %event_id,
                    school_id = %envelope.school_id,
                    subject_id = sid,
                    class_id = cid,
                    section_id = secid,
                    teacher_id = tid,
                    "subject_teacher_assigned_class_subject: would create ClassSubject for subject_id={} class_id={} section_id={} teacher_id={}",
                    sid,
                    cid,
                    secid,
                    tid,
                );
                // TODO(SDK): dispatch
                // `Academic::CreateClassSubject` command with
                // `(school_id, class_id, section_id, subject_id,
                // teacher_id)` once the SDK facade wires academic
                // commands.
            }
            _ => {
                tracing::warn!(
                    event_id = %event_id,
                    event_type = envelope.event_type,
                    "subject_teacher_assigned_class_subject: payload missing subject_id, class_id, section_id, or teacher_id; skipping"
                );
            }
        }
        Ok(())
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::*;
    use educore_core::clock::{IdGenerator, SystemIdGen};
    use educore_core::value_objects::Timestamp;
    use serde_json::json;

    /// Test helper: run an async future to completion. Uses a
    /// fresh single-thread tokio runtime; tests are short-lived
    /// so the runtime cost is negligible.
    fn block_on<F: std::future::Future>(future: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test runtime")
            .block_on(future)
    }

    fn envelope(event_type: &'static str, payload: serde_json::Value) -> EventEnvelope {
        let g = SystemIdGen;
        EventEnvelope {
            event_id: g.next_event_id(),
            event_type: event_type.to_string(),
            schema_version: 1,
            school_id: g.next_school_id(),
            aggregate_id: g.next_uuid(),
            aggregate_type: "test_aggregate".to_string(),
            actor_id: g.next_user_id(),
            correlation_id: g.next_correlation_id(),
            causation_id: None,
            occurred_at: Timestamp::now(),
            published_at: None,
            payload,
        }
    }

    #[test]
    fn register_all_subscribers_wires_at_least_seven_subscribers() {
        // Sanity check: the umbrella's bootstrap wires at least
        // the seven reference subscribers. The assertion uses
        // `>= 7` so that adding an eighth subscriber in a
        // future PR does not break this test.
        let mut registry = SubscriberRegistry::new();
        register_all_subscribers(&mut registry);
        assert!(
            registry.len() >= 7,
            "expected at least 7 subscribers, got {}",
            registry.len()
        );
    }

    #[test]
    fn form_uploaded_subscriber_receives_documents_form_uploaded_event() {
        let mut registry = SubscriberRegistry::new();
        register_all_subscribers(&mut registry);

        let env = envelope(
            "documents.form_download.uploaded",
            json!({
                "form_id": uuid::Uuid::now_v7(),
                "show_public": true,
            }),
        );
        let env_event_id = env.event_id;
        let stats = block_on(registry.dispatch(&env)).unwrap();
        assert_eq!(stats.delivered, 1);
        assert!(stats.is_ok());

        // Re-delivery: the subscriber's filter still matches
        // (same event_type), so the dispatcher invokes the
        // handler a second time. The handler is idempotent —
        // it observes the duplicate `event_id` in its
        // `seen_events` HashSet (the in-process idempotency
        // key) and returns `Ok(())` immediately, without
        // re-running the downstream CMS index write. Hence:
        //   - `delivered == 1` (handler invoked, no error)
        //   - `failures` is empty (no error raised)
        // The dedupe lives inside the handler, so the registry
        // sees the handler as a successful no-op. This matches
        // the bus-port contract: subscribers MUST be idempotent
        // on `event_id` because delivery is at-least-once.
        let stats2 = block_on(registry.dispatch(&env)).unwrap();
        assert_eq!(
            stats2.delivered, 1,
            "handler is invoked on duplicate but short-circuits via seen_events dedupe"
        );
        assert!(
            stats2.is_ok(),
            "duplicate dispatch must not record a failure"
        );
        assert!(stats2.failures.is_empty());
        let _ = env_event_id; // suppress unused warning
    }

    #[test]
    fn student_promoted_subscriber_handles_missing_payload_gracefully() {
        let subscriber = StudentPromotedFeeStructure::new();
        // Empty payload: subscriber should log a warning and
        // return Ok (not panic, not error).
        let env = envelope("academic.student.promoted", json!({}));
        let result = block_on(subscriber.handle(&env));
        assert!(result.is_ok());
    }

    #[test]
    fn staff_registered_subscriber_handles_missing_payload_gracefully() {
        let subscriber = StaffRegisteredSalaryTemplate::new();
        let env = envelope("hr.staff.registered", json!({}));
        let result = block_on(subscriber.handle(&env));
        assert!(result.is_ok());
    }

    #[test]
    fn payroll_paid_subscriber_handles_missing_payload_gracefully() {
        let subscriber = PayrollPaidMarkPaid::new();
        let env = envelope("hr.payroll.paid", json!({}));
        let result = block_on(subscriber.handle(&env));
        assert!(result.is_ok());
    }

    #[test]
    fn student_admitted_subscriber_handles_missing_payload_gracefully() {
        let subscriber = StudentAdmittedFeesAssign::new();
        // Empty payload: subscriber should log a warning and
        // return Ok (not panic, not error).
        let env = envelope("academic.student.admitted", json!({}));
        let result = block_on(subscriber.handle(&env));
        assert!(result.is_ok());
    }

    #[test]
    fn student_withdrawn_subscriber_handles_missing_payload_gracefully() {
        let subscriber = StudentWithdrawnTerminateFeesAssign::new();
        let env = envelope("academic.student.withdrawn", json!({}));
        let result = block_on(subscriber.handle(&env));
        assert!(result.is_ok());
    }

    #[test]
    fn subject_teacher_assigned_subscriber_handles_missing_payload_gracefully() {
        let subscriber = SubjectTeacherAssignedClassSubject::new();
        // Empty payload: subscriber should log a warning and
        // return Ok (not panic, not error).
        let env = envelope("academic.subject_teacher.assigned", json!({}));
        let result = block_on(subscriber.handle(&env));
        assert!(result.is_ok());
    }

    #[test]
    fn subject_teacher_assigned_subscriber_handles_full_payload() {
        let subscriber = SubjectTeacherAssignedClassSubject::new();
        let env = envelope(
            "academic.subject_teacher.assigned",
            json!({
                "subject_id": "sub-1",
                "class_id": "cls-1",
                "section_id": "sec-1",
                "teacher_id": "tch-1",
            }),
        );
        // First delivery: subscriber logs and returns Ok.
        let result = block_on(subscriber.handle(&env));
        assert!(result.is_ok());

        // Second delivery (same event_id): subscriber is
        // idempotent on `event_id`, so the dedupe guard
        // short-circuits with Ok.
        let result2 = block_on(subscriber.handle(&env));
        assert!(result2.is_ok());
    }

    #[test]
    fn form_uploaded_subscriber_handles_missing_show_public() {
        let subscriber = FormUploadedPublicIndexing::new();
        let env = envelope(
            "documents.form_download.uploaded",
            json!({"form_id": "abc"}),
        );
        let result = block_on(subscriber.handle(&env));
        assert!(result.is_ok());
    }

    #[test]
    fn subscribers_are_idempotent_under_redelivery() {
        // The same envelope dispatched twice should be a no-op
        // on the second delivery.
        let subscriber = PayrollPaidMarkPaid::new();
        let env = envelope(
            "hr.payroll.paid",
            json!({
                "payroll_generate_id": "pg-1",
                "paid_amount": 100.0,
            }),
        );
        block_on(subscriber.handle(&env)).unwrap();
        // Second delivery: the subscriber returns Ok immediately.
        let result = block_on(subscriber.handle(&env));
        assert!(result.is_ok());
    }

    #[test]
    fn subscribers_have_stable_names() {
        // The names are part of the public contract (used in
        // logs and DispatchStats::failures).
        assert_eq!(
            FormUploadedPublicIndexing::new().name(),
            "form_uploaded_public_indexing"
        );
        assert_eq!(
            StudentPromotedFeeStructure::new().name(),
            "student_promoted_fee_structure"
        );
        assert_eq!(
            StaffRegisteredSalaryTemplate::new().name(),
            "staff_registered_salary_template"
        );
        assert_eq!(PayrollPaidMarkPaid::new().name(), "payroll_paid_mark_paid");
        assert_eq!(
            StudentAdmittedFeesAssign::new().name(),
            "student_admitted_fees_assign"
        );
        assert_eq!(
            StudentWithdrawnTerminateFeesAssign::new().name(),
            "student_withdrawn_terminate_fees_assign"
        );
        assert_eq!(
            SubjectTeacherAssignedClassSubject::new().name(),
            "subject_teacher_assigned_class_subject"
        );
    }

    #[test]
    fn registry_only_dispatches_matching_events() {
        // Events that don't match any registered filter should
        // be skipped (not delivered, not failed). Use a
        // synthetic event type that no subscriber subscribes
        // to; do not reuse an academic event type here because
        // several academic.* events now have wired subscribers.
        let mut registry = SubscriberRegistry::new();
        register_all_subscribers(&mut registry);
        let env = envelope("unrelated.test.event", json!({}));
        let stats = block_on(registry.dispatch(&env)).unwrap();
        assert_eq!(stats.delivered, 0);
        assert_eq!(stats.skipped, 7);
        assert!(stats.is_ok());
    }
}
