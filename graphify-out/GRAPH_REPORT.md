# Graph Report - .  (2026-08-01)

## Corpus Check
- 51 files · ~107,256 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 921 nodes · 1741 edges · 81 communities (64 shown, 17 thin omitted)
- Extraction: 96% EXTRACTED · 4% INFERRED · 0% AMBIGUOUS · INFERRED: 61 edges (avg confidence: 0.89)
- Token cost: 345,808 input · 0 output

## Community Hubs (Navigation)
- Tauri Command Surface
- Interruption Stack State Machine
- Agent & Command Definitions
- Checksum Framing
- Task Template Storage
- Log Writer & Torn-Write Recovery
- Visual Redesign Direction
- Export Computation
- Global Hotkey Registration
- Export Settings Persistence
- Frontend API Bridge
- Sleep & Gap Recovery
- Interruption Vocabulary
- Log Replay
- Three-Field Metadata Model
- Risk Register
- Tauri App Configuration
- Capture & Naming Concepts
- Log Format & Identity Contract
- MVP Scope Definition
- Assumptions & Build-vs-Buy
- Desktop Capability Schema
- Windows Capability Schema
- Users & Success Metrics
- Application Icon Assets
- Hotkey Settings Persistence
- Architectural Constraints
- TypeScript Configuration
- Shared Frontend Types
- Runtime Dependencies
- Capability Array Schema
- Window Target Schema
- Capability Array Schema (Win)
- Window Target Schema (Win)
- Filesystem Path Resolution
- Dashboard View
- Build Tooling Dependencies
- Capability Identifier Schema
- Capability Identifier Schema (Win)
- Remote Capability Schema
- Remote Capability Schema (Win)
- Architecture Overview
- NPM Scripts
- Planning Epics & Milestones
- Default Capability Grant
- Capture Discipline Risk
- Persistent Block Identity
- Discovery Workflow Stages
- Produra Export Mapping
- Capability Schema Root
- Schema Description Fields
- Local Capability Fields
- Capability Schema Root (Win)
- Schema Description Fields (Win)
- Local Capability Fields (Win)
- Project Workflow Docs
- Number Schema Type
- Permission Entry Schema
- Number Schema Type (Win)
- Permission Entry Schema (Win)
- Svelte Configuration
- FnOnce Trait
- HotkeyBindings Type
- TaskTemplate Type
- PathBuf Type
- Duration Type
- Box Type
- Mutex Type
- Svelte Logo Asset
- Vite Logo Asset
- Prompts Index
- Retired CompletionReason
- ADR Template
- Switch Transition
- Feature Doc Template
- ExportSettings Type

## God Nodes (most connected - your core abstractions)
1. `apply_transition()` - 27 edges
2. `InterruptionStack` - 25 edges
3. `Visual Redesign (feature doc)` - 25 edges
4. `t()` - 24 edges
5. `start()` - 24 edges
6. `AppState` - 21 edges
7. `StackView` - 19 edges
8. `Timeline Reconstruction Feature` - 18 edges
9. `Export (XLSX / JSON) Feature` - 17 edges
10. `MVP Scope` - 17 edges

## Surprising Connections (you probably didn't know these)
- `App Favicon (32x32 PNG) - unmodified Tauri scaffold default, not Anchor branding` --conceptually_related_to--> `Visual Redesign (feature doc)`  [INFERRED]
  app/static/favicon.png → docs/product/features/visual-redesign.md
- `Tauri Framework Logo (SVG) - two interlocking arcs with dots in #FFC131 yellow and #24C8DB cyan, unmodified framework branding` --conceptually_related_to--> `Visual Redesign (feature doc)`  [INFERRED]
  app/static/tauri.svg → docs/product/features/visual-redesign.md
- `Prototypes` --semantically_similar_to--> `Research`  [INFERRED] [semantically similar]
  prototypes/README.md → research/README.md
- `R11 — Decisions Made on Unverified Assumptions` --semantically_similar_to--> `Discovery Stage 5 — Formal Review`  [INFERRED] [semantically similar]
  docs/risks.md → .claude/workflows/discovery.md
