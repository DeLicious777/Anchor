# Graph Report - .  (2026-08-07)

## Corpus Check
- 133 files · ~163,897 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 1075 nodes · 2127 edges · 82 communities (72 shown, 10 thin omitted)
- Extraction: 96% EXTRACTED · 4% INFERRED · 0% AMBIGUOUS · INFERRED: 79 edges (avg confidence: 0.84)
- Token cost: 358,393 input · 0 output

## Community Hubs (Navigation)
- Tauri Command Surface
- Interruption State Machine
- Snapshot and Compaction
- Log Replay and Recovery
- Team Agent Definitions
- Log Framing and Checksums
- Export and Billing Output
- Task Template Store
- Frontend IPC Layer
- Global Hotkey Bindings
- Export Settings Store
- Pause and Gap Semantics
- Documentation Governance
- Gap Recovery Rules
- Tauri App Configuration
- Visual Redesign
- Vision
- Time Block Metadata Model
- Reconstruction Editing Rules
- definitions
- definitions
- Timeline Reconstruction
- Anchor App Icon (Master)
- HotkeyBindings
- Anchor Design Principles
- +page.svelte
- $lib/types
- compilerOptions
- Interruption History (feature)
- Manual Time Block Entry
- Anchor Project Instructions
- package.json
- Visual Redesign — Layout,
- paths.rs
- Timeline Editor (feature)
- Roadmap
- permissions
- webviews
- permissions
- webviews
- Anchor Architecture Overview
- ADR 0004: Transition Log
- devDependencies
- properties
- properties
- R3 — Manual Tracking
- CapabilityRemote
- CapabilityRemote
- scripts
- Export (XLSX / JSON)
- Epic: Visual Redesign
- Planning Overview
- default.json
- ADR 0002: Desktop App
- Explicit Persisted Theme Preference
- Capability
- description
- local
- Capability
- description
- local
- Anchor Project Instructions (CLAUDE.md)
- Number
- PermissionEntry
- Number
- PermissionEntry
- Task Templates Feature
- Evidence Bundle
- svelte.config.js
- A5: Existing Tools Do
- Edit Identity
- Svelte Framework Logo
- Vite Framework Logo
- Prompts README
- ADR Template
- Feature Doc Template
- Produra (enterprise time-booking system)

## God Nodes (most connected - your core abstractions)
1. `t()` - 44 edges
2. `apply_transition()` - 41 edges
3. `InterruptionStack` - 37 edges
4. `start()` - 37 edges
5. `$lib/api` - 34 edges
6. `AppState` - 27 edges
7. `StackView` - 24 edges
8. `replay()` - 23 edges
9. `interrupt()` - 20 edges
10. `TransitionPayload` - 19 edges

## Surprising Connections (you probably didn't know these)
- `App Favicon (32x32 PNG) - unmodified Tauri scaffold default, not Anchor branding` --conceptually_related_to--> `Visual Redesign`  [INFERRED]
  app/static/favicon.png → docs/product/features/visual-redesign.md
- `Tauri Framework Logo (SVG) - two interlocking arcs with dots in #FFC131 yellow and #24C8DB cyan, unmodified framework branding` --conceptually_related_to--> `Visual Redesign`  [INFERRED]
  app/static/tauri.svg → docs/product/features/visual-redesign.md
- `Scaffold Title "Tauri + SvelteKit + Typescript App"` --conceptually_related_to--> `R13 — External Design System's Unpriced Costs`  [AMBIGUOUS]
  app/src/app.html → docs/risks.md
- `Persisted Enum Values and Auto-Names Must Not Translate` --semantically_similar_to--> `Persisted Project-to-Hue Mapping`  [INFERRED] [semantically similar]
  ideas/multi-language-ui-support.md → docs/product/features/visual-redesign.md
