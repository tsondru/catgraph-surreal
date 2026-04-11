# catgraph-surreal

SurrealDB persistence layer for [catgraph](https://github.com/tsondru/catgraph).

## Scope

Persists `Cospan`, `Span`, and `NamedCospan` from catgraph to SurrealDB 3.0.5 via two coexisting storage layers:

- **V1 (embedded arrays)**: each n-ary hyperedge stored as a single record with embedded arrays encoding the structural maps. O(1) reconstruction.
- **V2 (graph-native)**: first-class nodes, pairwise `RELATE` edges, hub-node reification for n-ary hyperedges, FTS + HNSW indexes for similarity search.

## Build & test

```sh
cargo test
```

Runs against an embedded `Mem` SurrealDB engine — no external DB required.

## Local development across repos

This crate depends on `catgraph` via a git tag. When co-editing both repos, uncomment the `[patch]` block at the bottom of `Cargo.toml` to redirect the catgraph dep to a local path at `../catgraph/catgraph`. Re-comment before pushing.

## Dependencies

- `catgraph` (git tag)
- `surrealdb` 3.0.5 with `kv-mem` feature
- `surrealdb-types` 3.0.5
- `serde` + `serde_json`
- `tokio` (full features)
- `thiserror`, `rust_decimal`

@.claude/refactor/architecture.md
@.claude/refactor/session-state.md
@.claude/refactor/CLAUDE.local.md
