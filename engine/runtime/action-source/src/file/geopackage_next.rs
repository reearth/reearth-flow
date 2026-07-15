//! New-geometry GeoPackage WKB parsing. Builds `reearth_flow_geometry::Geometry`
//! (per-leaf `CoordinateFrame`) from GeoPackage geometry blobs. Sibling of the
//! old-world logic in `geopackage.rs`; selected under `new-geometry`.

use std::sync::Arc;

use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use bytes::Bytes;
use indexmap::IndexMap;
use reearth_flow_geometry::{
    collection::{Collection2D, Collection3D},
    coordinate::{CoordinateFrame, EpsgCode},
    line_string::{LineString2D, LineString3D},
    point::{Point2D, Point3D},
    polygon::{Polygon2D, Polygon3D},
    Euclidean2DGeometry, Euclidean3DGeometry, Geometry, GeometryCollection,
};
use reearth_flow_sql::SqlAdapter;
use reearth_flow_types::{Attribute, AttributeValue, Feature};
use sqlx::{any::AnyRow, Column, Row};

use super::{GeoPackageReadMode, GeoPackageReaderCompiledParam};
use crate::errors::SourceError;

/// New-world entry point: open the GeoPackage bytes and read features (or metadata).
/// Mirrors the old-world `process_geopackage`; tiles remain disabled (dead in both worlds).
pub(super) async fn read(
    content: Bytes,
    params: &GeoPackageReaderCompiledParam,
) -> Result<Vec<Feature>, SourceError> {
    let temp_file = tempfile::NamedTempFile::new()
        .map_err(|e| err(format!("Failed to create temp file: {e}")))?;
    std::fs::write(temp_file.path(), content)
        .map_err(|e| err(format!("Failed to write temp file: {e}")))?;

    let db_url = format!("sqlite://{}", temp_file.path().display());
    let adapter = SqlAdapter::new(db_url, 1)
        .await
        .map_err(|e| err(format!("Failed to open GeoPackage: {e}")))?;

    super::verify_geopackage(&adapter).await?;

    match params.read_mode {
        GeoPackageReadMode::Features | GeoPackageReadMode::Tiles | GeoPackageReadMode::All => {
            read_features(&adapter, params).await
        }
        GeoPackageReadMode::MetadataOnly => super::read_metadata(&adapter, params).await,
    }
}

async fn read_features(
    adapter: &SqlAdapter,
    params: &GeoPackageReaderCompiledParam,
) -> Result<Vec<Feature>, SourceError> {
    let layers = if let Some(ref layer_name) = params.layer_name {
        vec![layer_name.clone()]
    } else {
        super::get_feature_layers(adapter).await?
    };

    let mut all_features = Vec::new();
    for layer in layers {
        all_features.extend(read_layer_features(adapter, &layer, params.force_2d).await?);
    }
    Ok(all_features)
}

async fn read_layer_features(
    adapter: &SqlAdapter,
    layer_name: &str,
    force_2d: bool,
) -> Result<Vec<Feature>, SourceError> {
    let geom_col = super::get_geometry_column(adapter, layer_name).await?;
    let srs_id = super::get_layer_srs_id(adapter, layer_name).await?;

    super::validate_table_name(layer_name)?;
    let query = format!("SELECT * FROM \"{}\"", super::escape_identifier(layer_name));
    let rows = adapter
        .fetch_many(&query)
        .await
        .map_err(|e| err(format!("Failed to query layer {layer_name}: {e}")))?;

    let mut features = Vec::new();
    for row in rows {
        let mut feature = row_to_feature(&row, &geom_col, srs_id, force_2d)?;
        feature.insert(
            "_geopackage_source",
            AttributeValue::String("features".to_string()),
        );
        feature.insert(
            "_geopackage_layer",
            AttributeValue::String(layer_name.to_string()),
        );
        features.push(feature);
    }
    Ok(features)
}

