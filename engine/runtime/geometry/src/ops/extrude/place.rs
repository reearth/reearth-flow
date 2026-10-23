//! Frame checks and where each vertex's base and top land.

use super::math::{add, norm, scale};
use super::{BaseMode, ExtrudeDirection, ExtrudeError, TopMode};
use crate::coordinate::CoordinateFrame;

/// A direction resolved against a frame and, for `SurfaceNormal`, a surface.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Placement {
    Vertical { base: BaseMode, top: TopMode },
    Offset([f64; 3]),
}

/// The frame's orientation sign (`+1` right-handed, `-1` reflected), or
/// `UnsupportedFrame` when the frame's axes cannot be determined.
pub(super) fn orientation_sign(frame: &CoordinateFrame) -> Result<f64, ExtrudeError> {
    frame
        .orientation_sign()
        .map(f64::from)
        .map_err(|e| ExtrudeError::UnsupportedFrame(e.to_string()))
}

/// Resolve `direction` in `frame`. `surface_normal` is the unit front normal
/// of a planar surface, `None` for points, lines and non-planar input.
pub(super) fn placement(
    frame: &CoordinateFrame,
    direction: &ExtrudeDirection,
    surface_normal: Option<[f64; 3]>,
) -> Result<Placement, ExtrudeError> {
    let offset = match *direction {
        ExtrudeDirection::Vertical { base, top } => return Ok(Placement::Vertical { base, top }),
        ExtrudeDirection::Vector { vector, length } => match length {
            None => vector,
            Some(length) => {
                let n = norm(vector);
                if n == 0.0 {
                    return Err(ExtrudeError::Degenerate(
                        "the direction vector has zero length",
                    ));
                }
                scale(vector, length / n)
            }
        },
        ExtrudeDirection::SurfaceNormal { distance } => {
            let Some(normal) = surface_normal else {
                return Err(ExtrudeError::UnsupportedType(
                    "only a planar surface has a normal to extrude along".to_string(),
                ));
            };
            scale(normal, distance)
        }
    };
    if !frame.has_linear_units() {
        return Err(ExtrudeError::UnsupportedFrame(
            "a vector or surface-normal extrusion needs a coordinate system in linear units"
                .to_string(),
        ));
    }
    if norm(offset) == 0.0 || offset.iter().any(|c| !c.is_finite()) {
        return Err(ExtrudeError::Degenerate("the extrusion has zero length"));
    }
    Ok(Placement::Offset(offset))
}

/// Base and top positions, in vertex order.
pub(super) type BaseTop = (Vec<[f64; 3]>, Vec<[f64; 3]>);

