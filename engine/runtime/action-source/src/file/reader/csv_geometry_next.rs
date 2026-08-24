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
            // Mirrors the WKT arm: a frame whose CRS declares reversed axis
            // order (e.g. EPSG:4326, latitude-first) stores (y, x), not the
            // text/column order. Z is never swapped.
            let [x, y] = if swaps_axes(&frame) { [y, x] } else { [x, y] };
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
            let parsed = parse_wkt_tolerantly(text)
                .map_err(|error| annotate_wkt_parse_error(error, column, text))?;
            // Computed once here, never per member: `orientation_sign` resolves
            // the CRS through PROJ, so a MULTIPOLYGON with a thousand members
            // must not repeat that resolution per member.
            let swap = swaps_axes(&frame);
            wkt_to_geometry(parsed, frame, swap)
        }
    }
}

/// A WKT cell can hold a several-thousand-vertex polygon, and the error text
/// this caps lands in a CSV cell of the rejected output alongside it. 120
/// characters is enough to recognize which value is at fault (its geometry
/// type, its first few coordinates) without the error cell dwarfing the row
/// it describes; the untruncated original always survives in the row's own
/// geometry column (see `csv.rs`), so nothing here is actually lost.
const WKT_ERROR_VALUE_MAX_CHARS: usize = 120;

/// Caps `value` to `WKT_ERROR_VALUE_MAX_CHARS`, appending `...` when cut.
/// Truncates on `char` boundaries so multi-byte input is never split mid-
/// character.
fn truncate_for_error(value: &str) -> String {
    if value.chars().count() <= WKT_ERROR_VALUE_MAX_CHARS {
        return value.to_string();
    }
    let mut truncated: String = value.chars().take(WKT_ERROR_VALUE_MAX_CHARS).collect();
    truncated.push_str("...");
    truncated
}

/// Wraps a WKT parse failure with the offending column name and a (truncated)
/// copy of its value, mirroring `InvalidCoordinate`'s column+value on the
/// coordinates path -- without this, `WktParsing`'s message is only ever the
/// `wkt` crate's own parser text, which names neither.
///
/// `GeometryParsingError::WktParsing` itself is not restructured to carry
/// `column`/`value` as fields: the old-world parser in `csv_geometry.rs`
/// constructs that same variant and is deliberately left byte-for-byte
/// untouched by this migration, so the detail is folded into the existing
/// `String` at this call site instead.
fn annotate_wkt_parse_error(
    error: GeometryParsingError,
    column: &str,
    value: &str,
) -> GeometryParsingError {
    match error {
        GeometryParsingError::WktParsing(message) => GeometryParsingError::WktParsing(format!(
            "column '{column}': {message} (value: {})",
            truncate_for_error(value)
        )),
        other => other,
    }
}

/// Parse WKT, accepting the bare 3D form alongside the tagged OGC form.
///
/// The `wkt` crate requires an explicit dimension tag: `POINT Z(1 2 3)` parses
/// and `POINT(1 2 3)` does not. Our own CSV Writer emits the bare form (a
/// reviewed parity decision with the old writer), so reading our own output
/// needs this. On failure, `bare_3d_retry` computes a tagged rewrite and we
/// retry once against it; if that still fails, or the input never looked like
/// bare 3D in the first place, the ORIGINAL parse error is reported, never
/// the retry's own.
///
/// Bare three-ordinate WKT is formally ambiguous between XYZ and XYM. We treat
/// it as XYZ, matching both our writer and the older PostGIS convention.
fn parse_wkt_tolerantly(text: &str) -> Result<wkt::Wkt<f64>, GeometryParsingError> {
    match wkt::Wkt::<f64>::from_str(text) {
        Ok(parsed) => Ok(parsed),
        Err(original) => match bare_3d_retry(text) {
            Some(tagged) => wkt::Wkt::<f64>::from_str(&tagged)
                // Report the ORIGINAL error, not the retry's: the retry is our
                // invention, and its message would confuse a user whose input
                // was simply malformed.
                .map_err(|_| GeometryParsingError::WktParsing(original.to_string())),
            None => Err(GeometryParsingError::WktParsing(original.to_string())),
        },
    }
}

