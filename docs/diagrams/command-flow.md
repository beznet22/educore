# Command Flow

Sequence diagrams for the engine's key commands. Each
diagram shows the call path from the consumer through
the dispatcher, the aggregate, the persistence layer,
the event bus, and the audit sink.

## 1. `AdmitStudent` — The Foundational Command

```mermaid
sequenceDiagram
    autonumber
    participant Caller as Caller
    participant Engine as Engine Facade
    participant Dispatcher as Command Dispatcher
    participant Auth as AuthProvider
    participant Cap as CapabilityCheck
    participant Repo as StudentRepository
    participant Aggregate as Student Aggregate
    participant Storage as Storage Port
    participant Bus as Event Bus
    participant Audit as Audit Sink
    participant Sub as Subscribers

    Caller->>Engine: engine.academic().admit_student(cmd)
    Engine->>Dispatcher: dispatch(cmd)
    Dispatcher->>Auth: resolve(actor)
    Auth-->>Dispatcher: TenantContext
    Dispatcher->>Cap: check(actor, Student.Admit)
    Cap-->>Dispatcher: Allow
    Dispatcher->>Dispatcher: validate struct<br/>validate references<br/>(class, section, year, guardians)
    Dispatcher->>Repo: load_by_admission_no(no)
    Repo-->>Dispatcher: None (new)
    Dispatcher->>Aggregate: new Student(student_spec)
    Aggregate->>Aggregate: enforce invariants
    Aggregate->>Aggregate: emit StudentAdmitted,<br/>StudentRecordCreated,<br/>GuardianLinked
    Aggregate-->>Dispatcher: state + events
    Dispatcher->>Storage: persist(state, events) [txn]
    Storage-->>Dispatcher: ok
    Dispatcher->>Bus: publish(events) [outbox]
    Bus-->>Sub: events
    Sub-->>Bus: ack
    Bus-->>Dispatcher: published
    Dispatcher->>Audit: write(audit_record)
    Audit-->>Dispatcher: ok
    Dispatcher-->>Engine: CommandOutcome
    Engine-->>Caller: AdmitStudentResult
```

## 2. `GenerateInvoice` — Fees Invoice Generation

```mermaid
sequenceDiagram
    autonumber
    participant Caller as Accountant
    participant Engine as Engine Facade
    participant Dispatcher as Command Dispatcher
    participant Cap as CapabilityCheck
    participant FeeRepo as FeesAssignRepository
    participant InvRepo as InvoiceRepository
    participant Aggregate as Invoice Aggregate
    participant Storage as Storage Port
    participant Bus as Event Bus
    participant Audit as Audit Sink

    Caller->>Engine: engine.finance().generate_invoice(cmd)
    Engine->>Dispatcher: dispatch(cmd)
    Dispatcher->>Cap: check(actor, Invoice.Generate)
    Cap-->>Dispatcher: Allow
    Dispatcher->>FeeRepo: list_assigns(class_id, year)
    FeeRepo-->>Dispatcher: Vec<FeesAssign>
    loop for each assign
        Dispatcher->>Aggregate: new Invoice(fees_assign)
        Aggregate->>Aggregate: compute line items,<br/>apply discounts,<br/>compute total
        Aggregate->>Aggregate: emit InvoiceGenerated
    end
    Aggregate-->>Dispatcher: state + events
    Dispatcher->>Storage: persist(invoices, events) [txn]
    Storage-->>Dispatcher: ok
    Dispatcher->>Bus: publish(events)
    Bus-->>Dispatcher: published
    Dispatcher->>Audit: write(audit_records)
    Audit-->>Dispatcher: ok
    Dispatcher-->>Engine: CommandOutcome
    Engine-->>Caller: GenerateInvoiceResult
```

## 3. `GeneratePayroll` — Monthly Payroll Generation