/// Base and top positions of `points` (one connected piece). For vertical
/// placements every vertex must move the same way and by a non-zero amount.
pub(super) fn place(points: &[[f64; 3]], placement: &Placement) -> Result<BaseTop, ExtrudeError> {
    let (base, top) = match *placement {
        Placement::Offset(v) => (points.to_vec(), points.iter().map(|p| add(*p, v)).collect()),
        Placement::Vertical { base, top } => {
            let lowest = points.iter().map(|p| p[2]).fold(f64::INFINITY, f64::min);
            let base_z = |p: &[f64; 3]| match base {
                BaseMode::KeepElevation => p[2],
                BaseMode::FlattenToLowest => lowest,
                BaseMode::FixedElevation(z) => z,
            };
            let base: Vec<[f64; 3]> = points.iter().map(|p| [p[0], p[1], base_z(p)]).collect();
            let lo = base.iter().map(|p| p[2]).fold(f64::INFINITY, f64::min);
            let hi = base.iter().map(|p| p[2]).fold(f64::NEG_INFINITY, f64::max);
            let top_z = |bz: f64| match top {
                TopMode::ByHeight(h) => bz + h,
                TopMode::ToElevation(z) => z,
                TopMode::AboveLowest(h) => lo + h,
                TopMode::AboveHighest(h) => hi + h,
            };
            let top_pts: Vec<[f64; 3]> = base.iter().map(|p| [p[0], p[1], top_z(p[2])]).collect();
            let direction = if matches!(top, TopMode::ByHeight(h) if h < 0.0) {
                -1.0
            } else {
                1.0
            };
            if base.iter().zip(&top_pts).any(|(b, t)| {
                // NaN fails the comparison, so it is rejected too.
                let rises = (t[2] - b[2]) * direction > 0.0 && t[2].is_finite();
                !rises
            }) {
                return Err(ExtrudeError::Degenerate("the top is not above the base"));
            }
            (base, top_pts)
        }
    };
    if base
        .iter()
        .chain(&top)
        .any(|p| p.iter().any(|c| !c.is_finite()))
    {
        return Err(ExtrudeError::InvalidSurface(
            "the geometry has non-finite coordinates",
        ));
    }
    Ok((base, top))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SLOPED: [[f64; 3]; 3] = [[0.0, 0.0, 1.0], [1.0, 0.0, 3.0], [0.0, 1.0, 2.0]];

    fn zs(v: &[[f64; 3]]) -> Vec<f64> {
        v.iter().map(|p| p[2]).collect()
    }

    fn vertical(base: BaseMode, top: TopMode) -> Placement {
        Placement::Vertical { base, top }
    }

    #[test]
    fn non_finite_coordinates_are_rejected() {
        let nan_x = [[f64::NAN, 0.0, 0.0]];
        let inf_y = [[0.0, 0.0, 0.0], [1.0, f64::INFINITY, 0.0], [0.0, 1.0, 0.0]];
        let placements = [
            vertical(BaseMode::KeepElevation, TopMode::ByHeight(1.0)),
            Placement::Offset([0.0, 0.0, 1.0]),
        ];
        for pl in &placements {
            for pts in [&nan_x[..], &inf_y[..]] {
                assert!(
                    matches!(place(pts, pl), Err(ExtrudeError::InvalidSurface(_))),
                    "{pts:?} {pl:?}"
                );
            }
        }
    }

    #[test]
    fn keep_by_height_follows_the_slope() {
        let (b, t) = place(
            &SLOPED,
            &vertical(BaseMode::KeepElevation, TopMode::ByHeight(10.0)),
        )
        .unwrap();
        assert_eq!(zs(&b), vec![1.0, 3.0, 2.0]);
        assert_eq!(zs(&t), vec![11.0, 13.0, 12.0]);
    }

    #[test]
    fn flatten_to_lowest_flattens_the_base() {
        let (b, t) = place(
            &SLOPED,
            &vertical(BaseMode::FlattenToLowest, TopMode::ByHeight(5.0)),
        )
        .unwrap();
        assert_eq!(zs(&b), vec![1.0, 1.0, 1.0]);
        assert_eq!(zs(&t), vec![6.0, 6.0, 6.0]);
    }

    #[test]
    fn fixed_base_and_absolute_top() {
        let (b, t) = place(
            &SLOPED,
            &vertical(BaseMode::FixedElevation(-2.0), TopMode::ToElevation(7.0)),
        )
        .unwrap();
        assert_eq!(zs(&b), vec![-2.0, -2.0, -2.0]);
        assert_eq!(zs(&t), vec![7.0, 7.0, 7.0]);
    }

    #[test]
    fn flat_tops_above_lowest_and_highest_keep_a_sloped_base() {
        let (b, t) = place(
            &SLOPED,
            &vertical(BaseMode::KeepElevation, TopMode::AboveLowest(4.0)),
        )
        .unwrap();
        assert_eq!(zs(&b), vec![1.0, 3.0, 2.0]);
        assert_eq!(zs(&t), vec![5.0, 5.0, 5.0]);
        let (_, t) = place(
            &SLOPED,
            &vertical(BaseMode::KeepElevation, TopMode::AboveHighest(4.0)),
        )
        .unwrap();
        assert_eq!(zs(&t), vec![7.0, 7.0, 7.0]);
    }

    #[test]
    fn negative_height_sweeps_down() {
        let (_, t) = place(
            &SLOPED,
            &vertical(BaseMode::KeepElevation, TopMode::ByHeight(-1.0)),
        )
        .unwrap();
        assert_eq!(zs(&t), vec![0.0, 2.0, 1.0]);
    }

    #[test]
    fn degenerate_heights_are_rejected() {
        for top in [
            TopMode::ByHeight(0.0),
            TopMode::ToElevation(2.0),
            TopMode::AboveLowest(1.0),
        ] {
            assert!(
                matches!(
                    place(&SLOPED, &vertical(BaseMode::KeepElevation, top)),
                    Err(ExtrudeError::Degenerate(_))
                ),
                "{top:?}"
            );
        }
    }

    #[test]
    fn vector_with_length_is_normalized_and_scaled() {
        let p = placement(
            &CoordinateFrame::Euclidean,
            &ExtrudeDirection::Vector {
                vector: [0.0, 3.0, 4.0],
                length: Some(10.0),
            },
            None,
        )
        .unwrap();
        let (_, t) = place(&[[0.0, 0.0, 0.0]], &p).unwrap();
        assert_eq!(t, vec![[0.0, 6.0, 8.0]]);
    }

    #[test]
    fn zero_vector_is_degenerate() {
        let r = placement(
            &CoordinateFrame::Euclidean,
            &ExtrudeDirection::Vector {
                vector: [0.0; 3],
                length: Some(1.0),
            },
            None,
        );
        assert!(matches!(r, Err(ExtrudeError::Degenerate(_))));
    }

    #[test]
    fn vector_on_an_angular_frame_is_an_unsupported_frame() {
        let r = placement(
            &CoordinateFrame::Crs(4326.into()),
            &ExtrudeDirection::Vector {
                vector: [0.0, 0.0, 1.0],
                length: None,
            },
            None,
        );
        assert!(matches!(r, Err(ExtrudeError::UnsupportedFrame(_))));
    }

    #[test]
    fn vertical_is_fine_on_an_angular_frame() {
        let d = ExtrudeDirection::Vertical {
            base: BaseMode::KeepElevation,
            top: TopMode::ByHeight(1.0),
        };
        assert!(placement(&CoordinateFrame::Crs(4326.into()), &d, None).is_ok());
    }

    #[test]
    fn surface_normal_without_a_surface_is_unsupported() {
        let d = ExtrudeDirection::SurfaceNormal { distance: 1.0 };
        assert!(matches!(
            placement(&CoordinateFrame::Euclidean, &d, None),
            Err(ExtrudeError::UnsupportedType(_))
        ));
    }
}