- `Prototypes` --semantically_similar_to--> `Research`  [INFERRED] [semantically similar]
  prototypes/README.md → research/README.md

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Design Workflow Feature Doc Role Collaboration** — claude_commands_new_feature, claude_agents_product_manager, claude_agents_ux_designer, claude_agents_technical_architect, claude_agents_senior_software_engineer, claude_agents_reviewer [EXTRACTED 1.00]
- **grill-with-docs Document Capture Flow** — claude_skills_grill_with_docs_skill, docs_glossary, docs_decisions_0000_adr_template, docs_assumptions [EXTRACTED 1.00]
- **Architecture Workflow Role Collaboration** — claude_workflows_architecture, claude_agents_technical_architect, claude_agents_senior_software_engineer, claude_agents_reviewer [EXTRACTED 1.00]
- **Discovery -> Design -> Planning Phase Gate Process** — claude_project_instructions, claude_workflows_discovery_workflow, claude_workflows_design_workflow, claude_workflows_planning_workflow [EXTRACTED 1.00]
- **Four UI Ideas Contending for the Same Two Windows** — ideas_visual_redesign_visual_redesign_idea, ideas_switch_between_mini_and_full_ui_switch_between_mini_and_full_ui, ideas_adjustable_timeline_view_adjustable_timeline_view, ideas_multi_language_ui_support_multi_language_ui_support [EXTRACTED 1.00]
- **Anchor App Icon Exported at Multiple Sizes/Platforms** — app_src_tauri_icons_icon_icon, app_src_tauri_icons_128x128_icon, app_src_tauri_icons_128x128_2x_icon, app_src_tauri_icons_32x32_icon, app_src_tauri_icons_square107x107logo_icon, app_src_tauri_icons_square142x142logo_icon, app_src_tauri_icons_square150x150logo_icon, app_src_tauri_icons_square284x284logo_icon, app_src_tauri_icons_square30x30logo_icon, app_src_tauri_icons_square310x310logo_icon, app_src_tauri_icons_square44x44logo_icon, app_src_tauri_icons_square71x71logo_icon, app_src_tauri_icons_square89x89logo_icon, app_src_tauri_icons_storelogo_icon [INFERRED 0.95]
- **The Compaction Snapshot Payload Contract** — docs_assumptions_a10_snapshot_persists_stack_frames, docs_assumptions_a14_snapshot_persists_block_ids, docs_assumptions_a15_snapshot_persists_last_durable_write, docs_decisions_0004_transition_log_format_and_torn_write_scheme_watermark_compaction [EXTRACTED 1.00]
- **The Three-Field Time Block Metadata Model and Its Canonical Projection** — docs_decisions_0005_event_model_time_block_metadata_and_reconstruction_transitions_end_determination, docs_decisions_0005_event_model_time_block_metadata_and_reconstruction_transitions_capture_origin, docs_decisions_0005_event_model_time_block_metadata_and_reconstruction_transitions_interruption_outcome, docs_decisions_0005_event_model_time_block_metadata_and_reconstruction_transitions_derived_interruption_status, docs_glossary_time_block [EXTRACTED 1.00]
- **Gap Recovery Flow Across Crash and Sleep/Wake Paths** — docs_product_features_interruption_stack_unified_gap_detection, docs_product_features_interruption_stack_heartbeat, docs_decisions_0007_auto_resume_after_a_short_gap_three_zone_gap_rule, docs_glossary_recovered_gap, docs_product_features_pause_paused_derived_state_rule [INFERRED 0.85]
- **The Five Reconstruction Operations** — docs_product_features_timeline_reconstruction_add, docs_product_features_timeline_reconstruction_move, docs_product_features_timeline_reconstruction_resize, docs_product_features_timeline_reconstruction_edit_identity, docs_product_features_timeline_reconstruction_delete [EXTRACTED 1.00]
- **M3 Blocker-Based Split** — planning_milestones_m3a_backend_integration, planning_milestones_m3b_visual_foundation, planning_milestones_m3c_surfaces, planning_milestones_split_by_blocker, docs_product_features_visual_redesign_design_system_not_in_repo [EXTRACTED 1.00]
- **Record-Honesty Provenance Chain** — docs_product_features_timeline_reconstruction_capture_origin, docs_product_features_timeline_reconstruction_end_determination, docs_product_features_visual_redesign_non_colour_encoding, docs_risks_r10, docs_risks_r4 [INFERRED 0.85]

## Communities (82 total, 10 thin omitted)

### Community 0 - "Tauri Command Surface"
Cohesion: 0.08
Nodes (84): a_command_surfaces_the_domain_error_rather_than_the_ui_preventing_it(), add_block(), apply_transition(), ClosedBlockView, complete(), create_template(), create_update_delete_template_round_trip(), delete_block() (+76 more)

