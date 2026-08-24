//! The new-geometry CSV geometry parser.
//!
//! Maps parsed input straight to the new geometry model. The old-world parser
//! converts through `geo_types` on its WKT path, and `geo_types` is 2D-only, so
//! it silently discards Z. Nothing here touches `geo_types`.

use std::str::FromStr;

use indexmap::IndexMap;
use reearth_flow_geometry::collection::{Collection2D, Collection3D};
use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};
use reearth_flow_geometry::line_string::{LineString2D, LineString3D};
use reearth_flow_geometry::point::{Point2D, Point3D};
use reearth_flow_geometry::polygon::{Polygon2D, Polygon3D};
use reearth_flow_geometry::{
    Euclidean2DGeometry, Euclidean3DGeometry, Geometry, GeometryCollection,
};

use super::{GeometryConfig, GeometryMode};
use crate::errors::GeometryParsingError;

/// The coordinate frame a config declares: a CRS when it names a non-zero EPSG,
/// otherwise bare Euclidean space. Mirrors `geopackage_next.rs`.
pub(crate) fn frame_for(config: &GeometryConfig) -> CoordinateFrame {
    match config.epsg {
        Some(code) if code > 0 => CoordinateFrame::Crs(EpsgCode::new(code)),
        _ => CoordinateFrame::Euclidean,
    }
}

/// Read one named column as an `f64`, naming the column and the offending value
/// on failure. That text reaches the user in the rejected row's error
/// attribute, so it has to be specific.
fn number(row: &IndexMap<String, String>, column: &str) -> Result<f64, GeometryParsingError> {
    let raw = row
        .get(column)
        .ok_or_else(|| GeometryParsingError::ColumnNotFound(column.to_string()))?;
    raw.parse()
        .map_err(|_| GeometryParsingError::InvalidCoordinate {
            column: column.to_string(),
            value: raw.clone(),
        })
}

/// Parse one row's geometry per `config`.
pub fn parse_geometry(
    row: &IndexMap<String, String>,
    config: &GeometryConfig,
) -> Result<Geometry, GeometryParsingError> {
    let frame = frame_for(config);
    match &config.mode {
        GeometryMode::Coordinates {
            x_column,
            y_column,
            z_column,
        } => {
            let x = number(row, x_column)?;
            let y = number(row, y_column)?;
            match z_column {
                None => Ok(Geometry::Euclidean2D(Euclidean2DGeometry::Point(
                    Point2D::new(frame, [x, y]),
                ))),
                Some(z_column) => {
                    let z = number(row, z_column)?;
                    Ok(Geometry::Euclidean3D(Euclidean3DGeometry::Point(
                        Point3D::new(frame, [x, y, z]),
                    )))
                }
            }
        }
        GeometryMode::Wkt { column } => {
            let text = row
                .get(column)
                .ok_or_else(|| GeometryParsingError::ColumnNotFound(column.clone()))?
                .trim();
            if text.is_empty() {
                return Ok(Geometry::None);
            }
            let parsed = parse_wkt_tolerantly(text)?;
            // Computed once here, never per member: `orientation_sign` resolves
            // the CRS through PROJ, so a MULTIPOLYGON with a thousand members
            // must not repeat that resolution per member.
            let swap = swaps_axes(&frame);
            wkt_to_geometry(parsed, frame, swap)
        }
    }
}

/// Parse WKT, accepting the bare 3D form alongside the tagged OGC form.
///
/// The `wkt` crate requires an explicit dimension tag: `POINT Z(1 2 3)` parses
/// and `POINT(1 2 3)` does not. Our own CSV Writer emits the bare form (a
/// reviewed parity decision with the old writer), so reading our own output
/// needs this. On failure we count the ordinates of the first coordinate group
/// and retry once with a `Z` tag inserted.
///
/// Bare three-ordinate WKT is formally ambiguous between XYZ and XYM. We treat
/// it as XYZ, matching both our writer and the older PostGIS convention.
fn parse_wkt_tolerantly(text: &str) -> Result<wkt::Wkt<f64>, GeometryParsingError> {
    match wkt::Wkt::<f64>::from_str(text) {
        Ok(parsed) => Ok(parsed),
        Err(original) => match with_z_tag(text) {
            Some(tagged) => wkt::Wkt::<f64>::from_str(&tagged)
                // Report the ORIGINAL error, not the retry's: the retry is our
                // invention, and its message would confuse a user whose input
                // was simply malformed.
                .map_err(|_| GeometryParsingError::WktParsing(original.to_string())),
            None => Err(GeometryParsingError::WktParsing(original.to_string())),
        },
    }
}

