# Graph Report - D:/Projekte/Anchor  (2026-07-24)

## Corpus Check
- Corpus is ~23,111 words - fits in a single context window. You may not need a graph.

## Summary
- 141 nodes · 302 edges · 12 communities (11 shown, 1 thin omitted)
- Extraction: 95% EXTRACTED · 5% INFERRED · 1% AMBIGUOUS · INFERRED: 14 edges (avg confidence: 0.8)
- Token cost: 47,346 input · 0 output

## Community Hubs (Navigation)
- Interruption Stack Mechanic
- Platform & Storage Architecture
- Review & Grilling Tooling
- Core Domain Glossary
- Team Roles & Discovery
- Architecture Overview Content
- Export Feature Design
- Documentation Standards
- Workflow Governance Rules
- MVP Scope & Planning
- Project Entry Points
- Prompts (Empty)

## God Nodes (most connected - your core abstractions)
1. `Interruption Stack (Switch / Interrupt / Return)` - 23 edges
2. `Documentation Standards` - 18 edges
3. `Concept (Anchor product concept)` - 17 edges
4. `Task Templates` - 16 edges
5. `grill-with-docs Skill` - 15 edges
6. `ADR 0002: Desktop App Framework and Target Platform` - 15 edges
7. `Architecture Workflow` - 14 edges
8. `Glossary` - 14 edges
9. `Product Manager Agent` - 13 edges
10. `Technical Architect Agent` - 13 edges

## Surprising Connections (you probably didn't know these)
- `Ideas Inbox` --conceptually_related_to--> `Interruption Stack (Switch / Interrupt / Return)`  [AMBIGUOUS]
  ideas/README.md → docs/product/features/interruption-stack.md
- `Prototypes` --semantically_similar_to--> `Research`  [INFERRED] [semantically similar]
  prototypes/README.md → research/README.md
- `Anchor README` --references--> `Documentation Index`  [EXTRACTED]
  README.md → docs/README.md
- `Research` --conceptually_related_to--> `Vision`  [INFERRED]
  research/README.md → docs/vision/vision.md
- `ADR 0003: Billable Classification Out of Scope` --semantically_similar_to--> `Recovered-Gap Review Enforcement (none, author's own discipline)`  [INFERRED] [semantically similar]
  docs/decisions/0003-billable-classification-out-of-scope.md → docs/product/features/export.md

## Hyperedges (group relationships)
- **Design Workflow Feature Doc Role Collaboration** — claude_commands_new_feature, claude_agents_product_manager, claude_agents_ux_designer, claude_agents_technical_architect, claude_agents_senior_software_engineer, claude_agents_reviewer [EXTRACTED 1.00]
- **grill-with-docs Document Capture Flow** — claude_skills_grill_with_docs_skill, docs_glossary, docs_decisions_0000_adr_template, docs_assumptions [EXTRACTED 1.00]
- **Architecture Workflow Role Collaboration** — claude_workflows_architecture, claude_agents_technical_architect, claude_agents_senior_software_engineer, claude_agents_reviewer [EXTRACTED 1.00]
- **Discovery -> Design -> Planning Phase Gate Process** — claude_project_instructions, claude_workflows_discovery_workflow, claude_workflows_design_workflow, claude_workflows_planning_workflow [EXTRACTED 1.00]
- **Rust/Tauri Persistence Architecture Chain** — docs_decisions_0002_desktop_app_framework_and_platform_adr, docs_decisions_0004_transition_log_format_and_torn_write_scheme_adr, docs_architecture_constraints_technical_constraints [INFERRED 0.85]
- **Sum-Then-Round Export Aggregation Pattern** — docs_product_features_export_export_feature, docs_product_features_export_json_export_shape, docs_product_features_export_excel_row_structure, docs_product_features_export_rounding_strategy [EXTRACTED 1.00]
- **Time Block as shared data model across features and risk** — docs_product_features_interruption_stack_time_block, docs_product_features_interruption_stack_completion_reason, docs_product_features_task_templates_task_templates, docs_risks_r2, planning_epics_export_export_epic [INFERRED 0.85]
- **recovered-gap staleness bound vs. unenforced review tradeoff** — docs_risks_r4, docs_product_features_interruption_stack_heartbeat, planning_epics_export_export_epic [INFERRED 0.75]
- **MVP Scope: three epics + MVP definition** — docs_product_mvp_mvp_scope, planning_epics_interruption_stack_interruption_stack_epic, planning_epics_task_templates_task_templates_epic, planning_epics_export_export_epic [INFERRED 0.85]

## Communities (12 total, 1 thin omitted)

### Community 0 - "Interruption Stack Mechanic"
Cohesion: 0.11
Nodes (32): Assumption: Manual tracking still delivers 'no work forgotten', ADR 0001: Manual, Assisted Tracking for MVP, completion_reason (explicit / auto-completed-on-skip / recovered-gap), Dashboard, Gap Recovery (crash / sleep-hibernate), Global Hotkeys, 60-second Heartbeat Write, Interruption Stack (Switch / Interrupt / Return) (+24 more)

### Community 1 - "Platform & Storage Architecture"
Cohesion: 0.16
Nodes (18): Frontend Stack Constraint (Svelte + TypeScript + Ramda), Technical Constraints, Assumptions Log, Assumption: Desktop-first with global hotkeys is sufficient for effortless capture, Assumption: Author's prior Rust experience is a manageable ramp-up cost, ADR Template, ADR 0002: Desktop App Framework and Target Platform, Rust Core Surface Area (hotkeys, tray, IPC, log I/O, heartbeat, sleep/hibernate detection) (+10 more)

