# catgraph-surreal

SurrealDB persistence layer for the [catgraph](https://github.com/tsondru/catgraph) workspace crates (`catgraph`, `catgraph-physics`, `catgraph-applied`).

Persists `Cospan`, `Span`, `NamedCospan`, hypergraph evolution, Petri nets, and wiring diagrams to SurrealDB 3.0.5 via two coexisting storage layers:

- **V1 (embedded arrays)**: each n-ary hyperedge stored as a single record with embedded arrays encoding the structural maps. O(1) reconstruction.
- **V2 (graph-native)**: first-class nodes, pairwise `RELATE` edges, hub-node reification for n-ary hyperedges, FTS + HNSW indexes for similarity search.

## Usage

```rust
use catgraph_surreal::{init_schema, init_schema_v2};
use catgraph_surreal::cospan_store::CospanStore;       // V1
use catgraph_surreal::node_store::NodeStore;            // V2
use catgraph_surreal::hyperedge::HyperedgeStore;        // V2
use surrealdb::{Surreal, engine::local::Mem};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Surreal::new::<Mem>(()).await?;
    db.use_ns("test").use_db("test").await?;

    init_schema(&db).await?;      // V1 tables
    init_schema_v2(&db).await?;   // V2 tables (can coexist)

    // V1: embedded array roundtrip
    let v1 = CospanStore::new(&db);
    let id = v1.save(&my_cospan).await?;
    let loaded: Cospan<char> = v1.load(&id).await?;

    // V2: graph-native decomposition
    let v2 = HyperedgeStore::new(&db);
    let hub_id = v2.decompose_cospan(&cospan, "reaction", props, |c| c.to_string()).await?;
    let sources = v2.sources(&hub_id).await?;
    let reconstructed: Cospan<char> = v2.reconstruct_cospan(&hub_id).await?;

    Ok(())
}
```

## Tables

### V1
- `cospan`, `named_cospan`, `span` — single-record embedded-array storage

### V2
- `graph_node` (with FTS + HNSW indexes), `graph_edge`
- `hyperedge_hub`, `source_of` (with `decimal weight`), `target_of` (with `decimal weight`)
- `petri_net`, `petri_place`, `petri_transition`, `pre_arc`, `post_arc`, `petri_marking`

## Local development

This crate depends on `catgraph`, `catgraph-physics`, and `catgraph-applied` via git tags (all three share the same tag for Cargo source deduplication). To work on changes that span repos, edit `Cargo.toml` and uncomment the `[patch]` block at the bottom — this redirects all three deps to your local workspace at `../catgraph/`.

Re-comment the patch before pushing to keep the released artifact reproducible.

## WASM / edge support (v0.10.0+)

The library compiles clean to `wasm32-wasip1-threads` for edge-device
deployment under Wasmtime / Wasmer / WasmEdge / Fermyon Spin. `tokio` is
now pulled with a trimmed feature set (`rt` + `sync` + `macros` + `time`
only — no `signal` / `process` / `net` / `io-std` / `fs`) and the
catgraph workspace is pinned to `v0.11.4` which gates rayon behind a
`parallel` feature for single-threaded WASI hosts.

```sh
cargo build --lib --target wasm32-wasip1-threads -p catgraph-surreal
```

See `examples/wasi_edge_client.rs` for a minimal sidecar-pattern smoke
test. Remote-engine generalization of the store API (so the sidecar can
talk to a native SurrealDB over WS instead of embedded `Mem`) is
scheduled for a follow-up patch — the stores currently type against
`Surreal<engine::local::Db>`.

## Dependencies

`catgraph`, `catgraph-physics`, `catgraph-applied` (all tag `v0.11.4`, shared for Cargo source dedup), `surrealdb` 3.0.5 (kv-mem), `surrealdb-types` 3.0.5, `serde` + `serde_json`, `tokio` (trimmed to `rt`/`sync`/`macros`/`time`), `thiserror`, `rust_decimal`

## Changelog

See [`CHANGELOG.md`](CHANGELOG.md) for release history.

## License

MIT
