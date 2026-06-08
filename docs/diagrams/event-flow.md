# Event Flow

Sequence diagrams for the engine's key domain events. Each
diagram shows the producer, the bus, the consumers, and the
side effects.

## 1. `StudentAdmitted` — Cross-Domain Fanout

```mermaid
sequenceDiagram
    autonumber
    participant Caller as Caller<br/>(web / mobile / agent)
    participant Engine as Engine Facade
    participant Cmd as Command Bus
    participant Auth as AuthProvider
    participant Cap as CapabilityCheck
    participant Academic as Academic Aggregate
    participant Bus as Event Bus
    participant Finance as Finance Subscriber
    participant Library as Library Subscriber
    participant Comm as Communication Subscriber
    participant Audit as Audit Sink

    Caller->>Engine: execute(AdmitStudentCommand)
    Engine->>Cmd: dispatch(cmd)
    Cmd->>Auth: authenticate(actor)
    Auth-->>Cmd: TenantContext
    Cmd->>Cap: check(actor, Student.Admit)
    Cap-->>Cmd: Allow
    Cmd->>Academic: load + admit(student_spec)
    Academic-->>Cmd: state + events
    Note over Academic: StudentAdmitted<br/>StudentRecordCreated<br/>GuardianLinked
    Cmd->>Bus: publish(events)
    par Subscribers
        Bus->>Finance: StudentAdmitted
        Finance->>Finance: assign fees
        Finance-->>Bus: FeesAssigned
    and
        Bus->>Library: StudentAdmitted
        Library->>Library: create member
        Library-->>Bus: MemberCreated
    and
        Bus->>Comm: StudentAdmitted
        Comm->>Comm: send welcome message
        Comm-->>Bus: NoticeSent
    end
    Cmd->>Audit: write(audit_record)
    Audit-->>Cmd: ok
    Cmd-->>Engine: CommandOutcome
    Engine-->>Caller: AdmitStudentResult
```

## 2. `PaymentReceived` — Collection, Receipt, Notification

```mermaid
sequenceDiagram
    autonumber
    participant Caller as Cashier / Online Gateway
    participant Engine as Engine Facade
    participant Cmd as Command Bus
    participant Finance as Finance Aggregate
    participant Bank as Bank Account
    participant Bus as Event Bus
    participant Comm as Communication Subscriber
    participant Reports as Report Projection
    participant Audit as Audit Sink

    Caller->>Engine: execute(CollectPaymentCommand)
    Engine->>Cmd: dispatch(cmd)
    Cmd->>Finance: load FeesAssign + record payment
    Finance->>Bank: record bank statement entry
    Bank-->>Finance: updated balance
    Finance-->>Cmd: state + PaymentCollected event
    Cmd->>Bus: publish(PaymentCollected)
    par Side Effects
        Bus->>Comm: PaymentCollected
        Comm->>Comm: send receipt to parent
    and
        Bus->>Reports: PaymentCollected
        Reports->>Reports: update collection summary
    end
    Cmd->>Audit: write(audit_record)
    Cmd-->>Engine: CommandOutcome
    Engine-->>Caller: CollectPaymentResult
```

## 3. `ResultPublished` — Cross-Domain Notification Cascade

```mermaid
sequenceDiagram
    autonumber
    participant Caller as Examination Officer
    participant Engine as Engine Facade
    participant Cmd as Command Bus
    participant Assessment as Assessment Aggregate
    participant Bus as Event Bus
    participant Comm as Communication Subscriber
    participant Academic as Academic Subscriber
    participant Reports as Report Projection
    participant Audit as Audit Sink

    Caller->>Engine: execute(PublishResultCommand)
    Engine->>Cmd: dispatch(cmd)
    Cmd->>Assessment: load Exam, mark Results as published
    Assessment-->>Cmd: state + ResultPublished events
    Cmd->>Bus: publish(ResultPublished)
    par Subscribers
        Bus->>Comm: ResultPublished
        Comm->>Comm: send "result published" notice to guardians
    and
        Bus->>Academic: ResultPublished
        Academic->>Academic: archive exam references for the year
    and
        Bus->>Reports: ResultPublished
        Reports->>Reports: aggregate pass/fail stats
    end
    Cmd->>Audit: write(audit_record)
    Cmd-->>Engine: CommandOutcome
    Engine-->>Caller: PublishResultResult
```

## 4. `AttendanceMarked` — Daily Cascade

```mermaid
sequenceDiagram
    autonumber
    participant Caller as Class Teacher (mobile)
    participant Engine as Engine Facade
    participant Cmd as Command Bus
    participant Attendance as Attendance Aggregate
    participant Bus as Event Bus
    participant Comm as Communication Subscriber
    participant Academic as Academic Subscriber
    participant Audit as Audit Sink

    Caller->>Engine: execute(MarkAttendanceCommand)
    Engine->>Cmd: dispatch(cmd)
    Cmd->>Attendance: load session, record entries
    Attendance-->>Cmd: state + AttendanceMarked event
    Cmd->>Bus: publish(AttendanceMarked)
    par Subscribers
        Bus->>Comm: AttendanceMarked
        Comm->>Comm: for each absent student,<br/>notify guardian
    and
        Bus->>Academic: AttendanceMarked
        Academic->>Academic: update attendance projection
    end
    Cmd->>Audit: write(audit_record)
    Cmd-->>Engine: CommandOutcome
    Engine-->>Caller: MarkAttendanceResult
```

