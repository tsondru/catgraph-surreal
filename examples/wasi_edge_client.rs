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
//! This example uses the embedded `Mem` engine (the current crate default)
//! because the remote-engine generalization of the store API is scheduled
//! for a follow-up patch (the stores currently hardcode
//! `Surreal<engine::local::Db>`). The calling pattern — async store +
//! tokio runtime + cospan roundtrip — is what a real edge sidecar exercises
//! once the remote engine lands.
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
use surrealdb::engine::local::Mem;
use surrealdb::Surreal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Sidecar-style setup: embedded in-memory SurrealDB. A production edge
    // sidecar would connect via `Surreal::new::<Ws>("127.0.0.1:8000")` to a
    // native daemon instead — see the doc comment above for the follow-up
    // plan that exposes that path.
    let db = Surreal::new::<Mem>(()).await?;
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
