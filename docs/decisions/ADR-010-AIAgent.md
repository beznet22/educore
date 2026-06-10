# ADR-010: AI-Agent Friendly Design

## Status

Accepted.

## Context

AI agents — large language models augmented with tool use
— are increasingly common. A school administrator may use
an agent to "admit these 30 students from the spreadsheet
and tell me which ones are missing documents." A
superintendent may use an agent to "give me the list of
all classes where the average marks dropped by more than
10% this term." A parent's AI assistant may use the
engine to ask "is the school bus running today?" and to
book a leave for the child.

An AI agent is **not** a human user. It is a tool-using
process with its own failure modes:

- It can hallucinate identifiers. A agent that
  hallucinates a `StudentId` will fail a "not found"
  check; the engine must return a typed error the
  agent can understand and recover from.
- It can issue a command it is not authorized for. The
  capability check must be enforced, but the error
  must be informative enough for the agent to
  understand what capability it needs and to request
  it.
- It can be tricked by a malicious prompt into
  attempting a destructive action. The engine must
  reject destructive actions that exceed the
  agent's capabilities.
- It can fail mid-workflow. The engine's idempotency
  must absorb retries.
- It can be slow or expensive. The engine must expose
  bulk commands and progress events for long
  operations.
- It can be wrong. The engine's audit log must
  capture every action the agent took, with the
  agent's user id, the prompt context, and the
  outcome.

The agent is also **not a privileged user**. It does
not get a "service role" by default. It inherits the
capabilities of the user it is acting on behalf of. An
agent acting for a parent has the parent's
capabilities; an agent acting for a school admin has
the admin's capabilities.

## Decision

Educore is **AI-agent friendly by design**. The
engine's command catalog, capability model, audit log,
and error semantics are designed to be driven safely
by tool-using LLM agents.

Concretely:

1. **The capability catalog is the agent's tool
   list.** A consumer's agent runtime reads
   `engine.capabilities()` and exposes them as tools.
2. **Capabilities are the authorization gate.** The
   agent cannot bypass them. There is no "service
   role" escape hatch.
3. **Errors are typed and informative.** A
   `Forbidden(CapabilityMissing)` error includes the
   required capability, the actor, and the
   remediation ("ask the user to grant the
   `Student.AssignSection` capability").
4. **Idempotency is structural.** The agent's
   runtime retries commands on transient failures;
   the engine deduplicates on `idempotency_key`.
5. **Bulk commands are first-class.** The agent
   does not loop single commands; it issues a bulk
   command with a transaction.
6. **Async commands have progress events.** A
   long-running command emits `CommandAccepted`,
   `CommandStarted`, `CommandProgress` (zero or
   more), `CommandCompleted`. The agent can poll
   for status.
7. **The audit log records the agent as a distinct
   `actor_type`.** Every command the agent invokes
   is audited with `actor_type = "agent"`, the
   `actor_id` of the underlying user, and a
   `metadata.prompt_context` field for the
   responsible prompt (where captured by the
   consumer's agent runtime).
8. **Capability groups for UI rendering are also
   useful for the agent.** The
   `PermissionSection` catalog groups capabilities
   into discoverable categories.
9. **The schema registry is exposed.** The agent
   can ask "what's the shape of `Academic.Student.
   Admit`?" before issuing it.
10. **AI-specific safety rails are encouraged.**
    Consumers may add a `RateLimit` policy per
    agent user; the engine supports it through a
    port.

The agent's contract with the engine is the same
as a human user's contract: capabilities, commands,
events, audit.

## Consequences

### Positive

- **Agents are first-class users.** The same
  surface serves a human in a web UI, a script
  in a CLI, and an agent in a chat.
- **Errors are recoverable.** A typed error tells
  the agent what went wrong and what to do
  next.
- **Audit is comprehensive.** Every agent action
  is traceable to the user, the prompt, and the
  outcome. A regulator can audit agent behavior
  the same way it audits human behavior.
- **Idempotency absorbs retries.** A flaky
  network does not produce a duplicate
  admission.
- **Bulk operations are atomic.** The agent's
  "admit 30 students" command either succeeds
  for all or fails for all, with per-item
  errors if the failure policy allows.
- **Progress is observable.** A long-running
  command does not leave the agent wondering;
  progress events feed the agent's UI.

### Negative

- **The agent runtime is the consumer's
  responsibility.** The engine does not ship
  one. The consumer integrates an LLM, a tool-
  use framework, and a prompt-template engine.
- **Capability lists can be long.** An agent
  must discover and reason over hundreds of
  capabilities. The `PermissionSection`
  grouping helps; consumers may also provide
  curated subsets per agent role.
- **Prompt injection is an unsolved problem.**
  The engine's defense is capability-gated
  commands and a comprehensive audit log.
  Defense in depth is the consumer's job.
- **The agent may not be able to recover from
  every error.** A `Precondition` error
  requires a human decision. The agent must
  escalate, not retry.

### Mitigations

- The `Capability` enum and the
  `PermissionSection` grouping make discovery
  manageable.
- The `Report.Generate` command lets agents ask
  for aggregated data without writing ad-hoc
  queries.
- The `RateLimit` port is built in; consumers
  can throttle agents per user / per minute.
- The `Audit` log is the safety net: every
  agent action is recorded, and the engine
  supports post-hoc "what did the agent do in
  the last hour?" queries.

## Alternatives Considered

### 1. A separate, "agent-only" API

Expose a different surface for agents. Rejected
because it doubles the surface, drifts from
the human surface, and creates a security
risk (the agent surface may have weaker
checks than the human surface).

### 2. Service-role authentication for agents

The agent authenticates as a service and
bypasses the per-user capability check.
Rejected because the agent's blast radius
becomes a service's blast radius. An agent
acting for a parent must have the parent's
capabilities, not a service's.

### 3. Read-only API for agents

Agents can read but not write. Rejected
because the agent's value comes from doing
(admitting, recording marks, scheduling
exams). A read-only agent is a search box;
a write-capable agent is a colleague.

### 4. Free-form SQL or GraphQL for agents

The agent writes its own queries. Rejected
for the same reasons as `ADR-006`: the
type system is the safety net.

### 5. A planner / orchestrator inside the
engine

The engine contains a planning module that
turns natural language into commands.
Rejected because the engine is a domain
kernel, not an LLM runtime. The consumer
provides the planner; the engine provides
the surface.
