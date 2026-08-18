//! Shapefile writing. Groups features by the shape they write and writes each
//! group as its own file set.

use std::collections::BTreeMap;
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
    /// What the feature's geometry writes to.
    shape: WrittenShape,
    /// The feature's attributes.
    attributes: &'a IndexMap<Attribute, AttributeValue>,
}

/// Write `upstream` as shapefiles under `base_path`, named after `key`.
///
/// A `.shp` holds records of one shape type, so features are grouped by the shape
/// they write and each group becomes its own file set. A group filling one bucket
/// is named after `key` alone; one filling several distinguishes its files by
/// bucket.
///
/// With `compress_output` set, each file set is gathered into its own ZIP archive
/// under that directory rather than left as loose files.
pub(super) fn pipeline(
    ctx: &Context,
    sandbox_root: &Uri,
    base_path: &str,
    compress_output: Option<&str>,
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

    if let Some(compress_output) = compress_output {
        let compress_out = crate::SinkOutput::new(sandbox_root, compress_output, resolver)
            .map_err(|e| {
                crate::errors::SinkError::ShapefileWriter(format!(
                    "Failed to create compressed output: {e}"
                ))
            })?;
        std::fs::create_dir_all(compress_out.uri().as_path())
            .map_err(crate::errors::SinkError::ShapefileWriterIo)?;
    }

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
        let written = match write_bucket(sandbox_root, base_path, &stem, bucket, &records, resolver)
        {
            Ok(written) => written,
            Err(err) => {
                ctx.event_hub.error_log(
                    None,
                    format!("Failed to write shapefile with: {:?}", err.to_string()),
                );
                continue;
            }
        };
        if let Some(compress_output) = compress_output {
            if let Err(err) = archive_file_set(
                sandbox_root,
                base_path,
                compress_output,
                &stem,
                &written,
                resolver,
            ) {
                ctx.event_hub.error_log(
                    None,
                    format!("Failed to archive shapefile with: {:?}", err.to_string()),
                );
            }
        }
    }
    Ok(())
}

/// Gather the file set `stem` names under `base_path` into
/// `{compress_output}/{stem}.zip`, and take the loose files away.
///
/// The components sit at the archive's root, named after `stem`, which is the layout
/// the Shapefile Reader takes.
fn archive_file_set(
    sandbox_root: &Uri,
    base_path: &str,
    compress_output: &str,
    stem: &str,
    extensions: &[&str],
    resolver: &Arc<StorageResolver>,
) -> crate::errors::Result<()> {
    let to_sink_error =
        |e: reearth_flow_common::Error| crate::errors::SinkError::ShapefileWriter(e.to_string());

    let archive =
        reearth_flow_common::zip::StreamingZipWriter::new(std::io::Cursor::new(Vec::new()));
    let mut gathered = Vec::new();
    for extension in extensions {
        let component = crate::SinkOutput::new(
            sandbox_root,
            &format!("{base_path}/{stem}.{extension}"),
            resolver,
        )
        .map_err(|e| crate::errors::SinkError::ShapefileWriter(e.to_string()))?;
        let bytes = std::fs::read(component.uri().as_path())
            .map_err(crate::errors::SinkError::ShapefileWriterIo)?;
        archive
            .write_entry(&format!("{stem}.{extension}"), &bytes)
            .map_err(to_sink_error)?;
        gathered.push(component.uri().clone());
    }
    let buffer = archive.finish().map_err(to_sink_error)?;

    crate::SinkOutput::new(
        sandbox_root,
        &format!("{compress_output}/{stem}.zip"),
        resolver,
    )
    .map_err(|e| crate::errors::SinkError::ShapefileWriter(e.to_string()))?
    .write(bytes::Bytes::from(buffer.into_inner()))
    .map_err(|e| crate::errors::SinkError::ShapefileWriter(e.to_string()))?;

    for uri in gathered {
        if let Err(error) = std::fs::remove_file(uri.as_path()) {
            tracing::warn!(
                %error,
                "leaving behind '{uri}', which the archive already holds"
            );
        }
    }
    Ok(())
}

