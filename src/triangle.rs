use crate::fragment::Fragment;
use crate::line::line;
use crate::vertex::Vertex;
use raylib::prelude::*;

// Fill rule shared by both filled modes: a pixel belongs to the triangle when
// its center (x + 0.5, y + 0.5) is strictly inside, or exactly on a top or
// left edge (top-left rule). Two triangles that share an edge never both
// produce a fragment for the same pixel, and no pixel is left out between them.

/// Triangle outline: three lines.
pub fn triangle(a: &Vertex, b: &Vertex, c: &Vertex) -> Vec<Fragment> {
    let mut fragments = line(a, b);
    fragments.extend(line(b, c));
    fragments.extend(line(c, a));
    fragments
}

/// Twice the signed area of (a, b, p). Positive when p is on the interior side
/// of a -> b for triangles we have oriented with `edge(a, b, c) > 0`.
fn edge(a: Vector3, b: Vector3, px: f32, py: f32) -> f32 {
    (b.x - a.x) * (py - a.y) - (b.y - a.y) * (px - a.x)
}

/// With the orientation above and screen y pointing down, an edge going up is
/// a left edge and a horizontal edge going right is a top edge.
fn is_top_left(a: Vector3, b: Vector3) -> bool {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    dy < 0.0 || (dy == 0.0 && dx > 0.0)
}

fn covers(w: f32, top_left: bool) -> bool {
    w > 0.0 || (w == 0.0 && top_left)
}

/// Fill by testing every pixel center in the bounding box with barycentric coordinates.
pub fn triangle_barycentric(a: &Vertex, b: &Vertex, c: &Vertex) -> Vec<Fragment> {
    let (pa, mut pb, mut pc) = (a.transformed_position, b.transformed_position, c.transformed_position);
    let mut area = edge(pa, pb, pc.x, pc.y);
    if area == 0.0 {
        return Vec::new(); // degenerate: the three vertices are collinear
    }
    if area < 0.0 {
        // reversed vertex order: swap so the edge tests keep the same sign
        std::mem::swap(&mut pb, &mut pc);
        area = -area;
    }
    let color = a.color;
    let (tl_bc, tl_ca, tl_ab) = (is_top_left(pb, pc), is_top_left(pc, pa), is_top_left(pa, pb));

    // pixels whose center lies inside the bounding box
    let min_x = (pa.x.min(pb.x).min(pc.x) - 0.5).ceil() as i32;
    let max_x = (pa.x.max(pb.x).max(pc.x) - 0.5).floor() as i32;
    let min_y = (pa.y.min(pb.y).min(pc.y) - 0.5).ceil() as i32;
    let max_y = (pa.y.max(pb.y).max(pc.y) - 0.5).floor() as i32;

    let mut fragments = Vec::new();
    for y in min_y..=max_y {
        let py = y as f32 + 0.5;
        for x in min_x..=max_x {
            let px = x as f32 + 0.5;
            let w_a = edge(pb, pc, px, py);
            let w_b = edge(pc, pa, px, py);
            let w_c = edge(pa, pb, px, py);
            if covers(w_a, tl_bc) && covers(w_b, tl_ca) && covers(w_c, tl_ab) {
                // barycentric coordinates: weights of a, b and c, summing to 1
                let (u, v, w) = (w_a / area, w_b / area, w_c / area);
                let depth = u * pa.z + v * pb.z + w * pc.z;
                fragments.push(Fragment::new(x as f32, y as f32, color, depth));
            }
        }
    }
    fragments
}

/// x and z where the edge p -> q crosses the horizontal line y.
fn cross_at(p: Vector3, q: Vector3, y: f32) -> (f32, f32) {
    let t = (y - p.y) / (q.y - p.y);
    (p.x + (q.x - p.x) * t, p.z + (q.z - p.z) * t)
}