### Community 1 - "Interruption State Machine"
Cohesion: 0.09
Nodes (68): a_block_may_not_end_in_the_future_or_end_before_it_starts(), add(), add_creates_an_independent_manual_entry_block(), an_added_block_consumes_its_auto_name(), complete_is_still_rejected_after_crash_recovery_with_an_open_stack(), complete_requires_empty_stack(), crashed_mid_interruption(), delete_is_permitted_once_the_frame_is_resolved() (+60 more)

### Community 2 - "Snapshot and Compaction"
Cohesion: 0.08
Nodes (40): a_missing_or_corrupt_snapshot_is_none_rather_than_an_error(), a_snapshot_from_a_different_version_is_refused(), compact(), CompactionTrigger, counts_toward_compaction(), only_user_triggered_lifecycle_transitions_count_toward_the_threshold(), preserves_stack_frames_and_time_block_ids_exactly(), round_trips_through_disk() (+32 more)

### Community 3 - "Log Replay and Recovery"
Cohesion: 0.08
Nodes (46): deliberately_corrupted_trailing_line_is_discarded_prior_lines_survive(), last_timestamp_reflects_the_last_good_line_leftover_active_included(), no_two_time_blocks_share_an_id_after_a_replay(), payload(), replay(), replay_from_a_snapshot_over_a_truncated_log_cannot_supply_a_gap_recovery_bound(), replay_of_missing_file_is_empty(), replay_reconstructs_stack_from_a_real_sequence() (+38 more)

### Community 4 - "Team Agent Definitions"
Cohesion: 0.13
Nodes (38): Documentation Steward Agent, Product Manager Agent, Agents README, Researcher Agent, Reviewer Agent, Senior Software Engineer Agent, Technical Architect Agent, UX Designer Agent (+30 more)

### Community 5 - "Log Framing and Checksums"
Cohesion: 0.14
Nodes (31): checksum_never_appears_inside_json_object(), decode_line(), detects_single_byte_corruption(), encode_line(), FramingError, missing_tab_is_no_tab_error(), round_trip(), Error (+23 more)

### Community 6 - "Export and Billing Output"
Cohesion: 0.16
Nodes (31): a_deleted_block_leaves_the_history_view_the_export_and_the_replayed_state(), a_genuinely_zero_duration_block_still_bills_nothing(), a_sub_second_block_still_bills_one_whole_interval(), active_entry_in_range_is_included_with_elapsed_so_far_duration_and_is_not_mutated(), blocks_in_range(), blocks_in_range_filters_by_start_time_not_end_time(), closed_block(), ExportRow (+23 more)

### Community 7 - "Task Template Store"
Cohesion: 0.12
Nodes (22): create_appends_and_assigns_fresh_uuid(), creating_two_templates_with_identical_name_project_client_is_allowed(), delete_by_id_removes_only_that_template(), delete_of_unknown_id_returns_err(), mutate_templates_rolls_back_in_memory_if_save_fails(), mutate_templates_round_trips_via_state(), AsRef, Into (+14 more)

### Community 9 - "Global Hotkey Bindings"
Cohesion: 0.20
Nodes (19): apply_remap(), bindings(), duplicate_accelerator_across_two_actions_is_detected(), duplicate_detection_is_case_insensitive(), find_duplicate(), HotkeyAction, HotkeyState, register_bindings() (+11 more)

### Community 10 - "Export Settings Store"
Cohesion: 0.16
Nodes (16): ExportSettings, ExportSettingsState, mutate_export_settings(), mutate_export_settings_rolls_back_in_memory_if_save_fails(), mutate_export_settings_round_trips_via_state(), AsRef, Default, FnOnce (+8 more)

### Community 11 - "Pause and Gap Semantics"
Cohesion: 0.12
Nodes (22): EndDetermination, active == None with a Non-Empty Stack Is a Legal State, Pause Is a Specialised Interrupt with No Successor, CONTINUITY_THRESHOLD (90 seconds), The Gap Itself Is Never Counted as Work, RESUME_LIMIT (1 hour), Three-Zone Gap Rule (Continuity / Resume / Deliberate), Anchor Name (+14 more)

