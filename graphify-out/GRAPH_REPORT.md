# Graph Report - .  (2026-07-27)

## Corpus Check
- 64 files · ~68,959 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 760 nodes · 1447 edges · 47 communities (45 shown, 2 thin omitted)
- Extraction: 96% EXTRACTED · 4% INFERRED · 0% AMBIGUOUS · INFERRED: 54 edges (avg confidence: 0.85)
- Token cost: 0 input · 143,433 output

## Community Hubs (Navigation)
- Tauri Commands & App State
- Product & Architecture Docs
- Feature Docs, Glossary & Risks
- Transition Log Format & Data Model
- Claude Agent/Workflow Config
- Interruption Stack State Machine
- Task Templates Backend
- Frontend API & UI
- Log Replay & App State Init
- Export Logic
- Build Config & Paths
- Hotkey Registration & Remap
- Export Settings Persistence
- NPM Dependencies
- Sleep/Hibernate Detection
- App Icons & Logos
- Tauri App Config
- Tauri Desktop Capability Schema (generated)
- Tauri Windows Capability Schema (generated)
- Hotkey Bindings Settings
- TypeScript Config
- Desktop Schema: Permissions/Platforms
- Desktop Schema: Windows/Webviews
- Windows Schema: Permissions/Platforms
- Windows Schema: Windows/Webviews
- Desktop Schema: Identifier/Remote
- Windows Schema: Identifier/Remote
- Desktop Schema: Capability Remote
- Windows Schema: Capability Remote
- Tauri Default Capability
- Desktop Schema: Capability Def
- Desktop Schema: Description Field
- Desktop Schema: Local Field
- Windows Schema: Capability Def
- Windows Schema: Description Field
- Windows Schema: Local Field
- Desktop Schema: Number Def
- Desktop Schema: Permission Entry
- Windows Schema: Number Def
- Windows Schema: Permission Entry
- Svelte Config
- Prompts Readme

## God Nodes (most connected - your core abstractions)
1. `Interruption Stack Feature Doc` - 23 edges
2. `apply_transition()` - 22 edges
3. `InterruptionStack` - 22 edges
4. `AppState` - 19 edges
5. `t()` - 18 edges
6. `start()` - 18 edges
7. `mutate_templates()` - 18 edges
8. `StackView` - 17 edges
9. `Glossary Doc` - 16 edges
10. `grill-with-docs Skill` - 15 edges

## Surprising Connections (you probably didn't know these)
- `Prototypes` --semantically_similar_to--> `Research`  [INFERRED] [semantically similar]
  prototypes/README.md → research/README.md
- `Research` --conceptually_related_to--> `Vision`  [INFERRED]
  research/README.md → docs/vision/vision.md
- `Anchor App README (Tauri + SvelteKit + TypeScript)` --conceptually_related_to--> `ADR 0002: Desktop App Framework and Platform`  [INFERRED]
  app/README.md → docs/glossary.md
- `Glossary Doc` --conceptually_related_to--> `Constraint (vs. Preference)`  [EXTRACTED]
  .claude/agents/documentation-steward.md → docs/glossary.md