/// Write one bucket's records as a `.shp`, `.shx`, `.dbf`, `.cpg` and, where one
/// CRS covers them and a `.prj` can describe it, a `.prj`. Returns the extensions
/// it wrote.
fn write_bucket(
    sandbox_root: &Uri,
    base_path: &str,
    stem: &str,
    bucket: Bucket,
    records: &[Record<'_>],
    resolver: &Arc<StorageResolver>,
) -> crate::errors::Result<Vec<&'static str>> {
    let output = |extension: &str| {
        crate::SinkOutput::new(
            sandbox_root,
            &format!("{base_path}/{stem}.{extension}"),
            resolver,
        )
        .map_err(|e| crate::errors::SinkError::ShapefileWriter(e.to_string()))
    };

    let (table_builder, fields) = make_table_builder(records.iter().map(|r| r.attributes))?;

    if bucket == Bucket::Null {
        write_null_bucket(
            &output("shp")?,
            &output("shx")?,
            records,
            table_builder,
            &fields,
        )?;
    } else {
        let shp_out = output("shp")?;
        let mut writer = shapefile::Writer::from_path(shp_out.uri().as_path(), table_builder)
            .map_err(to_sink_error)?;
        let multipoint = bucket_is_multipoint(bucket, records);
        for record in records {
            let attributes = attributes_to_record(record.attributes, &fields);
            let Some(payload) = &record.shape.payload else {
                continue;
            };
            write_shape(&mut writer, bucket, multipoint, payload, &attributes)?;
        }
    }

    let mut written = vec!["shp", "shx", "dbf", "cpg"];

    let mut cpg = Vec::new();
    write_cpg(BufWriter::new(&mut cpg))
        .map_err(|e| crate::errors::SinkError::ShapefileWriter(e.to_string()))?;
    output("cpg")?
        .write(bytes::Bytes::from(cpg))
        .map_err(|e| crate::errors::SinkError::ShapefileWriter(e.to_string()))?;

    let frames = records
        .iter()
        .fold(Frames::Nothing, |acc, r| acc.and(r.shape.frames.clone()));
    let Some(epsg) = frames.epsg() else {
        tracing::warn!("writing {stem}.shp without a .prj: no single CRS covers its coordinates");
        return Ok(written);
    };
    let mut buffer = Vec::new();
    match crs::write_prj(BufWriter::new(&mut buffer), epsg) {
        Ok(()) => {
            output("prj")?
                .write(bytes::Bytes::from(buffer))
                .map_err(|e| crate::errors::SinkError::ShapefileWriter(e.to_string()))?;
            written.push("prj");
        }
        Err(error) => tracing::warn!(
            %error,
            "writing {stem}.shp without a .prj: EPSG:{epsg} has no .prj form"
        ),
    }
    Ok(written)
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

/// Write one record: `payload` as the shape type `bucket` and `multipoint` settle
/// on, beside `record`.
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

/// Positions as shapefile points, their elevations dropped.
fn points(positions: &[[f64; 3]]) -> Vec<shapefile::Point> {
    positions
        .iter()
        .map(|[x, y, _]| shapefile::Point::new(*x, *y))
        .collect()
}

/// Positions as elevated shapefile points carrying no measure.
fn points_z(positions: &[[f64; 3]]) -> Vec<shapefile::PointZ> {
    positions
        .iter()
        .map(|[x, y, z]| shapefile::PointZ::new(*x, *y, *z, NO_DATA))
        .collect()
}

/// `points` as the outer or inner ring `ring` says it is.
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
    fields: &[Field],
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

/// A `shapefile` error as a sink error, an I/O error kept as one.
fn to_sink_error(err: shapefile::Error) -> crate::errors::SinkError {
    match err {
        shapefile::Error::IoError(io_err) => crate::errors::SinkError::ShapefileWriterIo(io_err),
        _ => crate::errors::SinkError::ShapefileWriter(err.to_string()),
    }
}
