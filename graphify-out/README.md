# Graphify Output

Generated knowledge graph for this repository — 1075 nodes, 2127 edges, 82 communities as of the last build (2026-08-07). Open [`graph.html`](graph.html) in a browser to explore it, or read [`GRAPH_REPORT.md`](GRAPH_REPORT.md) for god nodes, communities, and suggested questions.

- **Committed to version control** — this is a versioned snapshot of the project's knowledge graph, not a disposable build artifact.
- **Full rebuild** (`/graphify .`) when a Discovery/Design/Architecture/Planning phase completes, or a batch of ADRs lands — this is when community boundaries actually shift, not just node content (confirmed: fixing `docs/architecture/overview.md`'s empty Key Decisions table produced a genuinely new community, not just edge updates).
- **Incremental `/graphify . --update`** for a single-doc fix or small edit — the cache means unrelated files cost nothing, but cost still scales with how cross-referenced the changed doc is, not just its length: the initial 52-file build cost ~5.2K tokens/file on average, but re-extracting one doc after densifying its cross-references (the overview.md fix) cost 47K tokens on its own — about 18% of the entire initial build, for one file.
- **Not on every commit** — full rebuilds are too costly relative to value at typical doc-editing velocity; reserve them for the phase/ADR-batch cadence above. See `graphify-out/cost.json` for the running token ledger.

## Known graph health issues (2026-08-07 build)

Recorded here rather than in `GRAPH_REPORT.md` because that file is overwritten by every rebuild, and because a silently-degraded graph is worse than a visibly imperfect one.

The build's own diagnostic reported, against 2507 raw extracted edges:

- **209 dangling-endpoint edges** — edges whose endpoint node id was never produced. These are semantic (LLM) edges pointing at ids the AST pass did not generate, i.e. the ghost-duplicate-id failure the extraction spec warns about. They are dropped from the built graph, so **roughly 8% of extracted relationships are absent from `graph.json`**. A traversal that comes up empty may be hitting this rather than a real gap in the docs.
- **184 collapsed edges (undirected)** — mostly benign. The graph is a simple undirected `Graph`, so repeated `references` edges between the same pair of Rust symbols (e.g. `TransitionPayload` → `String`, 18 of them across different fields) flatten to one. Nothing is lost that a reader would miss.
- **0 missing-endpoint edges, 0 self-loops.**

Neither number is a reason to distrust the graph's shape — the community structure and god nodes match the architecture — but the dangling count should come down on the next full rebuild if id generation is aligned. Re-check it after the next `/graphify .` run rather than assuming it persists or resolves.