- `Persisted Enum Values and Auto-Names Must Not Translate` --semantically_similar_to--> `Persisted Project→Hue Mapping`  [INFERRED] [semantically similar]
  ideas/multi-language-ui-support.md → docs/product/features/visual-redesign.md

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Time Block's Three Orthogonal Metadata Fields Plus Their Canonical Projection** — docs_decisions_0005_event_model_time_block_metadata_and_reconstruction_transitions_end_determination, docs_decisions_0005_event_model_time_block_metadata_and_reconstruction_transitions_capture_origin, docs_decisions_0005_event_model_time_block_metadata_and_reconstruction_transitions_interruption_outcome, docs_decisions_0005_event_model_time_block_metadata_and_reconstruction_transitions_derived_interruption_status, docs_glossary_completion_reason [EXTRACTED 1.00]
- **The seq-Derived Time Block Identity Contract** — docs_decisions_0004_transition_log_format_and_torn_write_scheme_sequence_number, docs_decisions_0006_stable_persistent_time_block_identity_uuidv5_seq_derivation, docs_decisions_0006_stable_persistent_time_block_identity_anchor_namespace, docs_decisions_0006_stable_persistent_time_block_identity_one_block_per_transition_invariant, docs_decisions_0006_stable_persistent_time_block_identity_snapshot_must_persist_id, docs_product_features_timeline_reconstruction_payload_carries_uuid, docs_decisions_0004_transition_log_format_and_torn_write_scheme_r14_seq_reuse [EXTRACTED 1.00]
- **Crash-Safety and Replay Flow (Append, Torn-Write, Watermark, Heartbeat)** — docs_decisions_0004_transition_log_format_and_torn_write_scheme_json_lines_format, docs_decisions_0004_transition_log_format_and_torn_write_scheme_checksum_framing, docs_decisions_0004_transition_log_format_and_torn_write_scheme_watermark_compaction, docs_decisions_0004_transition_log_format_and_torn_write_scheme_heartbeat, docs_glossary_torn_write, docs_product_features_interruption_stack_gap_recovery, docs_decisions_0005_event_model_time_block_metadata_and_reconstruction_transitions_snapshot_payload_guarantee [EXTRACTED 1.00]
- **Four UI Ideas Contending for the Same Two Windows** — ideas_visual_redesign_visual_redesign_idea, ideas_switch_between_mini_and_full_ui_switch_between_mini_and_full_ui, ideas_adjustable_timeline_view_adjustable_timeline_view, ideas_multi_language_ui_support_multi_language_ui_support [EXTRACTED 1.00]
- **Manual-Tracking Trust Risk Cluster (R3/R4/R9/R10)** — docs_risks_r3, docs_risks_r4, docs_risks_r9, docs_risks_r10, docs_product_mvp_timeline_reconstruction [EXTRACTED 1.00]
- **Visual Redesign Design System (postures, tokens, densities)** — docs_product_features_visual_redesign_two_postures_one_system, docs_product_features_visual_redesign_semantic_design_tokens, docs_product_features_visual_redesign_compact_comfortable_densities, docs_product_features_visual_redesign_non_colour_encoding, docs_product_features_visual_redesign_timeline_primary_config_demoted [EXTRACTED 1.00]
- **Anchor App Icon Exported at Multiple Sizes/Platforms** — app_src_tauri_icons_icon_icon, app_src_tauri_icons_128x128_icon, app_src_tauri_icons_128x128_2x_icon, app_src_tauri_icons_32x32_icon, app_src_tauri_icons_square107x107logo_icon, app_src_tauri_icons_square142x142logo_icon, app_src_tauri_icons_square150x150logo_icon, app_src_tauri_icons_square284x284logo_icon, app_src_tauri_icons_square30x30logo_icon, app_src_tauri_icons_square310x310logo_icon, app_src_tauri_icons_square44x44logo_icon, app_src_tauri_icons_square71x71logo_icon, app_src_tauri_icons_square89x89logo_icon, app_src_tauri_icons_storelogo_icon [INFERRED 0.95]
- **Design Workflow Feature Doc Role Collaboration** — claude_commands_new_feature, claude_agents_product_manager, claude_agents_ux_designer, claude_agents_technical_architect, claude_agents_senior_software_engineer, claude_agents_reviewer [EXTRACTED 1.00]
- **grill-with-docs Document Capture Flow** — claude_skills_grill_with_docs_skill, docs_glossary, docs_decisions_0000_adr_template, docs_assumptions [EXTRACTED 1.00]
- **Architecture Workflow Role Collaboration** — claude_workflows_architecture, claude_agents_technical_architect, claude_agents_senior_software_engineer, claude_agents_reviewer [EXTRACTED 1.00]
- **Discovery -> Design -> Planning Phase Gate Process** — claude_project_instructions, claude_workflows_discovery_workflow, claude_workflows_design_workflow, claude_workflows_planning_workflow [EXTRACTED 1.00]