fn row_to_feature(
    row: &AnyRow,
    geom_col: &str,
    srs_id: i32,
    force_2d: bool,
) -> Result<Feature, SourceError> {
    let mut attributes = IndexMap::new();
    let mut geometry = None;

    for (idx, column) in row.columns().iter().enumerate() {
        let col_name = column.name();
        if col_name == geom_col {
            if let Ok(blob) = row.try_get::<Vec<u8>, _>(idx) {
                geometry = Some(parse_geopackage_geometry(&blob, srs_id, force_2d)?);
            }
        } else {
            let value = super::get_attribute_value(row, idx)?;
            attributes.insert(Attribute::new(col_name.to_string()), value);
        }
    }

    let mut feature = Feature::new_with_attributes(attributes);
    if let Some(geom) = geometry {
        feature.geometry = Arc::new(geom);
    }
    Ok(feature)
}

fn err(msg: impl Into<String>) -> SourceError {
    SourceError::GeoPackageReader(msg.into())
}

/// Per-leaf coordinate frame for a GeoPackage SRS id. `EpsgCode` is `u16`, so a
/// non-positive or out-of-range srs_id (e.g. an undefined `0`/`-1`, or a custom code
/// above 65535) can't be a CRS and falls back to `Euclidean` rather than truncating.
fn frame(srs_id: i32) -> CoordinateFrame {
    match u16::try_from(srs_id) {
        Ok(code) if code > 0 => CoordinateFrame::Crs(EpsgCode::new(code)),
        _ => CoordinateFrame::Euclidean,
    }
}

/// Parse a GeoPackage geometry blob (GP header + WKB body) into the new geometry.
pub(super) fn parse_geopackage_geometry(
    blob: &[u8],
    srs_id: i32,
    force_2d: bool,
) -> Result<Geometry, SourceError> {
    if blob.len() < 8 {
        return Err(err("Invalid geometry blob: too short"));
    }
    if blob[0] != 0x47 || blob[1] != 0x50 {
        return Err(err("Invalid GeoPackage geometry magic"));
    }
    let flags = blob[3];
    let envelope_len: usize = match (flags >> 1) & 0x07 {
        0 => 0,
        1 => 32,
        2 | 3 => 48,
        4 => 64,
        other => return Err(err(format!("Invalid envelope type: {other}"))),
    };
    // flags bit 0 sets the byte order of the header's srs_id (and envelope):
    // 1 = little-endian, 0 = big-endian.
    let srs_bytes = [blob[4], blob[5], blob[6], blob[7]];
    let gp_srs_id = if flags & 0x01 == 0x01 {
        i32::from_le_bytes(srs_bytes)
    } else {
        i32::from_be_bytes(srs_bytes)
    };
    let wkb_start = 8 + envelope_len;
    if blob.len() < wkb_start {
        return Err(err("Invalid geometry blob: truncated envelope"));
    }
    let wkb = &blob[wkb_start..];
    if wkb.is_empty() {
        return Err(err("Invalid geometry blob: missing WKB body"));
    }
    let final_srs_id = if gp_srs_id != 0 { gp_srs_id } else { srs_id };
    parse_wkb(wkb, final_srs_id, force_2d)
}

/// A single geometry leaf in one of the two embedding dimensions. The WKB header's
/// Z flag (and `force_2d`) decide which; a Multi* collects homogeneous leaves.
enum Leaf {
    D2(Euclidean2DGeometry),
    D3(Euclidean3DGeometry),
}

impl Leaf {
    fn into_geometry(self) -> Geometry {
        match self {
            Leaf::D2(g) => Geometry::Euclidean2D(g),
            Leaf::D3(g) => Geometry::Euclidean3D(g),
        }
    }
}

fn parse_wkb(wkb: &[u8], srs_id: i32, force_2d: bool) -> Result<Geometry, SourceError> {
    let mut cursor = std::io::Cursor::new(wkb);
    parse_wkb_geometry(&mut cursor, srs_id, force_2d)
}

