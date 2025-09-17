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
    // Prefer native wasm exports for springs and central gravity if available in the pkg.
    // Otherwise fall back to the JS adapters below. This allows progressive enhancement
    // while the Rust crate is updated and the pkg rebuilt.
    if (typeof pkg.compute_springs === "function") {
      window.forceatlas_compute_springs = pkg.compute_springs;
    }
    if (typeof pkg.compute_central_gravity === "function") {
      window.forceatlas_compute_central_gravity = pkg.compute_central_gravity;
    }
    // If native wasm helpers weren't found, provide JS fallbacks so callers can use the same API.
    /**
     * Compute spring forces in a typed arrays based representation.
     * positions: Float64Array [x0,y0,x1,y1,...]
     * fromIdx: Float64Array of integer indices (into node array)
     * toIdx: Float64Array of integer indices (into node array)
     * lengths: Float64Array of desired edge lengths
     * springConstant: number
     * returns Float64Array of forces [fx0,fy0,...]
     */
    if (typeof window.forceatlas_compute_springs !== "function") {
      window.forceatlas_compute_springs = function (positions, fromIdx, toIdx, lengths, springConstant) {
      const n = positions.length / 2;
      const forces = new Float64Array(n * 2);

      const m = fromIdx.length;
      for (let i = 0; i < m; i++) {
        const fi = Math.trunc(fromIdx[i]);
        const ti = Math.trunc(toIdx[i]);
        if (fi < 0 || fi >= n || ti < 0 || ti >= n) continue;

        const fx = positions[2 * fi] - positions[2 * ti];
        const fy = positions[2 * fi + 1] - positions[2 * ti + 1];
        let distance = Math.sqrt(fx * fx + fy * fy);
        distance = Math.max(distance, 0.01);

        const edgeLength = lengths && lengths.length > i ? lengths[i] : 0;
        const springForce = (springConstant * (edgeLength - distance)) / distance;

        const sx = fx * springForce;
        const sy = fy * springForce;

        forces[2 * fi] += sx;
        forces[2 * fi + 1] += sy;

        forces[2 * ti] -= sx;
        forces[2 * ti + 1] -= sy;
      }

      return forces;
      };
    }

    /**
     * Compute central gravity forces for nodes.
     * positions: Float64Array [x0,y0,...]
     * masses: Float64Array [m0,m1,...]
     * centralGravity: number
     * degrees: Float64Array degree multipliers
     * returns Float64Array of forces [fx0,fy0,...]
     */
    if (typeof window.forceatlas_compute_central_gravity !== "function") {
      window.forceatlas_compute_central_gravity = function (positions, masses, centralGravity, degrees) {
      const n = positions.length / 2;
      const forces = new Float64Array(n * 2);

      for (let i = 0; i < n; i++) {
        const dx = -positions[2 * i];
        const dy = -positions[2 * i + 1];
        const degree = degrees && degrees.length > i ? degrees[i] : 1;
        const mass = masses && masses.length > i ? masses[i] : 1;
        const gravityForce = centralGravity * degree * mass;
        forces[2 * i] = dx * gravityForce;
        forces[2 * i + 1] = dy * gravityForce;
      }

        return forces;
      };
    }
    return true;
  }
  return false;
}