## 5. `PayrollPaid` — Finance → HR Reconciliation

```mermaid
sequenceDiagram
    autonumber
    participant Caller as Accountant
    participant Engine as Engine Facade
    participant Cmd as Command Bus
    participant Finance as Finance Aggregate
    participant Bank as Bank Account
    participant Bus as Event Bus
    participant Hr as HR Subscriber
    participant Comm as Communication Subscriber
    participant Audit as Audit Sink

    Caller->>Engine: execute(PayPayrollCommand)
    Engine->>Cmd: dispatch(cmd)
    Cmd->>Finance: load Payroll, mark as paid
    Finance->>Bank: record bank statement
    Bank-->>Finance: updated balance
    Finance-->>Cmd: state + PayrollPaid event
    Cmd->>Bus: publish(PayrollPaid)
    par Subscribers
        Bus->>Hr: PayrollPaid
        Hr->>Hr: update staff payment history
    and
        Bus->>Comm: PayrollPaid
        Comm->>Comm: send payslip notification to staff
    end
    Cmd->>Audit: write(audit_record)
    Cmd-->>Engine: CommandOutcome
    Engine-->>Caller: PayPayrollResult
```

## 6. `StudentPromoted` — Year-End Workflow

```mermaid
sequenceDiagram
    autonumber
    participant Caller as Vice Principal
    participant Engine as Engine Facade
    participant Cmd as Command Bus
    participant Academic as Academic Aggregate
    participant Bus as Event Bus
    participant Finance as Finance Subscriber
    participant Attendance as Attendance Subscriber
    participant Assessment as Assessment Subscriber
    participant Audit as Audit Sink

    Caller->>Engine: execute(PromoteStudentCommand)
    Engine->>Cmd: dispatch(cmd)
    Cmd->>Academic: close old StudentRecord,<br/>open new one in next year
    Academic-->>Cmd: state + StudentPromoted event
    Cmd->>Bus: publish(StudentPromoted)
    par Subscribers
        Bus->>Finance: StudentPromoted
        Finance->>Finance: rollover balance,<br/>assign new fees
    and
        Bus->>Attendance: StudentPromoted
        Attendance->>Attendance: reset daily expectation
    and
        Bus->>Assessment: StudentPromoted
        Assessment->>Assessment: archive prior marks,<br/>prepare new exam schedule
    end
    Cmd->>Audit: write(audit_record)
    Cmd-->>Engine: CommandOutcome
    Engine-->>Caller: PromoteStudentResult
```

## 7. `StudentWithdrawn` — Cleanup Cascade

```mermaid
sequenceDiagram
    autonumber
    participant Caller as School Admin
    participant Engine as Engine Facade
    participant Cmd as Command Bus
    participant Academic as Academic Aggregate
    participant Bus as Event Bus
    participant Finance as Finance Subscriber
    participant Library as Library Subscriber
    participant Transport as Transport Subscriber
    participant Comm as Communication Subscriber
    participant Audit as Audit Sink

    Caller->>Engine: execute(WithdrawStudentCommand)
    Engine->>Cmd: dispatch(cmd)
    Cmd->>Academic: mark student as Withdrawn
    Academic-->>Cmd: state + StudentWithdrawn event
    Cmd->>Bus: publish(StudentWithdrawn)
    par Subscribers
        Bus->>Finance: StudentWithdrawn
        Finance->>Finance: finalize outstanding balance
    and
        Bus->>Library: StudentWithdrawn
        Library->>Library: mark unreturned books
    and
        Bus->>Transport: StudentWithdrawn
        Transport->>Transport: remove from route
    and
        Bus->>Comm: StudentWithdrawn
        Comm->>Comm: stop notifications
    end
    Cmd->>Audit: write(audit_record)
    Cmd-->>Engine: CommandOutcome
    Engine-->>Caller: WithdrawStudentResult
```

## 8. Eventual Consistency Window

```mermaid
sequenceDiagram
    autonumber
    participant Caller
    participant Engine
    participant Bus
    participant Sub1 as Subscriber 1
    participant Sub2 as Subscriber 2
    participant Sub3 as Subscriber 3

    Caller->>Engine: execute(AdmitStudent)
    Engine-->>Caller: CommandOutcome (success)
    Note over Engine,Bus: Events committed to outbox
    Bus->>Sub1: StudentAdmitted
    Bus->>Sub2: StudentAdmitted
    Bus->>Sub3: StudentAdmitted
    Note over Sub1,Sub2,Sub3: Subscribers process in parallel.
    Note over Bus: Delivery is at-least-once.
    Note over Sub1,Sub2,Sub3: Each subscriber is idempotent on event_id.
```

The event bus delivers at-least-once. Subscribers MUST
deduplicate on `event_id`. The engine's audit log
mirrors every event with the same `event_id`, providing
a single source of truth.