### Community 12 - "Documentation Governance"
Cohesion: 0.12
Nodes (21): When Not to Write an ADR (constrains-work-outside-the-feature test), Evidence Reopens a Decision, Preference Does Not, Documentation Governance Model, Constraint: The Event Log Is the Single Source of Truth, Future Import as an Input Path Producing Transitions, A10: Snapshot Persists Unresolved Interruption Stack Frames, A13: Replay-Stable Identity Scoped to One Log Lineage Is Sufficient, A14: Snapshot Persists Each Time Block's id (+13 more)

### Community 13 - "Gap Recovery Rules"
Cohesion: 0.24
Nodes (17): a_gap_under_the_continuity_threshold_is_not_a_gap_at_all(), a_long_gap_closes_without_resuming(), a_short_gap_closes_the_block_and_resumes_the_same_work(), active(), GapResolution, resolve(), DateTime, Option (+9 more)

### Community 14 - "Tauri App Configuration"
Cohesion: 0.11
Nodes (17): app, security, windows, build, beforeBuildCommand, beforeDevCommand, devUrl, frontendDist (+9 more)

### Community 15 - "Visual Redesign"
Cohesion: 0.16
Nodes (16): App Favicon (32x32 PNG) - unmodified Tauri scaffold default, not Anchor branding, Tauri Framework Logo (SVG) - two interlocking arcs with dots in #FFC131 yellow and #24C8DB cyan, unmodified framework branding, Move Operation, Resize Operation, Capture-First, Timeline-Assisted, Compact / Comfortable Density and 24x24 Target Floor, Two Postures, One System (Glanceable vs Inspectable), Visual Redesign (+8 more)

### Community 16 - "Vision"
Cohesion: 0.13
Nodes (16): ADR 0003: Billable Classification Out of Scope, The Interrupted Billable Developer (persona), Target Users, Adjustment Rate, Capture Latency (≤1s provisional target), Capture Rate (≥90% provisional target), ManicTime (competitor evidence), Memtime (competitor evidence) (+8 more)

### Community 17 - "Time Block Metadata Model"
Cohesion: 0.15
Nodes (16): CaptureOrigin, DerivedInterruptionStatus, InterruptionOutcome, Three Orthogonal Metadata Fields on TimeBlock, Inter-Replay Reference, Adjustment Rate, Capture Rate, Frame dismissal (+8 more)

### Community 18 - "Reconstruction Editing Rules"
Cohesion: 0.13
Nodes (16): Collision Clamping, Confirmed Delete Without Undo, Deferred Commit (Preferred Undo Evolution), Durable Undo (Tombstone + Restore), Rejected, The Editor Clamps; The Domain Rejects, Surface as Thin Adapter Over the State Machine, Hotkey-First, Discoverable-Second, Pause and First-Class Start (+8 more)

### Community 19 - "definitions"
Cohesion: 0.13
Nodes (14): anyOf, anyOf, description, definitions, Application, Target, Value, description (+6 more)

### Community 20 - "definitions"
Cohesion: 0.13
Nodes (14): anyOf, anyOf, description, definitions, Application, Target, Value, description (+6 more)

### Community 21 - "Timeline Reconstruction"
Cohesion: 0.19
Nodes (15): Delete Operation, Edit Identity Operation, History View Integration, Reconstruction Payloads Carry the Derived Uuid, Rename Transition, Three-Tier Block Editability Rule, Timeline Reconstruction, Two Orderings, Each Authoritative in Its Domain (+7 more)

### Community 22 - "Anchor App Icon (Master)"
Cohesion: 0.14
Nodes (14): App Icon 128x128@2x, App Icon 128x128, App Icon 32x32, Anchor App Icon (Master), Windows Tile Icon Square107x107Logo, Windows Tile Icon Square142x142Logo, Windows Tile Icon Square150x150Logo, Windows Tile Icon Square284x284Logo (+6 more)

### Community 23 - "HotkeyBindings"
Cohesion: 0.23
Nodes (8): HotkeyBindings, AsRef, Default, Path, Result, Self, String, save_then_load_round_trips()

