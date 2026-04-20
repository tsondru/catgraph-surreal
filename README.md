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
use surrealdb::engine::any;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Works with in-memory, on-disk, or remote endpoints — stores
    // type against `Surreal<Any>`, so swapping `"mem://"` for
    // `"ws://host:8000"` or `"surrealkv://path"` requires no other changes.
    let db = any::connect("mem://").await?;
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

### Engine generalization

Every store types against `Surreal<engine::any::Any>`, so the same code
works with in-memory, on-disk, WebSocket, or HTTP endpoints. Pick the
backend at connect time:

```rust
use surrealdb::engine::any;

// Embedded in-memory (native-embedded feature)
let db = any::connect("mem://").await?;

// Embedded on-disk (native-embedded feature)
let db = any::connect("surrealkv:///var/lib/catgraph").await?;

// Remote WS to a native SurrealDB daemon (remote-ws feature)
let db = any::connect("ws://127.0.0.1:8000").await?;
```

### Feature matrix

| Feature | Default | Enables | Use case |
|---|---|---|---|
| `native-embedded` | ✅ on | `surrealdb/kv-mem` + `surrealdb/kv-surrealkv` | Tests, single-process native, dev |
| `remote-ws` | ⬜ off | `surrealdb/protocol-ws` | WASM sidecar talking to a native SurrealDB over WS |
| `remote-http` | ⬜ off | `surrealdb/protocol-http` | RPC-over-HTTP clients (no live queries) |

Slim a native-embedded-only build out: `--no-default-features`.
Remote-only WASM sidecar: `--no-default-features --features remote-ws`.

### Tokio trim

`tokio` is pulled with a trimmed feature set (`rt` + `sync` + `macros` +
`time` only — no `signal` / `process` / `net` / `io-std` / `fs`). The
catgraph workspace is pinned to `v0.11.4` which gates rayon behind a
`parallel` feature for single-threaded WASI hosts.

### Build targets

```sh
# Native (default)
cargo build --lib

# WASI multi-threaded
cargo build --lib --target wasm32-wasip1-threads

# WASI single-threaded
cargo build --lib --target wasm32-wasip1 --no-default-features --features remote-ws
```

**Known blocker (upstream SurrealDB SDK):** the surrealdb 3.0.5 SDK's
WASM target assumes `wasm32-unknown-unknown` + a JS host
(`wasm-bindgen` + `web-sys` + `getrandom/wasm_js`). The **library**
compiles on `wasm32-wasip1-*` because our code only references the
`engine::any::Any` type, which is transport-agnostic — but actually
running a remote-ws client under WASI is blocked on SDK changes
upstream. The local-only v0.10.0 is held pending the next SurrealDB
release, which may either add WASI support directly or motivate a raw
WS client for the sidecar path. See `CHANGELOG.md` `[Unreleased]`.

See `examples/wasi_edge_client.rs` for a minimal sidecar-pattern smoke
test.

## Dependencies

`catgraph`, `catgraph-physics`, `catgraph-applied` (all tag `v0.11.4`, shared for Cargo source dedup), `surrealdb` 3.0.5 (engines opt-in via features — see above), `surrealdb-types` 3.0.5, `serde` + `serde_json`, `tokio` (trimmed to `rt`/`sync`/`macros`/`time`), `thiserror`, `rust_decimal`

## Changelog

See [`CHANGELOG.md`](CHANGELOG.md) for release history.

## License

MIT