## Communities (81 total, 17 thin omitted)

### Community 0 - "Tauri Command Surface"
Cohesion: 0.09
Nodes (64): ExportSettings, HotkeyBindings, apply_transition(), ClosedBlockView, complete(), create_template(), delete_template(), editing_and_deleting_a_template_does_not_change_an_already_recorded_time_block() (+56 more)

### Community 1 - "Interruption Stack State Machine"
Cohesion: 0.13
Nodes (42): complete_is_still_rejected_after_crash_recovery_with_an_open_stack(), complete_requires_empty_stack(), crashed_mid_interruption(), depth_12_interrupts_then_12_return_previous_unwinds_correctly(), depth_12_interrupts_then_return_original_skips_11(), derived_status_distinguishes_pending_from_never_interrupted(), derived_status_reports_resolved_outcomes(), interrupt() (+34 more)

### Community 2 - "Agent & Command Definitions"
Cohesion: 0.11
Nodes (43): Documentation Steward Agent, Product Manager Agent, Agents README, Researcher Agent, Reviewer Agent, Senior Software Engineer Agent, Technical Architect Agent, UX Designer Agent (+35 more)

### Community 3 - "Checksum Framing"
Cohesion: 0.13
Nodes (28): checksum_never_appears_inside_json_object(), decode_line(), detects_single_byte_corruption(), encode_line(), FramingError, missing_tab_is_no_tab_error(), round_trip(), Error (+20 more)

### Community 4 - "Task Template Storage"
Cohesion: 0.12
Nodes (26): create_appends_and_assigns_fresh_uuid(), creating_two_templates_with_identical_name_project_client_is_allowed(), delete_by_id_removes_only_that_template(), delete_of_unknown_id_returns_err(), mutate_templates(), mutate_templates_rolls_back_in_memory_if_save_fails(), mutate_templates_round_trips_via_state(), AsRef (+18 more)

### Community 5 - "Log Writer & Torn-Write Recovery"
Cohesion: 0.11
Nodes (28): a_complete_record_missing_only_its_newline_is_discarded_not_repaired(), a_log_containing_only_an_incomplete_write_truncates_to_empty(), append_assigns_increasing_sequence_numbers(), append_writes_lines_decodable_by_checksum_module(), LogWriter, opening_a_healthy_log_never_truncates_anything(), opening_a_missing_or_empty_log_is_a_no_op(), reopening_with_a_later_next_seq_continues_from_there() (+20 more)

### Community 6 - "Visual Redesign Direction"
Cohesion: 0.11
Nodes (29): Tauri + SvelteKit + TypeScript Scaffold README, Scaffold Title "Tauri + SvelteKit + Typescript App", SvelteKit App Shell (app.html), App Favicon (32x32 PNG) - unmodified Tauri scaffold default, not Anchor branding, Tauri Framework Logo (SVG) - two interlocking arcs with dots in #FFC131 yellow and #24C8DB cyan, unmodified framework branding, Bundled, Not Fetched Fonts, Capture-First, Timeline-Assisted, Compact / Comfortable Densities (+21 more)

### Community 7 - "Export Computation"
Cohesion: 0.18
Nodes (25): active_entry_in_range_is_included_with_elapsed_so_far_duration_and_is_not_mutated(), blocks_in_range(), blocks_in_range_filters_by_start_time_not_end_time(), closed_block(), ExportRow, group(), group_combines_same_task_separated_by_an_interrupting_task(), json_export() (+17 more)