```mermaid
sequenceDiagram
    autonumber
    participant Caller as HR / Accountant
    participant Engine as Engine Facade
    participant Dispatcher as Command Dispatcher
    participant Cap as CapabilityCheck
    participant StaffRepo as StaffRepository
    participant AttRepo as StaffAttendanceRepository
    participant LeaveRepo as LeaveRepository
    participant TplRepo as SalaryTemplateRepository
    participant Aggregate as Payroll Aggregate
    participant Storage as Storage Port
    participant Bus as Event Bus
    participant Audit as Audit Sink

    Caller->>Engine: engine.finance().generate_payroll(cmd)
    Engine->>Dispatcher: dispatch(cmd)
    Dispatcher->>Cap: check(actor, Payroll.Generate)
    Cap-->>Dispatcher: Allow
    Dispatcher->>StaffRepo: list_active_staff(period)
    StaffRepo-->>Dispatcher: Vec<Staff>
    loop for each staff
        Dispatcher->>AttRepo: attendance_summary(staff, period)
        AttRepo-->>Dispatcher: days_present, overtime
        Dispatcher->>LeaveRepo: leave_summary(staff, period)
        LeaveRepo-->>Dispatcher: leave_days, deductions
        Dispatcher->>TplRepo: salary_template(staff)
        TplRepo-->>Dispatcher: template
        Dispatcher->>Aggregate: new Payroll(staff, summary)
        Aggregate->>Aggregate: compute earnings,<br/>apply deductions,<br/>compute net
        Aggregate->>Aggregate: emit PayrollGenerated
    end
    Aggregate-->>Dispatcher: state + events
    Dispatcher->>Storage: persist(payrolls, events) [txn]
    Storage-->>Dispatcher: ok
    Dispatcher->>Bus: publish(events)
    Bus-->>Dispatcher: published
    Dispatcher->>Audit: write(audit_records)
    Audit-->>Dispatcher: ok
    Dispatcher-->>Engine: CommandOutcome
    Engine-->>Caller: GeneratePayrollResult
```

## 4. `MarkAttendance` — Daily Bulk Operation

```mermaid
sequenceDiagram
    autonumber
    participant Caller as Class Teacher
    participant Engine as Engine Facade
    participant Dispatcher as Command Dispatcher
    participant Cap as CapabilityCheck
    participant SessionRepo as AttendanceSessionRepository
    participant Aggregate as AttendanceSession Aggregate
    participant Storage as Storage Port
    participant Bus as Event Bus
    participant CommSub as Communication Subscriber
    participant Audit as Audit Sink

    Caller->>Engine: engine.attendance().mark(cmd)
    Engine->>Dispatcher: dispatch(cmd)
    Dispatcher->>Cap: check(actor, Attendance.Mark)
    Cap-->>Dispatcher: Allow
    Dispatcher->>SessionRepo: load_or_create(class_section, date)
    SessionRepo-->>Dispatcher: AttendanceSession
    Dispatcher->>Aggregate: record_entries(entries)
    Aggregate->>Aggregate: enforce invariants
    Aggregate->>Aggregate: emit AttendanceMarked
    Aggregate-->>Dispatcher: state + events
    Dispatcher->>Storage: persist(session, events) [txn]
    Storage-->>Dispatcher: ok
    Dispatcher->>Bus: publish(events)
    Bus->>CommSub: AttendanceMarked
    CommSub->>CommSub: for each absent,<br/>notify guardian
    CommSub-->>Bus: ack
    Bus-->>Dispatcher: published
    Dispatcher->>Audit: write(audit_record)
    Audit-->>Dispatcher: ok
    Dispatcher-->>Engine: CommandOutcome
    Engine-->>Caller: MarkAttendanceResult
```

## 5. `CollectPayment` — Fee Collection

```mermaid
sequenceDiagram
    autonumber
    participant Caller as Cashier / Gateway
    participant Engine as Engine Facade
    participant Dispatcher as Command Dispatcher
    participant Cap as CapabilityCheck
    participant AssignRepo as FeesAssignRepository
    participant BankRepo as BankRepository
    participant Aggregate as FeesPayment Aggregate
    participant Storage as Storage Port
    participant Bus as Event Bus
    participant Audit as Audit Sink

    Caller->>Engine: engine.finance().collect_payment(cmd)
    Engine->>Dispatcher: dispatch(cmd)
    Dispatcher->>Cap: check(actor, Payment.Collect)
    Cap-->>Dispatcher: Allow
    Dispatcher->>AssignRepo: load(assign_id)
    AssignRepo-->>Dispatcher: FeesAssign
    Dispatcher->>BankRepo: load(bank_account_id)
    BankRepo-->>Dispatcher: BankAccount
    Dispatcher->>Aggregate: new FeesPayment(amount, mode)
    Aggregate->>Aggregate: enforce paid <= balance
    Aggregate->>Aggregate: emit PaymentCollected
    Aggregate-->>Dispatcher: state + events
    Dispatcher->>Storage: persist(payment, bank_entry, events) [txn]
    Storage-->>Dispatcher: ok
    Dispatcher->>Bus: publish(events)
    Bus-->>Dispatcher: published
    Dispatcher->>Audit: write(audit_record)
    Audit-->>Dispatcher: ok
    Dispatcher-->>Engine: CommandOutcome
    Engine-->>Caller: CollectPaymentResult
```