/// Read one WKB geometry (its own byte-order + type header, then body) from the cursor.
fn parse_wkb_geometry(
    cursor: &mut std::io::Cursor<&[u8]>,
    srs_id: i32,
    force_2d: bool,
) -> Result<Geometry, SourceError> {
    let byte_order = read_byte_order(cursor)?;
    let wkb_type = read_u32(cursor, byte_order)?;
    let has_z = (wkb_type & 0x8000_0000) != 0;
    let has_m = (wkb_type & 0x4000_0000) != 0;
    let geom_type = wkb_type & 0x1FFF_FFFF;
    let three_d = has_z && !force_2d;

    match geom_type {
        1 => Ok(point_leaf(cursor, has_z, has_m, three_d, srs_id, byte_order)?.into_geometry()),
        2 => Ok(line_leaf(cursor, has_z, has_m, three_d, srs_id, byte_order)?.into_geometry()),
        3 => Ok(polygon_leaf(cursor, has_z, has_m, three_d, srs_id, byte_order)?.into_geometry()),
        4 => parse_multi(
            cursor, has_z, has_m, three_d, srs_id, byte_order, 1, point_leaf,
        ),
        5 => parse_multi(
            cursor, has_z, has_m, three_d, srs_id, byte_order, 2, line_leaf,
        ),
        6 => parse_multi(
            cursor,
            has_z,
            has_m,
            three_d,
            srs_id,
            byte_order,
            3,
            polygon_leaf,
        ),
        7 => parse_geometrycollection(cursor, srs_id, force_2d, byte_order),
        other => Err(err(format!("Unsupported geometry type: {other}"))),
    }
}

/// Read and validate a WKB byte-order byte: 0x00 (big-endian) or 0x01 (little-endian).
/// Anything else is a corrupt/invalid blob and fails fast.
fn read_byte_order(cursor: &mut std::io::Cursor<&[u8]>) -> Result<u8, SourceError> {
    match cursor
        .read_u8()
        .map_err(|e| err(format!("byte order: {e}")))?
    {
        b @ (0x00 | 0x01) => Ok(b),
        other => Err(err(format!("invalid WKB byte order: {other:#x}"))),
    }
}

fn read_u32(cursor: &mut std::io::Cursor<&[u8]>, byte_order: u8) -> Result<u32, SourceError> {
    if byte_order == 0x01 {
        cursor.read_u32::<LittleEndian>()
    } else {
        cursor.read_u32::<BigEndian>()
    }
    .map_err(|e| err(format!("u32: {e}")))
}

fn point_leaf(
    cursor: &mut std::io::Cursor<&[u8]>,
    has_z: bool,
    has_m: bool,
    three_d: bool,
    srs_id: i32,
    byte_order: u8,
) -> Result<Leaf, SourceError> {
    let coords = super::read_coordinates(cursor, 1, has_z, has_m, byte_order)?;
    let (x, y, z) = *coords.first().ok_or_else(|| err("empty point"))?;
    Ok(if three_d {
        Leaf::D3(Euclidean3DGeometry::Point(Point3D::new(
            frame(srs_id),
            [x, y, z.unwrap_or(0.0)],
        )))
    } else {
        Leaf::D2(Euclidean2DGeometry::Point(Point2D::new(
            frame(srs_id),
            [x, y],
        )))
    })
}

fn line_leaf(
    cursor: &mut std::io::Cursor<&[u8]>,
    has_z: bool,
    has_m: bool,
    three_d: bool,
    srs_id: i32,
    byte_order: u8,
) -> Result<Leaf, SourceError> {
    let count = read_u32(cursor, byte_order)?;
    let coords = super::read_coordinates(cursor, count, has_z, has_m, byte_order)?;
    Ok(if three_d {
        Leaf::D3(Euclidean3DGeometry::LineString(LineString3D::from_coords(
            frame(srs_id),
            coords.iter().map(|c| [c.0, c.1, c.2.unwrap_or(0.0)]),
        )))
    } else {
        Leaf::D2(Euclidean2DGeometry::LineString(LineString2D::from_coords(
            frame(srs_id),
            coords.iter().map(|c| [c.0, c.1]),
        )))
    })
}