/// `text` with a ` Z` tag inserted after the type keyword, when its first
/// coordinate group holds exactly three ordinates. `None` when the shape does
/// not look like bare 3D, in which case there is nothing to retry.
fn with_z_tag(text: &str) -> Option<String> {
    let open = text.find('(')?;
    let keyword = text[..open].trim_end();
    // An already-tagged or EMPTY input is not bare 3D.
    if keyword.is_empty() || keyword.split_whitespace().count() > 1 {
        return None;
    }
    // The first coordinate group is the run after the opening parens, so
    // `POLYGON((0 0 0, ...` and `POINT(0 0 0)` both work.
    let after_parens = text[open..].trim_start_matches(['(', ' ']);
    let first_coord = after_parens.split([',', ')']).next()?.trim();
    if first_coord.split_whitespace().count() != 3 {
        return None;
    }
    Some(format!("{keyword} Z{}", &text[open..]))
}

/// One `wkt` geometry as a new-model `Geometry`.
///
/// `EMPTY` forms parse successfully with an absent coord or empty ring lists, so
/// every arm checks for emptiness rather than unwrapping.
fn wkt_to_geometry(
    parsed: wkt::Wkt<f64>,
    frame: CoordinateFrame,
    swap: bool,
) -> Result<Geometry, GeometryParsingError> {
    use wkt::Wkt;
    match parsed {
        Wkt::Point(point) => match point.coord() {
            None => Ok(Geometry::None),
            Some(coord) => Ok(point_geometry(coord, frame, swap)),
        },
        Wkt::LineString(line) => {
            let coords = line.coords();
            if coords.is_empty() {
                return Ok(Geometry::None);
            }
            Ok(line_geometry(coords, frame, swap))
        }
        Wkt::Polygon(polygon) => {
            let rings = polygon.rings();
            if rings.is_empty() {
                return Ok(Geometry::None);
            }
            Ok(polygon_geometry(rings, frame, swap))
        }
        // The new model has no Multi* types, so a MULTI* becomes a same-
        // dimension `Collection` via `collect`. `EMPTY` members are filtered
        // out before folding, so an all-empty run yields `Geometry::None`
        // rather than an empty `Collection`.
        Wkt::MultiPoint(multi) => Ok(collect(
            multi
                .points()
                .iter()
                .filter_map(|p| p.coord())
                .map(|c| point_geometry(c, frame.clone(), swap)),
        )),
        Wkt::MultiLineString(multi) => Ok(collect(
            multi
                .line_strings()
                .iter()
                .map(|l| l.coords())
                .filter(|coords| !coords.is_empty())
                .map(|coords| line_geometry(coords, frame.clone(), swap)),
        )),
        Wkt::MultiPolygon(multi) => Ok(collect(
            multi
                .polygons()
                .iter()
                .map(|p| p.rings())
                .filter(|rings| !rings.is_empty())
                .map(|rings| polygon_geometry(rings, frame.clone(), swap)),
        )),
        // Closes the gap the old reader named in its own error type
        // (`GeometryParsingError::UnsupportedGeometryCollection`): the new
        // path can represent a GeometryCollection, mixed dimensions included.
        // Members are themselves WKT geometries, so this recurses — a nested
        // GEOMETRYCOLLECTION works the same as any other member. `swap` is
        // threaded through rather than recomputed, since it is derived from
        // the same frame at every recursion depth.
        Wkt::GeometryCollection(collection) => {
            let (geoms, _dim) = collection.into_inner();
            let members = geoms
                .into_iter()
                .map(|g| wkt_to_geometry(g, frame.clone(), swap))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .filter(|g| !matches!(g, Geometry::None))
                .collect::<Vec<_>>();
            if members.is_empty() {
                return Ok(Geometry::None);
            }
            Ok(Geometry::GeometryCollection(GeometryCollection::new(
                members,
            )))
        }
    }
}