- `R3: Fully Manual Tracking Depends on User Discipline` --references--> `Assumptions Doc`  [EXTRACTED]
  docs/risks.md → .claude/agents/product-manager.md

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Interruption Stack UX Surfaces (Mini Widget, Dashboard, Hotkeys)** — docs_product_features_interruption_stack, docs_product_features_interruption_stack_mini_widget, docs_product_features_interruption_stack_dashboard, docs_product_features_interruption_stack_global_hotkeys [EXTRACTED 0.90]
- **Naming Drift Risk Mitigation Chain (R2, Task Templates, Anchor Name, R8)** — docs_risks_r2, docs_product_features_task_templates, docs_glossary_anchor_name, docs_risks_r8 [INFERRED 0.85]
- **Durability/Crash-Safety Mechanisms (Append-only Log, Heartbeat, Torn Write, Compaction Risk)** — docs_product_features_interruption_stack_append_only_transition_log, docs_product_features_interruption_stack_heartbeat, docs_glossary_torn_write, docs_risks_r7 [INFERRED 0.85]
- **Anchor App Icon Exported at Multiple Sizes/Platforms** — app_src_tauri_icons_icon_icon, app_src_tauri_icons_128x128_icon, app_src_tauri_icons_128x128_2x_icon, app_src_tauri_icons_32x32_icon, app_src_tauri_icons_square107x107logo_icon, app_src_tauri_icons_square142x142logo_icon, app_src_tauri_icons_square150x150logo_icon, app_src_tauri_icons_square284x284logo_icon, app_src_tauri_icons_square30x30logo_icon, app_src_tauri_icons_square310x310logo_icon, app_src_tauri_icons_square44x44logo_icon, app_src_tauri_icons_square71x71logo_icon, app_src_tauri_icons_square89x89logo_icon, app_src_tauri_icons_storelogo_icon [INFERRED 0.95]
- **Unmodified Svelte+Tauri+Vite Scaffold Default Assets** — app_static_favicon_icon, app_static_svelte_logo, app_static_tauri_logo, app_static_vite_logo [INFERRED 0.85]
- **Design Workflow Feature Doc Role Collaboration** — claude_commands_new_feature, claude_agents_product_manager, claude_agents_ux_designer, claude_agents_technical_architect, claude_agents_senior_software_engineer, claude_agents_reviewer [EXTRACTED 1.00]
- **grill-with-docs Document Capture Flow** — claude_skills_grill_with_docs_skill, docs_glossary, docs_decisions_0000_adr_template, docs_assumptions [EXTRACTED 1.00]
- **Architecture Workflow Role Collaboration** — claude_workflows_architecture, claude_agents_technical_architect, claude_agents_senior_software_engineer, claude_agents_reviewer [EXTRACTED 1.00]
- **Discovery -> Design -> Planning Phase Gate Process** — claude_project_instructions, claude_workflows_discovery_workflow, claude_workflows_design_workflow, claude_workflows_planning_workflow [EXTRACTED 1.00]
- **Rust/Tauri Persistence Architecture Chain** — docs_decisions_0002_desktop_app_framework_and_platform_adr, docs_decisions_0004_transition_log_format_and_torn_write_scheme_adr, docs_architecture_constraints_technical_constraints [INFERRED 0.85]
- **Sum-Then-Round Export Aggregation Pattern** — docs_product_features_export_export_feature, docs_product_features_export_json_export_shape, docs_product_features_export_excel_row_structure, docs_product_features_export_rounding_strategy [EXTRACTED 1.00]
- **MVP Scope: three epics + MVP definition** — docs_product_mvp_mvp_scope, planning_epics_interruption_stack_interruption_stack_epic, planning_epics_task_templates_task_templates_epic, planning_epics_export_export_epic [INFERRED 0.85]

## Communities (47 total, 2 thin omitted)

### Community 0 - "Tauri Commands & App State"
Cohesion: 0.11
Nodes (55): apply_transition(), complete(), create_template(), delete_template(), editing_and_deleting_a_template_does_not_change_an_already_recorded_time_block(), emit_state_changed(), emit_templates_changed(), export_blocks_in_range() (+47 more)

### Community 1 - "Product & Architecture Docs"
Cohesion: 0.05
Nodes (61): Anchor Project Instructions (CLAUDE.md), Design Workflow, Discovery Workflow, Planning Workflow, Frontend Stack Constraint (Svelte + TypeScript + Ramda), Technical Constraints, App Framework & Platform Decision, Component Map (+53 more)

### Community 2 - "Feature Docs, Glossary & Risks"
Cohesion: 0.08
Nodes (52): app/src/routes/+page.svelte (dashboard), app/src/routes/widget/+page.svelte (mini widget), app/src-tauri/src/lib.rs (hotkey registration), app/src-tauri/tauri.conf.json (window config), Assumption: Auto-completing skipped interruptions matches user intent, Assumption: Existing time-tracking tools don't solve the interruption-stack problem, Concept Doc, Concept (Anchor product concept) (+44 more)

### Community 3 - "Transition Log Format & Data Model"
Cohesion: 0.10
Nodes (35): checksum_never_appears_inside_json_object(), decode_line(), detects_single_byte_corruption(), encode_line(), FramingError, missing_tab_is_no_tab_error(), round_trip(), Error (+27 more)

### Community 4 - "Claude Agent/Workflow Config"
Cohesion: 0.12
Nodes (41): Documentation Steward Agent, Product Manager Agent, Agents README, Researcher Agent, Reviewer Agent, Senior Software Engineer Agent, Technical Architect Agent, UX Designer Agent (+33 more)

### Community 5 - "Interruption Stack State Machine"
Cohesion: 0.17
Nodes (32): complete_requires_empty_stack(), depth_12_interrupts_then_12_return_previous_unwinds_correctly(), depth_12_interrupts_then_return_original_skips_11(), interrupt(), interrupt_closes_current_with_pending_reason_and_pushes_stack(), InterruptionStack, next_default_name_ignores_anchor_names_that_started_before_today(), next_default_name_ignores_non_anchor_names() (+24 more)

