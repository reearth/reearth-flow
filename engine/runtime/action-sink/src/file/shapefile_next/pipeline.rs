//! Shapefile writing. Groups features by the shape they write and writes each
//! group as its own file set.

use std::collections::{BTreeMap, HashMap};
use std::io::BufWriter;
use std::sync::Arc;

use indexmap::IndexMap;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::executor_operation::Context;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::{Attribute, AttributeValue, Feature};
use shapefile::NO_DATA;

use super::conversion::{attributes_to_record, make_table_builder, write_geometry, Field};
use super::crs;
use super::null_shape;
use super::shape::{Bucket, Frames, Payload, Ring, WrittenShape};

/// The geometry of one written feature, and the attributes to write beside it.
struct Record<'a> {
    shape: WrittenShape,
    attributes: &'a IndexMap<Attribute, AttributeValue>,
}

/// Write `upstream` as shapefiles under `base_path`, named after `key`.
///
/// A `.shp` holds records of one shape type, so features are grouped by the shape
/// they write and each group becomes its own file set. A group filling one bucket
/// is named after `key` alone; one filling several distinguishes its files by
/// bucket.
pub(super) fn pipeline(
    ctx: &Context,
    sandbox_root: &Uri,
    base_path: &str,
    key: &AttributeValue,
    upstream: &[Feature],
    resolver: &Arc<StorageResolver>,
) -> crate::errors::Result<()> {
    if upstream.is_empty() {
        return Ok(());
    }

    let base_out = crate::SinkOutput::new(sandbox_root, base_path, resolver).map_err(|e| {
        crate::errors::SinkError::ShapefileWriter(format!("Failed to create base output: {e}"))
    })?;
    std::fs::create_dir_all(base_out.uri().as_path())
        .map_err(crate::errors::SinkError::ShapefileWriterIo)?;

    let mut buckets: BTreeMap<Bucket, Vec<Record<'_>>> = BTreeMap::new();
    for feature in upstream {
        let shape = write_geometry(&feature.geometry);
        buckets.entry(shape.bucket()).or_default().push(Record {
            shape,
            attributes: &feature.attributes,
        });
    }

    let key_stem = key.to_string().replace('/', "-");
    let named_by_bucket = buckets.len() > 1;

    for (bucket, records) in buckets {
        let stem = if named_by_bucket {
            format!("{key_stem}_{}", bucket.suffix())
        } else {
            key_stem.clone()
        };
        if let Err(err) = write_bucket(sandbox_root, base_path, &stem, bucket, &records, resolver) {
            ctx.event_hub.error_log(
                None,
                format!("Failed to write shapefile with: {:?}", err.to_string()),
            );
        }
    }
    Ok(())
}

