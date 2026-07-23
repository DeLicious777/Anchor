# Target Users

_Last reviewed: 2026-07-23_

> No frontmatter on this doc (per `.claude/docs-standards.md`) — it's expected to change often as user understanding sharpens.

## Primary user segment

**A solo software developer doing billable work (freelance, contracting, or consulting) who also has internal/company obligations** — meetings, internal tickets, ad hoc requests — that need to be tracked as non-billable time within the same workday. Scope is explicitly personal use for now (the author); broader adoption by similar developers/knowledge workers is a possible later direction, not a current design target. See `docs/vision/vision.md` Non-goals.

## Persona sketch

**The interrupted billable developer** — works on client-billable tasks throughout the day but is routinely pulled away by calls, Slack/chat messages, internal meetings, or spontaneous requests. Needs to bill accurately for client work, account for internal/non-billable time honestly, and never lose track of what they were doing before the last interruption. Values speed and low friction above all — if tracking takes more than a couple of seconds of thought, it won't be used consistently.

## Needs & pain points

- Accurately splitting a full workday between billable and non-billable time, without after-the-fact guesswork — met by Anchor capturing accurate project/client attribution per Time Block; the billable/non-billable classification itself happens downstream, in whatever billing process the author already uses (see `docs/decisions/0003-billable-classification-out-of-scope.md`).
- Effortlessly switching into an interruption and back out again, without losing the mental thread of the original task.
- Trusting that time once tracked is never silently lost, even through several layers of nested interruptions.
- Low-friction entry for recurring activities (standups, recurring client work) so tracking discipline doesn't decay over the day.

## Explicitly not users (for now)

- Teams or organizations needing shared visibility into each other's time (no multi-user/collaboration features in scope).
- People who want passive/automatic tracking with no manual input (explicitly out of scope — see `docs/vision/vision.md`).
- Non-technical users requiring heavy onboarding/hand-holding — the primary persona is comfortable with hotkeys and a command palette.

---

**Keeping this current:** every feature doc's "Users" stage should reference the segment defined here. If a feature needs a user type not listed, that's a signal to update this doc first, not skip the reference.
