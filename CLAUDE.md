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

- `catgraph` (git tag `v0.11.4`) — F&S core (cospans, spans)
- `catgraph-physics` (git tag `v0.11.4`) — hypergraph evolution, gauge theory
- `catgraph-applied` (git tag `v0.11.4`) — Petri nets, wiring diagrams
- `surrealdb` 3.0.5 with `kv-mem` feature
- `surrealdb-types` 3.0.5
- `serde` + `serde_json`
- `tokio` (trimmed to `rt`, `sync`, `macros`, `time` in v0.10.0 for WASM compatibility)
- `thiserror`, `rust_decimal`

## WASM / edge support (v0.10.0+)

The library compiles to `wasm32-wasip1-threads` (parallel rayon path via
the catgraph `parallel` feature, inherited transitively) and
`wasm32-wasip1` (single-threaded). Store APIs currently type against
`Surreal<engine::local::Db>` (embedded) — remote-engine generalization
(so a WASM sidecar can talk to a native SurrealDB over WS) is a
follow-up patch. See `examples/wasi_edge_client.rs` for the smoke test.

@.claude/refactor/architecture.md
@.claude/refactor/session-state.md
@.claude/refactor/CLAUDE.local.md
