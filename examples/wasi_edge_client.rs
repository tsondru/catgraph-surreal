//! WASI edge-client smoke example — demonstrates the catgraph-surreal store
//! API pattern for a sidecar WASM process targeting `wasm32-wasip1-threads`
//! / `wasm32-wasip1`.
//!
//! The end-to-end edge-deployment picture (see the catgraph workspace plan
//! at `.claude/plans/i-realize-i-need-wise-stonebraker.md`) is:
//!
//! 1. SurrealDB runs native on the edge device (systemd service).
//! 2. The catgraph sidecar compiles to WASM, runs under wasmtime/wasmer,
//!    and talks to the native SurrealDB over WS (`surrealdb/protocol-ws`).
//! 3. Certain catgraph ops (cospan compose, pushout) additionally ship as
//!    Surrealism `.surli` modules loaded into SurrealDB via `DEFINE MODULE`
//!    for compute-near-data paths.
//!
//! With v0.10.0 the stores type against `Surreal<engine::any::Any>`, so
//! this example — and a real edge sidecar — can swap the in-memory
//! endpoint (`"mem://"`) for a remote one (`"ws://host:8000"`) without
//! changing any store code. The calling pattern below (async store +
//! tokio runtime + cospan roundtrip) is exactly what the WASM sidecar
//! exercises against a native SurrealDB over WS.
//!
//! ## Running
//!
//! Native: `cargo run --example wasi_edge_client -p catgraph-surreal`
//!
//! WASM (requires wasmtime or similar; after the remote engine lands this
//! example will also talk to a native SurrealDB):
//! ```sh
//! cargo build --lib --target wasm32-wasip1-threads -p catgraph-surreal
//! ```

use catgraph::category::HasIdentity;
use catgraph::cospan::Cospan;
use catgraph_surreal::cospan_store::CospanStore;
use catgraph_surreal::init_schema;
use surrealdb::engine::any;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Sidecar-style setup: embedded in-memory SurrealDB. Swap `"mem://"`
    // for `"ws://127.0.0.1:8000"` to point at a native SurrealDB daemon
    // instead — all stores in this crate type against `Surreal<Any>`, so
    // no other lines in the example change.
    let db = any::connect("mem://").await?;
    db.use_ns("edge").use_db("sidecar").await?;
    init_schema(&db).await?;
    let store = CospanStore::new(&db);

    // Build an identity cospan on three typed vertices. This is the
    // smallest non-trivial round-trippable morphism in `Cospan<char>` and
    // exercises the save/load/equality path end-to-end.
    let identity: Cospan<char> = Cospan::identity(&vec!['x', 'y', 'z']);
    println!("WASI edge-client: saving identity cospan on ['x', 'y', 'z']");

    let id = store.save(&identity).await?;
    let loaded: Cospan<char> = store.load(&id).await?;
    assert!(loaded.is_left_identity());
    assert!(loaded.is_right_identity());
    assert_eq!(identity.middle(), loaded.middle());

    println!("  record id: {id:?}");
    println!("  roundtrip verified (identity flags + middle set preserved)");

    store.delete(&id).await?;
    let remaining = store.list().await?;
    assert!(remaining.is_empty());
    println!("  delete + list empty: OK");

    Ok(())
}
