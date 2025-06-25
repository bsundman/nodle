//! Mathematical utilities for node graph operations

use egui::Pos2;

/// Calculates a point on a cubic Bézier curve at parameter t (0.0 to 1.0)
pub fn cubic_bezier_point(t: f32, p0: Pos2, p1: Pos2, p2: Pos2, p3: Pos2) -> Pos2 {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;

    Pos2::new(
        mt3 * p0.x + 3.0 * mt2 * t * p1.x + 3.0 * mt * t2 * p2.x + t3 * p3.x,
        mt3 * p0.y + 3.0 * mt2 * t * p1.y + 3.0 * mt * t2 * p2.y + t3 * p3.y,
    )
}

/// Calculates the minimum distance from a point to a line segment
pub fn distance_to_line_segment(point: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let ap = point - a;
    let ab_len_sq = ab.x * ab.x + ab.y * ab.y;

    if ab_len_sq == 0.0 {
        return (point - a).length();
    }

    let t = ((ap.x * ab.x + ap.y * ab.y) / ab_len_sq).clamp(0.0, 1.0);
    let projection = a + ab * t;
    (point - projection).length()
}