/// Write one bucket's records as a `.shp`, `.shx`, `.dbf`, `.cpg` and, where one
/// CRS covers them, a `.prj`.
fn write_bucket(
    sandbox_root: &Uri,
    base_path: &str,
    stem: &str,
    bucket: Bucket,
    records: &[Record<'_>],
    resolver: &Arc<StorageResolver>,
) -> crate::errors::Result<()> {
    let output = |extension: &str| {
        crate::SinkOutput::new(
            sandbox_root,
            &format!("{base_path}/{stem}.{extension}"),
            resolver,
        )
        .map_err(|e| crate::errors::SinkError::ShapefileWriter(e.to_string()))
    };

    // A field the first feature lacks would otherwise be missing from every
    // record, so the table covers the fields all of them carry.
    let (table_builder, fields) = make_table_builder(&union_of_attributes(records))?;

    if bucket == Bucket::Null {
        return write_null_bucket(
            &output("shp")?,
            &output("shx")?,
            records,
            table_builder,
            &fields,
        );
    }

    let shp_out = output("shp")?;
    // NOTE: Need to be scoped to drop the writer before the files are read back.
    {
        let mut writer = shapefile::Writer::from_path(shp_out.uri().as_path(), table_builder)
            .map_err(to_sink_error)?;
        // Every record in a file has one shape type, and a point bucket only
        // settles on `Point` when no feature in it writes more than one position.
        let multipoint = bucket_is_multipoint(bucket, records);
        for record in records {
            let attributes = attributes_to_record(record.attributes, &fields);
            let Some(payload) = &record.shape.payload else {
                continue;
            };
            write_shape(&mut writer, bucket, multipoint, payload, &attributes)?;
        }
    }

    let mut cpg = Vec::new();
    write_cpg(BufWriter::new(&mut cpg))
        .map_err(|e| crate::errors::SinkError::ShapefileWriter(e.to_string()))?;
    output("cpg")?
        .write(bytes::Bytes::from(cpg))
        .map_err(|e| crate::errors::SinkError::ShapefileWriter(e.to_string()))?;

    // A file whose records come from more than one CRS, or from none, is left
    // without a `.prj` rather than claiming a CRS that does not cover it.
    let frames = records
        .iter()
        .fold(Frames::Nothing, |acc, r| acc.and(r.shape.frames.clone()));
    match frames.epsg() {
        Some(epsg) => {
            let mut buffer = Vec::new();
            crs::write_prj(BufWriter::new(&mut buffer), epsg)
                .map_err(|e| crate::errors::SinkError::ShapefileWriter(e.to_string()))?;
            output("prj")?
                .write(bytes::Bytes::from(buffer))
                .map_err(|e| crate::errors::SinkError::ShapefileWriter(e.to_string()))?;
        }
        None => tracing::warn!(
            "writing {stem}.shp without a .prj: no single CRS covers its coordinates"
        ),
    }
    Ok(())
}

/// The attributes to build the table from: every field any feature carries, in the
/// order first seen, taking each field's type from its first non-null value.
fn union_of_attributes(records: &[Record<'_>]) -> IndexMap<Attribute, AttributeValue> {
    let mut union: IndexMap<Attribute, AttributeValue> = IndexMap::new();
    for record in records {
        for (name, value) in record.attributes {
            match union.get(name) {
                Some(AttributeValue::Null) | None => {
                    union.insert(name.clone(), value.clone());
                }
                Some(_) => {}
            }
        }
    }
    union
}

/// Whether a point bucket writes multipoint records: one feature holding more than
/// a single position settles it for the whole file.
fn bucket_is_multipoint(bucket: Bucket, records: &[Record<'_>]) -> bool {
    if !matches!(bucket, Bucket::Point | Bucket::PointZ) {
        return false;
    }
    records.iter().any(|record| match &record.shape.payload {
        Some(Payload::Points(points)) => points.len() != 1,
        _ => false,
    })
}

fn write_shape<T: std::io::Write + std::io::Seek>(
    writer: &mut shapefile::Writer<T>,
    bucket: Bucket,
    multipoint: bool,
    payload: &Payload,
    record: &shapefile::dbase::Record,
) -> crate::errors::Result<()> {
    let elevated = bucket.elevated();
    match payload {
        Payload::Points(positions) if multipoint && elevated => {
            writer.write_shape_and_record(&shapefile::MultipointZ::new(points_z(positions)), record)
        }
        Payload::Points(positions) if multipoint => {
            writer.write_shape_and_record(&shapefile::Multipoint::new(points(positions)), record)
        }
        Payload::Points(positions) if elevated => {
            let [x, y, z] = first_position(positions);
            writer.write_shape_and_record(&shapefile::PointZ::new(x, y, z, NO_DATA), record)
        }
        Payload::Points(positions) => {
            let [x, y, _] = first_position(positions);
            writer.write_shape_and_record(&shapefile::Point::new(x, y), record)
        }
        Payload::Curve(parts) if elevated => writer.write_shape_and_record(
            &shapefile::PolylineZ::with_parts(parts.iter().map(|p| points_z(p)).collect()),
            record,
        ),
        Payload::Curve(parts) => writer.write_shape_and_record(
            &shapefile::Polyline::with_parts(parts.iter().map(|p| points(p)).collect()),
            record,
        ),
        Payload::Area(rings) if elevated => writer.write_shape_and_record(
            &shapefile::PolygonZ::with_rings(
                rings
                    .iter()
                    .map(|ring| polygon_ring(ring, points_z(&ring.coords)))
                    .collect(),
            ),
            record,
        ),
        Payload::Area(rings) => writer.write_shape_and_record(
            &shapefile::Polygon::with_rings(
                rings
                    .iter()
                    .map(|ring| polygon_ring(ring, points(&ring.coords)))
                    .collect(),
            ),
            record,
        ),
    }
    .map_err(to_sink_error)
}

/// A bucket holds at least one position per record, so an empty one cannot occur;
/// the origin stands in for it rather than dropping the record's attributes.
fn first_position(positions: &[[f64; 3]]) -> [f64; 3] {
    positions.first().copied().unwrap_or([0.0, 0.0, 0.0])
}

fn points(positions: &[[f64; 3]]) -> Vec<shapefile::Point> {
    positions
        .iter()
        .map(|[x, y, _]| shapefile::Point::new(*x, *y))
        .collect()
}

fn points_z(positions: &[[f64; 3]]) -> Vec<shapefile::PointZ> {
    positions
        .iter()
        .map(|[x, y, z]| shapefile::PointZ::new(*x, *y, *z, NO_DATA))
        .collect()
}

fn polygon_ring<P>(ring: &Ring, points: Vec<P>) -> shapefile::PolygonRing<P> {
    if ring.outer {
        shapefile::PolygonRing::Outer(points)
    } else {
        shapefile::PolygonRing::Inner(points)
    }
}

/// Write a bucket of features carrying no geometry.
///
/// `shapefile::Writer` cannot write null records, so the `.dbf` is written through
/// it with a placeholder shape per record and the `.shp` and `.shx` it produced are
/// replaced with null-shape bytes.
fn write_null_bucket(
    shp_out: &crate::SinkOutput,
    shx_out: &crate::SinkOutput,
    records: &[Record<'_>],
    table_builder: shapefile::dbase::TableWriterBuilder,
    fields: &HashMap<String, Field>,
) -> crate::errors::Result<()> {
    {
        let mut writer = shapefile::Writer::from_path(shp_out.uri().as_path(), table_builder)
            .map_err(to_sink_error)?;
        for record in records {
            let attributes = attributes_to_record(record.attributes, fields);
            writer
                .write_shape_and_record(&shapefile::Point::default(), &attributes)
                .map_err(to_sink_error)?;
        }
    }

    let _ = std::fs::remove_file(shp_out.uri().as_path());
    let _ = std::fs::remove_file(shx_out.uri().as_path());

    let mut buffer = Vec::new();
    null_shape::write_shp(BufWriter::new(&mut buffer), records.len())
        .map_err(|e| crate::errors::SinkError::ShapefileWriter(e.to_string()))?;
    shp_out
        .write(bytes::Bytes::from(buffer))
        .map_err(|e| crate::errors::SinkError::ShapefileWriter(e.to_string()))?;

    let mut buffer = Vec::new();
    null_shape::write_shx(BufWriter::new(&mut buffer), records.len())
        .map_err(|e| crate::errors::SinkError::ShapefileWriter(e.to_string()))?;
    shx_out
        .write(bytes::Bytes::from(buffer))
        .map_err(|e| crate::errors::SinkError::ShapefileWriter(e.to_string()))?;
    Ok(())
}

/// Declare the `.dbf`'s encoding, which the table is written in.
fn write_cpg(mut writer: impl std::io::Write) -> std::io::Result<()> {
    writer.write_all(b"UTF-8")?;
    writer.flush()
}

fn to_sink_error(err: shapefile::Error) -> crate::errors::SinkError {
    match err {
        shapefile::Error::IoError(io_err) => crate::errors::SinkError::ShapefileWriterIo(io_err),
        _ => crate::errors::SinkError::ShapefileWriter(err.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn record(attributes: &IndexMap<Attribute, AttributeValue>) -> Record<'_> {
        Record {
            shape: WrittenShape::none(),
            attributes,
        }
    }

    fn attributes(pairs: &[(&str, AttributeValue)]) -> IndexMap<Attribute, AttributeValue> {
        pairs
            .iter()
            .map(|(k, v)| (Attribute::new(*k), v.clone()))
            .collect()
    }

    fn points_record<'a>(
        attributes: &'a IndexMap<Attribute, AttributeValue>,
        positions: Vec<[f64; 3]>,
    ) -> Record<'a> {
        Record {
            shape: WrittenShape {
                payload: Some(Payload::Points(positions)),
                elevated: false,
                frames: Frames::Nothing,
            },
            attributes,
        }
    }

    // Taking the table from the first feature alone would leave out a field only
    // later features carry.
    #[test]
    fn the_table_covers_fields_a_later_feature_introduces() {
        let first = attributes(&[("a", AttributeValue::String("x".into()))]);
        let second = attributes(&[("b", AttributeValue::Bool(true))]);
        let union = union_of_attributes(&[record(&first), record(&second)]);
        assert_eq!(union.len(), 2);
        assert!(union.contains_key(&Attribute::new("a")));
        assert!(union.contains_key(&Attribute::new("b")));
    }

    // A field's type comes from a value that has one, a null saying nothing about it.
    #[test]
    fn a_null_first_value_takes_its_type_from_a_later_feature() {
        let first = attributes(&[("a", AttributeValue::Null)]);
        let second = attributes(&[("a", AttributeValue::Bool(true))]);
        let union = union_of_attributes(&[record(&first), record(&second)]);
        assert_eq!(
            union.get(&Attribute::new("a")),
            Some(&AttributeValue::Bool(true))
        );
    }

    // A field a feature carries as null must still reach that feature's record:
    // a record short of a column the table declares is rejected outright.
    #[test]
    fn a_null_valued_field_is_still_written_for_every_record() {
        let union = attributes(&[("a", AttributeValue::String("x".into()))]);
        let (_, fields) = make_table_builder(&union).expect("the table is expected to build");
        let feature = attributes(&[("a", AttributeValue::Null)]);
        let record = attributes_to_record(&feature, &fields);
        assert_eq!(
            record.get("a"),
            Some(&shapefile::dbase::FieldValue::Character(None))
        );
    }

    #[test]
    fn one_multi_position_feature_makes_the_whole_file_multipoint() {
        let attributes = attributes(&[]);
        let records = vec![
            points_record(&attributes, vec![[1.0, 2.0, 0.0]]),
            points_record(&attributes, vec![[1.0, 2.0, 0.0], [3.0, 4.0, 0.0]]),
        ];
        assert!(bucket_is_multipoint(Bucket::Point, &records));
    }
}
