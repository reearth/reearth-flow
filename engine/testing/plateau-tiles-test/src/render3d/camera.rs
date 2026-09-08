use glam::{DVec3, Mat4, Vec3};

/// Camera pose and projection, kept in `f64` since it lives in the same
/// (potentially ECEF-scale) coordinate space as the source mesh data.
#[derive(Debug, Clone, Copy)]
pub struct Camera3d {
    pub position: DVec3,
    pub look_at: DVec3,
    pub up: DVec3,
    pub fov_y_deg: f64,
    pub near: f64,
    pub far: f64,
}

impl Camera3d {
    /// Combined view-projection matrix in `f32`, with `reference` subtracted from
    /// `position`/`look_at` in `f64` first. NDC z range is `[0, 1]` (glam's
    /// `perspective_rh` convention), unrelated to the rasterizer's own depth output.
    pub fn view_proj_f32(&self, width: usize, height: usize, reference: DVec3) -> Mat4 {
        let position: Vec3 = (self.position - reference).as_vec3();
        let look_at: Vec3 = (self.look_at - reference).as_vec3();
        // `up` is a direction, not a position, so no reference offset applies.
        let up: Vec3 = self.up.as_vec3();

        let view = Mat4::look_at_rh(position, look_at, up);
        let aspect = width as f32 / height as f32;
        let proj = Mat4::perspective_rh(
            (self.fov_y_deg as f32).to_radians(),
            aspect,
            self.near as f32,
            self.far as f32,
        );
        proj * view
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_camera() -> Camera3d {
        Camera3d {
            position: DVec3::new(0.0, 0.0, 0.0),
            look_at: DVec3::new(0.0, 0.0, -1.0),
            up: DVec3::new(0.0, 1.0, 0.0),
            fov_y_deg: 90.0,
            near: 1.0,
            far: 101.0,
        }
    }

    // A point on the look-at axis should land at the center of the screen (x=y=0 in NDC).
    #[test]
    fn test_view_proj_center_point() {
        let vp = test_camera().view_proj_f32(100, 100, DVec3::ZERO);
        let clip = vp * Vec3::new(0.0, 0.0, -51.0).extend(1.0);
        let ndc = clip.truncate() / clip.w;

        assert!(ndc.x.abs() < 1e-5, "x should be centered, got {}", ndc.x);
        assert!(ndc.y.abs() < 1e-5, "y should be centered, got {}", ndc.y);
        assert!(ndc.z > 0.0 && ndc.z < 1.0, "depth out of range: {}", ndc.z);
    }

    #[test]
    fn test_view_proj_near_and_far_planes() {
        let vp = test_camera().view_proj_f32(100, 100, DVec3::ZERO);

        let near_clip = vp * Vec3::new(0.0, 0.0, -1.0).extend(1.0);
        let near_ndc_z = near_clip.z / near_clip.w;
        assert!(
            near_ndc_z.abs() < 1e-5,
            "near plane depth should be ~0, got {}",
            near_ndc_z
        );

        let far_clip = vp * Vec3::new(0.0, 0.0, -101.0).extend(1.0);
        let far_ndc_z = far_clip.z / far_clip.w;
        assert!(
            (far_ndc_z - 1.0).abs() < 1e-5,
            "far plane depth should be ~1, got {}",
            far_ndc_z
        );
    }

    // Recentering by `reference` shouldn't change the projected result: shifting both
    // the camera and the reference by the same world-space offset is a no-op.
    #[test]
    fn test_reference_offset_is_invariant() {
        let camera = test_camera();
        let vp_a = camera.view_proj_f32(100, 100, DVec3::ZERO);
        let a = vp_a * Vec3::new(0.0, 0.0, -51.0).extend(1.0);

        let reference = DVec3::new(1000.0, -2000.0, 500.0);
        let shifted = Camera3d {
            position: camera.position + reference,
            look_at: camera.look_at + reference,
            ..camera
        };
        let vp_b = shifted.view_proj_f32(100, 100, reference);
        let b = vp_b * Vec3::new(0.0, 0.0, -51.0).extend(1.0);

        assert!((a.x / a.w - b.x / b.w).abs() < 1e-4);
        assert!((a.y / a.w - b.y / b.w).abs() < 1e-4);
        assert!((a.z / a.w - b.z / b.w).abs() < 1e-4);
    }
}
