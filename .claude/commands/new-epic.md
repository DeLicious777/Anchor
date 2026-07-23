---
description: Create a new epic that groups related feature docs and ADRs into planned, sequenced work.
argument-hint: <epic-name>
---

Follow `.claude/workflows/planning.md`. Only proceed if the feature docs being grouped are `status: accepted` and any driving ADRs are accepted — otherwise send the request back to `/new-feature` or `/new-adr` first.

Create `planning/epics/$ARGUMENTS.md` linking to its feature docs and ADRs. Act as **product-manager** for sequencing/prioritization and **technical-architect** for flagging cross-epic technical dependencies. Add the epic to `planning/milestones.md`.