/// Fold members into a `Collection` of their own dimension. Mixed dimensions
/// cannot share a `Collection`, so they fall back to a `GeometryCollection`,
/// which is the type that spans dimensions. An empty run is no geometry.
fn collect(members: impl Iterator<Item = Geometry>) -> Geometry {
    let members: Vec<Geometry> = members.collect();
    if members.is_empty() {
        return Geometry::None;
    }
    let all_2d = members
        .iter()
        .all(|g| matches!(g, Geometry::Euclidean2D(_)));
    let all_3d = members
        .iter()
        .all(|g| matches!(g, Geometry::Euclidean3D(_)));

    if all_2d {
        let leaves = members
            .into_iter()
            .filter_map(|g| match g {
                Geometry::Euclidean2D(leaf) => Some(leaf),
                _ => None,
            })
            .collect::<Vec<_>>();
        Geometry::Euclidean2D(Euclidean2DGeometry::Collection(Collection2D::new(leaves)))
    } else if all_3d {
        let leaves = members
            .into_iter()
            .filter_map(|g| match g {
                Geometry::Euclidean3D(leaf) => Some(leaf),
                _ => None,
            })
            .collect::<Vec<_>>();
        Geometry::Euclidean3D(Euclidean3DGeometry::Collection(Collection3D::new(leaves)))
    } else {
        Geometry::GeometryCollection(GeometryCollection::new(members))
    }
}

/// Whether any coord in a run carries a Z ordinate. A run is 3D if any of its
/// coords is, so a mixed run is lifted rather than truncated.
fn is_3d(coords: &[wkt::types::Coord<f64>]) -> bool {
    coords.iter().any(|c| c.z.is_some())
}

/// Whether a frame's CRS declares its axes in reversed order, so the first two
/// ordinates in the text are (y, x). Mirrors `swaps_axes` in the CSV writer's
/// `writer_geometry_next.rs`; the two must agree or the round-trip transposes.
///
/// A CRS whose axis order cannot be resolved is treated as not swapping, which
/// is the same fallback the writer takes. `Euclidean` and `Tangent` frames are
/// not CRSs and have no declared axis order, so `orientation_sign` returns
/// `Ok(1)` for them and this is always `false`.
fn swaps_axes(frame: &CoordinateFrame) -> bool {
    matches!(frame.orientation_sign(), Ok(sign) if sign < 0)
}

/// One coord's (x, y) in storage order. The Z ordinate, handled separately by
/// every caller, is never swapped.
fn xy(coord: &wkt::types::Coord<f64>, swap: bool) -> [f64; 2] {
    if swap {
        [coord.y, coord.x]
    } else {
        [coord.x, coord.y]
    }
}

fn point_geometry(coord: &wkt::types::Coord<f64>, frame: CoordinateFrame, swap: bool) -> Geometry {
    let [x, y] = xy(coord, swap);
    match coord.z {
        None => Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(frame, [x, y]))),
        Some(z) => {
            Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(frame, [x, y, z])))
        }
    }
}

fn line_geometry(
    coords: &[wkt::types::Coord<f64>],
    frame: CoordinateFrame,
    swap: bool,
) -> Geometry {
    if is_3d(coords) {
        Geometry::Euclidean3D(Euclidean3DGeometry::LineString(LineString3D::from_coords(
            frame,
            coords.iter().map(|c| {
                let [x, y] = xy(c, swap);
                [x, y, c.z.unwrap_or(0.0)]
            }),
        )))
    } else {
        Geometry::Euclidean2D(Euclidean2DGeometry::LineString(LineString2D::from_coords(
            frame,
            coords.iter().map(|c| xy(c, swap)),
        )))
    }
}

