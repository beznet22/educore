# Implementation Guides

This folder contains implementation guides for specific concerns that
cut across domains. Each guide provides a concrete, opinionated
walkthrough of a specific task.

## Available Guides

| Guide                                                | Purpose                                              |
| ---------------------------------------------------- | ---------------------------------------------------- |
| `multi-tenancy.md`                                   | Implementing SchoolId in a consumer application     |
| `audit-trail.md`                                     | Wiring the audit sink for compliance                |
| `offline-sync.md`                                    | Implementing offline-first mode                     |
| `capability-rbac.md`                                | Setting up roles, capabilities, and authorization  |
| `storage-adapter.md`                                 | Building a storage adapter (PostgreSQL, SQLite)     |
| `notification-templates.md`                          | Defining and rendering notification templates        |
| `fee-collection.md`                                 | End-to-end fee collection flow                       |
| `report-card-generation.md`                          | Generating report cards with GPA, grade, merit      |
| `payroll-calculation.md`                             | Calculating monthly payroll with templates & leave  |
| `idempotent-commands.md`                             | Implementing idempotent command handlers           |
| `event-replay.md`                                    | Replaying events for new projections                |
| `test-strategy.md`                                   | Test pyramid for SMSengine consumers                  |
| `crud-patterns.md`                                   | Standard CRUD command patterns                       |
| `school-onboarding.md`                               | First-run school setup workflow                      |
| `ai-agent-integration.md`                            | Exposing SMSengine to an AI agent                      |
| `ci-cd.md`                                           | CI/CD for an SMSengine consumer application           |
