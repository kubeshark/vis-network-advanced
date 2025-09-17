use wasm_bindgen::prelude::*;
use js_sys::Float64Array;

// Barnes-Hut based force computation for ForceAtlas2-like repulsion.
// Exports a single function `compute_forces(positions, masses, theta, gravitational, avoid_overlap, radii, degrees)`
// positions: Float64Array [x0,y0,x1,y1,...]
// masses: Float64Array [m0,m1,...]
// theta: threshold for Barnes-Hut approximation (typical ~0.5)
// gravitational: gravitationalConstant (can be negative for repulsion)
// avoid_overlap: numeric option (0..1) controlling overlap avoidance
// radii: Float64Array of node radii (optional, pass zeros if unused)
// degrees: Float64Array of node degree multipliers (edges.length + 1)

#[wasm_bindgen]
pub fn compute_forces(
    positions: &Float64Array,
    masses: &Float64Array,
    theta: f64,
    gravitational: f64,
    avoid_overlap: f64,
    radii: &Float64Array,
    degrees: &Float64Array,
) -> Float64Array {
    let len = positions.length() / 2;
    let n = len as usize;

    // copy inputs
    let mut pos = vec![0.0f64; n * 2];
    positions.copy_to(&mut pos[..]);

    let mut m = vec![1.0f64; n];
    if masses.length() as usize >= n {
        masses.copy_to(&mut m[..]);
    }

    let mut r = vec![0.0f64; n];
    if radii.length() as usize >= n {
        radii.copy_to(&mut r[..]);
    }

    let mut deg = vec![1.0f64; n];
    if degrees.length() as usize >= n {
        degrees.copy_to(&mut deg[..]);
    }

    // Build Barnes-Hut quadtree
    #[derive(Default)]
    struct Branch {
        mass: f64,
        com_x: f64,
        com_y: f64,
        minx: f64,
        miny: f64,
        size: f64,
        // if leaf and has a single point, store index; otherwise index = usize::MAX
        point_index: usize,
        children: Option<Box<[Branch; 4]>>,
    }

    impl Branch {
        fn new(minx: f64, miny: f64, size: f64) -> Self {
            Branch {
                mass: 0.0,
                com_x: 0.0,
                com_y: 0.0,
                minx,
                miny,
                size,
                point_index: usize::MAX,
                children: None,
            }
        }

        fn contains(&self, x: f64, y: f64) -> bool {
            x >= self.minx && x <= self.minx + self.size && y >= self.miny && y <= self.miny + self.size
        }

        fn center(&self) -> (f64, f64) {
            (self.minx + 0.5 * self.size, self.miny + 0.5 * self.size)
        }
    }

    // find bounding square
    let mut minx = pos[0];
    let mut maxx = pos[0];
    let mut miny = pos[1];
    let mut maxy = pos[1];
    for i in 1..n {
        let x = pos[2 * i];
        let y = pos[2 * i + 1];
        if x < minx { minx = x; }
        if x > maxx { maxx = x; }
        if y < miny { miny = y; }
        if y > maxy { maxy = y; }
    }
    let dx = maxx - minx;
    let dy = maxy - miny;
    let size = if dx > dy { dx } else { dy };
    let half_extension = 0.5 * (size - dx);
    let minx = minx - half_extension;
    let miny = miny - 0.5 * (size - dy);

    let mut root = Branch::new(minx, miny, if size == 0.0 { 1e-6 } else { size });

    // helper to update branch mass/center
    fn update_branch_mass(branch: &mut Branch, idx: usize, pos: &Vec<f64>, masses: &Vec<f64>) {
        let px = pos[2 * idx];
        let py = pos[2 * idx + 1];
        let total = branch.mass + masses[idx];
        if total > 0.0 {
            branch.com_x = (branch.com_x * branch.mass + px * masses[idx]) / total;
            branch.com_y = (branch.com_y * branch.mass + py * masses[idx]) / total;
            branch.mass = total;
        }
    }

    // insert point recursively
    fn insert_point(branch: &mut Branch, idx: usize, pos: &Vec<f64>, masses: &Vec<f64>) {
        // update mass
        update_branch_mass(branch, idx, pos, masses);

        // if leaf and no point stored -> store
        if branch.children.is_none() && branch.point_index == usize::MAX {
            branch.point_index = idx;
            return;
        }

        // if leaf and has a point, split
        if branch.children.is_none() {
            // create children
            let half = 0.5 * branch.size;
            let minx = branch.minx;
            let miny = branch.miny;
            let mut kids: Box<[Branch;4]> = Box::new([Branch::default(), Branch::default(), Branch::default(), Branch::default()]);
            kids[0] = Branch::new(minx, miny, half); // NW
            kids[1] = Branch::new(minx + half, miny, half); // NE
            kids[2] = Branch::new(minx, miny + half, half); // SW
            kids[3] = Branch::new(minx + half, miny + half, half); // SE
            // move existing point into child
            let existing = branch.point_index;
            branch.point_index = usize::MAX;
            branch.children = Some(kids);
            // place existing
            if existing != usize::MAX {
                // recursive insert into child
                let (ex, ey) = (pos[2 * existing], pos[2 * existing + 1]);
                if let Some(children) = &mut branch.children {
                    for child in children.iter_mut() {
                        if child.contains(ex, ey) {
                            insert_point(child, existing, pos, masses);
                            break;
                        }
                    }
                }
            }
        }

        // now insert new point into appropriate child
        if let Some(children) = &mut branch.children {
            let x = pos[2 * idx];
            let y = pos[2 * idx + 1];
            for child in children.iter_mut() {
                if child.contains(x, y) {
                    insert_point(child, idx, pos, masses);
                    return;
                }
            }
            // if none matched (edge case), put into first child
            insert_point(&mut children[0], idx, pos, masses);
        }
    }

    // build tree
    for i in 0..n {
        insert_point(&mut root, i, &pos, &m);
    }

    // compute forces by traversing tree
    let mut forces = vec![0.0f64; n * 2];

    // overlap avoidance factor translation (JS uses 1 - clamp(avoidOverlap,0,1))
    let overlap_avoidance_factor = 1.0 - if avoid_overlap.is_finite() {
        if avoid_overlap < 0.0 { 0.0 } else if avoid_overlap > 1.0 { 1.0 } else { avoid_overlap }
    } else { 0.0 };

    // recursive traversal computing force on index i
    fn accumulate_force(branch: &Branch, i: usize, pos: &Vec<f64>, masses: &Vec<f64>, forces: &mut Vec<f64>, theta: f64, g: f64, overlap_factor: f64, radii: &Vec<f64>, degrees: &Vec<f64>) {
        // avoid empty
        if branch.mass == 0.0 { return; }

        let xi = pos[2 * i];
        let yi = pos[2 * i + 1];

        // if this branch is a leaf and contains the same point
        if branch.children.is_none() && branch.point_index == i {
            return;
        }

        // compute dx,dy from center of mass
        let dx = branch.com_x - xi;
        let dy = branch.com_y - yi;
        let mut dist2 = dx * dx + dy * dy;
        if dist2 == 0.0 {
            // jitter
            dist2 = 1e-6;
        }
        let dist = dist2.sqrt();

        // if branch is sufficiently far, approximate
        if branch.size / dist < theta {
            // overlap avoidance similar to JS
            let mut d = dist;
            if overlap_factor < 1.0 && radii[i] > 0.0 {
                d = d.max(0.1 + overlap_factor * radii[i]);
                d = d.max(dist - radii[i]);
            }
            // Match JS ForceAtlas2-based solver: use degree multiplier and distance^2 scaling
            let degree = degrees[i];
            // gravity here may be negative for repulsion
            let force_mag = (g * branch.mass * masses[i] * degree) / (d * d);
            let fx = dx * force_mag;
            let fy = dy * force_mag;
            forces[2 * i] += fx;
            forces[2 * i + 1] += fy;
        } else {
            // descend if children exist
            if let Some(children) = &branch.children {
                for child in children.iter() {
                    accumulate_force(child, i, pos, masses, forces, theta, g, overlap_factor, radii, degrees);
                }
            } else {
                // leaf with single different point
                if branch.point_index != usize::MAX && branch.point_index != i {
                    // direct force (single other point)
                    let j = branch.point_index;
                    let dx = pos[2 * j] - xi;
                    let dy = pos[2 * j + 1] - yi;
                    let mut dist2 = dx * dx + dy * dy;
                    if dist2 == 0.0 { dist2 = 1e-6; }
                    let dist = dist2.sqrt();
                    let mut d = dist;
                    if overlap_factor < 1.0 && radii[i] > 0.0 {
                        d = d.max(0.1 + overlap_factor * radii[i]);
                        d = d.max(dist - radii[i]);
                    }
                    let degree = degrees[i];
                    let force_mag = (g * masses[j] * masses[i] * degree) / (d * d);
                    let fx = dx * force_mag;
                    let fy = dy * force_mag;
                    forces[2 * i] += fx;
                    forces[2 * i + 1] += fy;
                }
            }
        }
    }

    for i in 0..n {
        accumulate_force(&root, i, &pos, &m, &mut forces, theta, gravitational, overlap_avoidance_factor, &r, &deg);
    }

    let out = Float64Array::new_with_length((n * 2) as u32);
    out.copy_from(&forces[..]);
    out
}

