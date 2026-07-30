use glam::{DVec3, Vec3};

/// Subtracts `reference` from every vertex in `f64` (precision-safe for
/// ECEF-scale coordinates), then casts down to `f32` — the precision the
/// rasterizer's hot loop operates in. Call this once per camera, with
/// `reference` set to that camera's position, so precision is centered on the
/// actual viewpoint being rendered rather than lost against the absolute
/// coordinate origin.
pub fn recenter_and_cast(triangles: &[[DVec3; 3]], reference: DVec3) -> Vec<[Vec3; 3]> {
    triangles
        .iter()
        .map(|tri| {
            [
                (tri[0] - reference).as_vec3(),
                (tri[1] - reference).as_vec3(),
                (tri[2] - reference).as_vec3(),
            ]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recenter_and_cast_shifts_by_reference() {
        let triangles = vec![[
            DVec3::new(1000.0, 2000.0, 3000.0),
            DVec3::new(1001.0, 2000.0, 3000.0),
            DVec3::new(1000.0, 2001.0, 3000.0),
        ]];
        let reference = DVec3::new(1000.0, 2000.0, 3000.0);

        let result = recenter_and_cast(&triangles, reference);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0][0], Vec3::ZERO);
        assert_eq!(result[0][1], Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(result[0][2], Vec3::new(0.0, 1.0, 0.0));
    }

    // The whole point of recentering: a vertex far from the origin, close to
    // `reference`, keeps sub-millimeter precision after the f32 cast — whereas
    // casting the raw ECEF-scale coordinate straight to f32 would not.
    #[test]
    fn test_recenter_and_cast_preserves_local_precision() {
        let reference = DVec3::new(-3_957_123.456_789, 3_363_456.123_456, 3_704_567.987_654);
        let nearby = DVec3::new(
            reference.x + 0.0001,
            reference.y + 0.0002,
            reference.z + 0.0003,
        );
        let triangles = vec![[nearby, nearby, nearby]];

        let result = recenter_and_cast(&triangles, reference);
        let local = result[0][0];

        assert!((local.x - 0.0001).abs() < 1e-6, "x: {}", local.x);
        assert!((local.y - 0.0002).abs() < 1e-6, "y: {}", local.y);
        assert!((local.z - 0.0003).abs() < 1e-6, "z: {}", local.z);
    }
}
