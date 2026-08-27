//! Op-level tests for `DivideByGrid`.

use crate::coordinate::CoordinateFrame;
use crate::ops::grid::{CellCoverage, DivideByGrid, GridCell, GridDivideError, GridSpec};
use crate::point::Point3D;
use crate::polygon::{Polygon2D, Polygon3D};
use crate::Geometry;

fn unit_grid() -> GridSpec {
    GridSpec::new([0.0, 0.0], 1.0).expect("valid spec")
}

/// Collect everything the op emits, in order.
fn collect(
    g: &Geometry,
    grid: &GridSpec,
) -> Result<Vec<(GridCell, CellCoverage)>, GridDivideError> {
    let mut out = Vec::new();
    g.divide_by_grid(grid, &mut |cell, coverage, _geom| {
        out.push((cell, coverage));
    })?;
    Ok(out)
}

fn square_2d(min: f64, max: f64) -> Polygon2D {
    Polygon2D::from_rings(
        CoordinateFrame::default(),
        [[min, min], [max, min], [max, max], [min, max], [min, min]],
        std::iter::empty::<Vec<[f64; 2]>>(),
    )
}

#[test]
fn spec_rejects_non_positive_cell_size() {
    assert!(matches!(
        GridSpec::new([0.0, 0.0], 0.0),
        Err(GridDivideError::InvalidSpec(_))
    ));
    assert!(matches!(
        GridSpec::new([0.0, 0.0], -1.0),
        Err(GridDivideError::InvalidSpec(_))
    ));
    assert!(matches!(
        GridSpec::new([0.0, 0.0], f64::NAN),
        Err(GridDivideError::InvalidSpec(_))
    ));
    assert!(matches!(
        GridSpec::new([f64::INFINITY, 0.0], 1.0),
        Err(GridDivideError::InvalidSpec(_))
    ));
}

#[test]
fn polygon_spanning_four_cells_emits_four() {
    let poly = square_2d(0.0, 2.0);
    let geom = Geometry::Euclidean2D(crate::Euclidean2DGeometry::Polygon(Box::new(poly)));
    let out = collect(&geom, &unit_grid()).expect("divides");
    assert_eq!(out.len(), 4);
    assert!(out.iter().all(|(_, c)| *c == CellCoverage::Full));
}

#[test]
fn polygon_smaller_than_a_cell_reports_partial() {
    let poly = square_2d(0.25, 0.75);
    let geom = Geometry::Euclidean2D(crate::Euclidean2DGeometry::Polygon(Box::new(poly)));
    let out = collect(&geom, &unit_grid()).expect("divides");
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].0, GridCell { row: 0, col: 0 });
    assert_eq!(out[0].1, CellCoverage::Partial);
}

#[test]
fn explicit_origin_produces_negative_indices() {
    // Origin sits above and to the right of the data, so the data lands in
    // negative rows and columns rather than being clamped away.
    let poly = square_2d(-2.0, -1.0);
    let geom = Geometry::Euclidean2D(crate::Euclidean2DGeometry::Polygon(Box::new(poly)));
    let out = collect(&geom, &unit_grid()).expect("divides");
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].0, GridCell { row: -2, col: -2 });
}

#[test]
fn cells_are_emitted_row_major_and_reproducibly() {
    let poly = square_2d(0.0, 2.0);
    let geom = Geometry::Euclidean2D(crate::Euclidean2DGeometry::Polygon(Box::new(poly)));
    let first = collect(&geom, &unit_grid()).expect("divides");
    let second = collect(&geom, &unit_grid()).expect("divides");
    assert_eq!(first, second, "output must be reproducible");

    let cells: Vec<GridCell> = first.iter().map(|(c, _)| *c).collect();
    let mut sorted = cells.clone();
    sorted.sort_by_key(|c| (c.row, c.col));
    assert_eq!(cells, sorted, "emission must be row-major");
}

