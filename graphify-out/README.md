# Graphify Output

Generated knowledge graph for this repository — 141 nodes, 302 edges, 12 communities as of the last build. Open [`graph.html`](graph.html) in a browser to explore it, or read [`GRAPH_REPORT.md`](GRAPH_REPORT.md) for god nodes, communities, and suggested questions.

- **Committed to version control** — this is a versioned snapshot of the project's knowledge graph, not a disposable build artifact.
- **Full rebuild** (`/graphify .`) when a Discovery/Design/Architecture/Planning phase completes, or a batch of ADRs lands — this is when community boundaries actually shift, not just node content (confirmed: fixing `docs/architecture/overview.md`'s empty Key Decisions table produced a genuinely new community, not just edge updates).
- **Incremental `/graphify . --update`** for a single-doc fix or small edit — the cache means unrelated files cost nothing, but cost still scales with how cross-referenced the changed doc is, not just its length: the initial 52-file build cost ~5.2K tokens/file on average, but re-extracting one doc after densifying its cross-references (the overview.md fix) cost 47K tokens on its own — about 18% of the entire initial build, for one file.
- **Not on every commit** — full rebuilds are too costly relative to value at typical doc-editing velocity; reserve them for the phase/ADR-batch cadence above. See `graphify-out/cost.json` for the running token ledger.