### Community 8 - "Global Hotkey Registration"
Cohesion: 0.20
Nodes (19): apply_remap(), bindings(), duplicate_accelerator_across_two_actions_is_detected(), duplicate_detection_is_case_insensitive(), find_duplicate(), HotkeyAction, HotkeyState, register_bindings() (+11 more)

### Community 9 - "Export Settings Persistence"
Cohesion: 0.16
Nodes (16): ExportSettings, ExportSettingsState, mutate_export_settings(), mutate_export_settings_rolls_back_in_memory_if_save_fails(), mutate_export_settings_round_trips_via_state(), AsRef, Default, FnOnce (+8 more)

### Community 11 - "Sleep & Gap Recovery"
Cohesion: 0.15
Nodes (19): active_block(), boundary_at_exactly_the_threshold_counts_as_a_gap(), boundary_one_second_under_threshold_is_not_a_gap(), no_gap_when_last_activity_is_recent(), resolve_resume_gap(), resolves_to_recover_gap_alone_when_gap_exceeds_threshold(), AppHandle, DateTime (+11 more)

### Community 12 - "Interruption Vocabulary"
Cohesion: 0.12
Nodes (21): MVP Edit Surface: Add, Move, Resize, Edit Identity, Delete, Rejected Option G: UUIDv5 over seq + timestamp, Collision Clamping, Continue Session, Edit Identity, Interrupt, Interruption Stack, Interruption Stack Frame (+13 more)

### Community 13 - "Log Replay"
Cohesion: 0.19
Nodes (19): deliberately_corrupted_trailing_line_is_discarded_prior_lines_survive(), last_timestamp_reflects_the_last_good_line_leftover_active_included(), payload(), replay(), replay_of_missing_file_is_empty(), replay_reconstructs_stack_from_a_real_sequence(), ReplayError, ReplayResult (+11 more)

### Community 14 - "Three-Field Metadata Model"
Cohesion: 0.14
Nodes (20): A2: Auto-Completing Skipped Interruptions Matches User Intent, A8 (invalidated): JSON Export Should Round Per-Record, ADR 0005: Event Model — Time Block Metadata and Reconstruction Transitions, CaptureOrigin Field, DerivedInterruptionStatus Projection, EndDetermination Field, InterruptionOutcome Field, Amendment: The Metadata Split Is Not an On-Disk Change (+12 more)

### Community 15 - "Risk Register"
Cohesion: 0.18
Nodes (18): Discovery Stage 5 — Formal Review, Export (XLSX / JSON) Feature, Contrast and Non-Colour Encoding Only, R1 — Auto-Completed Skipped Interruptions Masking Unfinished Work, R11 — Decisions Made on Unverified Assumptions, R14 — A seq Can Be Consumed by an Append That Did Not Durably Complete, R2 — Export-Time Aggregation Fragments Billing Totals, R4 — Uncorrected Inferred End Time Undermines Billing Trust (+10 more)

### Community 16 - "Tauri App Configuration"
Cohesion: 0.11
Nodes (17): app, security, windows, build, beforeBuildCommand, beforeDevCommand, devUrl, frontendDist (+9 more)

### Community 17 - "Capture & Naming Concepts"
Cohesion: 0.14
Nodes (18): Scope Exception: settings.json Holds Templates, Hotkeys, Export Settings, A3: Exact-Match Aggregation Is Sufficient for Billing Reports, 60-Second Heartbeat Line, Wake Stops Auto-Starting (RecoverGap Only), Anchor Name, Recovered Gap, Task Template, Exact-Match Grouping Key (name, project, client) (+10 more)

### Community 18 - "Log Format & Identity Contract"
Cohesion: 0.16
Nodes (17): ADR 0004: Transition Log Format, Torn-Write Detection, and Compaction, Checksum Framed Outside the JSON Object, JSON Lines Record Encoding, R14: seq Consumed by a Non-Durable Append, Per-Line Sequence Number (seq), Watermark-Based Compaction (Clean Shutdown or N=500 Transitions), ANCHOR_NAMESPACE Constant, Architectural Invariant: One Block Per Creating Transition (+9 more)