/// Fill one row at a time between the two edges that cross it.
pub fn triangle_scanline(a: &Vertex, b: &Vertex, c: &Vertex) -> Vec<Fragment> {
    let mut p = [a.transformed_position, b.transformed_position, c.transformed_position];
    if edge(p[0], p[1], p[2].x, p[2].y) == 0.0 {
        return Vec::new(); // degenerate: the three vertices are collinear
    }
    let color = a.color;
    p.sort_by(|u, v| u.y.total_cmp(&v.y));
    let [top, mid, bottom] = p;

    // rows whose center is in [top.y, bottom.y): a flat top edge is included,
    // a flat bottom edge is not
    let first_row = (top.y - 0.5).ceil() as i32;
    let end_row = (bottom.y - 0.5).ceil() as i32;

    let mut fragments = Vec::new();
    for y in first_row..end_row {
        let py = y as f32 + 0.5;
        let long = cross_at(top, bottom, py);
        // at py == mid.y use the lower edge; this also skips a flat top edge
        let short = if py < mid.y { cross_at(top, mid, py) } else { cross_at(mid, bottom, py) };
        let (left, right) = if long.0 <= short.0 { (long, short) } else { (short, long) };

        // columns whose center is in [left, right): left edge in, right edge out
        let first_col = (left.0 - 0.5).ceil() as i32;
        let end_col = (right.0 - 0.5).ceil() as i32;
        let width = right.0 - left.0;
        for x in first_col..end_col {
            let px = x as f32 + 0.5;
            let t = if width > 0.0 { (px - left.0) / width } else { 0.0 };
            let depth = left.1 + (right.1 - left.1) * t;
            fragments.push(Fragment::new(x as f32, y as f32, color, depth));
        }
    }
    fragments
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn v(x: f32, y: f32) -> Vertex {
        Vertex::new(Vector3::new(x, y, 0.0), Vector3::new(1.0, 1.0, 1.0))
    }

    fn pixels(fragments: &[Fragment]) -> BTreeSet<(i32, i32)> {
        fragments
            .iter()
            .map(|f| (f.position.x as i32, f.position.y as i32))
            .collect()
    }

    /// Both fills generate exactly the same pixels, with no duplicates.
    fn assert_same(a: Vertex, b: Vertex, c: Vertex) -> usize {
        let bary = triangle_barycentric(&a, &b, &c);
        let scan = triangle_scanline(&a, &b, &c);
        let (pb, ps) = (pixels(&bary), pixels(&scan));
        assert_eq!(pb.len(), bary.len(), "barycentric produced duplicates");
        assert_eq!(ps.len(), scan.len(), "scanline produced duplicates");
        assert_eq!(pb, ps);
        bary.len()
    }

    #[test]
    fn small_and_large() {
        assert!(assert_same(v(1.0, 1.0), v(4.0, 2.0), v(2.0, 5.0)) > 0);
        assert!(assert_same(v(10.0, 20.0), v(700.0, 80.0), v(300.0, 550.0)) > 100_000);
    }

    #[test]
    fn horizontal_and_vertical_edges() {
        // flat top, flat bottom, vertical left edge
        assert_same(v(10.0, 10.0), v(90.0, 10.0), v(40.0, 70.0));
        assert_same(v(40.0, 10.0), v(90.0, 70.0), v(10.0, 70.0));
        assert_same(v(10.0, 10.0), v(10.0, 90.0), v(70.0, 50.0));
    }

    #[test]
    fn reversed_order_gives_same_pixels() {
        let (a, b, c) = (v(10.0, 10.0), v(90.0, 30.0), v(40.0, 70.0));
        let forward = pixels(&triangle_barycentric(&a, &b, &c));
        let reversed = pixels(&triangle_barycentric(&c, &b, &a));
        assert!(!forward.is_empty());
        assert_eq!(forward, reversed);
        assert_eq!(forward, pixels(&triangle_scanline(&c, &b, &a)));
    }

    #[test]
    fn partially_outside() {
        assert_same(v(-50.0, -30.0), v(120.0, 10.0), v(20.0, 900.0));
    }

    #[test]
    fn collinear_is_empty() {
        assert_eq!(assert_same(v(0.0, 0.0), v(50.0, 50.0), v(100.0, 100.0)), 0);
        assert_eq!(assert_same(v(0.0, 5.0), v(50.0, 5.0), v(100.0, 5.0)), 0);
    }

    #[test]
    fn shared_edge_has_no_gaps_or_overlap() {
        // a square split along its diagonal covers each pixel exactly once
        let (a, b, c, d) = (v(0.0, 0.0), v(40.0, 0.0), v(40.0, 40.0), v(0.0, 40.0));
        for fill in [triangle_barycentric, triangle_scanline] {
            let mut all = pixels(&fill(&a, &b, &c));
            let second = pixels(&fill(&a, &c, &d));
            assert!(all.is_disjoint(&second));
            all.extend(second);
            assert_eq!(all.len(), 40 * 40);
        }
    }
}