#[test]
fn z_is_preserved_and_interpolated_on_a_sloped_face() {
    // A face on the plane z = x, spanning two cells. Every emitted vertex must
    // still satisfy z == x.
    let poly = Polygon3D::from_rings(
        CoordinateFrame::default(),
        [
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 2.0],
            [2.0, 1.0, 2.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ],
        std::iter::empty::<Vec<[f64; 3]>>(),
    );
    let geom = Geometry::Euclidean3D(crate::Euclidean3DGeometry::Polygon(Box::new(poly)));

    let mut checked = 0;
    geom.divide_by_grid(&unit_grid(), &mut |_cell, _coverage, piece| {
        if let Geometry::Euclidean3D(crate::Euclidean3DGeometry::Polygon(p)) = piece {
            for v in p.exterior() {
                assert!((v[2] - v[0]).abs() < 1e-12, "z {} != x {}", v[2], v[0]);
                checked += 1;
            }
        }
    })
    .expect("divides");
    assert!(checked > 0, "no vertices were checked");
}

#[test]
fn point_is_unsupported() {
    let geom = Geometry::Euclidean3D(crate::Euclidean3DGeometry::Point(Point3D::new(
        CoordinateFrame::default(),
        [0.5, 0.5, 0.0],
    )));
    assert!(matches!(
        collect(&geom, &unit_grid()),
        Err(GridDivideError::Unsupported(_))
    ));
}

#[test]
fn geometry_none_is_empty() {
    assert!(matches!(
        collect(&Geometry::None, &unit_grid()),
        Err(GridDivideError::Empty)
    ));
}

#[test]
fn explicit_uv_is_preserved_and_interpolated_on_a_partial_cut() {
    use crate::appearance::UvSource;
    use crate::test_support::{explicit_uv, textured, theme};

    // A 2x1 rectangle whose UV is the unit quad (plus the closing-duplicate
    // entry the stored, closed exterior ring needs): u == x / 2, v == y
    // exactly, so any corner the clip produces (original or cut) can be
    // checked against a closed-form expectation rather than a hand-picked
    // table.
    let mut poly = Polygon2D::from_rings(
        CoordinateFrame::default(),
        [[0.0, 0.0], [2.0, 0.0], [2.0, 1.0], [0.0, 1.0], [0.0, 0.0]],
        std::iter::empty::<Vec<[f64; 2]>>(),
    );
    let uv = explicit_uv(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [0.0, 0.0]]);
    poly.set_appearance(theme("rgb"), textured(), Some(uv))
        .unwrap();
    let geom = Geometry::Euclidean2D(crate::Euclidean2DGeometry::Polygon(Box::new(poly)));

    let mut checked = 0;
    geom.divide_by_grid(&unit_grid(), &mut |_cell, _coverage, piece| {
        if let Geometry::Euclidean2D(crate::Euclidean2DGeometry::Polygon(p)) = piece {
            let app = p.appearance().as_ref().expect("appearance carried through");
            let UvSource::Explicit(uv) = &app.themes()[0].uv_sets[0].uv else {
                panic!("expected an explicit output UV set");
            };
            assert_eq!(
                uv.len(),
                p.exterior().len(),
                "uv must be parallel to the piece's own corner buffer, not the source's"
            );
            for (corner, uv) in p.exterior().iter().zip(uv.iter()) {
                assert!(
                    (uv[0] - corner[0] / 2.0).abs() < 1e-12,
                    "u {} != x/2 {}",
                    uv[0],
                    corner[0] / 2.0
                );
                assert!(
                    (uv[1] - corner[1]).abs() < 1e-12,
                    "v {} != y {}",
                    uv[1],
                    corner[1]
                );
                checked += 1;
            }
        }
    })
    .expect("divides");
    assert!(checked > 0, "no corners were checked");
}