### Community 6 - "Task Templates Backend"
Cohesion: 0.11
Nodes (27): create_update_delete_template_round_trip(), templates_persist_across_restart(), create_appends_and_assigns_fresh_uuid(), creating_two_templates_with_identical_name_project_client_is_allowed(), delete_by_id_removes_only_that_template(), delete_of_unknown_id_returns_err(), mutate_templates(), mutate_templates_rolls_back_in_memory_if_save_fails() (+19 more)

### Community 7 - "Frontend API & UI"
Cohesion: 0.07
Nodes (10): svelte, updateTemplate(), CompletionReason, ExportSettings, HotkeyBindings, StackFrame, StackView, TaskTemplate (+2 more)

### Community 8 - "Log Replay & App State Init"
Cohesion: 0.09
Nodes (30): deliberately_corrupted_trailing_line_is_discarded_prior_lines_survive(), last_timestamp_reflects_the_last_good_line_leftover_active_included(), payload(), replay(), replay_of_missing_file_is_empty(), replay_reconstructs_stack_from_a_real_sequence(), ReplayError, ReplayResult (+22 more)

### Community 9 - "Export Logic"
Cohesion: 0.18
Nodes (25): active_entry_in_range_is_included_with_elapsed_so_far_duration_and_is_not_mutated(), blocks_in_range(), blocks_in_range_filters_by_start_time_not_end_time(), closed_block(), ExportRow, group(), group_combines_same_task_separated_by_an_interrupting_task(), json_export() (+17 more)

### Community 10 - "Build Config & Paths"
Cohesion: 0.14
Nodes (24): devDependencies, svelte-check, @sveltejs/adapter-static, @sveltejs/kit, @sveltejs/vite-plugin-svelte, @tauri-apps/cli, @types/ramda, typescript (+16 more)

### Community 11 - "Hotkey Registration & Remap"
Cohesion: 0.20
Nodes (19): apply_remap(), bindings(), duplicate_accelerator_across_two_actions_is_detected(), duplicate_detection_is_case_insensitive(), find_duplicate(), HotkeyAction, HotkeyState, register_bindings() (+11 more)

### Community 12 - "Export Settings Persistence"
Cohesion: 0.16
Nodes (16): ExportSettings, ExportSettingsState, mutate_export_settings(), mutate_export_settings_rolls_back_in_memory_if_save_fails(), mutate_export_settings_round_trips_via_state(), AsRef, Default, FnOnce (+8 more)

### Community 13 - "NPM Dependencies"
Cohesion: 0.11
Nodes (17): dependencies, ramda, @tauri-apps/api, @tauri-apps/plugin-dialog, @tauri-apps/plugin-opener, description, license, name (+9 more)

### Community 14 - "Sleep/Hibernate Detection"
Cohesion: 0.15
Nodes (17): active_block(), boundary_at_exactly_the_threshold_counts_as_a_gap(), boundary_one_second_under_threshold_is_not_a_gap(), no_gap_when_last_activity_is_recent(), resolve_resume_gap(), resolves_recover_gap_then_start_when_gap_exceeds_threshold(), AppHandle, DateTime (+9 more)

### Community 15 - "App Icons & Logos"
Cohesion: 0.11
Nodes (18): App Icon 128x128@2x, App Icon 128x128, App Icon 32x32, Anchor App Icon (Master), Windows Tile Icon Square107x107Logo, Windows Tile Icon Square142x142Logo, Windows Tile Icon Square150x150Logo, Windows Tile Icon Square284x284Logo (+10 more)

### Community 16 - "Tauri App Config"
Cohesion: 0.11
Nodes (17): app, security, windows, build, beforeBuildCommand, beforeDevCommand, devUrl, frontendDist (+9 more)

### Community 17 - "Tauri Desktop Capability Schema (generated)"
Cohesion: 0.13
Nodes (14): anyOf, anyOf, description, definitions, Application, Target, Value, description (+6 more)

### Community 18 - "Tauri Windows Capability Schema (generated)"
Cohesion: 0.13
Nodes (14): anyOf, anyOf, description, definitions, Application, Target, Value, description (+6 more)

### Community 19 - "Hotkey Bindings Settings"
Cohesion: 0.23
Nodes (8): HotkeyBindings, AsRef, Default, Path, Result, Self, String, save_then_load_round_trips()

### Community 20 - "TypeScript Config"
Cohesion: 0.17
Nodes (11): compilerOptions, allowJs, checkJs, esModuleInterop, forceConsistentCasingInFileNames, moduleResolution, resolveJsonModule, skipLibCheck (+3 more)

