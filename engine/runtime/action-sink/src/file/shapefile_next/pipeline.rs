//! Writing features as shapefile file sets, one per shape type.

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
use super::shape::{Bucket, Frames, Patch, Payload, Ring, WrittenShape};

/// One feature to write.
struct Record<'a> {
    /// What the feature's geometry writes to.
    shape: WrittenShape,
    /// The feature's attributes.
    attributes: &'a IndexMap<Attribute, AttributeValue>,
}

/// Write `upstream` as shapefiles under `base_path`: one file set per bucket,
/// named after `key` and, where there are several, the bucket. With
/// `compress_output`, each file set becomes a ZIP archive under that directory.
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

/// Move the file set `stem` under `base_path` into `{compress_output}/{stem}.zip`,
/// its components at the archive's root.
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

/// Write one bucket's records as `.shp`, `.shx`, `.dbf`, `.cpg` and, where one
/// CRS covers them, `.prj`. Returns the extensions written.
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

/// Whether a point bucket writes multipoint records: it does once any feature
/// holds other than one position.
fn bucket_is_multipoint(bucket: Bucket, records: &[Record<'_>]) -> bool {
    if !matches!(bucket, Bucket::Point | Bucket::PointZ) {
        return false;
    }
    records.iter().any(|record| match &record.shape.payload {
        Some(Payload::Points(points)) => points.len() != 1,
        _ => false,
    })
}

/// Write `payload` as the shape type `bucket` and `multipoint` settle on, beside
/// `record`.
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
        Payload::Surface(patches) => writer.write_shape_and_record(
            &shapefile::Multipatch::with_parts(patches.iter().map(patch).collect()),
            record,
        ),
    }
    .map_err(to_sink_error)
}

/// The first position, or the origin for none.
fn first_position(positions: &[[f64; 3]]) -> [f64; 3] {
    positions.first().copied().unwrap_or([0.0, 0.0, 0.0])
}

/// Positions as shapefile points without elevation.
fn points(positions: &[[f64; 3]]) -> Vec<shapefile::Point> {
    positions
        .iter()
        .map(|[x, y, _]| shapefile::Point::new(*x, *y))
        .collect()
}

/// Positions as elevated shapefile points.
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

/// A patch as a multipatch part.
fn patch(patch: &Patch) -> shapefile::Patch {
    match patch {
        Patch::Ring(ring) if ring.outer => shapefile::Patch::OuterRing(points_z(&ring.coords)),
        Patch::Ring(ring) => shapefile::Patch::InnerRing(points_z(&ring.coords)),
        Patch::Triangle(corners) => shapefile::Patch::TriangleStrip(points_z(corners)),
    }
}

/// Write a bucket of features carrying no geometry: the `.dbf` through
/// `shapefile::Writer` with placeholder shapes, then the `.shp` and `.shx`
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

/// Write the `.cpg` declaring the table's encoding.
fn write_cpg(mut writer: impl std::io::Write) -> std::io::Result<()> {
    writer.write_all(b"UTF-8")?;
    writer.flush()
}

/// A `shapefile` error as a sink error.
fn to_sink_error(err: shapefile::Error) -> crate::errors::SinkError {
    match err {
        shapefile::Error::IoError(io_err) => crate::errors::SinkError::ShapefileWriterIo(io_err),
        _ => crate::errors::SinkError::ShapefileWriter(err.to_string()),
    }
}
