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

Runs against an embedded `Mem` SurrealDB engine (via `surrealdb::engine::any::connect("mem://")`) — no external DB required.

## Local development across repos

This crate depends on three sibling crates in the catgraph workspace (`catgraph`, `catgraph-physics`, `catgraph-applied`), all via the same git tag for Cargo source deduplication. When co-editing, uncomment the `[patch]` block at the bottom of `Cargo.toml` to redirect all three to local paths at `../catgraph/{catgraph,catgraph-physics,catgraph-applied}`. Re-comment before pushing.

## Dependencies

- `catgraph` (git tag `v0.11.4`) — F&S core (cospans, spans)
- `catgraph-physics` (git tag `v0.11.4`) — hypergraph evolution, gauge theory
- `catgraph-applied` (git tag `v0.11.4`) — Petri nets, wiring diagrams
- `surrealdb` 3.0.5 — engines are feature-gated; see "WASM / edge support" below
- `surrealdb-types` 3.0.5
- `serde` + `serde_json`
- `tokio` (trimmed to `rt`, `sync`, `macros`, `time` in v0.10.0 for WASM compatibility)
- `thiserror`, `rust_decimal`

## WASM / edge support (v0.10.0+, local-only, unreleased)

The library compiles to `wasm32-wasip1-threads` (parallel rayon path via
the catgraph `parallel` feature, inherited transitively) and
`wasm32-wasip1` (single-threaded). **All store APIs generalized to
`Surreal<engine::any::Any>`** — the same code works with `"mem://"`,
`"surrealkv://path"`, `"ws://host:8000"`, `"http://host:8000"` endpoints.

**Cargo features:**
- `native-embedded` (default-on) → `surrealdb/kv-mem` + `surrealdb/kv-surrealkv`
- `remote-ws` (opt-in) → `surrealdb/protocol-ws` (WASM sidecar target)
- `remote-http` (opt-in) → `surrealdb/protocol-http`

**Upstream SurrealDB SDK blocker:** 3.0.5's WASM target is browser-only
(`wasm32-unknown-unknown` + `wasm-bindgen` + `web-sys`). Our library
compiles on `wasm32-wasip1-*` because it only references
`engine::any::Any`, but actually running a remote WS client under WASI
needs either SurrealDB to add WASI support or a raw WS client on our
side. v0.10.0 is held local pending the next SurrealDB release.

See `examples/wasi_edge_client.rs` for the smoke test.

@.claude/refactor/architecture.md
@.claude/refactor/session-state.md
@.claude/refactor/CLAUDE.local.md
