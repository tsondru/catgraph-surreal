# catgraph-surreal

SurrealDB persistence layer for the [catgraph](https://github.com/tsondru/catgraph) workspace (`catgraph`, `catgraph-physics`, `catgraph-applied`).

## Scope

Persists F&S core types (`Cospan`, `Span`, `NamedCospan`), Wolfram-physics types (`Hypergraph`, `HypergraphEvolution`), and applied-CT types (`PetriNet`, `WiringDiagram`) from the catgraph workspace to SurrealDB 3.0.5 via two coexisting storage layers:

- **V1 (embedded arrays)**: each n-ary hyperedge stored as a single record with embedded arrays encoding the structural maps. O(1) reconstruction.
- **V2 (graph-native)**: first-class nodes, pairwise `RELATE` edges, hub-node reification for n-ary hyperedges, FTS + HNSW indexes for similarity search.

## Build & test

```sh
cargo test
```

Runs against an embedded `Mem` SurrealDB engine — no external DB required.

## Local development across repos

This crate depends on three sibling crates in the catgraph workspace (`catgraph`, `catgraph-physics`, `catgraph-applied`), all via the same git tag for Cargo source deduplication. When co-editing, uncomment the `[patch]` block at the bottom of `Cargo.toml` to redirect all three to local paths at `../catgraph/{catgraph,catgraph-physics,catgraph-applied}`. Re-comment before pushing.

## Dependencies

- `catgraph` (git tag) — F&S core (cospans, spans)
- `catgraph-physics` (git tag) — hypergraph evolution, gauge theory
- `catgraph-applied` (git tag) — Petri nets, wiring diagrams (added Phase 3.2, 2026-04-14)
- `surrealdb` 3.0.5 with `kv-mem` feature
- `surrealdb-types` 3.0.5
- `serde` + `serde_json`
- `tokio` (full features)
- `thiserror`, `rust_decimal`

@.claude/refactor/architecture.md
@.claude/refactor/session-state.md
@.claude/refactor/CLAUDE.local.md
