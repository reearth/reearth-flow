use crate::rasterize::Canvas;
use glam::{Mat4, Vec3};

/// Renders a depth buffer of world-space `triangles`, already camera-relative
/// (see `input::recenter_and_cast`), as a `Canvas`. Pixel values are the actual
/// Euclidean distance from the camera to the nearest surface, in the same
/// units as the input geometry; background pixels are `f32::INFINITY`.
///
/// Distance (not normalized `[0, 1]` NDC depth) so the result carries a fixed,
/// scale-independent meaning regardless of a scene's size, and so an absolute
/// positioning regression (e.g. a global coordinate offset bug) still shows up
/// as a real difference instead of being invisible under a per-scene-relative
/// normalization.
///
/// Backface-culls CCW-front triangles (glTF convention). No near-plane
/// clipping: a triangle with a vertex behind the camera is dropped whole.
pub fn render_depth(triangles: &[[Vec3; 3]], view_proj: Mat4, width: usize, height: usize) -> Canvas {
    let mut zbuffer = vec![f32::INFINITY; width * height];

    for tri in triangles {
        if let Some(screen) = project_triangle(tri, view_proj, width, height) {
            rasterize_triangle(&screen, &mut zbuffer, width, height);
        }
    }

    Canvas {
        data: zbuffer,
        width,
        height,
    }
}

#[derive(Clone, Copy)]
struct ScreenVert {
    x: f32,
    y: f32,
    /// 1/clip.w, for perspective-correct interpolation of `dist_over_w`.
    inv_w: f32,
    /// Euclidean distance from the camera to this vertex, divided by clip.w.
    dist_over_w: f32,
}

/// Projects a world-space (camera-relative) triangle to screen space. Returns
/// `None` if any vertex is behind the camera.
fn project_triangle(
    tri: &[Vec3; 3],
    view_proj: Mat4,
    width: usize,
    height: usize,
) -> Option<[ScreenVert; 3]> {
    let mut out = [ScreenVert {
        x: 0.0,
        y: 0.0,
        inv_w: 0.0,
        dist_over_w: 0.0,
    }; 3];
    for (i, v) in tri.iter().enumerate() {
        let clip = view_proj * v.extend(1.0);
        if clip.w <= 0.0 {
            return None;
        }
        let ndc = clip.truncate() / clip.w;
        let inv_w = 1.0 / clip.w;
        out[i] = ScreenVert {
            x: (ndc.x * 0.5 + 0.5) * width as f32,
            y: (1.0 - (ndc.y * 0.5 + 0.5)) * height as f32,
            inv_w,
            dist_over_w: v.length() * inv_w,
        };
    }
    Some(out)
}

fn edge(a: (f32, f32), b: (f32, f32), p: (f32, f32)) -> f32 {
    (b.0 - a.0) * (p.1 - a.1) - (b.1 - a.1) * (p.0 - a.0)
}