### Community 21 - "Desktop Schema: Permissions/Platforms"
Cohesion: 0.20
Nodes (10): $ref, description, items, type, uniqueItems, description, items, type (+2 more)

### Community 22 - "Desktop Schema: Windows/Webviews"
Cohesion: 0.20
Nodes (10): type, webviews, windows, items, description, items, type, description (+2 more)

### Community 23 - "Windows Schema: Permissions/Platforms"
Cohesion: 0.20
Nodes (10): $ref, description, items, type, uniqueItems, description, items, type (+2 more)

### Community 24 - "Windows Schema: Windows/Webviews"
Cohesion: 0.20
Nodes (10): type, webviews, windows, items, description, items, type, description (+2 more)

### Community 25 - "Desktop Schema: Identifier/Remote"
Cohesion: 0.22
Nodes (9): properties, Identifier, description, oneOf, type, identifier, remote, anyOf (+1 more)

### Community 26 - "Windows Schema: Identifier/Remote"
Cohesion: 0.22
Nodes (9): properties, Identifier, description, oneOf, type, identifier, remote, anyOf (+1 more)

### Community 27 - "Desktop Schema: Capability Remote"
Cohesion: 0.25
Nodes (8): description, properties, required, type, CapabilityRemote, urls, description, type

### Community 28 - "Windows Schema: Capability Remote"
Cohesion: 0.25
Nodes (8): description, properties, required, type, CapabilityRemote, urls, description, type

### Community 29 - "Tauri Default Capability"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 30 - "Desktop Schema: Capability Def"
Cohesion: 0.50
Nodes (4): description, required, type, Capability

### Community 31 - "Desktop Schema: Description Field"
Cohesion: 0.50
Nodes (4): default, description, type, description

### Community 32 - "Desktop Schema: Local Field"
Cohesion: 0.50
Nodes (4): default, description, type, local

### Community 33 - "Windows Schema: Capability Def"
Cohesion: 0.50
Nodes (4): description, required, type, Capability

### Community 34 - "Windows Schema: Description Field"
Cohesion: 0.50
Nodes (4): default, description, type, description

### Community 35 - "Windows Schema: Local Field"
Cohesion: 0.50
Nodes (4): default, description, type, local

### Community 36 - "Desktop Schema: Number Def"
Cohesion: 0.67
Nodes (3): Number, anyOf, description

### Community 37 - "Desktop Schema: Permission Entry"
Cohesion: 0.67
Nodes (3): PermissionEntry, anyOf, description

### Community 38 - "Windows Schema: Number Def"
Cohesion: 0.67
Nodes (3): Number, anyOf, description

### Community 39 - "Windows Schema: Permission Entry"
Cohesion: 0.67
Nodes (3): PermissionEntry, anyOf, description

## Ambiguous Edges - Review These
- `Anchor App Icon (Master)` → `Default SvelteKit Favicon (unbranded)`  [AMBIGUOUS]
  app/static/favicon.png · relation: conceptually_related_to

## Knowledge Gaps
- **166 isolated node(s):** `Prompts README`, `mattpocock/skills grill-with-docs (Source)`, `/grilling Skill`, `/domain-modeling Skill`, `ADR Template` (+161 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **2 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **What is the exact relationship between `Anchor App Icon (Master)` and `Default SvelteKit Favicon (unbranded)`?**
  _Edge tagged AMBIGUOUS (relation: conceptually_related_to) - confidence is low._
- **Why does `Tauri Framework` connect `Build Config & Paths` to `Tauri Commands & App State`, `Feature Docs, Glossary & Risks`, `Sleep/Hibernate Detection`?**
  _High betweenness centrality (0.260) - this node is a cross-community bridge._
- **Why does `Anchor App README (Tauri + SvelteKit + TypeScript)` connect `Build Config & Paths` to `Feature Docs, Glossary & Risks`?**
  _High betweenness centrality (0.145) - this node is a cross-community bridge._
- **Why does `R6: Tauri Ecosystem Maturity Risk for Solo Maintainer` connect `Feature Docs, Glossary & Risks` to `Build Config & Paths`?**
  _High betweenness centrality (0.138) - this node is a cross-community bridge._
- **Are the 2 inferred relationships involving `apply_transition()` (e.g. with `run()` and `handle_resume()`) actually correct?**
  _`apply_transition()` has 2 INFERRED edges - model-reasoned connections that need verification._
- **What connects `Prompts README`, `mattpocock/skills grill-with-docs (Source)`, `/grilling Skill` to the rest of the system?**
  _178 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Tauri Commands & App State` be split into smaller, more focused modules?**
  _Cohesion score 0.1110523532522475 - nodes in this community are weakly interconnected._