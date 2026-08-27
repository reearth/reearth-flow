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

#[test]
fn multi_channel_default_theme_channel_loss_does_not_discard_the_whole_appearance() {
    use crate::appearance::{ChannelId, TexMatrix, UvSource};
    use crate::polygon::PolygonFace;
    use crate::test_support::{bare, explicit_uv, theme, two_channel};
    use std::collections::BTreeMap;

    // Spans four cells, so every emitted piece is genuinely cut.
    let mut poly = square_2d(0.0, 2.0);

    // The default theme's front material needs TWO channels: channel 0 (the
    // default slot -- recoverable, since it's exactly what gets threaded
    // through the clip) and channel 1 (not the default slot, so it cannot
    // be threaded). Losing channel 1's uv makes the whole `Uniform` binding
    // unusable, even though channel 0's own uv is gathered successfully;
    // this is the case that used to trip the unconditional `return None` at
    // the old bug site -- a *partial* front failure, not the wholesale one
    // `world_to_texture_theme_survives_a_sibling_default_theme_drop` covers.
    let ring5 = |uv: [[f64; 2]; 4]| explicit_uv(&[uv[0], uv[1], uv[2], uv[3], uv[0]]);
    let mut uv = BTreeMap::new();
    uv.insert(
        ChannelId(0),
        ring5([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]),
    );
    uv.insert(
        ChannelId(1),
        ring5([[0.1, 0.1], [0.2, 0.2], [0.3, 0.3], [0.4, 0.4]]),
    );
    let front = PolygonFace {
        material: two_channel(0, 1),
        uv,
    };
    // A trivial, colour-only back so `set_two_sided_appearance` (the only
    // public entry point that accepts a multi-channel `PolygonFace`) has
    // something valid to pair `front` with; it carries no UV of its own, so
    // it is never at risk in this test.
    let back = PolygonFace::single(bare(), None);
    poly.set_two_sided_appearance(theme("rgb"), front, back)
        .unwrap();

    // A second, unrelated theme whose only UV is `WorldToTexture`. Only the
    // *default* theme's default slot is ever threaded through a clip, so an
    // `Explicit` UV on any other theme is unrecoverable regardless of its
    // channel; `WorldToTexture` is the one thing a non-default theme can
    // carry that survives a real cut, which is exactly why it is the right
    // fixture for "a theme that was never at risk".
    let matrix = TexMatrix([
        [0.25, 0.0, 0.0, 0.0],
        [0.0, 0.25, 0.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);
    poly.set_appearance(
        theme("ir"),
        crate::test_support::textured(),
        Some(UvSource::WorldToTexture(matrix)),
    )
    .unwrap();

    let geom = Geometry::Euclidean2D(crate::Euclidean2DGeometry::Polygon(Box::new(poly)));

    let mut checked = 0;
    geom.divide_by_grid(&unit_grid(), &mut |_cell, _coverage, piece| {
        if let Geometry::Euclidean2D(crate::Euclidean2DGeometry::Polygon(p)) = piece {
            let app = p
                .appearance()
                .as_ref()
                .expect("the appearance must not be wholly discarded");
            assert_eq!(
                app.themes().len(),
                1,
                "the multi-channel default theme is unusable and must be dropped whole, \
                 but the unrelated second theme must not be swept away with it"
            );
            assert_eq!(
                app.themes()[0].theme,
                theme("ir"),
                "the surviving theme is the one that was never at risk"
            );
            assert_eq!(
                *app.default_theme(),
                theme("ir"),
                "default_theme must be re-pointed once the original default is gone"
            );
            assert_appearance_couples(app, p.exterior().len());
            checked += 1;
        }
    })
    .expect("divides");
    assert_eq!(checked, 4, "one piece per quadrant");
}

#[test]
fn world_to_texture_theme_survives_a_sibling_default_theme_drop() {
    use crate::appearance::{ChannelId, TexMatrix, UvSource};
    use crate::polygon::PolygonFace;
    use crate::test_support::{bare, explicit_uv, theme, two_channel};
    use std::collections::BTreeMap;

    let mut poly = square_2d(0.0, 2.0);

    // The default theme's front references only non-default channels, so
    // neither of its uv sets is the default slot and both are unrecoverable
    // once the layout changes; the whole theme must go.
    let ring5 = |uv: [[f64; 2]; 4]| explicit_uv(&[uv[0], uv[1], uv[2], uv[3], uv[0]]);
    let mut uv = BTreeMap::new();
    uv.insert(
        ChannelId(1),
        ring5([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]),
    );
    uv.insert(
        ChannelId(2),
        ring5([[0.1, 0.1], [0.2, 0.2], [0.3, 0.3], [0.4, 0.4]]),
    );
    let front = PolygonFace {
        material: two_channel(1, 2),
        uv,
    };
    let back = PolygonFace::single(bare(), None);
    poly.set_two_sided_appearance(theme("rgb"), front, back)
        .unwrap();

    // A second theme whose only UV is `WorldToTexture`: positional, so it
    // never needed threading in the first place and must survive untouched.
    let matrix = TexMatrix([
        [0.25, 0.0, 0.0, 0.0],
        [0.0, 0.25, 0.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);
    poly.set_appearance(
        theme("ir"),
        crate::test_support::textured(),
        Some(UvSource::WorldToTexture(matrix)),
    )
    .unwrap();

    let geom = Geometry::Euclidean2D(crate::Euclidean2DGeometry::Polygon(Box::new(poly)));

    let mut checked = 0;
    geom.divide_by_grid(&unit_grid(), &mut |_cell, _coverage, piece| {
        if let Geometry::Euclidean2D(crate::Euclidean2DGeometry::Polygon(p)) = piece {
            let app = p
                .appearance()
                .as_ref()
                .expect("the appearance must not be wholly discarded");
            assert_eq!(
                app.themes().len(),
                1,
                "only the broken default theme is dropped"
            );
            assert_eq!(app.themes()[0].theme, theme("ir"));
            assert!(matches!(
                app.themes()[0].uv_sets[0].uv,
                UvSource::WorldToTexture(out) if out == matrix
            ));
            assert_appearance_couples(app, p.exterior().len());
            checked += 1;
        }
    })
    .expect("divides");
    assert_eq!(checked, 4, "one piece per quadrant");
}

use crate::polygon_mesh::PolygonMesh3D;

/// Two faces meeting along x = 0.5, together covering the unit cell exactly.
/// Neither face alone fills it.
fn split_cover_mesh() -> PolygonMesh3D {
    let left = Polygon3D::from_rings(
        CoordinateFrame::default(),
        [
            [0.0, 0.0, 0.0],
            [0.5, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ],
        std::iter::empty::<Vec<[f64; 3]>>(),
    );
    let right = Polygon3D::from_rings(
        CoordinateFrame::default(),
        [
            [0.5, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.5, 1.0, 0.0],
            [0.5, 0.0, 0.0],
        ],
        std::iter::empty::<Vec<[f64; 3]>>(),
    );
    PolygonMesh3D::from_polygons(CoordinateFrame::default(), [&left, &right]).expect("valid mesh")
}

#[test]
fn mesh_whose_faces_together_fill_a_cell_reports_full() {
    // The behaviour change recorded as B1 in the spec: the old per-polygon check
    // called this Partial and, with completeCellsOnly, dropped the cell.
    let geom = Geometry::Euclidean3D(crate::Euclidean3DGeometry::PolygonMesh(Box::new(
        split_cover_mesh(),
    )));
    let out = collect(&geom, &unit_grid()).expect("divides");
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].1, CellCoverage::Full, "faces together fill the cell");
}

#[test]
fn clipped_mesh_stays_a_mesh() {
    let geom = Geometry::Euclidean3D(crate::Euclidean3DGeometry::PolygonMesh(Box::new(
        split_cover_mesh(),
    )));
    let mut kinds = Vec::new();
    geom.divide_by_grid(&unit_grid(), &mut |_c, _v, piece| {
        kinds.push(matches!(
            piece,
            Geometry::Euclidean3D(crate::Euclidean3DGeometry::PolygonMesh(_))
        ));
    })
    .expect("divides");
    assert!(!kinds.is_empty());
    assert!(kinds.iter().all(|k| *k), "leaf kind must be preserved");
}

#[test]
fn triangular_mesh_clipped_to_a_pentagon_stays_a_triangular_mesh() {
    use crate::triangular_mesh::TriangularMesh3D;

    // One triangle overhanging the cell's top-right corner. Clipping it against
    // the cell yields a pentagon, which must be fan-triangulated back.
    let mesh = TriangularMesh3D::from_parts(
        CoordinateFrame::default(),
        vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]],
        [0u32, 1, 2],
    )
    .expect("valid mesh");
    let geom = Geometry::Euclidean3D(crate::Euclidean3DGeometry::TriangularMesh(Box::new(mesh)));

    let mut seen = 0;
    geom.divide_by_grid(&unit_grid(), &mut |_c, _v, piece| {
        assert!(
            matches!(
                piece,
                Geometry::Euclidean3D(crate::Euclidean3DGeometry::TriangularMesh(_))
            ),
            "a clipped triangular mesh must stay a triangular mesh"
        );
        seen += 1;
    })
    .expect("divides");
    assert!(seen > 0);
}

#[test]
fn mesh_explicit_uv_interpolates_at_a_cut_and_stays_position_parallel() {
    use crate::appearance::UvSource;
    use crate::test_support::explicit_uv_appearance;
    use crate::triangular_mesh::TriangularMesh3D;

    // One triangle spanning two cells (x in 0..2), with UV running 0..1 across
    // x on its three corners. After dividing at x = 1 of a 1-unit grid, every
    // new cut corner (on the hypotenuse and the base) must carry u = 0.5.
    let mut mesh = TriangularMesh3D::from_parts(
        CoordinateFrame::default(),
        vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]],
        [0u32, 1, 2],
    )
    .expect("valid mesh");
    *mesh.appearance_mut() = Some(explicit_uv_appearance(&[
        [0.0, 0.0],
        [1.0, 0.0],
        [0.0, 1.0],
    ]));

    let geom = Geometry::Euclidean3D(crate::Euclidean3DGeometry::TriangularMesh(Box::new(mesh)));

    let mut saw_cut = false;
    geom.divide_by_grid(&unit_grid(), &mut |_c, _v, piece| {
        let Geometry::Euclidean3D(crate::Euclidean3DGeometry::TriangularMesh(m)) = piece else {
            panic!("expected a triangular mesh piece");
        };
        let app = m.appearance().as_ref().expect("uv must survive the cut");
        let UvSource::Explicit(uv) = &app.themes()[0].uv_sets[0].uv else {
            panic!("expected an explicit uv set");
        };
        let verts = m.vertices();
        for (tri, [a, b, c]) in m.triangles().enumerate() {
            for (corner, &vi) in [a, b, c].iter().enumerate() {
                let pos = verts[vi as usize];
                let u = uv[3 * tri + corner][0];
                if pos[0] == 1.0 {
                    assert!((u - 0.5).abs() < 1e-12, "u at the cut was {u}");
                    saw_cut = true;
                }
            }
        }
    })
    .expect("divides");
    assert!(saw_cut, "the division must actually cut the triangle");
}