fn polygon_leaf(
    cursor: &mut std::io::Cursor<&[u8]>,
    has_z: bool,
    has_m: bool,
    three_d: bool,
    srs_id: i32,
    byte_order: u8,
) -> Result<Leaf, SourceError> {
    let num_rings = read_u32(cursor, byte_order)?;
    let mut rings = Vec::with_capacity(num_rings as usize);
    for _ in 0..num_rings {
        let count = read_u32(cursor, byte_order)?;
        rings.push(super::read_coordinates(
            cursor, count, has_z, has_m, byte_order,
        )?);
    }
    Ok(if three_d {
        let mut it = rings.into_iter().map(|r| {
            r.iter()
                .map(|c| [c.0, c.1, c.2.unwrap_or(0.0)])
                .collect::<Vec<_>>()
        });
        let exterior = it.next().unwrap_or_default();
        Leaf::D3(Euclidean3DGeometry::Polygon(Box::new(
            Polygon3D::from_rings(frame(srs_id), exterior, it),
        )))
    } else {
        let mut it = rings
            .into_iter()
            .map(|r| r.iter().map(|c| [c.0, c.1]).collect::<Vec<_>>());
        let exterior = it.next().unwrap_or_default();
        Leaf::D2(Euclidean2DGeometry::Polygon(Box::new(
            Polygon2D::from_rings(frame(srs_id), exterior, it),
        )))
    })
}

/// A Multi* geometry: `num` embedded WKB members (each with its own byte-order + type
/// header), all the same kind, collected into a homogeneous `Collection`.
#[allow(clippy::too_many_arguments)]
fn parse_multi<F>(
    cursor: &mut std::io::Cursor<&[u8]>,
    has_z: bool,
    has_m: bool,
    three_d: bool,
    srs_id: i32,
    byte_order: u8,
    expected_type: u32,
    read_leaf: F,
) -> Result<Geometry, SourceError>
where
    F: Fn(&mut std::io::Cursor<&[u8]>, bool, bool, bool, i32, u8) -> Result<Leaf, SourceError>,
{
    let count = read_u32(cursor, byte_order)?;
    let mut members_2d = Vec::new();
    let mut members_3d = Vec::new();
    for _ in 0..count {
        let member_order = read_byte_order(cursor)?;
        // Each member must be the corresponding single type with the same Z/M as the
        // enclosing Multi*; a mismatch means a malformed blob, so fail fast rather than
        // misread the byte stream.
        let member_type = read_u32(cursor, member_order)?;
        let member_base = member_type & 0x1FFF_FFFF;
        let member_has_z = (member_type & 0x8000_0000) != 0;
        let member_has_m = (member_type & 0x4000_0000) != 0;
        if member_base != expected_type || member_has_z != has_z || member_has_m != has_m {
            return Err(err(format!(
                "Multi* member type mismatch: expected type {expected_type} (z={has_z}, m={has_m}), \
                 got type {member_base} (z={member_has_z}, m={member_has_m})"
            )));
        }
        match read_leaf(cursor, has_z, has_m, three_d, srs_id, member_order)? {
            Leaf::D2(g) => members_2d.push(g),
            Leaf::D3(g) => members_3d.push(g),
        }
    }
    Ok(if three_d {
        Geometry::Euclidean3D(Euclidean3DGeometry::Collection(Collection3D::new(
            members_3d,
        )))
    } else {
        Geometry::Euclidean2D(Euclidean2DGeometry::Collection(Collection2D::new(
            members_2d,
        )))
    })
}