## 6. `PublishResult` — Examination Publication

```mermaid
sequenceDiagram
    autonumber
    participant Caller as Examination Officer
    participant Engine as Engine Facade
    participant Dispatcher as Command Dispatcher
    participant Cap as CapabilityCheck
    participant ExamRepo as ExamRepository
    participant ResultRepo as ResultRepository
    participant Aggregate as Exam Aggregate
    participant Storage as Storage Port
    participant Bus as Event Bus
    participant CommSub as Communication Subscriber
    participant ReportProj as Report Projection
    participant Audit as Audit Sink

    Caller->>Engine: engine.assessment().publish_result(cmd)
    Engine->>Dispatcher: dispatch(cmd)
    Dispatcher->>Cap: check(actor, Result.Publish)
    Cap-->>Dispatcher: Allow
    Dispatcher->>ExamRepo: load(exam_id)
    ExamRepo-->>Dispatcher: Exam
    Dispatcher->>ResultRepo: list_for_exam(exam_id)
    ResultRepo-->>Dispatcher: Vec<Result>
    loop for each result
        Dispatcher->>Aggregate: mark_published(result)
        Aggregate->>Aggregate: emit ResultPublished
    end
    Aggregate-->>Dispatcher: state + events
    Dispatcher->>Storage: persist(results, events) [txn]
    Storage-->>Dispatcher: ok
    Dispatcher->>Bus: publish(events)
    par Subscribers
        Bus->>CommSub: ResultPublished
        CommSub->>CommSub: notify guardians
    and
        Bus->>ReportProj: ResultPublished
        ReportProj->>ReportProj: aggregate stats
    end
    Bus-->>Dispatcher: published
    Dispatcher->>Audit: write(audit_records)
    Audit-->>Dispatcher: ok
    Dispatcher-->>Engine: CommandOutcome
    Engine-->>Caller: PublishResultResult
```

## 7. Command Pipeline (Generic)

```mermaid
sequenceDiagram
    autonumber
    participant Caller
    participant Engine
    participant Dispatcher
    participant Auth as AuthProvider
    participant Cap as CapabilityCheck
    participant Idem as Idempotency Store
    participant Repo as Repository
    participant Aggregate
    participant Storage
    participant Outbox as Outbox
    participant Bus as Event Bus
    participant Audit as Audit Sink

    Caller->>Engine: execute(cmd)
    Engine->>Dispatcher: dispatch(cmd)
    Dispatcher->>Auth: resolve actor
    Auth-->>Dispatcher: TenantContext
    Dispatcher->>Cap: check capability
    Cap-->>Dispatcher: Allow | Deny
    alt Deny
        Dispatcher-->>Caller: Forbidden
    end
    Dispatcher->>Idem: lookup(key)
    alt cached
        Idem-->>Dispatcher: prior outcome
        Dispatcher-->>Caller: prior outcome
    else not cached
        Dispatcher->>Repo: load(aggregate)
        Repo-->>Dispatcher: aggregate
        Dispatcher->>Aggregate: handle(cmd)
        Aggregate-->>Dispatcher: new state + events
        Dispatcher->>Storage: persist(state, events) [txn]
        Storage->>Outbox: write events [txn]
        Storage-->>Dispatcher: ok
        Dispatcher->>Idem: store outcome
        Dispatcher->>Bus: publish (via outbox relay)
        Bus-->>Dispatcher: published
        Dispatcher->>Audit: write record
        Audit-->>Dispatcher: ok
        Dispatcher-->>Caller: CommandOutcome
    end
```

## 8. Idempotency Replay

```mermaid
sequenceDiagram
    autonumber
    participant Caller
    participant Engine
    participant Dispatcher
    participant Idem as Idempotency Store

    Caller->>Engine: execute(cmd with idempotency_key=K)
    Engine->>Dispatcher: dispatch(cmd)
    Dispatcher->>Idem: lookup(K)
    Idem-->>Dispatcher: prior CommandOutcome
    Dispatcher-->>Engine: prior CommandOutcome
    Engine-->>Caller: same result, no re-execution

    Note over Caller,Idem: The caller sees the same outcome as<br/>the first call. The aggregate is not reloaded,<br/>no events are re-emitted.
```