/// Computes a bare-3D-tolerant rewrite of `text`, dispatching on shape: a
/// `GEOMETRYCOLLECTION` needs each member individually `Z`-tagged (see
/// `with_z_tag_on_geometrycollection` for why), everything else uses the
/// single outer-keyword tag from `with_z_tag`.
fn bare_3d_retry(text: &str) -> Option<String> {
    let open = text.find('(')?;
    let keyword = text[..open].trim_end();
    if keyword == "GEOMETRYCOLLECTION" {
        with_z_tag_on_geometrycollection(text)
    } else {
        with_z_tag(text)
    }
}

/// `text` with a ` Z` tag inserted after the type keyword, when its first
/// coordinate group holds exactly three ordinates. `None` when the shape does
/// not look like bare 3D, in which case there is nothing to retry.
///
/// Not meaningful for a `GEOMETRYCOLLECTION`: its "first coordinate group" is
/// actually a nested member's type keyword plus parens, not a coordinate, so
/// this must not be called on one directly (`bare_3d_retry` routes those to
/// `with_z_tag_on_geometrycollection` instead).
///
/// Requires the text to end exactly at its own closing paren: `wkt` does not
/// require EOF after a complete geometry, so without this, a cell like
/// `POINT(1 2 3) EXTRA` or `POINT(1 2 3),POINT(4 5 6)` -- both rejected by
/// `wkt` as written -- would rewrite to `POINT Z(1 2 3) EXTRA` /
/// `POINT Z(1 2 3),POINT(4 5 6)`, both of which parse, silently dropping the
/// trailing text (or a second geometry) rather than erroring on what is
/// genuinely malformed input.
fn with_z_tag(text: &str) -> Option<String> {
    let open = text.find('(')?;
    let keyword = text[..open].trim_end();
    // An already-tagged or EMPTY input is not bare 3D.
    if keyword.is_empty() || keyword.split_whitespace().count() > 1 {
        return None;
    }
    if !balanced_to_end(&text[open..]) {
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

/// Whether `s` -- which starts with its own opening paren -- is a single
/// balanced parenthesized group that runs to the end of the string, with
/// nothing after its closing paren. WKT has no quoted strings, so a plain
/// depth counter is enough.
fn balanced_to_end(s: &str) -> bool {
    let mut depth = 0i32;
    for (i, ch) in s.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return i == s.len() - 1;
                }
            }
            _ => {}
        }
    }
    false
}

/// `text` with each bare-3D member of a `GEOMETRYCOLLECTION` individually
/// `Z`-tagged. Unlike `MULTI*`, a `GEOMETRYCOLLECTION`'s outer tag alone does
/// not cover its members: verified directly against the `wkt` crate,
/// `GEOMETRYCOLLECTION Z(POINT(0 0 0))` is rejected ("Missing closing
/// parenthesis for type") while `GEOMETRYCOLLECTION(POINT Z(0 0 0))` parses.
/// So this reuses `with_z_tag` (via `bare_3d_retry`, so a member that is
/// itself a bare-3D `GEOMETRYCOLLECTION` recurses correctly) per member
/// rather than tagging the outer keyword.
///
/// Members are split on commas at paren depth zero: WKT has no quoted
/// strings, so a plain depth counter tells a member-separating comma apart
/// from one inside a member's own coordinate list. `None` when `text` is not
/// a `GEOMETRYCOLLECTION(...)`, or when no member needed tagging.
///
/// If the rewrite is wrong for any reason (a malformed member, an unbalanced
/// paren), the caller's follow-up parse simply fails and the ORIGINAL error
/// is reported — this never fabricates a geometry from a bad rewrite.
fn with_z_tag_on_geometrycollection(text: &str) -> Option<String> {
    let open = text.find('(')?;
    let keyword = text[..open].trim_end();
    if keyword != "GEOMETRYCOLLECTION" || !balanced_to_end(&text[open..]) {
        return None;
    }
    let inner = &text[open + 1..text.len() - 1];
    let mut changed = false;
    let members: Vec<String> = split_top_level(inner)
        .into_iter()
        .map(|member| {
            let trimmed = member.trim();
            match bare_3d_retry(trimmed) {
                Some(tagged) => {
                    changed = true;
                    tagged
                }
                None => trimmed.to_string(),
            }
        })
        .collect();
    changed.then(|| format!("{keyword}({})", members.join(", ")))
}

