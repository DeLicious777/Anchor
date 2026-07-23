# Glossary

Shared vocabulary for this project. A term earns an entry here once it's used more than once, is disputed, or gets redefined mid-conversation — typically surfaced during a `grill-with-docs` session via `/domain-modeling`. Feeds Graphify's knowledge graph directly, so keep entries atomic (one term, one definition) rather than prose paragraphs.

| Term | Definition | First defined in |
|---|---|---|
| Time Block | The atomic unit of tracked work: a name, start time, end time, duration, optional project/client, and a completion reason (`explicit`, `auto-completed-on-skip`, or `recovered-gap`). Independent entry in the timeline — not tied to a persistent Task entity. | `docs/concept/concept.md` |
| Timeline | The ordered sequence of all Time Blocks for a user; the single source of truth from which all reports and exports are derived. | `docs/concept/concept.md` |
| Switch | A deliberate move from one task to another with no expectation of returning to the one just left. Distinct from Interrupt. | `docs/concept/concept.md` |
| Interrupt | Pausing the current task because something requires immediate attention, with intent to return; pushes the current task onto the interruption stack. | `docs/concept/concept.md` |
| Interruption Stack | The internal, arbitrarily deep, true stack structure holding nested interrupted tasks in order. Not directly exposed in the UI. | `docs/concept/concept.md` |
| Return to Previous Task | Resume the most recently interrupted task by stepping back exactly one level in the interruption stack. | `docs/concept/concept.md` |
| Return to Original Task | Jump directly to the root task of the current interruption chain. All intermediate interruptions skipped this way are marked completed with reason `auto-completed-on-skip` — distinct from an explicit completion, so nothing is silently indistinguishable from work the user actually finished. | `docs/concept/concept.md` |
| Completion Reason | A field on every Time Block recording how it ended: `explicit` (user-finished), `auto-completed-on-skip` (via Return to Original Task), or `recovered-gap` (active entry reconciled after a detected gap — an ungraceful crash or a resume from sleep/hibernate — with an inferred end time). Exists so anything other than a clean, user-chosen completion stays reviewable rather than silent. | `docs/concept/concept.md` (extended in `docs/product/features/interruption-stack.md`) |
| Task Template | A reusable preset that pre-populates a Time Block's name/project/client, for fast starts on recurring activities (e.g. daily standup). | `docs/product/mvp.md` |
| Torn Write | A write to the persisted transition log that was interrupted mid-operation (e.g. by a crash), leaving a partial/corrupt record. Detected via an explicit framing/checksum scheme and discarded on next launch — every prior record is unaffected. | `docs/product/features/interruption-stack.md`; the concrete detection scheme (parse-validity + checksum, framed outside the JSON object) is decided in [ADR 0004](decisions/0004-transition-log-format-and-torn-write-scheme.md), not ADR 0002, which explicitly deferred it |
| Constraint (vs. Preference) | A non-negotiable architectural boundary recorded in `docs/architecture/constraints.md` — distinct from a preference, which can lose to a better trade-off. A preference is promoted to a constraint via an ADR when it's doing the work of eliminating options outright, not just tipping a close call. | `docs/architecture/constraints.md`, first applied in `docs/decisions/0002-desktop-app-framework-and-platform.md` |

---

**Keeping this current:** if two docs use the same term with subtly different meanings, that's a bug — reconcile it here and update both docs to match, don't let the drift stand.