### Community 24 - "Anchor Design Principles"
Cohesion: 0.18
Nodes (14): active == None with a Non-Empty Stack Is a Legal State, Anchor Product Concept, Build-vs-Buy Record (ManicTime, Memtime, Toggl, task-stack), Capture-First, Timeline-Assisted, Five Actions, Five Distinct Intents (Start/Switch/Interrupt/Pause/Return), MVP Edit Surface: Add, Move, Resize, Edit Identity, Delete, Capture-First, Timeline-Assisted, Reconstruction Workspace (+6 more)

### Community 25 - "+page.svelte"
Cohesion: 0.18
Nodes (7): svelte, editIdentity(), resizeBlock(), updateTemplate(), $lib/time, fromLocalInputValue(), if()

### Community 26 - "$lib/types"
Cohesion: 0.17
Nodes (13): $lib/types, CaptureOrigin, ClosedBlock, DerivedInterruptionStatus, EndDetermination, ExportSettings, HotkeyBindings, InterruptionOutcome (+5 more)

### Community 27 - "compilerOptions"
Cohesion: 0.17
Nodes (11): compilerOptions, allowJs, checkJs, esModuleInterop, forceConsistentCasingInFileNames, moduleResolution, resolveJsonModule, skipLibCheck (+3 more)

### Community 28 - "Interruption History (feature)"
Cohesion: 0.17
Nodes (12): A17: A Disclosure the User Must Open Will Be Opened When It Matters, History View, Interruption History (glossary term), Interruption Stack, Progressive Disclosure (of the stack), Return preview, Time Block, Timeline (+4 more)

### Community 29 - "Manual Time Block Entry"
Cohesion: 0.20
Nodes (12): Persisted Project-to-Hue Mapping, Timeline Primary, Configuration Demoted, Flat Timeline Data Model, MVP Scope, R2 — Export Aggregation Fragmentation, The Timeline as a Reconstruction Workspace, Flat Chronological Timeline Rendering, Five-Operation Edit Surface (add, move, resize, edit identity, delete) (+4 more)

### Community 30 - "Anchor Project Instructions"
Cohesion: 0.20
Nodes (11): Decisions surviving grilling become sequential ADRs, grill-with-docs skill, source-command-discovery-session skill, ADR Numbering and Append-Only Rule, Definition of Ready, Frontmatter Convention, Graphify Regeneration Cadence, Multidisciplinary Agent Team (+3 more)

### Community 31 - "package.json"
Cohesion: 0.18
Nodes (10): dependencies, ramda, @tauri-apps/api, @tauri-apps/plugin-dialog, @tauri-apps/plugin-opener, description, license, name (+2 more)

### Community 32 - "Visual Redesign — Layout,"
Cohesion: 0.18
Nodes (11): Tauri + SvelteKit + TypeScript Scaffold README, Scaffold Title "Tauri + SvelteKit + Typescript App", SvelteKit App Shell (app.html), MVP Build Order, Open Scope Trade Deferred to Planning, Alignment, Not Re-Planning, Multi-Language UI Support (idea), A Strings Layer, Not a Localization Program (+3 more)

### Community 33 - "paths.rs"
Cohesion: 0.56
Nodes (10): export_settings_file_path(), log_file_path(), AppHandle, Box, Error, PathBuf, Result, settings_file_path() (+2 more)

### Community 34 - "Timeline Editor (feature)"
Cohesion: 0.18
Nodes (11): Cluster, Collision Clamping, Context block, Marker gutter, Today mode / Range mode, Proportionality Is Absolute; Interaction Degrades, The Editor Clamps; the Domain Rejects, Membership Governs the Set; Occupancy Governs the Canvas (+3 more)

### Community 35 - "Roadmap"
Cohesion: 0.20
Nodes (10): Discovery Workflow, Discovery Exit Criteria, Discovery Stage 1 — Elicit, Discovery Stage 2 — Deepen, Discovery Stage 3 — Challenge (inline), Discovery Stage 4 — Record, Discovery Stage 5 — Formal Review, Discovery Stage 6 — Remediate (+2 more)

### Community 36 - "permissions"
Cohesion: 0.20
Nodes (10): $ref, description, items, type, uniqueItems, description, items, type (+2 more)

### Community 37 - "webviews"
Cohesion: 0.20
Nodes (10): type, webviews, windows, items, description, items, type, description (+2 more)

### Community 38 - "permissions"
Cohesion: 0.20
Nodes (10): $ref, description, items, type, uniqueItems, description, items, type (+2 more)

