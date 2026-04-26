# Changelog

All notable changes to this crate are documented in this file.

Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); this crate adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Pending v0.10.1 — catgraph v0.12.0 bump (2026-04-26)

Bumped `catgraph` / `catgraph-physics` / `catgraph-applied` shared git tag from `v0.11.4` → `v0.12.0` so downstream consumers (irreducible v0.6.1+) can move to the v0.12.0 Corel co-release without hitting the diamond-dep `Cospan<Lambda>` type mismatch (two different catgraph SHAs unifying through this crate). The v0.12.0 catgraph release is additive at the surface this crate touches (`Cospan`, `Span`, `Hyperedge`); no source changes were required and the full test suite (25 binaries, 0 ignored) is still green.

This bump is folded into the still-untagged v0.10.x line — package version is now `0.10.1`. Tagging is still gated on the SurrealDB WASI blocker tracked below.

### Tagging strategy for v0.10.0

The v0.10.0 code is **complete and on `main`** (commits `9b1d03a` tokio-trim + `0f69dbd` engine-gen + native-embedded). No git tag has been cut yet. The gate on tagging is the resolution of [surrealdb/surrealdb#7283](https://github.com/surrealdb/surrealdb/issues/7283) (filed by this project on 2026-04-21):

- **If upstream accepts the cfg-split PR 1** (Cargo + source gates split into browser `target_os = "unknown"` vs. WASI `target_os = "wasi"`, no new transports) → bump the surrealdb dep to whatever point release ships the fix, verify `cargo build --target wasm32-wasip1 --no-default-features` and `--target wasm32-wasip1-threads --no-default-features --features remote-ws` both run to instantiation under wasmtime, then tag `v0.10.0`.
- **If upstream accepts PRs 2 + 3 as well** (WASI-native WebSocket/HTTP transports over `std::net`) → no changes needed in this crate; tag `v0.10.0` straight off the current `main` once the dep is published.
- **If upstream stays on the browser-only WASM path** → add a raw WS client behind a new feature flag (e.g. `remote-ws-wasi`) against SurrealDB's RPC protocol for the sidecar build only, ship as `v0.10.1` with that addition, tag, and push.
- **If the SDK requires any source changes** to catgraph-surreal (API breakage, feature renames) → fold into whichever path above is chosen, bumping to `v0.10.1` if post-engine-gen code has to change.

In all three cases the bump is 0.10.0 → 0.10.0 (or 0.10.1). We do not jump to 0.11.0 because the engine-generalization is additive from a consumer's perspective — `Surreal<Any>` still accepts the `mem://` endpoints tests use. The only "breaking" change was `Surreal<engine::local::Db>` → `Surreal<engine::any::Any>` in user-facing APIs, and there are no known external consumers of this crate yet.

**Do not** tag v0.10.0 without the SurrealDB-release gate — if the SDK path forces code changes, we want them folded into the tagged artifact.

### Upstream SurrealDB SDK WASI blocker (as of surrealdb 3.0.5)

Tracked upstream at **[surrealdb/surrealdb#7283](https://github.com/surrealdb/surrealdb/issues/7283)** (filed 2026-04-21). The underlying feature request has been open since beta.8 as #1641.

**Root cause.** The SDK uses `cfg(target_family = "wasm")` to split native vs. WASM dependencies throughout `surrealdb/Cargo.toml`, `surrealdb/core/Cargo.toml`, the two `build.rs` files, and a few dozen source-level gates. That predicate matches both `wasm32-unknown-unknown` (browser / JS host) *and* `wasm32-wasip1*` / `wasm32-wasip2` (WASI runtimes like wasmtime) — so WASI builds pull in the browser dep set: `getrandom/wasm_js`, `ring/wasm32_unknown_unknown_js`, `js-sys`, `wasm-bindgen`, `wasm-bindgen-futures`, `web-sys`, `tokio-tungstenite-wasm`, `uuid/js`, `wasmtimer`.

**Observed behaviour.** Counter to our earlier note in this file, the crate graph actually *compiles* clean on `wasm32-wasip1`; `wasm-ld` links without error. But the resulting `.wasm` contains hundreds of `__wbindgen_placeholder__::__wbg_*` imports (browser `WebSocket`, `setTimeout`, `performance.now`, etc.) that no WASI runtime can satisfy, so instantiation fails:

```
$ wasmtime ./probe.wasm
Error: failed to run main module
Caused by:
    0: failed to instantiate
    1: unknown import: `__wbindgen_placeholder__::__wbindgen_describe` has not been defined
```

**Fix path.** The proposed upstream fix is a three-arm cfg split — `target_os = "unknown"` (browser, unchanged), `target_os = "wasi"` (new, no wasm-bindgen deps), `not(target_family = "wasm")` (native, unchanged). PR 1 of the three-PR plan in #7283 — the Cargo + source cfg split without any new transport implementations — is partially drafted on local branch `wasi-cfg-split` of `/home/oryx/Documents/surreal/surrealdb` (commit `fd392ea05`, unpushed). The Cargo.toml split, core Cargo.toml split, and the three `build.rs` cfg-emission fixes are in that commit; the ~60 source-level cfg narrowings that accompany the split are tracked as pickup work. See `.claude/refactor/wasi-pr1-pickup.md` for the file-level checklist, verification commands, and fork-and-push flow for when upstream maintainers sign off on the cfg shape. PRs 2 and 3 would add WASI-native WebSocket and HTTP transports over `std::net` / `wasi-sockets`.

**What our library does in the meantime.** catgraph-surreal v0.10.0 is already structurally ready for this — every store generalized to `Surreal<engine::any::Any>`, embedded backends gated behind `native-embedded` so a remote-client-only build skips SurrealKV. The library itself compiles clean on `wasm32-wasip1-threads` and `wasm32-wasip1`. The only remaining blocker to shipping a WASI sidecar is the SDK-side wasm-bindgen coupling tracked in #7283.

## [0.10.0] - 2026-04-19 (on main, untagged)

Phase W.2 — WASM prep + remote-engine generalization + embedded-backend feature gate. Library compiles clean to `wasm32-wasip1-threads` and `wasm32-wasip1`; tokio features trimmed to the minimum the crate actually uses; every store generalized from `Surreal<engine::local::Db>` to `Surreal<engine::any::Any>` so the same code works against in-memory, on-disk, WebSocket, and HTTP endpoints; `native-embedded` feature gates the embedded kv backends so a remote-client-only build (WASM sidecar) can skip pulling SurrealKV and the in-memory store. catgraph workspace pinned to `v0.11.4` (the Phase W.1 co-release tag that introduces the `parallel` feature gate).

### Added

- **`native-embedded` cargo feature** (default-on) — wires `surrealdb/kv-mem` and `surrealdb/kv-surrealkv`. Disable with `--no-default-features` on builds that only need a remote client.
- **`remote-ws` / `remote-http` cargo features** (opt-in) — enable the `protocol-ws` / `protocol-http` client transports independently of the embedded backends. Combine with `native-embedded` when a process wants both dialects.
- `examples/wasi_edge_client.rs` — sidecar-pattern smoke test showing the async store workflow (cospan save → load → delete) under a trimmed tokio runtime. Demonstrates the `"mem://"` ↔ `"ws://host:8000"` swap that the engine-generalization now unlocks.

### Changed

- **Store API generalized to `Surreal<engine::any::Any>`.** Every store (`CospanStore`, `SpanStore`, `NamedCospanStore`, `NodeStore`, `EdgeStore`, `HyperedgeStore`, `PetriNetStore`, `WiringDiagramStore`, `HypergraphEvolutionStore`, `FingerprintEngine`, `QueryHelper`) and the two free `init_schema` / `init_schema_v2` functions now take `&Surreal<Any>`. Call sites that previously built `Surreal::new::<Mem>(())` switch to `surrealdb::engine::any::connect("mem://")`; swapping to `"ws://host:8000"` or `"surrealkv://path"` requires no other changes. **Breaking** for any external consumer that held a `Surreal<Db>` — but there are no known external consumers of this crate yet.
- `tokio` dep moved from `features = ["full"]` to `default-features = false, features = ["rt", "sync", "macros", "time"]`. `time` retained for `tokio::time::sleep` in `hyperedge::decompose` retry backoff. `full`'s `signal`/`process`/`net`/`io-std`/`fs` bits are not used directly by this crate and don't build on `wasm32-wasip1-*`.
- `catgraph`, `catgraph-physics`, `catgraph-applied` dep tags bumped from `v0.11.0` to `v0.11.4` (same tag across all three for Cargo source deduplication). v0.11.4 introduces the `parallel` feature across the workspace; this crate does not exercise it directly but the inherited default-on behavior preserves existing semantics.
- `README.md` gained a "WASM / edge support" section documenting the `wasm32-wasip1-threads` build story and the engine-feature matrix.
- `CLAUDE.md` dep list updated to reflect tokio trim + pinned catgraph tag + engine generalization.

### Removed

- `use surrealdb::engine::local::{Db, Mem}` imports from every store, test, and example. The `local` module is now only reachable through `any::connect("mem://")` / `any::connect("surrealkv://path")`.

### Verified

- Native default features: `cargo test` — **171 tests pass**, 0 failed, 0 ignored — unchanged from v0.9.0 baseline.
- `cargo check --lib --no-default-features` (zero engine backends): builds clean. The library compiles against `Surreal<Any>` regardless of which transports are enabled.
- `cargo check --lib --no-default-features --features remote-ws`: builds clean.
- `cargo clippy --lib`: zero warnings.
- WASM (native-embedded-on): `cargo build --lib --target wasm32-wasip1-threads` — verified clean under W.2 partial commit; surrealdb SDK itself blocks running the full remote-client build on WASI (SDK targets `wasm32-unknown-unknown` + JS host). Tracked in `[Unreleased]`.

## [0.9.0] - 2026-04-14

Phase 3.2 — caught up with the catgraph workspace's Phase 3 relocation of applied-CT modules (`petri_net`, `wiring_diagram`, etc.) from `catgraph` to the new `catgraph-applied` workspace member.

### Added

- Direct dependency on `catgraph-applied` (git tag `v0.11.0`, shared with the other two deps for Cargo source dedup).

### Changed

- Import sites for `PetriNetStore`, `WiringDiagramStore`, and the matching V2 tests + examples rewritten from `catgraph::{petri_net,wiring_diagram}::*` to `catgraph_applied::*`. Affected files: `src/petri_net_store.rs`, `src/wiring_store.rs`, `tests/v2_petri_net.rs`, `tests/v2_wiring_diagram.rs`, `examples/petri_net_persistence.rs`.
- `catgraph` + `catgraph-physics` dep tags bumped `v0.10.6` → `v0.11.0` (the catgraph slim-baseline release).

### Removed

- `src/multiway_store.rs` stub module — it only carried a placeholder comment, never implemented save/load, and had zero downstream consumers. Removed the `pub mod` declaration from `lib.rs` and the file itself.
- V1 `vertex` table DDL (3 lines in `src/schema.rs`) — defined in the original V1 schema but never written to or read from by any store. Dead weight.

### Kept (plan was wrong)

- `PersistError::Json` variant was flagged for deletion in the Phase 3.2 scope, but a grep showed it is used via `?` on `serde_json::from_value` in `src/hyperedge/provenance.rs`. Left in place.

## [0.8.0] - 2026-04-12

Phase 2 — workspace restructure in the catgraph repo introduced the `catgraph-physics` workspace member; this release bumps the catgraph dep tag and takes a direct dep on the new crate.

### Added

- Direct dependency on `catgraph-physics` (git tag `v0.10.6`, shared tag with `catgraph` for Cargo source dedup).

### Changed

- Import sites for `HypergraphEvolutionStore` and matching tests rewritten from `catgraph::hypergraph::*` to `catgraph_physics::hypergraph::*`. Affected files: `src/hypergraph_evolution_store.rs`, `tests/v2_hypergraph_evolution.rs`.
- `catgraph` dep tag bumped `v0.10.5` → `v0.10.6`.

## [0.7.2] - 2026-04-11

Phase 1 — catgraph workspace moved eight modules back to the `irreducible` sibling repo; no imports in catgraph-surreal were affected.

### Changed

- `catgraph` dep tag bumped `v0.10.4` → `v0.10.5`. No source changes.

## [0.7.1] - 2026-04-11

Phase 0.5 — catgraph workspace closed five F&S audit gaps; no API surface consumed by this crate changed.

### Changed

- `catgraph` dep tag bumped `v0.10.3` → `v0.10.4`. No source changes.

## [0.7.0] - 2026-04-11

Phase 0.0 — initial release as a sibling repo. `catgraph` was restructured into a virtual workspace and the SurrealDB persistence layer was extracted from the catgraph tree into a standalone repository with its own release cadence.

### Added

- Cospan, NamedCospan, Span V1 stores (embedded-array schema).
- V2 graph-native schema with first-class nodes, RELATE edges, hub-node reification for n-ary hyperedges.
- FTS + HNSW indexes for similarity search.
- Petri net + wiring diagram stores (V2 decompose patterns).
- Hyperedge decompose + provenance modules.
- Fingerprint module for content-addressable persistence.

[Unreleased]: https://github.com/tsondru/catgraph-surreal/compare/v0.10.0...HEAD
[0.10.0]: https://github.com/tsondru/catgraph-surreal/releases/tag/v0.10.0
[0.9.0]: https://github.com/tsondru/catgraph-surreal/releases/tag/v0.9.0
[0.8.0]: https://github.com/tsondru/catgraph-surreal/releases/tag/v0.8.0
[0.7.2]: https://github.com/tsondru/catgraph-surreal/releases/tag/v0.7.2
[0.7.1]: https://github.com/tsondru/catgraph-surreal/releases/tag/v0.7.1
[0.7.0]: https://github.com/tsondru/catgraph-surreal/releases/tag/v0.7.0