fn polygon_geometry(
    rings: &[wkt::types::LineString<f64>],
    frame: CoordinateFrame,
    swap: bool,
) -> Geometry {
    let any_3d = rings.iter().any(|r| is_3d(r.coords()));
    let exterior = rings[0].coords();
    let interiors = &rings[1..];
    if any_3d {
        Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
            Polygon3D::from_rings(
                frame,
                exterior
                    .iter()
                    .map(|c| {
                        let [x, y] = xy(c, swap);
                        [x, y, c.z.unwrap_or(0.0)]
                    })
                    .collect::<Vec<_>>(),
                interiors
                    .iter()
                    .map(|r| {
                        r.coords()
                            .iter()
                            .map(|c| {
                                let [x, y] = xy(c, swap);
                                [x, y, c.z.unwrap_or(0.0)]
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>(),
            ),
        )))
    } else {
        Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
            Polygon2D::from_rings(
                frame,
                exterior.iter().map(|c| xy(c, swap)).collect::<Vec<_>>(),
                interiors
                    .iter()
                    .map(|r| r.coords().iter().map(|c| xy(c, swap)).collect::<Vec<_>>())
                    .collect::<Vec<_>>(),
            ),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};
    use reearth_flow_geometry::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

    fn row(pairs: &[(&str, &str)]) -> IndexMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn coords_config(z: Option<&str>, epsg: Option<u16>) -> GeometryConfig {
        GeometryConfig {
            mode: GeometryMode::Coordinates {
                x_column: "lon".to_string(),
                y_column: "lat".to_string(),
                z_column: z.map(|s| s.to_string()),
            },
            epsg,
        }
    }

    #[test]
    fn x_and_y_build_a_2d_point() {
        let g = parse_geometry(
            &row(&[("lon", "1.5"), ("lat", "2.5")]),
            &coords_config(None, None),
        )
        .unwrap();
        match g {
            Geometry::Euclidean2D(Euclidean2DGeometry::Point(p)) => {
                assert_eq!(*p.frame(), CoordinateFrame::Euclidean);
            }
            other => panic!("expected a 2D point, got {other:?}"),
        }
    }

    #[test]
    fn x_y_and_z_build_a_3d_point() {
        let g = parse_geometry(
            &row(&[("lon", "1.0"), ("lat", "2.0"), ("h", "3.0")]),
            &coords_config(Some("h"), None),
        )
        .unwrap();
        match g {
            Geometry::Euclidean3D(Euclidean3DGeometry::Point(_)) => {}
            other => panic!("expected a 3D point, got {other:?}"),
        }
    }

    /// A declared EPSG becomes a CRS frame; its absence stays Euclidean.
    #[test]
    fn an_epsg_becomes_a_crs_frame() {
        assert_eq!(
            frame_for(&coords_config(None, Some(6677))),
            CoordinateFrame::Crs(EpsgCode::new(6677))
        );
        assert_eq!(
            frame_for(&coords_config(None, None)),
            CoordinateFrame::Euclidean
        );
        // Zero is not a CRS.
        assert_eq!(
            frame_for(&coords_config(None, Some(0))),
            CoordinateFrame::Euclidean
        );
    }

    /// The error names the column and the value, because that text is what
    /// reaches the user in the rejected row's error attribute.
    #[test]
    fn a_non_numeric_cell_errors_naming_the_column_and_value() {
        let err = parse_geometry(
            &row(&[("lon", "abc"), ("lat", "2.0")]),
            &coords_config(None, None),
        )
        .unwrap_err();
        let text = err.to_string();
        assert!(text.contains("lon"), "{text}");
        assert!(text.contains("abc"), "{text}");
    }

    #[test]
    fn a_missing_column_errors_naming_the_column() {
        let err = parse_geometry(&row(&[("lat", "2.0")]), &coords_config(None, None)).unwrap_err();
        assert!(err.to_string().contains("lon"), "{err}");
    }

    fn wkt_config(epsg: Option<u16>) -> GeometryConfig {
        GeometryConfig {
            mode: GeometryMode::Wkt {
                column: "geom".to_string(),
            },
            epsg,
        }
    }

    fn parse(text: &str) -> Result<Geometry, GeometryParsingError> {
        parse_geometry(&row(&[("geom", text)]), &wkt_config(None))
    }

    #[test]
    fn a_2d_point_parses() {
        match parse("POINT(1 2)").unwrap() {
            Geometry::Euclidean2D(Euclidean2DGeometry::Point(_)) => {}
            other => panic!("expected a 2D point, got {other:?}"),
        }
    }

    /// The case the old reader silently truncated to 2D by hopping through
    /// `geo_types`. Z must survive.
    #[test]
    fn an_ogc_3d_point_keeps_its_z() {
        match parse("POINT Z(1 2 3)").unwrap() {
            Geometry::Euclidean3D(Euclidean3DGeometry::Point(p)) => {
                assert_eq!(p.position()[2], 3.0);
            }
            other => panic!("expected a 3D point, got {other:?}"),
        }
    }

    #[test]
    fn a_linestring_parses_in_both_dimensions() {
        assert!(matches!(
            parse("LINESTRING(0 0, 1 1)").unwrap(),
            Geometry::Euclidean2D(Euclidean2DGeometry::LineString(_))
        ));
        assert!(matches!(
            parse("LINESTRING Z(0 0 0, 1 1 1)").unwrap(),
            Geometry::Euclidean3D(Euclidean3DGeometry::LineString(_))
        ));
    }

    #[test]
    fn a_polygon_keeps_its_hole() {
        let text = "POLYGON((0 0, 4 0, 4 4, 0 4, 0 0), (1 1, 2 1, 2 2, 1 2, 1 1))";
        match parse(text).unwrap() {
            Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(p)) => {
                assert_eq!(p.interiors().count(), 1, "the hole must survive");
            }
            other => panic!("expected a 2D polygon, got {other:?}"),
        }
    }

    /// EMPTY is a parse *success* in the wkt crate, yielding `coord: None` and
    /// empty ring lists. An `unwrap()` on those panics on valid input.
    #[test]
    fn empty_geometries_become_an_absent_geometry_not_a_panic() {
        for text in ["POINT EMPTY", "POLYGON EMPTY", "LINESTRING EMPTY"] {
            assert!(
                matches!(parse(text).unwrap(), Geometry::None),
                "{text} should map to Geometry::None"
            );
        }
    }

    /// A blank cell is a row without geometry, not a malformed one.
    #[test]
    fn a_blank_cell_is_an_absent_geometry_not_an_error() {
        assert!(matches!(parse("").unwrap(), Geometry::None));
        assert!(matches!(parse("   ").unwrap(), Geometry::None));
    }

    #[test]
    fn malformed_wkt_errors_rather_than_panicking() {
        assert!(parse("NOT WKT AT ALL").is_err());
        assert!(parse("POINT(").is_err());
    }

    /// Our own CSV Writer emits the bare 3D form. The `wkt` crate refuses it, so
    /// reading back what we wrote used to abort the whole file. Verified
    /// against wkt 0.14: `POINT(1 2 3)` fails with "Missing closing parenthesis
    /// for type".
    #[test]
    fn the_bare_3d_form_our_writer_emits_is_accepted() {
        match parse("POINT(1 2 3)").unwrap() {
            Geometry::Euclidean3D(Euclidean3DGeometry::Point(p)) => {
                assert_eq!(p.position()[2], 3.0);
            }
            other => panic!("expected a 3D point, got {other:?}"),
        }
    }

    #[test]
    fn bare_3d_linestrings_and_polygons_are_accepted() {
        assert!(matches!(
            parse("LINESTRING(0 0 0, 1 1 1)").unwrap(),
            Geometry::Euclidean3D(Euclidean3DGeometry::LineString(_))
        ));
        assert!(matches!(
            parse("POLYGON((0 0 0, 4 0 0, 4 4 1, 0 0 0))").unwrap(),
            Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(_))
        ));
    }

    /// Nested parens are where a normalising pre-pass is most likely to go
    /// wrong, so a holed bare-3D polygon is the case that matters.
    #[test]
    fn a_bare_3d_polygon_with_a_hole_keeps_both_rings() {
        let text = "POLYGON((0 0 0, 9 0 0, 9 9 0, 0 9 0, 0 0 0), (1 1 0, 2 1 0, 2 2 0, 1 1 0))";
        match parse(text).unwrap() {
            Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(p)) => {
                assert_eq!(p.interiors().count(), 1);
            }
            other => panic!("expected a 3D polygon, got {other:?}"),
        }
    }

    /// Tolerance must not swallow real errors, and must not mangle 2D input.
    #[test]
    fn tolerance_does_not_change_2d_or_hide_malformed_input() {
        assert!(matches!(
            parse("POINT(1 2)").unwrap(),
            Geometry::Euclidean2D(Euclidean2DGeometry::Point(_))
        ));
        assert!(parse("POINT(1 2 3 4 5)").is_err());
        assert!(parse("POINT EMPTY").is_ok());
    }

    /// The retry-also-fails branch of `parse_wkt_tolerantly` is dead unless an
    /// input both (a) looks like bare 3D (three tokens in its first coordinate
    /// group, triggering the rewrite) and (b) is still malformed after the `Z`
    /// tag is added. `"POINT(1 2 x)"` is such an input: it rewrites to
    /// `"POINT Z(1 2 x)"`, which still fails, landing on the map_err.
    ///
    /// The assertion that matters is not that this errors (a weaker test would
    /// pass even if the branch reported the wrong thing) but that the reported
    /// text is the error from parsing the ORIGINAL input, not the rewritten
    /// one. Verified directly: `wkt` reports "Missing closing parenthesis for
    /// type" for the bare original and "Expected a number for the Z
    /// coordinate" for the retried rewrite. Those two strings differ, so
    /// asserting the former and refuting the latter actually pins which one
    /// surfaces.
    #[test]
    fn a_failed_retry_reports_the_original_error_not_the_rewritten_ones() {
        let err = parse("POINT(1 2 x)").unwrap_err().to_string();
        assert!(
            err.contains("Missing closing parenthesis for type"),
            "expected the ORIGINAL parse error, got: {err}"
        );
        assert!(
            !err.contains("Expected a number for the Z coordinate"),
            "leaked the RETRY's error instead of the original: {err}"
        );
    }

    /// `with_z_tag` in isolation, asserting its exact return value rather than
    /// inferring its behavior through the parser. The dangerous failure mode
    /// for this function is a rewrite that is syntactically valid but
    /// semantically wrong (e.g. tagging the wrong keyword, or splicing at the
    /// wrong offset) — such a rewrite would still parse downstream and would
    /// only be caught by pinning the exact string.
    #[test]
    fn with_z_tag_returns_the_exact_expected_rewrite() {
        // 2D must never be rewritten.
        assert_eq!(with_z_tag("POINT(1 2)"), None);
        // The canonical bare-3D case.
        assert_eq!(
            with_z_tag("POINT(1 2 3)"),
            Some("POINT Z(1 2 3)".to_string())
        );
        // Already tagged: must not double-tag.
        assert_eq!(with_z_tag("POINT Z(1 2 3)"), None);
        // No paren at all.
        assert_eq!(with_z_tag("POINT EMPTY"), None);
        // The coordinate run is two parens deep.
        assert_eq!(
            with_z_tag("POLYGON((0 0 0, 4 0 0, 4 4 1, 0 0 0))"),
            Some("POLYGON Z((0 0 0, 4 0 0, 4 4 1, 0 0 0))".to_string())
        );
        // A second ring follows the first; only the keyword gets tagged, the
        // rest of the text (including the hole) passes through untouched.
        assert_eq!(
            with_z_tag(
                "POLYGON((0 0 0, 9 0 0, 9 9 0, 0 9 0, 0 0 0), (1 1 0, 2 1 0, 2 2 0, 1 1 0))"
            ),
            Some(
                "POLYGON Z((0 0 0, 9 0 0, 9 9 0, 0 9 0, 0 0 0), (1 1 0, 2 1 0, 2 2 0, 1 1 0))"
                    .to_string()
            )
        );
        // A bare multi-geometry whose first point is 2D must not be rewritten.
        assert_eq!(with_z_tag("MULTIPOINT(0 0, 1 1)"), None);
        // A bare multi-geometry whose first point is 3D: only the outer
        // keyword is tagged.
        assert_eq!(
            with_z_tag("MULTIPOINT((0 0 0), (1 1 1))"),
            Some("MULTIPOINT Z((0 0 0), (1 1 1))".to_string())
        );
        // Five ordinates is not bare 3D.
        assert_eq!(with_z_tag("POINT(1 2 3 4 5)"), None);
    }

    /// The new model has no Multi* types, so a MULTI* becomes a Collection.
    /// The writer emits MULTIPOINT in the flat spelling, so both must work.
    #[test]
    fn multipoint_becomes_a_collection_in_both_spellings() {
        for text in ["MULTIPOINT(0 0, 1 1)", "MULTIPOINT((0 0), (1 1))"] {
            match parse(text).unwrap() {
                Geometry::Euclidean2D(Euclidean2DGeometry::Collection(c)) => {
                    assert_eq!(c.members().len(), 2, "{text}");
                }
                other => panic!("{text} gave {other:?}"),
            }
        }
    }

    #[test]
    fn multilinestring_becomes_a_collection() {
        match parse("MULTILINESTRING((0 0, 1 1), (2 2, 3 3))").unwrap() {
            Geometry::Euclidean2D(Euclidean2DGeometry::Collection(c)) => {
                assert_eq!(c.members().len(), 2);
            }
            other => panic!("expected a 2D collection, got {other:?}"),
        }
    }

    #[test]
    fn multipolygon_becomes_a_collection_keeping_holes() {
        let text = "MULTIPOLYGON(((0 0, 9 0, 9 9, 0 0), (1 1, 2 1, 2 2, 1 1)), ((20 20, 21 20, 21 21, 20 20)))";
        match parse(text).unwrap() {
            Geometry::Euclidean2D(Euclidean2DGeometry::Collection(c)) => {
                assert_eq!(c.members().len(), 2);
            }
            other => panic!("expected a 2D collection, got {other:?}"),
        }
    }

    /// The gap the old reader named in its own error type:
    /// "GeometryCollection is not yet supported in CSV reader".
    #[test]
    fn a_geometrycollection_is_supported_and_holds_mixed_members() {
        match parse("GEOMETRYCOLLECTION(POINT(1 2), LINESTRING(0 0, 1 1))").unwrap() {
            Geometry::GeometryCollection(c) => {
                assert_eq!(c.members().len(), 2);
            }
            other => panic!("expected a GeometryCollection, got {other:?}"),
        }
    }

    #[test]
    fn empty_collections_become_an_absent_geometry() {
        for text in ["MULTIPOINT EMPTY", "GEOMETRYCOLLECTION EMPTY"] {
            assert!(matches!(parse(text).unwrap(), Geometry::None), "{text}");
        }
    }

    /// A 3D MULTI* yields a 3D collection, so the writer's mesh output
    /// (emitted as MULTIPOLYGON) reads back in 3D.
    #[test]
    fn a_3d_multipolygon_yields_a_3d_collection() {
        let text = "MULTIPOLYGON Z(((0 0 0, 9 0 0, 9 9 1, 0 0 0)))";
        assert!(matches!(
            parse(text).unwrap(),
            Geometry::Euclidean3D(Euclidean3DGeometry::Collection(_))
        ));
    }

    /// The writer swaps axes when a CRS declares reversed order, so for
    /// EPSG:4326 it writes latitude first. The reader must mirror that or the
    /// round-trip transposes coordinates. Driven by the frame, never hardcoded:
    /// EPSG:3857 (Web Mercator, metres, easting-first) must NOT swap.
    ///
    /// The brief for this task named EPSG:6677 (a Japan Plane Rectangular CS
    /// zone) as the "must not swap" fixture, on the assumption that it stores
    /// (x, y) in standard order. That assumption is wrong: `axis_order_sign`
    /// resolves it to `-1` (northing-first), the same family member as
    /// EPSG:6669, which `reearth-flow-geometry`'s own
    /// `northing_first_projected_is_negative` test already documents as
    /// northing-first. Verified directly against this crate's PROJ build
    /// before substituting. EPSG:3857 is used instead: it is asserted `+1`
    /// (easting-first) by that same crate's `easting_first_projected_is_positive`
    /// test, so it is a genuine, verified non-swapping counterexample.
    #[test]
    fn a_reversed_axis_crs_reads_transposed_and_a_normal_one_does_not() {
        let text = "POINT(10 20)";

        let swapped = parse_geometry(&row(&[("geom", text)]), &wkt_config(Some(4326))).unwrap();
        let plain = parse_geometry(&row(&[("geom", text)]), &wkt_config(Some(3857))).unwrap();

        let position = |g: Geometry| match g {
            Geometry::Euclidean2D(Euclidean2DGeometry::Point(p)) => p.position(),
            other => panic!("expected a 2D point, got {other:?}"),
        };

        assert_eq!(position(plain), [10.0, 20.0], "EPSG:3857 must not swap");
        assert_eq!(position(swapped), [20.0, 10.0], "EPSG:4326 must swap");
    }
}
