# Planning

Bridges finished design/architecture work to executable GitHub issues.

- [`epics/`](epics/) — one file per epic ([interruption-stack.md](epics/interruption-stack.md), [task-templates.md](epics/task-templates.md), [export.md](epics/export.md)), each linking to the feature docs and ADRs it depends on
- [`milestones.md`](milestones.md) — sequencing (M1 Core Tracking Loop, M2 MVP Complete)
- [`issue-conventions.md`](issue-conventions.md) — labeling, naming, required links, and GitHub Project field/view conventions

Planning artifacts here should only be created once the relevant feature/architecture work has cleared the Design and Architecture workflows — not before. See `.claude/workflows/planning.md`.

## GitHub

- Project board: https://github.com/users/DeLicious777/projects/4
- Milestones: [M1 — Core Tracking Loop](https://github.com/DeLicious777/Anchor/milestone/1), [M2 — MVP Complete](https://github.com/DeLicious777/Anchor/milestone/2)
- Epic issues: [#1 Interruption Stack](https://github.com/DeLicious777/Anchor/issues/1), [#2 Task Templates](https://github.com/DeLicious777/Anchor/issues/2), [#3 Export](https://github.com/DeLicious777/Anchor/issues/3)

Note: `gh project` doesn't support creating additional views (Table-by-Epic, Table-by-Milestone) via CLI — only the default Board view exists so far. The other two views from `issue-conventions.md` need to be added manually in the web UI.
