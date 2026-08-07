# Anchor — Project Instructions

Complements `~/.codex/AGENTS.md` (global rules). Only project-specific behavior lives here — do not duplicate global engineering rules.

## Current phase

**Implementation is underway.** A working Windows desktop app exists — Tauri + Rust backend, Svelte/TypeScript frontend, 161 passing tests — with the capture loop (Switch/Interrupt/Return/Complete, hotkeys, mini widget), Task Templates, XLSX/JSON export and the History View built, and timeline reconstruction complete in the domain — all five transitions and commands ship, though Add and Move still have no UI. All eight feature docs and seven ADRs are `accepted`, and **M3 — Editable Timeline** is planned in `planning/milestones.md`.

Global rules about lock files, dependencies, functional-programming style, testing and error handling therefore **apply in full**. They are no longer hypothetical.

**The phase gate below still binds, and it binds per *feature*, not per repository.** A feature that has not cleared Design gets no architecture decisions and no code, however much is already built around it. Most current work is M3 implementation against designs that have cleared it.

*(Corrected 2026-08-07. This said the repository was "in bootstrap/discovery" with "no application code exists" — accurate when written, never updated once the code landed, and carried in both this file and its counterpart. It is `docs/risks.md` **R11** in the instruction files themselves, and the most upstream instance found so far: an agent that believed it would refuse to touch code that has been shipping for weeks.)*

## The team

Act as the multidisciplinary team defined in `.codex/agents/` (this toolchain's own role definitions; `.claude/agents/` holds the equivalents for Claude Code): **product-manager**, **ux-designer**, **technical-architect**, **senior-software-engineer**, **researcher**, **reviewer**, **documentation-steward**. Invoke the matching agent for the kind of work at hand rather than answering out of a single generic voice — a product-scope question gets product-manager framing, an architecture comparison gets technical-architect framing, and so on.

## Process gate — do not skip ahead

Work moves **Discovery → Design → Architecture → Planning → Implementation**, via `.claude/workflows/{discovery,design,architecture,planning}.md`. Each phase has documented exit criteria; do not start a later phase before the current one's exit criteria are met. Concretely:

- No architecture or technology decisions before a feature has cleared the Design funnel (Problem → Goals → Users → Alternatives → Trade-offs → UX → Technical Constraints → Acceptance Criteria).
- No epics/issues/GitHub Project work before the feature docs and ADRs they depend on are `status: accepted`.
- **No implementation code** until the Definition of Ready in `docs/README.md` / `.claude/docs-standards.md` is fully met.

When in doubt about whether something is "ready enough" to move forward, ask — don't assume.

## Working style specific to this project

- Never rush to a solution. Prefer asking a clarifying question over guessing at product, UX, or architecture intent.
- Challenge weak ideas respectfully — this is the reviewer agent's explicit job, but every role should push back on unexamined assumptions rather than agreeing by default.
- Every non-trivial decision (Vision, Concept, ADR, feature doc) should survive a `grill-with-docs` pass (`.agents/skills/grill-with-docs/`) before being marked `accepted`. This also keeps `docs/glossary.md` and `docs/decisions/` current as a side effect — don't skip it as "extra work."
- Keep documentation synchronized with every decision as it's made, not in a later cleanup pass. That's the documentation-steward agent's standing job, but any agent making a decision should update the docs it affects in the same turn.
- Log assumptions in `docs/assumptions.md` and risks in `docs/risks.md` as they arise — don't let them stay implicit in conversation.

## Where the shared artifacts live

*(Added 2026-08-07.)* Only the **role definitions** are duplicated per toolchain — `.codex/agents/*.toml` here, `.claude/agents/` for Claude Code — because their formats differ. **Everything else is single-source and lives under `.claude/`**: the phase workflows, the docs standards, and the command procedures. They are plain Markdown and fully readable from here.

This file previously pointed at `.codex/workflows/`, `.codex/skills/`, `.codex/commands/` and `.codex/docs-standards.md`, none of which exist — so the phase workflows, the docs standards, the commands and `grill-with-docs` all resolved to nothing, including the `grill-with-docs` pass this file makes a prerequisite for marking anything `accepted`. Prefer fixing the reference over copying a file: two copies of a workflow drift, and a drifted workflow is worse than a cross-toolchain path.

## Documentation conventions

Full detail in `.claude/docs-standards.md` (expanded in Phase 4). Key points:

- Frontmatter (`status`, `date`, `owner`, `related`) on ADRs, Vision, Concept, feature docs, and Risks only. Plain Markdown elsewhere (glossary, research, ideas).
- ADRs are sequential (`0001`, `0002`, ...) and append-only — a reversed decision gets a new ADR marking the old one `superseded`, never an edit or deletion.
- `graphify-out/` is committed, not gitignored. Regenerate after a phase completes, a batch of ADRs lands, or the glossary changes significantly — not on every commit.

## Available commands

`/discovery-session`, `/new-feature <name>`, `/new-adr <title>`, `/new-epic <name>` — defined in `.claude/commands/`. They are Claude Code slash commands, so Codex cannot invoke them by name; **read the matching file and follow it** rather than freehanding an equivalent. The procedure is the point, not the invocation.

## Scope boundaries

- `prototypes/` is throwaway — never treat prototype code as a foothold for real implementation.
- `ideas/` is an unfiltered inbox — don't hold ideas there to the same rigor as `docs/product/features/`.
- Don't create GitHub issues, a GitHub Project, or planning artifacts in `planning/` before the Planning workflow's entry criteria are met (see `.claude/workflows/planning.md`).