### Community 19 - "MVP Scope Definition"
Cohesion: 0.21
Nodes (16): Discovery Exit Criteria, Discovery Stage 6 — Remediate, Durable Persistence With Gap Recovery, Exports (XLSX and JSON), Progressive Disclosure of the Interruption Stack (interruption-history panel), MVP Scope, Open Scope Trade (deferred to Planning), Pause, and Start as a First-Class Action (+8 more)

### Community 20 - "Assumptions & Build-vs-Buy"
Cohesion: 0.17
Nodes (16): A10: Snapshot Persists Unresolved Interruption Stack Frames, A14: Snapshot Persists Each Time Block's id (or Its seq), A5 (invalidated): Existing Tools Do Not Solve the Interruption Stack, A9: No Evaluated Tool Combines Depth, Root Recovery, Skip Provenance, Assumptions Log, Build-vs-Buy Record (ManicTime, Memtime, Toggl, task-stack), ADR 0001: Manual, Assisted Tracking for MVP, ADR 0002: Desktop App Framework and Target Platform (+8 more)

### Community 21 - "Desktop Capability Schema"
Cohesion: 0.13
Nodes (14): anyOf, anyOf, description, definitions, Application, Target, Value, description (+6 more)

### Community 22 - "Windows Capability Schema"
Cohesion: 0.13
Nodes (14): anyOf, anyOf, description, definitions, Application, Target, Value, description (+6 more)

### Community 23 - "Users & Success Metrics"
Cohesion: 0.14
Nodes (15): ADR 0003: Billable Classification Out of Scope, The Interrupted Billable Developer (persona), Target Users, Adjustment Rate, Capture Rate (≥90% provisional target), ManicTime (competitor evidence), Memtime (competitor evidence), Vision Non-Goals (+7 more)

### Community 24 - "Application Icon Assets"
Cohesion: 0.14
Nodes (14): App Icon 128x128@2x, App Icon 128x128, App Icon 32x32, Anchor App Icon (Master), Windows Tile Icon Square107x107Logo, Windows Tile Icon Square142x142Logo, Windows Tile Icon Square150x150Logo, Windows Tile Icon Square284x284Logo (+6 more)

### Community 25 - "Hotkey Settings Persistence"
Cohesion: 0.23
Nodes (8): HotkeyBindings, AsRef, Default, Path, Result, Self, String, save_then_load_round_trips()

### Community 26 - "Architectural Constraints"
Cohesion: 0.20
Nodes (14): Constraint: The Event Log Is the Single Source of Truth, Constraint: Svelte + TypeScript + Ramda Frontend Stack, Technical Constraints Register, active == None with a Non-Empty Stack Is a Legal State, Anchor Product Concept, Capture-First, Timeline-Assisted, Five Actions, Five Distinct Intents (Start/Switch/Interrupt/Pause/Return), Pause as a Specialised Interrupt (+6 more)

### Community 27 - "TypeScript Configuration"
Cohesion: 0.17
Nodes (11): compilerOptions, allowJs, checkJs, esModuleInterop, forceConsistentCasingInFileNames, moduleResolution, resolveJsonModule, skipLibCheck (+3 more)

### Community 28 - "Shared Frontend Types"
Cohesion: 0.20
Nodes (10): CaptureOrigin, ClosedBlock, DerivedInterruptionStatus, EndDetermination, InterruptionOutcome, StackFrame, StackView, TaskTemplate (+2 more)

### Community 29 - "Runtime Dependencies"
Cohesion: 0.20
Nodes (9): dependencies, @tauri-apps/api, @tauri-apps/plugin-dialog, @tauri-apps/plugin-opener, description, license, name, type (+1 more)

### Community 30 - "Capability Array Schema"
Cohesion: 0.20
Nodes (10): $ref, description, items, type, uniqueItems, description, items, type (+2 more)

### Community 31 - "Window Target Schema"
Cohesion: 0.20
Nodes (10): type, webviews, windows, items, description, items, type, description (+2 more)

