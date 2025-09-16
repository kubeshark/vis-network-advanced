Build instructions for the wasm prototype

This crate exposes a single function `compute_forces(positions, masses, gravitational_constant)`
that takes JS Float64Array objects and returns a Float64Array of forces [fx0, fy0, fx1, fy1, ...].

Quick build (recommended):

1) Install `wasm-pack` (https://rustwasm.github.io/wasm-pack/installer/)

2) From repo root:

```bash
cd wasm/forceatlas
wasm-pack build --target web
```

This produces `wasm/forceatlas/pkg` with JS glue. Copy or serve that `pkg` to the browser and expose
`window.forceatlas_compute_forces = pkg.compute_forces` (or load the module with dynamic import).

Alternative (manual): build with `cargo build --target wasm32-unknown-unknown` and run `wasm-bindgen`.

Notes:
- Current implementation is pairwise O(N^2) as a correctness-first prototype. The Barnes-Hut tree (O(N log N))
  should be implemented in Rust next for large graphs.

WebAssembly threading notes
--------------------------
Wasm supports native threads via Rust's `rayon` and `wasm-bindgen` if the browser environment provides
SharedArrayBuffer and cross-origin isolation headers (COOP/COEP). For large graphs, parallelizing the tree
traversal or force accumulation can significantly improve performance.

However, enabling threads requires:
- Serving the page with COOP and COEP response headers to enable cross-origin isolation.
- Building the wasm with `--features=console_error_panic_hook` and thread support and using `wasm-bindgen`/`wasm-pack` with appropriate flags.

Given the extra deployment and complexity constraints, I recommend:
1) First implement the single-threaded Barnes-Hut in Rust and validate correctness and speedups.
2) If further speed is needed and COOP/COEP headers are feasible for your deployment, extend the crate to use rayon and wasm threads.

