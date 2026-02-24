use std::f32::consts::PI;

const TICK_SPACING: f32 = 0.1;
const AXIS_HALF_T: f32 = 0.005; // half-thickness of main axis lines in NDC Y units
const GRID_HALF_T: f32 = 0.003; // thick enough to survive Retina/HiDPI scaling
const DOT_RADIUS: f32 = 0.007; // in NDC Y units
const DOT_SEGMENTS: u32 = 12;
// Grid lines always fill the full NDC range regardless of arm_len
const GRID_FULL_SPAN: f32 = 1.0;

const DARK_BLUE: [f32; 4] = [0.05, 0.15, 0.7, 1.0];
const GREY: [f32; 4] = [0.6, 0.6, 0.6, 1.0];

fn vert(x: f32, y: f32, c: [f32; 4]) -> [f32; 6] {
    [x, y, c[0], c[1], c[2], c[3]]
}

// Horizontal quad from (x0, y-ty) to (x1, y+ty)
fn h_quad(v: &mut Vec<[f32; 6]>, x0: f32, x1: f32, y: f32, ty: f32, c: [f32; 4]) {
    v.push(vert(x0, y - ty, c));
    v.push(vert(x1, y - ty, c));
    v.push(vert(x1, y + ty, c));
    v.push(vert(x0, y - ty, c));
    v.push(vert(x1, y + ty, c));
    v.push(vert(x0, y + ty, c));
}

// Vertical quad from (x-tx, y0) to (x+tx, y1)
// tx is already aspect-corrected by the caller
fn v_quad(v: &mut Vec<[f32; 6]>, x: f32, y0: f32, y1: f32, tx: f32, c: [f32; 4]) {
    v.push(vert(x - tx, y0, c));
    v.push(vert(x + tx, y0, c));
    v.push(vert(x + tx, y1, c));
    v.push(vert(x - tx, y0, c));
    v.push(vert(x + tx, y1, c));
    v.push(vert(x - tx, y1, c));
}

// Filled circle (dot) at (cx, cy). rx = aspect-corrected x radius, ry = y radius.
fn dot(v: &mut Vec<[f32; 6]>, cx: f32, cy: f32, rx: f32, ry: f32, c: [f32; 4]) {
    for i in 0..DOT_SEGMENTS {
        let a1 = 2.0 * PI * i as f32 / DOT_SEGMENTS as f32;
        let a2 = 2.0 * PI * (i + 1) as f32 / DOT_SEGMENTS as f32;
        v.push(vert(cx, cy, c));
        v.push(vert(cx + a1.cos() * rx, cy + a1.sin() * ry, c));
        v.push(vert(cx + a2.cos() * rx, cy + a2.sin() * ry, c));
    }
}

/// Generates TriangleList vertices for axes (thick quads) and tick dots.
/// `aspect` = window width / height, used to keep lines and dots visually square.
pub fn generate_vertices(arm_len: f32, grid: bool, aspect: f32) -> Vec<[f32; 6]> {
    let mut v: Vec<[f32; 6]> = Vec::new();
    let steps = (arm_len / TICK_SPACING).round() as u32;

    // X thickness corrected for aspect so vertical elements look the same width as horizontal
    let axis_tx = AXIS_HALF_T / aspect;
    let dot_rx = DOT_RADIUS / aspect;
    let grid_tx = GRID_HALF_T / aspect;

    // Grid quads first (behind everything).
    // Lines span the full screen (±GRID_FULL_SPAN) and cover all positions up to ±GRID_FULL_SPAN
    // so the grid always fills the viewport regardless of arm_len.
    if grid {
        let grid_steps = (GRID_FULL_SPAN / TICK_SPACING).round() as u32;
        for i in 1..=grid_steps {
            let t = i as f32 * TICK_SPACING;
            h_quad(&mut v, -GRID_FULL_SPAN, GRID_FULL_SPAN,  t, GRID_HALF_T, GREY);
            h_quad(&mut v, -GRID_FULL_SPAN, GRID_FULL_SPAN, -t, GRID_HALF_T, GREY);
            v_quad(&mut v,  t, -GRID_FULL_SPAN, GRID_FULL_SPAN, grid_tx, GREY);
            v_quad(&mut v, -t, -GRID_FULL_SPAN, GRID_FULL_SPAN, grid_tx, GREY);
        }
    }

    // Main axes (thick, dark blue) — span full screen in grid mode, arm_len in axis mode
    let axes_span = if grid { GRID_FULL_SPAN } else { arm_len };
    h_quad(&mut v, -axes_span, axes_span, 0.0, AXIS_HALF_T, DARK_BLUE);
    v_quad(&mut v, 0.0, -axes_span, axes_span, axis_tx, DARK_BLUE);

    // Tick dots at each interval position
    for i in 1..=steps {
        let t = i as f32 * TICK_SPACING;
        dot(&mut v,  t, 0.0, dot_rx, DOT_RADIUS, DARK_BLUE);
        dot(&mut v, -t, 0.0, dot_rx, DOT_RADIUS, DARK_BLUE);
        dot(&mut v, 0.0,  t, dot_rx, DOT_RADIUS, DARK_BLUE);
        dot(&mut v, 0.0, -t, dot_rx, DOT_RADIUS, DARK_BLUE);
    }

    v
}