### Community 39 - "webviews"
Cohesion: 0.20
Nodes (10): type, webviews, windows, items, description, items, type, description (+2 more)

### Community 40 - "Anchor Architecture Overview"
Cohesion: 0.24
Nodes (10): Constraint: Svelte + TypeScript + Ramda Frontend Stack, Anchor Architecture Overview, No External Services (Local-Only, Windows-First MVP), Persisted State: JSONL Transition Log + Snapshot, Rust Core (Tauri Backend), Svelte/TypeScript Frontend (Mini Widget + Dashboard), A18: The Always-On-Top Widget Makes a Forgotten Pause Self-Correcting, Constraint (vs. Preference) (+2 more)

### Community 41 - "ADR 0004: Transition Log"
Cohesion: 0.22
Nodes (10): ADR 0004: Transition Log Format, Torn-Write Detection, and Compaction, Compaction Trigger: Clean Shutdown or 500 User-Triggered Transitions, JSONL with Checksum Framed Outside the JSON Object, Sequence Number (seq), Watermark-Based Compaction and Replay, R14: A seq Consumed by a Non-Durable Append, Torn Write, Principle 4: Only Materialised State Survives (+2 more)

### Community 42 - "devDependencies"
Cohesion: 0.22
Nodes (9): devDependencies, svelte-check, @sveltejs/adapter-static, @sveltejs/kit, @sveltejs/vite-plugin-svelte, @tauri-apps/cli, @types/ramda, typescript (+1 more)

### Community 43 - "properties"
Cohesion: 0.22
Nodes (9): properties, Identifier, description, oneOf, type, identifier, remote, anyOf (+1 more)

### Community 44 - "properties"
Cohesion: 0.22
Nodes (9): properties, Identifier, description, oneOf, type, identifier, remote, anyOf (+1 more)

### Community 45 - "R3 — Manual Tracking"
Cohesion: 0.25
Nodes (9): Add Operation, CaptureOrigin Provenance Model, EndDetermination Travels With the End Value, Contrast and Non-Colour Encoding Only, R10 — Reconstruction Erodes Capture Discipline, R19 — A Forgotten Pause Is Untracked Work, R3 — Manual Tracking Depends on User Discipline, A Hotkey Binding Cannot Ship Dormant (+1 more)

### Community 46 - "CapabilityRemote"
Cohesion: 0.25
Nodes (8): description, properties, required, type, CapabilityRemote, urls, description, type

### Community 47 - "CapabilityRemote"
Cohesion: 0.25
Nodes (8): description, properties, required, type, CapabilityRemote, urls, description, type

### Community 48 - "scripts"
Cohesion: 0.29
Nodes (7): scripts, build, check, check:watch, dev, preview, tauri

### Community 49 - "Export (XLSX / JSON)"
Cohesion: 0.38
Nodes (7): A3: Export-Time Aggregation by Exact Name/Project/Client Match Is Sufficient, View range vs. Export range, Export (XLSX / JSON), No Metadata Field May Become an Aggregation Key, Range Membership by a Block's start, First Sum Then Round, Shared View Range Over Timeline and History View; Export Decoupled

### Community 50 - "Epic: Visual Redesign"
Cohesion: 0.33
Nodes (7): Design System Not in This Repository, Tokens Transfer, Components Are Rebuilt, R13 — External Design System's Unpriced Costs, Epic: Visual Redesign, M3 — Editable Timeline, M3b — Visual Foundation, M3 Split by Blocker, Not by Feature

### Community 51 - "Planning Overview"
Cohesion: 0.38
Nodes (7): Graphify Regeneration (Oldest Open Item), Epic: Export (XLSX / JSON), Epic: Interruption Stack, Epic: Task Templates, Issue and GitHub Project Conventions, Milestones, Planning Overview

### Community 52 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 53 - "ADR 0002: Desktop App"
Cohesion: 0.50
Nodes (5): ADR 0001: Manual, Assisted Tracking for MVP, ADR 0002: Desktop App Framework and Target Platform, Rust Core Surface Area (hotkeys, tray, IPC, log I/O, heartbeat, sleep/hibernate detection), Tauri + Svelte/TypeScript Decision, Two-Window Architecture (mini widget + dashboard)