// Compute spring (edge) forces in Rust and return a Float64Array of per-node forces [fx,fy,...]
#[wasm_bindgen]
pub fn compute_springs(
    positions: &Float64Array,
    from_idx: &Float64Array,
    to_idx: &Float64Array,
    lengths: &Float64Array,
    spring_constant: f64,
) -> Float64Array {
    let n = (positions.length() / 2) as usize;

    // copy positions
    let mut pos = vec![0.0f64; n * 2];
    positions.copy_to(&mut pos[..]);

    let m = from_idx.length() as usize;
    let mut from = vec![0.0f64; m];
    from_idx.copy_to(&mut from[..]);
    let mut to = vec![0.0f64; m];
    to_idx.copy_to(&mut to[..]);

    let mut lens = vec![0.0f64; m];
    if lengths.length() as usize >= m {
        lengths.copy_to(&mut lens[..]);
    }

    let mut forces = vec![0.0f64; n * 2];

    for i in 0..m {
        let fi_f = from[i];
        let ti_f = to[i];
        // allow -1 or invalid indices; ignore those
        let fi = if fi_f.is_finite() { fi_f as isize } else { -1 };
        let ti = if ti_f.is_finite() { ti_f as isize } else { -1 };
        if fi < 0 || ti < 0 { continue; }
        let fi_usize = fi as usize;
        let ti_usize = ti as usize;
        if fi_usize >= n || ti_usize >= n { continue; }

        let fx = pos[2 * fi_usize] - pos[2 * ti_usize];
        let fy = pos[2 * fi_usize + 1] - pos[2 * ti_usize + 1];
        let mut distance = (fx * fx + fy * fy).sqrt();
        if distance < 0.01 { distance = 0.01; }

        let edge_length = if i < lens.len() { lens[i] } else { 0.0 };
        let spring_force = (spring_constant * (edge_length - distance)) / distance;

        let sx = fx * spring_force;
        let sy = fy * spring_force;

        forces[2 * fi_usize] += sx;
        forces[2 * fi_usize + 1] += sy;

        forces[2 * ti_usize] -= sx;
        forces[2 * ti_usize + 1] -= sy;
    }

    let out = Float64Array::new_with_length((n * 2) as u32);
    out.copy_from(&forces[..]);
    out
}

// Compute central gravity per-node in Rust and return Float64Array [fx,fy,...]
#[wasm_bindgen]
pub fn compute_central_gravity(
    positions: &Float64Array,
    masses: &Float64Array,
    central_gravity: f64,
    degrees: &Float64Array,
) -> Float64Array {
    let n = (positions.length() / 2) as usize;

    let mut pos = vec![0.0f64; n * 2];
    positions.copy_to(&mut pos[..]);

    let mut m = vec![1.0f64; n];
    if masses.length() as usize >= n {
        masses.copy_to(&mut m[..]);
    }

    let mut deg = vec![1.0f64; n];
    if degrees.length() as usize >= n {
        degrees.copy_to(&mut deg[..]);
    }

    let mut forces = vec![0.0f64; n * 2];
    for i in 0..n {
        let dx = -pos[2 * i];
        let dy = -pos[2 * i + 1];
        let degree = deg[i];
        let mass = m[i];
        let gravity_force = central_gravity * degree * mass;
        forces[2 * i] = dx * gravity_force;
        forces[2 * i + 1] = dy * gravity_force;
    }

    let out = Float64Array::new_with_length((n * 2) as u32);
    out.copy_from(&forces[..]);
    out
}