/// Splits `s` on commas that sit at paren depth zero: the top-level
/// separators between a `GEOMETRYCOLLECTION`'s members, as opposed to a comma
/// inside one member's own coordinate list (e.g. the ring separator in a
/// member `POLYGON`).
fn split_top_level(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    for (i, ch) in s.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(&s[start..]);
    parts
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

    /// Mirrors `a_reversed_axis_crs_reads_transposed_and_a_normal_one_does_not`
    /// below, but for `GeometryMode::Coordinates`: the x/y columns are read in
    /// declared (lon, lat) order regardless of CRS, so the frame's axis order
    /// must still be applied when the values are stored. EPSG:4326 swaps
    /// (latitude-first); EPSG:3857 does not.
    #[test]
    fn a_coordinates_row_in_a_swapping_crs_stores_its_ordinates_exchanged() {
        let g = parse_geometry(
            &row(&[("lon", "10.0"), ("lat", "20.0")]),
            &coords_config(None, Some(4326)),
        )
        .unwrap();
        match g {
            Geometry::Euclidean2D(Euclidean2DGeometry::Point(p)) => {
                assert_eq!(p.position(), [20.0, 10.0], "EPSG:4326 must swap");
            }
            other => panic!("expected a 2D point, got {other:?}"),
        }
    }

    #[test]
    fn a_coordinates_row_in_a_non_swapping_crs_does_not_exchange_ordinates() {
        let g = parse_geometry(
            &row(&[("lon", "10.0"), ("lat", "20.0")]),
            &coords_config(None, Some(3857)),
        )
        .unwrap();
        match g {
            Geometry::Euclidean2D(Euclidean2DGeometry::Point(p)) => {
                assert_eq!(p.position(), [10.0, 20.0], "EPSG:3857 must not swap");
            }
            other => panic!("expected a 2D point, got {other:?}"),
        }
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

    /// A prior version of `with_z_tag` rewrote `POINT(1 2 3) EXTRA` to
    /// `POINT Z(1 2 3) EXTRA`, which `wkt` parses -- silently dropping the
    /// trailing text and turning genuinely malformed input into a successful
    /// (and wrong) 2-point-looking parse. Same for a second geometry crammed
    /// into one cell after a comma. `balanced_to_end` closes this: all three
    /// are rejected by `wkt` as written, and must stay rejected after the
    /// retry too.
    #[test]
    fn trailing_content_after_the_geometry_is_not_silently_dropped() {
        for text in [
            "POINT(1 2 3) EXTRA",
            "POINT(1 2 3)trailing",
            "POINT(1 2 3),POINT(4 5 6)",
        ] {
            assert!(parse(text).is_err(), "{text} must still error");
        }
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

    /// The swap exchanges only the first two ordinates. A reversed-axis CRS on
    /// a 3D point must swap X and Y but leave Z untouched — asserting the full
    /// position, not just Z, so a regression that stops swapping X/Y would
    /// still be caught even though it would not move Z.
    #[test]
    fn a_reversed_axis_crs_swaps_xy_but_never_z() {
        let g = parse_geometry(
            &row(&[("geom", "POINT Z(10 20 30)")]),
            &wkt_config(Some(4326)),
        )
        .unwrap();
        match g {
            Geometry::Euclidean3D(Euclidean3DGeometry::Point(p)) => {
                assert_eq!(p.position(), [20.0, 10.0, 30.0]);
            }
            other => panic!("expected a 3D point, got {other:?}"),
        }
    }

    /// The bare 3D form our own writer emits for a reversed-axis CRS: it
    /// routes through Task 4's tolerant `Z`-tag retry as well as the swap, so
    /// it exercises both together.
    #[test]
    fn a_bare_3d_point_under_a_swapping_crs_swaps_xy_but_never_z() {
        let g = parse_geometry(
            &row(&[("geom", "POINT(10 20 30)")]),
            &wkt_config(Some(4326)),
        )
        .unwrap();
        match g {
            Geometry::Euclidean3D(Euclidean3DGeometry::Point(p)) => {
                assert_eq!(p.position(), [20.0, 10.0, 30.0]);
            }
            other => panic!("expected a 3D point, got {other:?}"),
        }
    }
    // Every assertion above the following block is on a `POINT`. `line_geometry`
    // and `polygon_geometry` each call `xy(c, swap)` at multiple independent
    // sites (2D and 3D, exterior and interior ring), and none of those sites
    // were exercised under a swapping CRS: a review mutated all six call sites
    // to ignore `swap` and the full suite still passed. The tests below close
    // that gap, one per dimension/shape, plus a collection (which routes
    // through the same functions via `collect`'s fold rather than being called
    // directly).

    /// Guards `line_geometry`'s 2D `xy(c, swap)` call site.
    #[test]
    fn a_reversed_axis_crs_swaps_a_2d_linestrings_coordinates() {
        let g = parse_geometry(
            &row(&[("geom", "LINESTRING(10 20, 30 40)")]),
            &wkt_config(Some(4326)),
        )
        .unwrap();
        match g {
            Geometry::Euclidean2D(Euclidean2DGeometry::LineString(l)) => {
                assert_eq!(l.coords(), &[[20.0, 10.0], [40.0, 30.0]]);
            }
            other => panic!("expected a 2D linestring, got {other:?}"),
        }
    }

    /// Guards `line_geometry`'s 3D `xy(c, swap)` call site. Z must survive
    /// untouched alongside the swapped X/Y.
    #[test]
    fn a_reversed_axis_crs_swaps_a_3d_linestrings_coordinates() {
        let g = parse_geometry(
            &row(&[("geom", "LINESTRING Z(10 20 1, 30 40 2)")]),
            &wkt_config(Some(4326)),
        )
        .unwrap();
        match g {
            Geometry::Euclidean3D(Euclidean3DGeometry::LineString(l)) => {
                assert_eq!(l.coords(), &[[20.0, 10.0, 1.0], [40.0, 30.0, 2.0]]);
            }
            other => panic!("expected a 3D linestring, got {other:?}"),
        }
    }

    /// Guards `polygon_geometry`'s 2D exterior and interior `xy(c, swap)` call
    /// sites together: a holed polygon exercises both.
    #[test]
    fn a_reversed_axis_crs_swaps_a_2d_polygons_coordinates_including_its_hole() {
        let text = "POLYGON((0 10, 4 10, 4 14, 0 14, 0 10), (1 11, 2 11, 2 12, 1 11))";
        let g = parse_geometry(&row(&[("geom", text)]), &wkt_config(Some(4326))).unwrap();
        match g {
            Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(p)) => {
                assert_eq!(
                    p.exterior(),
                    &[
                        [10.0, 0.0],
                        [10.0, 4.0],
                        [14.0, 4.0],
                        [14.0, 0.0],
                        [10.0, 0.0],
                    ]
                );
                let interior = p.interiors().next().expect("the hole must survive");
                assert_eq!(
                    interior,
                    &[[11.0, 1.0], [11.0, 2.0], [12.0, 2.0], [11.0, 1.0]]
                );
            }
            other => panic!("expected a 2D polygon, got {other:?}"),
        }
    }

    /// Guards `polygon_geometry`'s 3D exterior and interior `xy(c, swap)` call
    /// sites. Z must survive untouched.
    #[test]
    fn a_reversed_axis_crs_swaps_a_3d_polygons_coordinates_including_its_hole() {
        let text = "POLYGON Z((0 10 5, 4 10 5, 4 14 5, 0 14 5, 0 10 5), \
                     (1 11 5, 2 11 5, 2 12 5, 1 11 5))";
        let g = parse_geometry(&row(&[("geom", text)]), &wkt_config(Some(4326))).unwrap();
        match g {
            Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(p)) => {
                assert_eq!(
                    p.exterior(),
                    &[
                        [10.0, 0.0, 5.0],
                        [10.0, 4.0, 5.0],
                        [14.0, 4.0, 5.0],
                        [14.0, 0.0, 5.0],
                        [10.0, 0.0, 5.0],
                    ]
                );
                let interior = p.interiors().next().expect("the hole must survive");
                assert_eq!(
                    interior,
                    &[
                        [11.0, 1.0, 5.0],
                        [11.0, 2.0, 5.0],
                        [12.0, 2.0, 5.0],
                        [11.0, 1.0, 5.0],
                    ]
                );
            }
            other => panic!("expected a 3D polygon, got {other:?}"),
        }
    }

    /// A collection routes members through `line_geometry`/`polygon_geometry`
    /// via `collect`'s fold rather than being called directly, so it is a
    /// distinct path worth its own coverage under a swapping CRS.
    #[test]
    fn a_reversed_axis_crs_swaps_a_multipolygons_member_coordinates() {
        let text = "MULTIPOLYGON(((0 10, 4 10, 4 14, 0 10)))";
        let g = parse_geometry(&row(&[("geom", text)]), &wkt_config(Some(4326))).unwrap();
        match g {
            Geometry::Euclidean2D(Euclidean2DGeometry::Collection(c)) => {
                assert_eq!(c.members().len(), 1);
                match &c.members()[0] {
                    Euclidean2DGeometry::Polygon(p) => {
                        assert_eq!(
                            p.exterior(),
                            &[[10.0, 0.0], [10.0, 4.0], [14.0, 4.0], [10.0, 0.0]]
                        );
                    }
                    other => panic!("expected a polygon member, got {other:?}"),
                }
            }
            other => panic!("expected a 2D collection, got {other:?}"),
        }
    }

    /// The gap the review found: our own writer emits bare-3D members inside a
    /// `GEOMETRYCOLLECTION` (e.g. `GEOMETRYCOLLECTION(POINT(0 0 0), ...)`),
    /// and unlike `MULTI*`, tagging only the outer keyword does not parse --
    /// verified directly: `GEOMETRYCOLLECTION Z(POINT(0 0 0))` is rejected by
    /// the `wkt` crate ("Missing closing parenthesis for type"). Each member
    /// needs its own tag, which `with_z_tag_on_geometrycollection` supplies.
    #[test]
    fn a_bare_3d_geometrycollection_from_our_own_writer_round_trips() {
        let text = "GEOMETRYCOLLECTION(POINT(1 2 3), LINESTRING(0 0 0, 1 1 1))";
        match parse(text).unwrap() {
            Geometry::GeometryCollection(c) => {
                assert_eq!(c.members().len(), 2);
                assert!(
                    matches!(
                        c.members()[0],
                        Geometry::Euclidean3D(Euclidean3DGeometry::Point(_))
                    ),
                    "the point member must keep its Z"
                );
                assert!(
                    matches!(
                        c.members()[1],
                        Geometry::Euclidean3D(Euclidean3DGeometry::LineString(_))
                    ),
                    "the linestring member must keep its Z"
                );
            }
            other => panic!("expected a GeometryCollection, got {other:?}"),
        }
    }

    /// A holed polygon member makes sure the top-level comma splitter isn't
    /// fooled by the ring-separating comma inside a member's own coordinate
    /// list.
    #[test]
    fn a_bare_3d_geometrycollection_with_a_holed_polygon_member_round_trips() {
        let text = "GEOMETRYCOLLECTION(POINT(9 9 9), \
                     POLYGON((0 0 0, 4 0 0, 4 4 0, 0 4 0, 0 0 0), (1 1 0, 2 1 0, 2 2 0, 1 1 0)))";
        match parse(text).unwrap() {
            Geometry::GeometryCollection(c) => {
                assert_eq!(c.members().len(), 2);
                match &c.members()[1] {
                    Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(p)) => {
                        assert_eq!(p.interiors().count(), 1, "the hole must survive");
                    }
                    other => panic!("expected a 3D polygon member, got {other:?}"),
                }
            }
            other => panic!("expected a GeometryCollection, got {other:?}"),
        }
    }

    /// Mixed dimensions within one `GEOMETRYCOLLECTION`: only the members that
    /// actually look like bare 3D get tagged, so a 2D member is left alone
    /// rather than being force-tagged into a spurious Z.
    #[test]
    fn a_geometrycollection_mixing_a_bare_3d_and_a_2d_member_round_trips() {
        let text = "GEOMETRYCOLLECTION(POINT(1 2 3), POINT(4 5))";
        match parse(text).unwrap() {
            Geometry::GeometryCollection(c) => {
                assert_eq!(c.members().len(), 2);
                assert!(matches!(
                    c.members()[0],
                    Geometry::Euclidean3D(Euclidean3DGeometry::Point(_))
                ));
                assert!(matches!(
                    c.members()[1],
                    Geometry::Euclidean2D(Euclidean2DGeometry::Point(_))
                ));
            }
            other => panic!("expected a GeometryCollection, got {other:?}"),
        }
    }

    /// `with_z_tag_on_geometrycollection` in isolation, pinning both the
    /// top-level comma split and that an already-tagged member passes through
    /// unchanged.
    #[test]
    fn with_z_tag_on_geometrycollection_tags_only_the_members_that_need_it() {
        assert_eq!(
            with_z_tag_on_geometrycollection(
                "GEOMETRYCOLLECTION(POINT(1 2 3), POINT Z(4 5 6), POINT(7 8))"
            ),
            Some("GEOMETRYCOLLECTION(POINT Z(1 2 3), POINT Z(4 5 6), POINT(7 8))".to_string())
        );
        // Nothing needs tagging: no rewrite.
        assert_eq!(
            with_z_tag_on_geometrycollection("GEOMETRYCOLLECTION(POINT(1 2), POINT(3 4))"),
            None
        );
        // Not a GEOMETRYCOLLECTION at all.
        assert_eq!(
            with_z_tag_on_geometrycollection("MULTIPOINT((0 0 0))"),
            None
        );
    }

    /// The sibling of `trailing_content_after_the_geometry_is_not_silently_dropped`,
    /// but for the `GEOMETRYCOLLECTION` path: a re-review found
    /// `with_z_tag_on_geometrycollection` guarded with `text.ends_with(')')`
    /// rather than `balanced_to_end`, so trailing content after the
    /// collection's own closing paren was silently dropped once any member
    /// needed a bare-3D rewrite. Before the fix this parsed (dropping
    /// `EXTRA)`); it must error instead, matching how `with_z_tag` already
    /// treats the same shape of malformed input.
    #[test]
    fn trailing_content_after_a_geometrycollection_is_not_silently_dropped() {
        let text = "GEOMETRYCOLLECTION(POINT(1 2 3), POINT(4 5))EXTRA)";
        assert!(parse(text).is_err(), "{text} must still error");
    }

    /// `balanced_to_end` is stricter than the `ends_with(')')` it replaces, so
    /// the risk of tightening the `GEOMETRYCOLLECTION` guard is over-rejection
    /// of legitimate input that happens to end in more than one closing
    /// paren. Re-verify every such shape the type supports still round-trips.
    #[test]
    fn balanced_to_end_does_not_over_reject_legitimate_geometrycollections() {
        // The original sibling-fix case: a bare-3D GEOMETRYCOLLECTION.
        assert!(parse("GEOMETRYCOLLECTION(POINT(1 2 3), LINESTRING(0 0 0, 1 1 1))").is_ok());
        // A GEOMETRYCOLLECTION nested inside a GEOMETRYCOLLECTION.
        assert!(parse("GEOMETRYCOLLECTION(GEOMETRYCOLLECTION(POINT(1 2 3)))").is_ok());
        // A member with interior rings (extra closing parens before the
        // collection's own).
        assert!(parse(
            "GEOMETRYCOLLECTION(MULTIPOLYGON(((0 0 0, 4 0 0, 4 4 0, 0 4 0, 0 0 0), \
             (1 1 0, 2 1 0, 2 2 0, 1 1 0))))"
        )
        .is_ok());
        // EMPTY has no parens at all to balance.
        assert!(parse("GEOMETRYCOLLECTION EMPTY").is_ok());
        // A single member and no comma.
        assert!(parse("GEOMETRYCOLLECTION(POINT(1 2 3))").is_ok());
    }
}