### Community 32 - "Capability Array Schema (Win)"
Cohesion: 0.20
Nodes (10): $ref, description, items, type, uniqueItems, description, items, type (+2 more)

### Community 33 - "Window Target Schema (Win)"
Cohesion: 0.20
Nodes (10): type, webviews, windows, items, description, items, type, description (+2 more)

### Community 34 - "Filesystem Path Resolution"
Cohesion: 0.56
Nodes (9): export_settings_file_path(), log_file_path(), AppHandle, Box, Error, PathBuf, Result, settings_file_path() (+1 more)

### Community 35 - "Dashboard View"
Cohesion: 0.22
Nodes (5): ramda, svelte, updateTemplate(), if(), $lib/time

### Community 36 - "Build Tooling Dependencies"
Cohesion: 0.22
Nodes (9): devDependencies, svelte-check, @sveltejs/adapter-static, @sveltejs/kit, @sveltejs/vite-plugin-svelte, @tauri-apps/cli, @types/ramda, typescript (+1 more)

### Community 37 - "Capability Identifier Schema"
Cohesion: 0.22
Nodes (9): properties, Identifier, description, oneOf, type, identifier, remote, anyOf (+1 more)

### Community 38 - "Capability Identifier Schema (Win)"
Cohesion: 0.22
Nodes (9): properties, Identifier, description, oneOf, type, identifier, remote, anyOf (+1 more)

### Community 39 - "Remote Capability Schema"
Cohesion: 0.25
Nodes (8): description, properties, required, type, CapabilityRemote, urls, description, type

### Community 40 - "Remote Capability Schema (Win)"
Cohesion: 0.25
Nodes (8): description, properties, required, type, CapabilityRemote, urls, description, type

### Community 41 - "Architecture Overview"
Cohesion: 0.36
Nodes (8): Anchor Architecture Overview, No External Services (Local-Only, Windows-First MVP), Persisted State: JSONL Transition Log + Snapshot, Rust Core (Tauri Backend), Svelte/TypeScript Frontend (Mini Widget + Dashboard), History View, Timeline Editor, Always-On-Top Mini Widget

### Community 42 - "NPM Scripts"
Cohesion: 0.29
Nodes (7): scripts, build, check, check:watch, dev, preview, tauri

### Community 43 - "Planning Epics & Milestones"
Cohesion: 0.57
Nodes (7): Core Interruption Model, Epic: Export (XLSX / JSON), Epic: Interruption Stack, Epic: Task Templates, Issue & GitHub Project Conventions, M1 — Core Tracking Loop, Planning Overview

### Community 44 - "Default Capability Grant"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 45 - "Capture Discipline Risk"
Cohesion: 0.47
Nodes (6): A1: Fully Manual Tracking Still Delivers 'No Work Forgotten', A11: A Reconstruction Workspace Will Not Erode Capture Discipline, R10 — Reconstruction Workspace Eroding Capture Discipline, R3 — Fully Manual Tracking Depends on User Discipline, The Timeline as a Reconstruction Workspace, Five-Operation Edit Surface (add, move, resize, edit identity, delete)

### Community 46 - "Persistent Block Identity"
Cohesion: 0.40
Nodes (6): A13: Log-Lineage-Scoped Identity Is Sufficient (No Global Uniqueness), ADR 0006: Stable, Persistent Time Block Identity, Inter-Replay Reference (First in the Product), Non-Goal: Global Uniqueness Across Log Lineages, Open Tension: Undo of a Delete vs. seq-Derived Identity, Delete Must Be Either Undoable or Confirmed

### Community 47 - "Discovery Workflow Stages"
Cohesion: 0.40
Nodes (5): Discovery Workflow, Discovery Stage 1 — Elicit, Discovery Stage 2 — Deepen, Discovery Stage 3 — Challenge (inline), Discovery Stage 4 — Record

### Community 48 - "Produra Export Mapping"
Cohesion: 0.50
Nodes (5): Flat Timeline Data Model, Produra-Compatible Export Column Structure, R12 — Flat Model May Not Map Onto Produra's Hierarchy, Produra (enterprise time-booking system), Side-by-Side, Flat Chronological Timeline