#[test]
fn polygon_mesh_whole_mesh_inside_one_cell_preserves_a_second_theme_verbatim() {
    use crate::appearance::{
        Appearance, ChannelId, FaceBinding, MaterialIndex, Side, TexMatrix, ThemeBinding, UvSet,
        UvSource,
    };
    use crate::test_support::{textured, theme};

    // A single quad face exactly matching the unit cell (boundary-inclusive,
    // so the clip never touches it), built via the bare `from_parts`
    // constructor rather than `PolygonMesh3D::from_polygons`: welding a
    // `WorldToTexture` theme through `from_polygons` bakes it to `Explicit`
    // *at construction time* (a welded mesh's faces cannot share one
    // matrix), which would make this fixture unable to carry a
    // `WorldToTexture` theme at all, regardless of how it later divides. The
    // appearance is attached afterwards through the raw `appearance_mut`
    // escape hatch instead, the same pattern `test_support::
    // explicit_uv_appearance` uses, so it genuinely starts out
    // `WorldToTexture` and this test is actually exercising what happens to
    // it across a division.
    let mut mesh = PolygonMesh3D::from_parts(
        CoordinateFrame::default(),
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ],
        [[0u32, 1, 2, 3]],
    )
    .expect("valid mesh");
    let matrix = TexMatrix([
        [0.25, 0.0, 0.0, 0.0],
        [0.0, 0.25, 0.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);
    let app = Appearance::from_parts(
        vec![textured(), textured()],
        vec![
            ThemeBinding {
                theme: theme("rgb"),
                front: FaceBinding::Uniform(MaterialIndex::new(0).unwrap()),
                back: None,
                uv_sets: vec![UvSet {
                    side: Side::Front,
                    channel: ChannelId::default(),
                    uv: UvSource::Explicit(Box::new([
                        [0.0, 0.0],
                        [1.0, 0.0],
                        [1.0, 1.0],
                        [0.0, 1.0],
                    ])),
                }],
            },
            ThemeBinding {
                theme: theme("ir"),
                front: FaceBinding::Uniform(MaterialIndex::new(1).unwrap()),
                back: None,
                uv_sets: vec![UvSet {
                    side: Side::Front,
                    channel: ChannelId::default(),
                    uv: UvSource::WorldToTexture(matrix),
                }],
            },
        ],
        theme("rgb"),
    );
    *mesh.appearance_mut() = Some(app);
    let geom = Geometry::Euclidean3D(crate::Euclidean3DGeometry::PolygonMesh(Box::new(mesh)));

    let mut checked = 0;
    geom.divide_by_grid(&unit_grid(), &mut |_cell, coverage, piece| {
        assert_eq!(coverage, CellCoverage::Full);
        let Geometry::Euclidean3D(crate::Euclidean3DGeometry::PolygonMesh(m)) = piece else {
            panic!("expected a polygon mesh piece");
        };
        let app = m
            .appearance()
            .as_ref()
            .expect("appearance carried through untouched");
        assert_eq!(
            app.themes().len(),
            2,
            "both themes must survive a division that cuts nothing, not just the default"
        );
        let ir = app
            .themes()
            .iter()
            .find(|t| t.theme == theme("ir"))
            .expect("second theme present");
        assert!(
            matches!(
                ir.uv_sets[0].uv,
                UvSource::WorldToTexture(out) if out == matrix
            ),
            "WorldToTexture must not be baked to Explicit by a weld the mesh never needed"
        );
        checked += 1;
    })
    .expect("divides");
    assert_eq!(checked, 1, "the whole mesh fits in one cell");
}

#[test]
fn triangular_mesh_whole_mesh_inside_one_cell_preserves_a_second_theme_verbatim() {
    use crate::appearance::{
        Appearance, ChannelId, FaceBinding, MaterialIndex, Side, TexMatrix, ThemeBinding, UvSet,
        UvSource,
    };
    use crate::test_support::{textured, theme};
    use crate::triangular_mesh::TriangularMesh3D;

    // One triangle wholly inside the unit cell, with two themes built
    // straight through `Appearance::from_parts` (the same raw escape hatch
    // `test_support::explicit_uv_appearance` uses) so this test does not
    // depend on `TriangularMesh::set_appearance`'s own validation to get a
    // second theme attached.
    let mut mesh = TriangularMesh3D::from_parts(
        CoordinateFrame::default(),
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        [0u32, 1, 2],
    )
    .expect("valid mesh");

    let matrix = TexMatrix([
        [0.25, 0.0, 0.0, 0.0],
        [0.0, 0.25, 0.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);
    let app = Appearance::from_parts(
        vec![textured(), textured()],
        vec![
            ThemeBinding {
                theme: theme("rgb"),
                front: FaceBinding::Uniform(MaterialIndex::new(0).unwrap()),
                back: None,
                uv_sets: vec![UvSet {
                    side: Side::Front,
                    channel: ChannelId::default(),
                    uv: UvSource::Explicit(Box::new([[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]])),
                }],
            },
            ThemeBinding {
                theme: theme("ir"),
                front: FaceBinding::Uniform(MaterialIndex::new(1).unwrap()),
                back: None,
                uv_sets: vec![UvSet {
                    side: Side::Front,
                    channel: ChannelId::default(),
                    uv: UvSource::WorldToTexture(matrix),
                }],
            },
        ],
        theme("rgb"),
    );
    *mesh.appearance_mut() = Some(app);

    let geom = Geometry::Euclidean3D(crate::Euclidean3DGeometry::TriangularMesh(Box::new(mesh)));
    let mut checked = 0;
    geom.divide_by_grid(&unit_grid(), &mut |_c, _v, piece| {
        let Geometry::Euclidean3D(crate::Euclidean3DGeometry::TriangularMesh(m)) = piece else {
            panic!("expected a triangular mesh piece");
        };
        let app = m
            .appearance()
            .as_ref()
            .expect("appearance carried through untouched");
        assert_eq!(
            app.themes().len(),
            2,
            "both themes must survive a division that cuts nothing"
        );
        let ir = app
            .themes()
            .iter()
            .find(|t| t.theme == theme("ir"))
            .expect("second theme present");
        assert!(
            matches!(
                ir.uv_sets[0].uv,
                UvSource::WorldToTexture(out) if out == matrix
            ),
            "WorldToTexture must not be baked to Explicit by a rebuild the mesh never needed"
        );
        checked += 1;
    })
    .expect("divides");
    assert_eq!(checked, 1, "the whole mesh fits in one cell");
}

#[test]
fn polygon_mesh_per_face_material_binding_survives_division() {
    use crate::appearance::{FaceBinding, Material, PhongMaterial};
    use crate::test_support::theme;

    fn colored(diffuse: [f32; 3]) -> Material {
        Material::Phong(PhongMaterial {
            diffuse,
            specular: [0.0; 3],
            emissive: [0.0; 3],
            ambient_intensity: 0.0,
            shininess: 0.0,
            transparency: 0.0,
            diffuse_map: None,
            emissive_map: None,
            normal_map: None,
        })
    }

    // Three faces, one per grid column, so the division lands each in its
    // own cell without cutting any of them: this exercises `face_appearance`
    // slicing a per-face `FaceBinding::PerFace` entry back out through a
    // genuine (multi-cell) division, not the whole-mesh-unchanged fast path.
    let mut face0 = Polygon3D::from_rings(
        CoordinateFrame::default(),
        [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ],
        std::iter::empty::<Vec<[f64; 3]>>(),
    );
    face0
        .set_appearance(theme("rgb"), colored([1.0, 0.0, 0.0]), None)
        .unwrap();

    let mut face1 = Polygon3D::from_rings(
        CoordinateFrame::default(),
        [
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [2.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, 0.0, 0.0],
        ],
        std::iter::empty::<Vec<[f64; 3]>>(),
    );
    face1
        .set_appearance(theme("rgb"), colored([0.0, 1.0, 0.0]), None)
        .unwrap();

    // Bare: no appearance at all, so this face is unbound under "rgb" once
    // welded -- `face_appearance` must drop the theme for this face alone,
    // not for its neighbours.
    let face2 = Polygon3D::from_rings(
        CoordinateFrame::default(),
        [
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [3.0, 1.0, 0.0],
            [2.0, 1.0, 0.0],
            [2.0, 0.0, 0.0],
        ],
        std::iter::empty::<Vec<[f64; 3]>>(),
    );

    let mesh = PolygonMesh3D::from_polygons(CoordinateFrame::default(), [&face0, &face1, &face2])
        .expect("valid mesh");
    let geom = Geometry::Euclidean3D(crate::Euclidean3DGeometry::PolygonMesh(Box::new(mesh)));

    let mut seen: std::collections::BTreeMap<i64, Option<[f32; 3]>> =
        std::collections::BTreeMap::new();
    geom.divide_by_grid(&unit_grid(), &mut |cell, _coverage, piece| {
        let Geometry::Euclidean3D(crate::Euclidean3DGeometry::PolygonMesh(m)) = piece else {
            panic!("expected a polygon mesh piece");
        };
        let diffuse = m.appearance().as_ref().map(|app| {
            let binding = app
                .themes()
                .iter()
                .find(|t| t.theme == theme("rgb"))
                .expect("rgb theme present");
            let idx = match &binding.front {
                FaceBinding::Uniform(idx) => *idx,
                FaceBinding::PerFace(v) => v
                    .first()
                    .copied()
                    .flatten()
                    .expect("this cell's one face is bound"),
            };
            let Material::Phong(p) = &app.materials()[idx.get() as usize] else {
                panic!("expected a phong material");
            };
            p.diffuse
        });
        seen.insert(cell.col, diffuse);
    })
    .expect("divides");

    assert_eq!(seen.len(), 3, "one cell per face");
    assert_eq!(
        seen[&0],
        Some([1.0, 0.0, 0.0]),
        "column 0 keeps face0's own material"
    );
    assert_eq!(
        seen[&1],
        Some([0.0, 1.0, 0.0]),
        "column 1 keeps face1's own material"
    );
    assert_eq!(
        seen[&2], None,
        "column 2's face was unbound under this theme, so no appearance survives it"
    );
}