#[test]
fn world_to_texture_uv_carries_through_untouched() {
    use crate::appearance::{TexMatrix, UvSource};
    use crate::test_support::{textured, theme};

    let mut poly = square_2d(0.0, 2.0);
    let matrix = TexMatrix([
        [0.25, 0.0, 0.0, 0.0],
        [0.0, 0.25, 0.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);
    poly.set_appearance(
        theme("rgb"),
        textured(),
        Some(UvSource::WorldToTexture(matrix)),
    )
    .unwrap();
    let geom = Geometry::Euclidean2D(crate::Euclidean2DGeometry::Polygon(Box::new(poly)));

    let mut checked = 0;
    geom.divide_by_grid(&unit_grid(), &mut |_cell, _coverage, piece| {
        if let Geometry::Euclidean2D(crate::Euclidean2DGeometry::Polygon(p)) = piece {
            let app = p.appearance().as_ref().expect("appearance carried through");
            assert!(matches!(
                app.themes()[0].uv_sets[0].uv,
                UvSource::WorldToTexture(out) if out == matrix
            ));
            checked += 1;
        }
    })
    .expect("divides");
    assert!(checked > 0, "no pieces were checked");
}

/// Every theme/side binding's referenced UV channels are satisfied and no
/// `uv_set` is orphaned -- the same coupling the crate's own setters enforce
/// at construction (`Appearance::set_appearance` etc.), re-checked here
/// because a geometric op building an `Appearance` via `from_parts` bypasses
/// that enforcement and must uphold the invariant by construction instead.
fn assert_appearance_couples(app: &crate::appearance::Appearance, corner_count: usize) {
    use crate::appearance::{validate_uv_coupling, FaceBinding, Side};
    use std::collections::{BTreeMap, BTreeSet};

    for theme in app.themes() {
        for (side, binding) in [
            (Side::Front, Some(&theme.front)),
            (Side::Back, theme.back.as_ref()),
        ] {
            let Some(binding) = binding else {
                continue;
            };
            let mut referenced = BTreeSet::new();
            match binding {
                FaceBinding::Uniform(idx) => {
                    referenced.extend(app.materials()[idx.get() as usize].referenced_channels());
                }
                FaceBinding::PerFace(faces) => {
                    for idx in faces.iter().flatten() {
                        referenced
                            .extend(app.materials()[idx.get() as usize].referenced_channels());
                    }
                }
            }
            let uvs: BTreeMap<_, _> = theme
                .uv_sets
                .iter()
                .filter(|u| u.side == side)
                .map(|u| (u.channel, u.uv.clone()))
                .collect();
            validate_uv_coupling(&referenced, &uvs, corner_count).unwrap_or_else(|e| {
                panic!(
                    "theme {:?} side {side:?} fails coupling: {e:?}",
                    theme.theme
                )
            });
        }
    }
}

#[test]
fn full_coverage_untouched_cell_preserves_a_second_theme_verbatim() {
    use crate::appearance::UvSource;
    use crate::test_support::{explicit_uv, textured, theme};

    // Exactly matches the unit cell, so the clip's `contains` (inclusive of
    // the boundary) never cuts it: `corner_layout_unchanged` must see this,
    // even though a naive check keyed on `CellCoverage::Full` would also
    // (correctly, but for the wrong reason) accept a genuinely rewritten
    // full-area piece elsewhere -- see `polygon_spanning_four_cells_emits_four`.
    let mut poly = square_2d(0.0, 1.0);
    let default_uv = explicit_uv(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [0.0, 0.0]]);
    let second_uv = explicit_uv(&[[0.1, 0.2], [0.3, 0.4], [0.5, 0.6], [0.7, 0.8], [0.1, 0.2]]);
    poly.set_appearance(theme("rgb"), textured(), Some(default_uv))
        .unwrap();
    poly.set_appearance(theme("ir"), textured(), Some(second_uv.clone()))
        .unwrap();
    let geom = Geometry::Euclidean2D(crate::Euclidean2DGeometry::Polygon(Box::new(poly)));

    let mut checked = 0;
    geom.divide_by_grid(&unit_grid(), &mut |_cell, coverage, piece| {
        assert_eq!(coverage, CellCoverage::Full);
        if let Geometry::Euclidean2D(crate::Euclidean2DGeometry::Polygon(p)) = piece {
            let app = p.appearance().as_ref().expect("appearance carried through");
            assert_eq!(app.themes().len(), 2, "the second theme must survive");
            let ir = app
                .themes()
                .iter()
                .find(|t| t.theme == theme("ir"))
                .expect("second theme present");
            let UvSource::Explicit(uv) = &ir.uv_sets[0].uv else {
                panic!("expected an explicit uv set");
            };
            let UvSource::Explicit(expected) = &second_uv else {
                unreachable!("fixture is explicit");
            };
            assert_eq!(
                uv, expected,
                "non-default theme's uv must survive byte for byte"
            );
            checked += 1;
        }
    })
    .expect("divides");
    assert_eq!(checked, 1, "the whole polygon fits in one cell");
}

#[test]
fn full_coverage_untouched_cell_preserves_a_back_side_uv_set_verbatim() {
    use crate::appearance::UvSource;
    use crate::polygon::PolygonFace;
    use crate::test_support::{explicit_uv, textured, theme};

    let mut poly = square_2d(0.0, 1.0);
    let front_uv = explicit_uv(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [0.0, 0.0]]);
    let back_uv = explicit_uv(&[[1.0, 1.0], [0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]);
    poly.set_two_sided_appearance(
        theme("rgb"),
        PolygonFace::single(textured(), Some(front_uv)),
        PolygonFace::single(textured(), Some(back_uv.clone())),
    )
    .unwrap();
    let geom = Geometry::Euclidean2D(crate::Euclidean2DGeometry::Polygon(Box::new(poly)));

    let mut checked = 0;
    geom.divide_by_grid(&unit_grid(), &mut |_cell, _coverage, piece| {
        if let Geometry::Euclidean2D(crate::Euclidean2DGeometry::Polygon(p)) = piece {
            let app = p.appearance().as_ref().expect("appearance carried through");
            assert!(app.themes()[0].back.is_some(), "back binding must survive");
            let back_set = app.themes()[0]
                .uv_sets
                .iter()
                .find(|u| u.side == crate::appearance::Side::Back)
                .expect("back uv set present");
            let UvSource::Explicit(uv) = &back_set.uv else {
                panic!("expected an explicit uv set");
            };
            let UvSource::Explicit(expected) = &back_uv else {
                unreachable!("fixture is explicit");
            };
            assert_eq!(uv, expected, "back uv must survive byte for byte");
            checked += 1;
        }
    })
    .expect("divides");
    assert_eq!(checked, 1, "the whole polygon fits in one cell");
}

#[test]
fn partial_coverage_clip_drops_unrecoverable_uv_without_orphaning_a_binding() {
    use crate::polygon::PolygonFace;
    use crate::test_support::{explicit_uv, textured, theme};

    // Spans four cells, so every emitted piece is genuinely cut -- none of
    // this polygon's rings survive any one cell's clip untouched.
    let mut poly = square_2d(0.0, 2.0);
    let front_uv = explicit_uv(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [0.0, 0.0]]);
    let back_uv = explicit_uv(&[[1.0, 1.0], [0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]);
    // Default theme, two-sided: the front (default slot) is threaded through
    // every cut; the back cannot be, and must be dropped along with its
    // binding, not left dangling.
    poly.set_two_sided_appearance(
        theme("rgb"),
        PolygonFace::single(textured(), Some(front_uv)),
        PolygonFace::single(textured(), Some(back_uv)),
    )
    .unwrap();
    // A second theme's front cannot be threaded either, and `front` is not
    // optional, so the whole theme must go rather than leave it half-wired.
    poly.set_appearance(
        theme("ir"),
        textured(),
        Some(explicit_uv(&[
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 1.0],
            [0.0, 1.0],
            [0.0, 0.0],
        ])),
    )
    .unwrap();
    let geom = Geometry::Euclidean2D(crate::Euclidean2DGeometry::Polygon(Box::new(poly)));

    let mut checked = 0;
    geom.divide_by_grid(&unit_grid(), &mut |_cell, coverage, piece| {
        assert_eq!(coverage, CellCoverage::Full, "each quadrant fills its cell");
        if let Geometry::Euclidean2D(crate::Euclidean2DGeometry::Polygon(p)) = piece {
            let app = p
                .appearance()
                .as_ref()
                .expect("default theme's front survives");
            assert_eq!(
                app.themes().len(),
                1,
                "the second theme must be dropped whole, not left with a missing uv"
            );
            assert!(
                app.themes()[0].back.is_none(),
                "the back binding must be dropped along with its unrecoverable uv"
            );
            assert!(
                app.themes()[0]
                    .uv_sets
                    .iter()
                    .all(|u| u.side == crate::appearance::Side::Front),
                "no back-side uv set may linger once the back binding is gone"
            );
            assert_appearance_couples(app, p.exterior().len());
            checked += 1;
        }
    })
    .expect("divides");
    assert_eq!(checked, 4, "one piece per quadrant");
}