### Community 2 - "Review & Grilling Tooling"
Cohesion: 0.26
Nodes (14): Reviewer Agent, Technical Architect Agent, /new-adr Command, /new-epic Command, Commands README, /domain-modeling Skill, /grilling Skill, grill-with-docs Skill (+6 more)

### Community 3 - "Core Domain Glossary"
Cohesion: 0.27
Nodes (13): Assumption: Auto-completing skipped interruptions matches user intent, Assumption: Existing time-tracking tools don't solve the interruption-stack problem, Concept (Anchor product concept), Completion Reason, Glossary, Interrupt, Interruption Stack, Return to Original Task (+5 more)

### Community 4 - "Team Roles & Discovery"
Cohesion: 0.32
Nodes (12): Product Manager Agent, Agents README, Researcher Agent, Senior Software Engineer Agent, UX Designer Agent, /discovery-session Command, /new-feature Command, Definition of Ready (+4 more)

### Community 5 - "Architecture Overview Content"
Cohesion: 0.29
Nodes (11): App Framework & Platform Decision, Component Map, Data Model Scope Decision, Key Decisions, No External Services, Persisted State (On Disk), Rust Core (Tauri Backend), Svelte/TypeScript Frontend (+3 more)

### Community 6 - "Export Feature Design"
Cohesion: 0.27
Nodes (10): Assumption: Billable/non-billable classification need not be tracked in Anchor's data model, Assumption: Exact-match export aggregation is sufficient for billing, Assumption (invalidated): JSON export should stay raw, independently rounded per record, ADR 0003: Billable Classification Out of Scope, Export Date Range Selection, Excel Row Structure (grouped-then-rounded per task), Export Feature (XLSX / JSON), JSON Export Shape (sum-then-round parity with XLSX) (+2 more)

### Community 7 - "Documentation Standards"
Cohesion: 0.39
Nodes (9): Documentation Steward Agent, Documentation Standards, ADR Numbering Rule, Frontmatter Convention, Graphify Regeneration Policy, Architecture Overview Doc, Glossary Doc, Roadmap Doc (+1 more)

### Community 8 - "Workflow Governance Rules"
Cohesion: 0.39
Nodes (8): Architecture Workflow, ADR 0002 (Desktop Framework/Platform), Preference vs Constraint Guidance, Two-Pass Review Requirement, Design Workflow, Planning Workflow, Workflows README, Architecture Constraints Doc

### Community 9 - "MVP Scope & Planning"
Cohesion: 0.75
Nodes (8): Roadmap, Epic: Export (XLSX / JSON), Epic: Interruption Stack, Epic: Task Templates, Issue & GitHub Project Conventions, M1 — Core Tracking Loop, M2 — MVP Complete, Planning Overview

### Community 10 - "Project Entry Points"
Cohesion: 0.60
Nodes (5): Anchor Project Instructions (CLAUDE.md), Design Workflow, Discovery Workflow, Planning Workflow, Anchor README

## Ambiguous Edges - Review These
- `Frontmatter Convention` → `Graphify Regeneration Policy`  [AMBIGUOUS]
  .claude/docs-standards.md · relation: conceptually_related_to
- `Interruption Stack (Switch / Interrupt / Return)` → `Ideas Inbox`  [AMBIGUOUS]
  ideas/README.md · relation: conceptually_related_to

## Knowledge Gaps
- **15 isolated node(s):** `Prompts README`, `mattpocock/skills grill-with-docs (Source)`, `/grilling Skill`, `/domain-modeling Skill`, `ADR Template` (+10 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **1 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **What is the exact relationship between `Frontmatter Convention` and `Graphify Regeneration Policy`?**
  _Edge tagged AMBIGUOUS (relation: conceptually_related_to) - confidence is low._
- **What is the exact relationship between `Interruption Stack (Switch / Interrupt / Return)` and `Ideas Inbox`?**
  _Edge tagged AMBIGUOUS (relation: conceptually_related_to) - confidence is low._
- **Why does `Interruption Stack (Switch / Interrupt / Return)` connect `Interruption Stack Mechanic` to `Platform & Storage Architecture`, `Core Domain Glossary`, `Export Feature Design`, `MVP Scope & Planning`?**
  _High betweenness centrality (0.180) - this node is a cross-community bridge._
- **Why does `ADR 0002: Desktop App Framework and Target Platform` connect `Platform & Storage Architecture` to `Interruption Stack Mechanic`, `Architecture Overview Content`, `Export Feature Design`?**
  _High betweenness centrality (0.103) - this node is a cross-community bridge._
- **Why does `Concept (Anchor product concept)` connect `Core Domain Glossary` to `Interruption Stack Mechanic`, `Export Feature Design`?**
  _High betweenness centrality (0.090) - this node is a cross-community bridge._
- **What connects `ADR Numbering Rule`, `Prompts README`, `mattpocock/skills grill-with-docs (Source)` to the rest of the system?**
  _26 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Interruption Stack Mechanic` be split into smaller, more focused modules?**
  _Cohesion score 0.11088709677419355 - nodes in this community are weakly interconnected._