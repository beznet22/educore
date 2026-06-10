# AI Agent Integration Guide

## Goal

Expose Educore to AI agents (LLM tool-use) safely. The agent can
admit students, mark attendance, generate invoices, and answer
questions, but cannot bypass business rules.

## Capability Gating

Every command requires a capability. The agent's tool surface is
derived from the capabilities the operator grants the agent.

```rust
let agent_session = Session {
    user_id: agent_user_id,
    school_id: target_school_id,
    capabilities: btreeset! {
        Capability::StudentRead,
        Capability::StudentAdmit,
        Capability::AttendanceMark,
    },
    ..
};
```

The agent can call any command in this set. Anything outside is
rejected with `DomainError::Forbidden`.

## Tool Surface

Each capability corresponds to one or more "tools" the agent can
invoke. The engine provides a tool catalog:

```rust
pub struct Tool {
    pub name: &'static str,
    pub description: &'static str,
    pub input_schema: serde_json::Value,    // JSON Schema
    pub output_schema: serde_json::Value,
    pub required_capability: Capability,
    pub example: Option<&'static str>,
}
```

The consumer's agent runtime reads the catalog and exposes the tools
to the LLM. Capabilities the agent lacks are filtered out.

## Input Schema

Inputs are JSON objects matching the command struct. The engine
generates a JSON Schema for each command:

```json
{
  "type": "object",
  "properties": {
    "admission_no": { "type": "string", "pattern": "^[A-Z0-9-]+$" },
    "first_name": { "type": "string", "minLength": 1, "maxLength": 200 },
    "date_of_birth": { "type": "string", "format": "date" },
    "class_id": { "type": "string", "format": "uuid" }
  },
  "required": ["admission_no", "first_name", "date_of_birth", "class_id"]
}
```

The agent runtime validates inputs against the schema before
dispatching. Invalid inputs are rejected at the boundary, not inside
the engine.

## Output Schema

Outputs are JSON representations of the resulting aggregate:

```json
{
  "student_id": "f1e2d3c4-b5a6-9788-...",
  "admission_no": "ADM-2026-0001",
  "full_name": "Ada Lovelace",
  "status": "Active",
  "class_id": "...",
  "section_id": "..."
}
```

The agent reads the output and decides the next action.

## Workflow Choreography

A complex workflow (e.g. "admit a student") is a sequence of tool
calls. The agent runtime does not need a workflow engine; the LLM
chooses the next tool based on prior outputs.

```text
User: "Admit Ada Lovelace to class 5A."
Agent: calls AdmitStudentTool
Engine: returns Student aggregate
Agent: "Ada Lovelace is admitted. Admission number ADM-2026-0001.
        Her class is 5A, section A."
```

## Sandboxing

The agent's session can be restricted further:

- Read-only: only `*.Read` capabilities.
- Limited to a single class: the engine enforces `class_id` filter.
- Time-limited: the session expires after N minutes.
- Action-limited: the agent can admit at most 10 students per session.

The consumer enforces these limits. The engine enforces the
underlying business rules.

## Confirmation Prompts

For destructive commands (withdraw, refund, delete), the agent
runtime SHOULD require a human confirmation before dispatching. The
engine does not enforce this; the runtime does.

```rust
let outcome = if cmd.is_destructive() {
    runtime.confirm_with_human(&format!(
        "The agent wants to {}. Proceed?", cmd.describe()
    ))?;
    engine.dispatch(cmd).await
} else {
    engine.dispatch(cmd).await
};
```

## Audit

Every agent action is audited with `actor_id = agent_user_id`. The
audit trail includes:

- The tool called.
- The input (full JSON, redacted for PII).
- The output (or error).
- The capability check result.
- The eventual state change (event id).

Auditors can answer: "What did the agent do on 2026-06-08 between
14:00 and 15:00?" with a single query.

## Testing Agents

The engine provides a `educore-agent-test` crate:

```rust
let agent = TestAgent::new("test-agent", capabilities![
    Capability::StudentRead,
    Capability::AttendanceMark,
]);

let outcome = agent.invoke("Mark John Doe present today.").await?;
assert!(outcome.contains("attendance marked"));
```

The test agent is a deterministic simulator that exercises the
command surface end-to-end.

## Worked Example

A consumer integrates an LLM agent:

```rust
let engine = Engine::builder()
    .storage(...)
    .auth(...)
    .build().await?;

let tools = engine.tools().for_session(agent_session);
let agent = MyLlmAgent::new(llm_client, tools);

let user_input = "Mark all students in class 5A present for today.";
let response = agent.invoke(user_input).await?;
```

The agent calls `ListStudentsInClass` to enumerate, then calls
`MarkStudentAttendance` for each. The engine enforces the class scope
and writes audit records.

## Anti-Patterns

- ❌ Exposing raw SQL to the agent.
- ❌ Allowing the agent to issue commands without a capability.
- ❌ Bypassing business rules (e.g. "force-admit" command).
- ❌ Letting the agent read PII without consent.
- ❌ Letting the agent cross tenants.
- ❌ Letting the agent delete audit records.

The engine has no "force" or "admin override" commands. Operators
issue commands; agents issue commands under the same rules.
