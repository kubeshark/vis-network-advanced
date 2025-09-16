// Example loader for the wasm package produced by `wasm-pack build --target web`
// Put the generated `pkg` folder from wasm-pack in the same directory or serve it.

export async function loadForceAtlasWasm() {
  // adjust path to pkg as needed
  const pkg = await import("./pkg/forceatlas_wasm.js");
  // wasm-pack modules export a default async initializer. Call it before using exported functions.
  try {
    if (typeof pkg.default === "function") {
      await pkg.default();
    } else if (typeof pkg.init === "function") {
      await pkg.init();
    }
  } catch (e) {
    // initialization failed
    console.error("Failed to initialize ForceAtlas wasm module:", e);
    return false;
  }

  if (pkg && typeof pkg.compute_forces === "function") {
    window.forceatlas_compute_forces = pkg.compute_forces;
    return true;
  }
  return false;
}