### Community 49 - "Capability Schema Root"
Cohesion: 0.50
Nodes (4): description, required, type, Capability

### Community 50 - "Schema Description Fields"
Cohesion: 0.50
Nodes (4): default, description, type, description

### Community 51 - "Local Capability Fields"
Cohesion: 0.50
Nodes (4): default, description, type, local

### Community 52 - "Capability Schema Root (Win)"
Cohesion: 0.50
Nodes (4): description, required, type, Capability

### Community 53 - "Schema Description Fields (Win)"
Cohesion: 0.50
Nodes (4): default, description, type, description

### Community 54 - "Local Capability Fields (Win)"
Cohesion: 0.50
Nodes (4): default, description, type, local

### Community 55 - "Project Workflow Docs"
Cohesion: 0.67
Nodes (4): Anchor Project Instructions (CLAUDE.md), Design Workflow, Planning Workflow, Anchor README

### Community 56 - "Number Schema Type"
Cohesion: 0.67
Nodes (3): Number, anyOf, description

### Community 57 - "Permission Entry Schema"
Cohesion: 0.67
Nodes (3): PermissionEntry, anyOf, description

### Community 58 - "Number Schema Type (Win)"
Cohesion: 0.67
Nodes (3): Number, anyOf, description

### Community 59 - "Permission Entry Schema (Win)"
Cohesion: 0.67
Nodes (3): PermissionEntry, anyOf, description

## Ambiguous Edges - Review These
- `Scaffold Title "Tauri + SvelteKit + Typescript App"` → `R13 — External Design System Carries Unpriced Costs (JSX, fonts, csp:null)`  [AMBIGUOUS]
  app/src/app.html · relation: conceptually_related_to

## Knowledge Gaps
- **186 isolated node(s):** `Prompts README`, `mattpocock/skills grill-with-docs (Source)`, `/grilling Skill`, `/domain-modeling Skill`, `Anchor README` (+181 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **17 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **What is the exact relationship between `Scaffold Title "Tauri + SvelteKit + Typescript App"` and `R13 — External Design System Carries Unpriced Costs (JSX, fonts, csp:null)`?**
  _Edge tagged AMBIGUOUS (relation: conceptually_related_to) - confidence is low._
- **Why does `HotkeyBindings` connect `Tauri Command Surface` to `Frontend API Bridge`, `Shared Frontend Types`?**
  _High betweenness centrality (0.055) - this node is a cross-community bridge._
- **Why does `InterruptionStack` connect `Interruption Stack State Machine` to `Tauri Command Surface`, `Log Writer & Torn-Write Recovery`, `Log Replay`?**
  _High betweenness centrality (0.048) - this node is a cross-community bridge._
- **Why does `Export (XLSX / JSON) Feature` connect `Risk Register` to `Visual Redesign Direction`, `Interruption Vocabulary`, `Persistent Block Identity`, `Three-Field Metadata Model`, `Capture & Naming Concepts`, `Log Format & Identity Contract`, `MVP Scope Definition`, `Users & Success Metrics`?**
  _High betweenness centrality (0.022) - this node is a cross-community bridge._
- **Are the 2 inferred relationships involving `apply_transition()` (e.g. with `run()` and `handle_resume()`) actually correct?**
  _`apply_transition()` has 2 INFERRED edges - model-reasoned connections that need verification._
- **Are the 2 inferred relationships involving `Visual Redesign (feature doc)` (e.g. with `App Favicon (32x32 PNG) - unmodified Tauri scaffold default, not Anchor branding` and `Tauri Framework Logo (SVG) - two interlocking arcs with dots in #FFC131 yellow and #24C8DB cyan, unmodified framework branding`) actually correct?**
  _`Visual Redesign (feature doc)` has 2 INFERRED edges - model-reasoned connections that need verification._
- **What connects `Prompts README`, `mattpocock/skills grill-with-docs (Source)`, `/grilling Skill` to the rest of the system?**
  _198 weakly-connected nodes found - possible documentation gaps or missing edges._