/// A GeometryCollection: `num` heterogeneous members, each a full WKB geometry with its
/// own dimension, collected into the cross-dimensional `Geometry::GeometryCollection`.
fn parse_geometrycollection(
    cursor: &mut std::io::Cursor<&[u8]>,
    srs_id: i32,
    force_2d: bool,
    byte_order: u8,
) -> Result<Geometry, SourceError> {
    let count = read_u32(cursor, byte_order)?;
    let mut members = Vec::with_capacity(count as usize);
    for _ in 0..count {
        members.push(parse_wkb_geometry(cursor, srs_id, force_2d)?);
    }
    Ok(Geometry::GeometryCollection(GeometryCollection::new(
        members,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn crs(code: u16) -> CoordinateFrame {
        CoordinateFrame::Crs(EpsgCode::new(code))
    }

    /// Wrap a WKB body in a minimal GeoPackage geometry blob header
    /// (magic "GP", version 0, little-endian, no envelope).
    fn gp_blob(srs_id: i32, wkb: &[u8]) -> Vec<u8> {
        let mut b = vec![0x47, 0x50, 0x00, 0x01];
        b.extend_from_slice(&srs_id.to_le_bytes());
        b.extend_from_slice(wkb);
        b
    }

    /// GeoPackage blob with a big-endian header (flags bit 0 = 0), srs_id in BE.
    fn gp_blob_be(srs_id: i32, wkb: &[u8]) -> Vec<u8> {
        let mut b = vec![0x47, 0x50, 0x00, 0x00];
        b.extend_from_slice(&srs_id.to_be_bytes());
        b.extend_from_slice(wkb);
        b
    }

    fn wkb_point_2d(x: f64, y: f64) -> Vec<u8> {
        let mut w = vec![0x01]; // little-endian
        w.extend_from_slice(&1u32.to_le_bytes()); // WKB type 1 = Point
        w.extend_from_slice(&x.to_le_bytes());
        w.extend_from_slice(&y.to_le_bytes());
        w
    }

    fn wkb_point_3d(x: f64, y: f64, z: f64) -> Vec<u8> {
        let mut w = vec![0x01];
        w.extend_from_slice(&(1u32 | 0x8000_0000).to_le_bytes()); // Point + Z flag
        w.extend_from_slice(&x.to_le_bytes());
        w.extend_from_slice(&y.to_le_bytes());
        w.extend_from_slice(&z.to_le_bytes());
        w
    }

    fn wkb_linestring_2d(coords: &[[f64; 2]]) -> Vec<u8> {
        let mut w = vec![0x01];
        w.extend_from_slice(&2u32.to_le_bytes()); // LineString
        w.extend_from_slice(&(coords.len() as u32).to_le_bytes());
        for c in coords {
            w.extend_from_slice(&c[0].to_le_bytes());
            w.extend_from_slice(&c[1].to_le_bytes());
        }
        w
    }

    fn wkb_linestring_3d(coords: &[[f64; 3]]) -> Vec<u8> {
        let mut w = vec![0x01];
        w.extend_from_slice(&(2u32 | 0x8000_0000).to_le_bytes()); // LineString + Z
        w.extend_from_slice(&(coords.len() as u32).to_le_bytes());
        for c in coords {
            w.extend_from_slice(&c[0].to_le_bytes());
            w.extend_from_slice(&c[1].to_le_bytes());
            w.extend_from_slice(&c[2].to_le_bytes());
        }
        w
    }

    fn wkb_polygon_2d(rings: &[Vec<[f64; 2]>]) -> Vec<u8> {
        let mut w = vec![0x01];
        w.extend_from_slice(&3u32.to_le_bytes()); // Polygon
        w.extend_from_slice(&(rings.len() as u32).to_le_bytes());
        for ring in rings {
            w.extend_from_slice(&(ring.len() as u32).to_le_bytes());
            for c in ring {
                w.extend_from_slice(&c[0].to_le_bytes());
                w.extend_from_slice(&c[1].to_le_bytes());
            }
        }
        w
    }

    fn wkb_polygon_3d(rings: &[Vec<[f64; 3]>]) -> Vec<u8> {
        let mut w = vec![0x01];
        w.extend_from_slice(&(3u32 | 0x8000_0000).to_le_bytes()); // Polygon + Z
        w.extend_from_slice(&(rings.len() as u32).to_le_bytes());
        for ring in rings {
            w.extend_from_slice(&(ring.len() as u32).to_le_bytes());
            for c in ring {
                w.extend_from_slice(&c[0].to_le_bytes());
                w.extend_from_slice(&c[1].to_le_bytes());
                w.extend_from_slice(&c[2].to_le_bytes());
            }
        }
        w
    }

    #[test]
    fn point_2d_wkb_converts_to_euclidean_2d() {
        let blob = gp_blob(4326, &wkb_point_2d(139.7, 35.6));

        let geom = parse_geopackage_geometry(&blob, 4326, false).unwrap();

        assert_eq!(
            geom,
            Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(
                crs(4326),
                [139.7, 35.6],
            )))
        );
    }

    #[test]
    fn point_3d_wkb_converts_to_euclidean_3d() {
        let blob = gp_blob(4979, &wkb_point_3d(139.7, 35.6, 12.5));

        let geom = parse_geopackage_geometry(&blob, 4979, false).unwrap();

        assert_eq!(
            geom,
            Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
                crs(4979),
                [139.7, 35.6, 12.5],
            )))
        );
    }

    // force_2d drops Z: a 3D point WKB yields a 2D geometry.
    #[test]
    fn point_3d_wkb_with_force_2d_yields_2d() {
        let blob = gp_blob(4326, &wkb_point_3d(139.7, 35.6, 12.5));

        let geom = parse_geopackage_geometry(&blob, 4326, true).unwrap();

        assert_eq!(
            geom,
            Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(
                crs(4326),
                [139.7, 35.6],
            )))
        );
    }

    #[test]
    fn line_string_2d_wkb_converts() {
        let blob = gp_blob(4326, &wkb_linestring_2d(&[[0.0, 0.0], [1.0, 2.0]]));

        let geom = parse_geopackage_geometry(&blob, 4326, false).unwrap();

        assert_eq!(
            geom,
            Geometry::Euclidean2D(Euclidean2DGeometry::LineString(LineString2D::from_coords(
                crs(4326),
                [[0.0, 0.0], [1.0, 2.0]],
            )))
        );
    }

    #[test]
    fn line_string_3d_wkb_converts() {
        let blob = gp_blob(
            4979,
            &wkb_linestring_3d(&[[0.0, 0.0, 1.0], [1.0, 2.0, 3.0]]),
        );

        let geom = parse_geopackage_geometry(&blob, 4979, false).unwrap();

        assert_eq!(
            geom,
            Geometry::Euclidean3D(Euclidean3DGeometry::LineString(LineString3D::from_coords(
                crs(4979),
                [[0.0, 0.0, 1.0], [1.0, 2.0, 3.0]],
            )))
        );
    }

    #[test]
    fn polygon_2d_with_hole_wkb_converts() {
        let exterior = vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 0.0]];
        let hole = vec![[1.0, 1.0], [2.0, 1.0], [1.0, 2.0], [1.0, 1.0]];
        let blob = gp_blob(4326, &wkb_polygon_2d(&[exterior, hole]));

        let geom = parse_geopackage_geometry(&blob, 4326, false).unwrap();

        assert_eq!(
            geom,
            Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
                Polygon2D::from_rings(
                    crs(4326),
                    [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 0.0]],
                    [[[1.0, 1.0], [2.0, 1.0], [1.0, 2.0], [1.0, 1.0]]],
                )
            )))
        );
    }

    #[test]
    fn polygon_3d_wkb_converts() {
        let exterior = vec![
            [0.0, 0.0, 1.0],
            [4.0, 0.0, 1.0],
            [4.0, 4.0, 1.0],
            [0.0, 0.0, 1.0],
        ];
        let blob = gp_blob(4979, &wkb_polygon_3d(&[exterior]));

        let geom = parse_geopackage_geometry(&blob, 4979, false).unwrap();

        assert_eq!(
            geom,
            Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
                Polygon3D::from_rings(
                    crs(4979),
                    [
                        [0.0, 0.0, 1.0],
                        [4.0, 0.0, 1.0],
                        [4.0, 4.0, 1.0],
                        [0.0, 0.0, 1.0]
                    ],
                    std::iter::empty::<Vec<[f64; 3]>>(),
                )
            )))
        );
    }

    /// A Multi*/GeometryCollection WKB body: type code + count + member WKB bodies
    /// (each member is itself a full WKB geometry, so single-geometry builders reuse).
    fn wkb_multi(type_code: u32, members: &[Vec<u8>]) -> Vec<u8> {
        let mut w = vec![0x01];
        w.extend_from_slice(&type_code.to_le_bytes());
        w.extend_from_slice(&(members.len() as u32).to_le_bytes());
        for m in members {
            w.extend_from_slice(m);
        }
        w
    }

    #[test]
    fn multi_point_2d_wkb_converts_to_collection() {
        let blob = gp_blob(
            4326,
            &wkb_multi(4, &[wkb_point_2d(0.0, 0.0), wkb_point_2d(1.0, 1.0)]),
        );

        let geom = parse_geopackage_geometry(&blob, 4326, false).unwrap();

        assert_eq!(
            geom,
            Geometry::Euclidean2D(Euclidean2DGeometry::Collection(Collection2D::new([
                Euclidean2DGeometry::Point(Point2D::new(crs(4326), [0.0, 0.0])),
                Euclidean2DGeometry::Point(Point2D::new(crs(4326), [1.0, 1.0])),
            ])))
        );
    }

    #[test]
    fn multi_polygon_2d_wkb_converts_to_collection() {
        let ring = vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [0.0, 0.0]];
        let blob = gp_blob(4326, &wkb_multi(6, &[wkb_polygon_2d(&[ring])]));

        let geom = parse_geopackage_geometry(&blob, 4326, false).unwrap();

        assert_eq!(
            geom,
            Geometry::Euclidean2D(Euclidean2DGeometry::Collection(Collection2D::new([
                Euclidean2DGeometry::Polygon(Box::new(Polygon2D::from_rings(
                    crs(4326),
                    [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [0.0, 0.0]],
                    std::iter::empty::<Vec<[f64; 2]>>(),
                ))),
            ])))
        );
    }

    // An srs_id beyond u16 range can't be an EpsgCode, so it falls back to Euclidean
    // rather than silently truncating.
    #[test]
    fn srs_id_beyond_u16_range_falls_back_to_euclidean() {
        let blob = gp_blob(900913, &wkb_point_2d(1.0, 2.0));

        let geom = parse_geopackage_geometry(&blob, 900913, false).unwrap();

        assert_eq!(
            geom,
            Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(
                CoordinateFrame::Euclidean,
                [1.0, 2.0],
            )))
        );
    }

    // A big-endian GeoPackage header must have its srs_id read big-endian.
    #[test]
    fn big_endian_header_reads_srs_id() {
        let blob = gp_blob_be(4326, &wkb_point_2d(1.0, 2.0));

        // db srs_id 0 so the header's srs_id is what's used.
        let geom = parse_geopackage_geometry(&blob, 0, false).unwrap();

        assert_eq!(
            geom,
            Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(
                crs(4326),
                [1.0, 2.0]
            )))
        );
    }

    // An invalid WKB byte-order byte fails fast rather than misparsing as big-endian.
    #[test]
    fn invalid_wkb_byte_order_is_rejected() {
        let bad_wkb = vec![0x02, 1, 0, 0, 0]; // byte order 0x02 is neither BE nor LE
        let blob = gp_blob(4326, &bad_wkb);

        let e = parse_geopackage_geometry(&blob, 4326, false).unwrap_err();

        assert!(
            format!("{e}").to_lowercase().contains("byte order"),
            "expected a byte-order error, got: {e}"
        );
    }

    // A blob with a valid header but no WKB body gets a clear error, not a raw IO error.
    #[test]
    fn empty_wkb_body_is_rejected() {
        let blob = gp_blob(4326, &[]);

        let e = parse_geopackage_geometry(&blob, 4326, false).unwrap_err();

        assert!(
            format!("{e}").to_lowercase().contains("wkb body"),
            "expected a missing-WKB-body error, got: {e}"
        );
    }

    // A Multi* whose member's base type doesn't match the enclosing kind fails fast.
    #[test]
    fn multi_with_mismatched_member_type_is_rejected() {
        // MultiPoint (type 4) whose sole member is a LineString body.
        let blob = gp_blob(
            4326,
            &wkb_multi(4, &[wkb_linestring_2d(&[[0.0, 0.0], [1.0, 1.0]])]),
        );

        let e = parse_geopackage_geometry(&blob, 4326, false).unwrap_err();

        assert!(
            format!("{e}").to_lowercase().contains("member type"),
            "expected a member-type mismatch error, got: {e}"
        );
    }

    // GeometryCollection members may differ in dimension (each has its own WKB header).
    #[test]
    fn geometry_collection_wkb_converts_with_mixed_dimension_members() {
        let blob = gp_blob(
            4326,
            &wkb_multi(7, &[wkb_point_2d(0.0, 0.0), wkb_point_3d(1.0, 1.0, 2.0)]),
        );

        let geom = parse_geopackage_geometry(&blob, 4326, false).unwrap();

        assert_eq!(
            geom,
            Geometry::GeometryCollection(GeometryCollection::new([
                Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(
                    crs(4326),
                    [0.0, 0.0],
                ))),
                Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
                    crs(4326),
                    [1.0, 1.0, 2.0],
                ))),
            ]))
        );
    }
}