/// Edge-function rasterization over the triangle's screen-space bounding box.
fn rasterize_triangle(v: &[ScreenVert; 3], zbuffer: &mut [f32], width: usize, height: usize) {
    let (p0, p1, p2) = ((v[0].x, v[0].y), (v[1].x, v[1].y), (v[2].x, v[2].y));
    let area = edge(p0, p1, p2);
    // CCW-front (glTF) becomes negative-area in screen space, since the NDC->screen
    // y-flip reverses winding sense. area >= 0 is degenerate or backfacing.
    if area >= 0.0 {
        return;
    }

    let min_x_f = p0.0.min(p1.0).min(p2.0).floor();
    let max_x_f = p0.0.max(p1.0).max(p2.0).ceil();
    let min_y_f = p0.1.min(p1.1).min(p2.1).floor();
    let max_y_f = p0.1.max(p1.1).max(p2.1).ceil();

    if max_x_f < 0.0 || max_y_f < 0.0 || min_x_f >= width as f32 || min_y_f >= height as f32 {
        return;
    }

    let min_x = min_x_f.max(0.0) as usize;
    let min_y = min_y_f.max(0.0) as usize;
    let max_x = (max_x_f.min(width as f32 - 1.0)) as usize;
    let max_y = (max_y_f.min(height as f32 - 1.0)) as usize;

    for py in min_y..=max_y {
        for px in min_x..=max_x {
            let p = (px as f32 + 0.5, py as f32 + 0.5);
            let w0 = edge(p1, p2, p);
            let w1 = edge(p2, p0, p);
            let w2 = edge(p0, p1, p);
            if w0 > 0.0 || w1 > 0.0 || w2 > 0.0 {
                continue;
            }
            // Perspective-correct interpolation: 1/w and (attr/w) are affine in
            // screen space, so interpolate those, then recover the attribute.
            let inv_w = (w0 * v[0].inv_w + w1 * v[1].inv_w + w2 * v[2].inv_w) / area;
            let dist_over_w =
                (w0 * v[0].dist_over_w + w1 * v[1].dist_over_w + w2 * v[2].dist_over_w) / area;
            let depth = dist_over_w / inv_w;
            let idx = py * width + px;
            if depth < zbuffer[idx] {
                zbuffer[idx] = depth;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_depth_single_triangle_covers_expected_pixels() {
        let width = 64;
        let height = 64;
        let view_proj = Mat4::orthographic_rh(-1.0, 1.0, -1.0, 1.0, 0.1, 10.0);
        let triangles = vec![[
            Vec3::new(-0.5, -0.5, -1.0),
            Vec3::new(0.5, -0.5, -1.0),
            Vec3::new(0.0, 0.5, -1.0),
        ]];

        let canvas = render_depth(&triangles, view_proj, width, height);

        let center_idx = (height / 2) * width + width / 2;
        assert!(
            canvas.data[center_idx].is_finite(),
            "center pixel should be covered by the triangle, got {}",
            canvas.data[center_idx]
        );

        let corner_idx = 0;
        assert_eq!(
            canvas.data[corner_idx],
            f32::INFINITY,
            "corner pixel should be background"
        );
    }

    #[test]
    fn test_render_depth_nearer_triangle_wins_zbuffer() {
        let width = 16;
        let height = 16;
        let view_proj = Mat4::orthographic_rh(-1.0, 1.0, -1.0, 1.0, 0.1, 10.0);

        let far_tri = [
            Vec3::new(-1.0, -1.0, -5.0),
            Vec3::new(1.0, -1.0, -5.0),
            Vec3::new(0.0, 1.0, -5.0),
        ];
        let near_tri = [
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, -1.0, -1.0),
            Vec3::new(0.0, 1.0, -1.0),
        ];

        let far_first = render_depth(&[far_tri, near_tri], view_proj, width, height);
        let near_first = render_depth(&[near_tri, far_tri], view_proj, width, height);

        let center_idx = (height / 2) * width + width / 2;
        assert!((far_first.data[center_idx] - near_first.data[center_idx]).abs() < 1e-6,
            "z-buffer result should not depend on draw order: {} vs {}", far_first.data[center_idx], near_first.data[center_idx]);
    }

    #[test]
    fn test_render_depth_triangle_behind_camera_is_dropped() {
        let width = 8;
        let height = 8;
        let view_proj = Mat4::perspective_rh(90.0_f32.to_radians(), 1.0, 1.0, 100.0)
            * Mat4::look_at_rh(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0), Vec3::Y);
        // Entirely behind the camera (positive z, looking down -z).
        let triangles = vec![[
            Vec3::new(-1.0, -1.0, 5.0),
            Vec3::new(1.0, -1.0, 5.0),
            Vec3::new(0.0, 1.0, 5.0),
        ]];

        let canvas = render_depth(&triangles, view_proj, width, height);
        assert!(
            canvas.data.iter().all(|&d| d == f32::INFINITY),
            "no pixel should be covered"
        );
    }

    #[test]
    fn test_render_depth_culls_backfacing_triangle() {
        let width = 32;
        let height = 32;
        let view_proj = Mat4::orthographic_rh(-1.0, 1.0, -1.0, 1.0, 0.1, 10.0);
        // CCW (front-facing): visible.
        let front = [
            Vec3::new(-0.5, -0.5, -1.0),
            Vec3::new(0.5, -0.5, -1.0),
            Vec3::new(0.0, 0.5, -1.0),
        ];
        // Reversed winding: backfacing, culled.
        let back = [front[0], front[2], front[1]];

        let front_canvas = render_depth(&[front], view_proj, width, height);
        let back_canvas = render_depth(&[back], view_proj, width, height);

        let center_idx = (height / 2) * width + width / 2;
        assert!(
            front_canvas.data[center_idx].is_finite(),
            "front-facing triangle should be visible"
        );
        assert!(
            back_canvas.data.iter().all(|&d| d == f32::INFINITY),
            "backfacing triangle should be culled"
        );
    }
}
