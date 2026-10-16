use crate::fragment::Fragment;
use crate::vertex::Vertex;
use raylib::prelude::*;

/// Bresenham line between the transformed positions of two vertices.
pub fn line(a: &Vertex, b: &Vertex) -> Vec<Fragment> {
    let start = a.transformed_position;
    let end = b.transformed_position;
    let white = Vector3::new(1.0, 1.0, 1.0);

    let mut x0 = start.x.round() as i32;
    let mut y0 = start.y.round() as i32;
    let x1 = end.x.round() as i32;
    let y1 = end.y.round() as i32;

    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let steps = dx.max(-dy).max(1) as f32;

    let mut fragments = Vec::new();
    let mut i = 0.0;
    loop {
        let depth = start.z + (end.z - start.z) * (i / steps);
        fragments.push(Fragment::new(x0 as f32, y0 as f32, white, depth));
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
        i += 1.0;
    }
    fragments
}
