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
///
/// Takes the polygon rather than a caller-supplied corner count: an
/// `Appearance`'s `Explicit` UV is parallel to the *whole* `coords` buffer
/// (exterior, then every hole, each stored closed), so passing
/// `exterior().len()` -- which is only the true total for a hole-free face --
/// would let a piece whose UV is out of step with its rings slip through.
fn assert_appearance_couples(p: &Polygon2D) {
    use crate::appearance::{validate_uv_coupling, FaceBinding, Side};
    use std::collections::{BTreeMap, BTreeSet};

    let Some(app) = p.appearance().as_ref() else {
        panic!("expected an appearance to check coupling on");
    };
    let corner_count = p.exterior().len() + p.interiors().map(|r| r.len()).sum::<usize>();

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
            // The `layout_unchanged` fast path clones the source appearance
            // verbatim, so the piece's own rings must still be stored the way
            // the source's were -- closed -- or the cloned 5-entry UV would
            // sit on a 4-corner face.
            assert_appearance_couples(&p);
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
            // As above: the fast path's verbatim clone is only valid because
            // the piece's rings are stored closed like the source's.
            assert_appearance_couples(&p);
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
            assert_appearance_couples(&p);
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
            assert_appearance_couples(&p);
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
            assert_appearance_couples(&p);
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

#[test]
fn polygon_mesh_world_to_texture_survives_multi_cell_uncut_division() {
    use crate::appearance::{
        Appearance, ChannelId, FaceBinding, MaterialIndex, Side, TexMatrix, ThemeBinding, UvSet,
        UvSource,
    };
    use crate::test_support::{textured, theme};

    // Three faces, one per grid column, none cut by any grid line -- same
    // layout as `polygon_mesh_per_face_material_binding_survives_division`,
    // so the fast path cannot fire (`buckets.len() == 3`, not `1`) even
    // though no individual piece is cut either. This is exactly the gap a
    // prior round missed: `PolygonMesh3D::from_polygons` bakes
    // `WorldToTexture` to `Explicit` at weld time regardless of whether any
    // face was actually clipped, so routing genuinely-uncut per-cell pieces
    // through that constructor (as every non-fast-path cell does) must not
    // let that baking show through.
    let mut mesh = PolygonMesh3D::from_parts(
        CoordinateFrame::default(),
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [2.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [3.0, 1.0, 0.0],
            [2.0, 1.0, 0.0],
        ],
        [[0u32, 1, 2, 3], [4, 5, 6, 7], [8, 9, 10, 11]],
    )
    .expect("valid mesh");
    let matrix = TexMatrix([
        [0.25, 0.0, 0.0, 0.0],
        [0.0, 0.25, 0.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);
    let app = Appearance::from_parts(
        vec![textured()],
        vec![ThemeBinding {
            theme: theme("ir"),
            front: FaceBinding::Uniform(MaterialIndex::new(0).unwrap()),
            back: None,
            uv_sets: vec![UvSet {
                side: Side::Front,
                channel: ChannelId::default(),
                uv: UvSource::WorldToTexture(matrix),
            }],
        }],
        theme("ir"),
    );
    *mesh.appearance_mut() = Some(app);

    let geom = Geometry::Euclidean3D(crate::Euclidean3DGeometry::PolygonMesh(Box::new(mesh)));
    let mut checked = 0;
    geom.divide_by_grid(&unit_grid(), &mut |_cell, _coverage, piece| {
        let Geometry::Euclidean3D(crate::Euclidean3DGeometry::PolygonMesh(m)) = piece else {
            panic!("expected a polygon mesh piece");
        };
        let app = m
            .appearance()
            .as_ref()
            .expect("appearance carried through the weld");
        assert!(
            matches!(
                app.themes()[0].uv_sets[0].uv,
                UvSource::WorldToTexture(out) if out == matrix
            ),
            "WorldToTexture must not be baked to Explicit by a weld none of these faces needed"
        );
        checked += 1;
    })
    .expect("divides");
    assert_eq!(checked, 3, "one cell per face, none of them cut");
}

#[test]
fn polygon_mesh_world_to_texture_survives_a_genuine_cut() {
    use crate::appearance::{
        Appearance, ChannelId, FaceBinding, MaterialIndex, Side, TexMatrix, ThemeBinding, UvSet,
        UvSource,
    };
    use crate::test_support::{textured, theme};

    // One face spanning two cells (x in 0..2), genuinely severed by the
    // grid line at x = 1 -- the case the *unconditional* rule (survive
    // regardless of cut status) actually needs to cover, since an
    // uncut-only fix would miss it.
    let mut mesh = PolygonMesh3D::from_parts(
        CoordinateFrame::default(),
        vec![
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [2.0, 1.0, 0.0],
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
        vec![textured()],
        vec![ThemeBinding {
            theme: theme("ir"),
            front: FaceBinding::Uniform(MaterialIndex::new(0).unwrap()),
            back: None,
            uv_sets: vec![UvSet {
                side: Side::Front,
                channel: ChannelId::default(),
                uv: UvSource::WorldToTexture(matrix),
            }],
        }],
        theme("ir"),
    );
    *mesh.appearance_mut() = Some(app);

    let geom = Geometry::Euclidean3D(crate::Euclidean3DGeometry::PolygonMesh(Box::new(mesh)));
    let mut checked = 0;
    geom.divide_by_grid(&unit_grid(), &mut |_cell, _coverage, piece| {
        let Geometry::Euclidean3D(crate::Euclidean3DGeometry::PolygonMesh(m)) = piece else {
            panic!("expected a polygon mesh piece");
        };
        let app = m
            .appearance()
            .as_ref()
            .expect("appearance carried through a genuine cut");
        assert!(
            matches!(
                app.themes()[0].uv_sets[0].uv,
                UvSource::WorldToTexture(out) if out == matrix
            ),
            "WorldToTexture must survive the cut piece too, unbaked"
        );
        checked += 1;
    })
    .expect("divides");
    assert_eq!(checked, 2, "the face is genuinely severed across two cells");
}

#[test]
fn collection_members_together_filling_a_cell_report_full() {
    let left = square_2d_at(0.0, 0.0, 0.5, 1.0);
    let right = square_2d_at(0.5, 0.0, 1.0, 1.0);
    let coll = crate::collection::Collection2D::new([
        crate::Euclidean2DGeometry::Polygon(Box::new(left)),
        crate::Euclidean2DGeometry::Polygon(Box::new(right)),
    ]);
    let geom = Geometry::Euclidean2D(crate::Euclidean2DGeometry::Collection(coll));

    let out = collect(&geom, &unit_grid()).expect("divides");
    assert_eq!(out.len(), 1, "one collection per cell, not one per member");
    assert_eq!(out[0].1, CellCoverage::Full);
}

#[test]
fn collection_with_an_undividable_member_still_divides_the_rest() {
    // A bag holding one face and one point divides the face and skips the
    // point, rather than the point costing the caller the face.
    let face = Polygon3D::from_rings(
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
    let mixed = crate::collection::Collection3D::new([
        crate::Euclidean3DGeometry::Polygon(Box::new(face)),
        crate::Euclidean3DGeometry::Point(Point3D::new(
            CoordinateFrame::default(),
            [0.5, 0.5, 0.0],
        )),
    ]);
    let geom = Geometry::Euclidean3D(crate::Euclidean3DGeometry::Collection(mixed));
    let out = collect(&geom, &unit_grid()).expect("the face still divides");
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].1, CellCoverage::Full, "the face fills the cell");
}

#[test]
fn collection_of_only_undividable_members_is_empty() {
    // Empty, not Unsupported: a bag is something we know how to divide, it just
    // had nothing to give.
    let only_point = crate::collection::Collection3D::new([crate::Euclidean3DGeometry::Point(
        Point3D::new(CoordinateFrame::default(), [0.5, 0.5, 0.0]),
    )]);
    let geom = Geometry::Euclidean3D(crate::Euclidean3DGeometry::Collection(only_point));
    assert!(matches!(
        collect(&geom, &unit_grid()),
        Err(GridDivideError::Empty)
    ));
}

fn square_2d_at(x0: f64, y0: f64, x1: f64, y1: f64) -> Polygon2D {
    Polygon2D::from_rings(
        CoordinateFrame::default(),
        [[x0, y0], [x1, y0], [x1, y1], [x0, y1], [x0, y0]],
        std::iter::empty::<Vec<[f64; 2]>>(),
    )
}

/// A 2x1 rectangle with one square hole, both rings stored closed. Used to
/// check that *every* output ring is stored the way the source's were, not
/// just the exterior.
fn holed_rect_2d() -> Polygon2D {
    Polygon2D::from_rings(
        CoordinateFrame::default(),
        [[0.0, 0.0], [2.0, 0.0], [2.0, 1.0], [0.0, 1.0], [0.0, 0.0]],
        [vec![
            [0.2, 0.2],
            [0.2, 0.8],
            [0.8, 0.8],
            [0.8, 0.2],
            [0.2, 0.2],
        ]],
    )
}

#[test]
fn a_2_5d_face_keeps_its_elevation_on_every_divided_piece() {
    // `Polygon2D::from_rings` -- what the rebuild goes through -- hardcodes
    // `z: None`, so without an explicit carry the elevation of a 2.5D face
    // (the shape `polygon/feature_write.rs`, `ops/hole.rs`, `ops/elevation.rs`
    // and mesh face decomposition all produce) is silently dropped by a
    // division. A grid clip only cuts in XY, so it must come through
    // unchanged.
    let poly = Polygon2D::from_rings_at_elevation(
        CoordinateFrame::default(),
        [[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0], [0.0, 0.0]],
        std::iter::empty::<Vec<[f64; 2]>>(),
        42.5,
    );
    assert_eq!(poly.elevation(), Some(42.5), "fixture is 2.5D");
    let geom = Geometry::Euclidean2D(crate::Euclidean2DGeometry::Polygon(Box::new(poly)));

    let mut checked = 0;
    geom.divide_by_grid(&unit_grid(), &mut |_cell, _coverage, piece| {
        let Geometry::Euclidean2D(crate::Euclidean2DGeometry::Polygon(p)) = piece else {
            panic!("expected a polygon piece");
        };
        assert_eq!(
            p.elevation(),
            Some(42.5),
            "a grid clip never changes Z, so the elevation must carry through"
        );
        checked += 1;
    })
    .expect("divides");
    assert_eq!(checked, 4, "one piece per quadrant");
}

#[test]
fn a_pure_2d_face_stays_pure_2d_through_a_division() {
    // The other half of the elevation carry: `None` must stay `None` rather
    // than becoming some fabricated height.
    let geom = Geometry::Euclidean2D(crate::Euclidean2DGeometry::Polygon(Box::new(square_2d(
        0.0, 2.0,
    ))));
    let mut checked = 0;
    geom.divide_by_grid(&unit_grid(), &mut |_cell, _coverage, piece| {
        let Geometry::Euclidean2D(crate::Euclidean2DGeometry::Polygon(p)) = piece else {
            panic!("expected a polygon piece");
        };
        assert_eq!(p.elevation(), None);
        checked += 1;
    })
    .expect("divides");
    assert_eq!(checked, 4);
}

/// Every ring of `p` is stored closed (first == last), the way a well-formed
/// polygon is -- what `validation_next`'s `check_unclosed_ring` requires and
/// what `ops::containment`'s `coord_pos_relative_to_ring` assumes.
fn assert_rings_stored_closed(p: &Polygon2D) {
    for (i, ring) in std::iter::once(p.exterior())
        .chain(p.interiors())
        .enumerate()
    {
        assert!(
            ring.len() >= 4,
            "ring {i} is too short to be closed: {ring:?}"
        );
        assert_eq!(
            ring.first(),
            ring.last(),
            "ring {i} left the division open: {ring:?}"
        );
    }
}

#[test]
fn a_cut_piece_leaves_with_closed_rings_and_a_uv_that_matches() {
    use crate::appearance::UvSource;
    use crate::test_support::{explicit_uv, textured, theme};

    // Spans four cells with a hole in the lower-left one, so pieces come off
    // both the cut path (rewritten rings) and with more than one ring to
    // re-close. The UV is parallel to the stored, closed coords -- ten
    // entries for 5 + 5 -- exactly as `Appearance` requires.
    let mut poly = Polygon2D::from_rings(
        CoordinateFrame::default(),
        [[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0], [0.0, 0.0]],
        [vec![
            [0.2, 0.2],
            [0.2, 0.8],
            [0.8, 0.8],
            [0.8, 0.2],
            [0.2, 0.2],
        ]],
    );
    let uv: Vec<[f64; 2]> = std::iter::once(poly.exterior())
        .chain(poly.interiors())
        .flat_map(|r| r.iter())
        .map(|c| [c[0] / 2.0, c[1] / 2.0])
        .collect();
    poly.set_appearance(theme("rgb"), textured(), Some(explicit_uv(&uv)))
        .unwrap();
    let geom = Geometry::Euclidean2D(crate::Euclidean2DGeometry::Polygon(Box::new(poly)));

    let mut checked = 0;
    let mut saw_hole = false;
    geom.divide_by_grid(&unit_grid(), &mut |_cell, _coverage, piece| {
        let Geometry::Euclidean2D(crate::Euclidean2DGeometry::Polygon(p)) = piece else {
            panic!("expected a polygon piece");
        };
        assert_rings_stored_closed(&p);
        saw_hole |= p.interiors().count() > 0;

        // The UV must be parallel to the *stored* coords, closing duplicates
        // included -- and hole corners counted, which is what
        // `assert_appearance_couples` re-checks through `validate_uv_coupling`.
        let app = p.appearance().as_ref().expect("uv survives the cut");
        let UvSource::Explicit(out_uv) = &app.themes()[0].uv_sets[0].uv else {
            panic!("expected an explicit uv set");
        };
        let corners = p.exterior().len() + p.interiors().map(|r| r.len()).sum::<usize>();
        assert_eq!(out_uv.len(), corners, "uv must stay parallel to the coords");
        assert_appearance_couples(&p);

        // u == x/2, v == y/2 on the source, and the clip interpolates with the
        // same parameter, so it holds on every output corner too -- including
        // the restored closing duplicates.
        for (corner, uv) in std::iter::once(p.exterior())
            .chain(p.interiors())
            .flat_map(|r| r.iter())
            .zip(out_uv.iter())
        {
            assert!((uv[0] - corner[0] / 2.0).abs() < 1e-12);
            assert!((uv[1] - corner[1] / 2.0).abs() < 1e-12);
        }
        checked += 1;
    })
    .expect("divides");
    assert_eq!(checked, 4, "one piece per quadrant");
    assert!(saw_hole, "the hole must survive into at least one piece");
}

#[test]
fn an_untouched_piece_is_stored_exactly_like_its_source() {
    // The `layout_unchanged` fast path clones the source appearance verbatim,
    // which is only sound when the piece's stored corner buffer matches the
    // source's. Exactly matching the cell means the clip touches nothing, so
    // the piece must come back byte-identical to its source.
    let source = holed_rect_2d();
    let grid = GridSpec::new([0.0, 0.0], 2.0).expect("valid spec");
    let geom = Geometry::Euclidean2D(crate::Euclidean2DGeometry::Polygon(Box::new(
        source.clone(),
    )));

    let mut checked = 0;
    geom.divide_by_grid(&grid, &mut |_cell, _coverage, piece| {
        let Geometry::Euclidean2D(crate::Euclidean2DGeometry::Polygon(p)) = piece else {
            panic!("expected a polygon piece");
        };
        assert_rings_stored_closed(&p);
        assert_eq!(p.exterior(), source.exterior());
        assert!(p.interiors().eq(source.interiors()));
        checked += 1;
    })
    .expect("divides");
    assert_eq!(checked, 1);
}

#[test]
fn a_source_stored_open_stays_open_so_the_mesh_leaves_can_reuse_this_op() {
    // `polygon_mesh/ops.rs` rebuilds each mesh face as a bare polygon whose
    // rings are open -- a mesh's CSR face buffers leave the closing edge
    // implied -- divides that, and welds the pieces back with
    // `PolygonMesh3D::from_polygons`, which re-closes each ring itself and
    // compares pieces against their source faces ring-for-ring. Closing an
    // open source's pieces here would give them one more stored coord than
    // their appearance's UV. So the op mirrors the source's storage rather
    // than normalising it.
    let open = Polygon2D::from_rings(
        CoordinateFrame::default(),
        [[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]],
        std::iter::empty::<Vec<[f64; 2]>>(),
    );
    let geom = Geometry::Euclidean2D(crate::Euclidean2DGeometry::Polygon(Box::new(open)));

    let mut checked = 0;
    geom.divide_by_grid(&unit_grid(), &mut |_cell, _coverage, piece| {
        let Geometry::Euclidean2D(crate::Euclidean2DGeometry::Polygon(p)) = piece else {
            panic!("expected a polygon piece");
        };
        assert_ne!(
            p.exterior().first(),
            p.exterior().last(),
            "an open source's pieces must stay open"
        );
        checked += 1;
    })
    .expect("divides");
    assert_eq!(checked, 4);
}

#[test]
fn coverage_is_judged_against_the_cell_s_own_window_not_the_square_of_its_side() {
    use crate::ops::grid::COVERAGE_TOLERANCE;

    // A cell's true area is `(max - min)^2` over its *own* corners, which is
    // not `cell_size^2` in floating point: `cell_bounds` rounds `origin +
    // n * cell_size`, so the width a clip actually snaps a full piece to
    // differs from the nominal side. At a projected-CRS-sized origin the gap
    // is past `COVERAGE_TOLERANCE`, so judging against `cell_size^2` calls an
    // exactly-full piece `Partial` -- and `completeCellsOnly` then drops it.
    let cell_size = 0.3;
    let grid = GridSpec::new([3_900_000.0, 3_900_000.0], cell_size).expect("valid spec");
    let cell = GridCell { row: 0, col: 0 };
    let window_area = grid.window(cell).area();
    let nominal = cell_size * cell_size;

    let relative_gap = (window_area - nominal) / nominal;
    assert!(
        relative_gap.abs() > COVERAGE_TOLERANCE,
        "fixture must actually separate the two comparands, gap was {relative_gap:e}"
    );

    // A piece whose area is exactly the cell's window -- the clip's own
    // guarantee, since a cut vertex takes the cell's coordinate verbatim --
    // is Full against the window and Partial against the nominal square.
    assert_eq!(
        CellCoverage::from_area(window_area, window_area),
        CellCoverage::Full,
        "an exactly-full piece must be Full against its own cell"
    );
    assert_eq!(
        CellCoverage::from_area(window_area, nominal),
        CellCoverage::Partial,
        "this is the data loss `cell_size * cell_size` reintroduced"
    );
}

#[test]
fn every_leaf_judges_coverage_against_the_same_cell_area() {
    // Guards the five call sites that used to compute the cell's area as
    // `cell_size * cell_size` while `polygon/ops.rs` used `window.area()`:
    // the polygon, polygon-mesh, triangular-mesh and collection leaves must
    // all agree about the same cell, on a grid whose origin is not the
    // coordinate origin.
    //
    // This pins the leaves against each other; the *reason* the comparand has
    // to be the cell's own window is pinned separately by
    // `coverage_is_judged_against_the_cell_s_own_window_not_the_square_of_its_side`.
    // The two cannot be one test: separating the comparands needs an origin
    // several million times the cell size, and at that magnitude the shoelace
    // every leaf measures area with (`signed_area_xy`, on absolute
    // coordinates) loses far more precision than the gap being measured, so
    // every leaf reports `Partial` either way.
    use crate::triangular_mesh::TriangularMesh3D;

    let grid = GridSpec::new([10.5, 20.25], 1.0).expect("valid spec");
    let (mn, mx) = grid.cell_bounds(GridCell { row: 0, col: 0 });

    let face = |x0: f64, x1: f64| {
        Polygon3D::from_rings(
            CoordinateFrame::default(),
            [
                [x0, mn[1], 0.0],
                [x1, mn[1], 0.0],
                [x1, mx[1], 0.0],
                [x0, mx[1], 0.0],
                [x0, mn[1], 0.0],
            ],
            std::iter::empty::<Vec<[f64; 3]>>(),
        )
    };
    let mid = mn[0] + (mx[0] - mn[0]) / 2.0;

    let polygon = Geometry::Euclidean3D(crate::Euclidean3DGeometry::Polygon(Box::new(face(
        mn[0], mx[0],
    ))));
    let mesh = Geometry::Euclidean3D(crate::Euclidean3DGeometry::PolygonMesh(Box::new(
        PolygonMesh3D::from_polygons(
            CoordinateFrame::default(),
            [&face(mn[0], mid), &face(mid, mx[0])],
        )
        .expect("valid mesh"),
    )));
    let tri = Geometry::Euclidean3D(crate::Euclidean3DGeometry::TriangularMesh(Box::new(
        TriangularMesh3D::from_parts(
            CoordinateFrame::default(),
            vec![
                [mn[0], mn[1], 0.0],
                [mx[0], mn[1], 0.0],
                [mx[0], mx[1], 0.0],
                [mn[0], mx[1], 0.0],
            ],
            [0u32, 1, 2, 0, 2, 3],
        )
        .expect("valid mesh"),
    )));
    let collection = Geometry::Euclidean3D(crate::Euclidean3DGeometry::Collection(
        crate::collection::Collection3D::new([
            crate::Euclidean3DGeometry::Polygon(Box::new(face(mn[0], mid))),
            crate::Euclidean3DGeometry::Polygon(Box::new(face(mid, mx[0]))),
        ]),
    ));

    for (name, geom) in [
        ("polygon", polygon),
        ("polygon mesh", mesh),
        ("triangular mesh", tri),
        ("collection", collection),
    ] {
        let out = collect(&geom, &grid).expect("divides");
        assert_eq!(out.len(), 1, "{name} must land in exactly one cell");
        assert_eq!(
            out[0].1,
            CellCoverage::Full,
            "{name} exactly fills the cell and must report Full"
        );
    }
}

#[test]
fn a_solid_in_a_foreign_frame_makes_a_collection_mixed_frames() {
    use crate::coordinate::EpsgCode;
    use crate::solid::Solid;
    use crate::triangular_mesh::TriangularMesh3DData;

    // `Solid::frame()` exists and `Geometry::frame()` reads it, so the
    // collection's own frame check must read it too -- otherwise the
    // `MixedFrames` verdict and `Geometry::frame()` disagree about the same
    // geometry, and the grid-divider action's angular-frame warning is judged
    // on a frame set the divider itself never considered.
    let plane = CoordinateFrame::default();
    let other = CoordinateFrame::Crs(EpsgCode::from(4326));

    let face = Polygon3D::from_rings(
        plane.clone(),
        [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ],
        std::iter::empty::<Vec<[f64; 3]>>(),
    );
    let shell = TriangularMesh3DData::from_parts(
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        [0u32, 1, 2],
    )
    .expect("valid shell");
    let solid = Solid::from_exterior(other, shell);

    let mixed = crate::collection::Collection3D::new([
        crate::Euclidean3DGeometry::Polygon(Box::new(face)),
        crate::Euclidean3DGeometry::Solid(Box::new(solid)),
    ]);
    let geom = Geometry::Euclidean3D(crate::Euclidean3DGeometry::Collection(mixed));

    assert_eq!(
        geom.frame(),
        None,
        "the two leaves disagree, so there is no single frame"
    );
    assert!(
        matches!(
            collect(&geom, &unit_grid()),
            Err(GridDivideError::MixedFrames)
        ),
        "the frame check must see the solid's frame, like `Geometry::frame` does"
    );
}

#[test]
fn a_solid_sharing_the_collection_s_frame_does_not_block_the_division() {
    use crate::solid::Solid;
    use crate::triangular_mesh::TriangularMesh3DData;

    // The flip side: collecting the solid's frame must not turn an agreeing
    // collection into a `MixedFrames` failure. The solid itself is still
    // `Unsupported` for division and is simply skipped.
    let frame = CoordinateFrame::default();
    let face = Polygon3D::from_rings(
        frame.clone(),
        [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ],
        std::iter::empty::<Vec<[f64; 3]>>(),
    );
    let shell = TriangularMesh3DData::from_parts(
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        [0u32, 1, 2],
    )
    .expect("valid shell");
    let agreeing = crate::collection::Collection3D::new([
        crate::Euclidean3DGeometry::Polygon(Box::new(face)),
        crate::Euclidean3DGeometry::Solid(Box::new(Solid::from_exterior(frame, shell))),
    ]);
    let geom = Geometry::Euclidean3D(crate::Euclidean3DGeometry::Collection(agreeing));

    let out = collect(&geom, &unit_grid()).expect("the face still divides");
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].1, CellCoverage::Full);
}