### Community 54 - "Explicit Persisted Theme Preference"
Cohesion: 0.40
Nodes (5): Explicit Persisted Theme Preference (Not OS-Follow), Semantic Design Tokens With Two Value Sets, Theme Applied Before First Paint, theme.json as a Fourth Settings File, Widget Light-Theme Edge Treatment

### Community 55 - "Capability"
Cohesion: 0.50
Nodes (4): description, required, type, Capability

### Community 56 - "description"
Cohesion: 0.50
Nodes (4): default, description, type, description

### Community 57 - "local"
Cohesion: 0.50
Nodes (4): default, description, type, local

### Community 58 - "Capability"
Cohesion: 0.50
Nodes (4): description, required, type, Capability

### Community 59 - "description"
Cohesion: 0.50
Nodes (4): default, description, type, description

### Community 60 - "local"
Cohesion: 0.50
Nodes (4): default, description, type, local

### Community 61 - "Anchor Project Instructions (CLAUDE.md)"
Cohesion: 0.67
Nodes (4): Anchor Project Instructions (CLAUDE.md), Design Workflow, Planning Workflow, Anchor README

### Community 62 - "Number"
Cohesion: 0.67
Nodes (3): Number, anyOf, description

### Community 63 - "PermissionEntry"
Cohesion: 0.67
Nodes (3): PermissionEntry, anyOf, description

### Community 64 - "Number"
Cohesion: 0.67
Nodes (3): Number, anyOf, description

### Community 65 - "PermissionEntry"
Cohesion: 0.67
Nodes (3): PermissionEntry, anyOf, description

### Community 66 - "Task Templates Feature"
Cohesion: 0.67
Nodes (3): Task Templates Feature, Template Edits Do Not Propagate to Recorded Time Blocks, Quick-Input Autocomplete Invocation

### Community 67 - "Evidence Bundle"
Cohesion: 0.67
Nodes (3): Red Abort — Do Not Relaunch, Evidence Bundle, Validated Baseline Tag

## Ambiguous Edges - Review These
- `Scaffold Title "Tauri + SvelteKit + Typescript App"` → `R13 — External Design System's Unpriced Costs`  [AMBIGUOUS]
  app/src/app.html · relation: conceptually_related_to
- `A3: Export-Time Aggregation by Exact Name/Project/Client Match Is Sufficient` → `Export (XLSX / JSON)`  [AMBIGUOUS]
  docs/product/features/export.md · relation: references

## Knowledge Gaps
- **212 isolated node(s):** `name`, `version`, `description`, `type`, `dev` (+207 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **10 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **What is the exact relationship between `Scaffold Title "Tauri + SvelteKit + Typescript App"` and `R13 — External Design System's Unpriced Costs`?**
  _Edge tagged AMBIGUOUS (relation: conceptually_related_to) - confidence is low._
- **What is the exact relationship between `A3: Export-Time Aggregation by Exact Name/Project/Client Match Is Sufficient` and `Export (XLSX / JSON)`?**
  _Edge tagged AMBIGUOUS (relation: references) - confidence is low._
- **Why does `InterruptionStack` connect `Interruption State Machine` to `Tauri Command Surface`, `Snapshot and Compaction`, `Log Replay and Recovery`?**
  _High betweenness centrality (0.058) - this node is a cross-community bridge._
- **Why does `TransitionPayload` connect `Snapshot and Compaction` to `Tauri Command Surface`, `Interruption State Machine`, `Log Replay and Recovery`, `Log Framing and Checksums`, `Gap Recovery Rules`?**
  _High betweenness centrality (0.025) - this node is a cross-community bridge._
- **Why does `apply_transition()` connect `Tauri Command Surface` to `Interruption State Machine`, `Snapshot and Compaction`, `Log Replay and Recovery`, `Export and Billing Output`?**
  _High betweenness centrality (0.020) - this node is a cross-community bridge._
- **Are the 7 inferred relationships involving `apply_transition()` (e.g. with `run()` and `handle_resume()`) actually correct?**
  _`apply_transition()` has 7 INFERRED edges - model-reasoned connections that need verification._
- **What connects `name`, `version`, `description` to the rest of the system?**
  _245 weakly-connected nodes found - possible documentation gaps or missing edges._