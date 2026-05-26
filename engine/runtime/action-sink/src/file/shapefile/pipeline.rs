use std::io::BufWriter;

use itertools::Itertools;
use rayon::iter::{ParallelBridge, ParallelIterator};
use reearth_flow_runtime::executor_operation::Context;
use reearth_flow_types::{AttributeValue, Feature};

use super::{
    conversion::{attributes_to_record, feature_to_shape, make_table_builder},
    crs::{self, ProjectionRepository},
    null_shape,
};

pub(super) fn pipeline(
    ctx: &Context,
    output: &crate::SinkOutput,
    key: &AttributeValue,
    upstream: &[Feature],
) -> crate::errors::Result<()> {
    let (sender, receiver) = std::sync::mpsc::sync_channel(1000);
    let feature = upstream
        .first()
        .ok_or(crate::errors::SinkError::ShapefileWriter(
            "No feature".to_string(),
        ))?;
    let Some(epsg) = feature.geometry.epsg else {
        return Err(crate::errors::SinkError::ShapefileWriter(
            "No EPSG code".to_string(),
        ));
    };
    std::fs::create_dir_all(output.uri().as_path())
        .map_err(crate::errors::SinkError::ShapefileWriterIo)?;

    let (table_builder, fields_default) = make_table_builder(&feature.attributes)?;

    let (ra, rb) = rayon::join(
        || {
            // Convert CityObjects to Shapefile objects
            upstream
                .iter()
                .par_bridge()
                .try_for_each_with(sender, |sender, feature| {
                    let shape = feature_to_shape(feature)?;
                    if sender.send((shape, feature.attributes.clone())).is_err() {
                        return Err(crate::errors::SinkError::ShapefileWriter(
                            "Failed to send data".to_string(),
                        ));
                    };
                    Ok(())
                })
        },
        || {
            // Write Shapefile to a file

            // Attribute fields for the features
            // FieldName byte representation cannot exceed 11 bytes
            let shapes = receiver.into_iter().collect_vec();

            // Create all the files needed for the shapefile to be complete (.shp, .shx, .dbf)
            let key_stem = key.to_string().replace('/', "-");
            let shp_out = output.join(&format!("{key_stem}.shp")).map_err(|err| {
                crate::errors::SinkError::ShapefileWriter(format!("Failed to join path: {err}"))
            })?;
            let shp_path = shp_out.uri();
            let feature_count = shapes.len();
            let has_no_geometry = shapes
                .iter()
                .all(|(shape, _)| matches!(shape, shapefile::Shape::NullShape));

            // NOTE: Need to be scoped to drop the writer before removing .shp/.shx
            {
                let mut writer = shapefile::Writer::from_path(shp_path.as_path(), table_builder)
                    .map_err(|err| match err {
                        shapefile::Error::IoError(io_err) => {
                            crate::errors::SinkError::ShapefileWriterIo(io_err)
                        }
                        _ => crate::errors::SinkError::ShapefileWriter(err.to_string()),
                    })?;

                // Write each feature
                for (shape, attributes) in shapes {
                    let record = attributes_to_record(&attributes, &fields_default);

                    match shape {
                        shapefile::Shape::PolygonZ(polygon) => {
                            writer
                                .write_shape_and_record(&polygon, &record)
                                .map_err(|err| match err {
                                    shapefile::Error::IoError(io_err) => {
                                        crate::errors::SinkError::ShapefileWriterIo(io_err)
                                    }
                                    _ => crate::errors::SinkError::ShapefileWriter(err.to_string()),
                                })?;
                        }
                        shapefile::Shape::NullShape if has_no_geometry => {
                            // Write dummy data once because shapefile-rs cannot write NullShape file
                            let point = shapefile::Point::default();
                            writer.write_shape_and_record(&point, &record).map_err(
                                |err| match err {
                                    shapefile::Error::IoError(io_err) => {
                                        crate::errors::SinkError::ShapefileWriterIo(io_err)
                                    }
                                    _ => crate::errors::SinkError::ShapefileWriter(err.to_string()),
                                },
                            )?;
                        }
                        _ => {}
                    }
                }
            }

            let shx_out = output
                .join(&format!("{key_stem}.shx"))
                .map_err(|e| crate::errors::SinkError::ShapefileWriter(e.to_string()))?;

            if has_no_geometry {
                // Remove the files written by shapefile::Writer and replace with null-shape bytes
                let _ = std::fs::remove_file(shp_path.as_path());
                let _ = std::fs::remove_file(shx_out.uri().as_path());
                let mut buffer = Vec::new();
                null_shape::write_shp(BufWriter::new(&mut buffer), feature_count)
                    .map_err(|e| crate::errors::SinkError::ShapefileWriter(e.to_string()))?;
                shp_out
                    .write(bytes::Bytes::from(buffer))
                    .map_err(|e| crate::errors::SinkError::ShapefileWriter(e.to_string()))?;

                let mut buffer = Vec::new();
                null_shape::write_shx(BufWriter::new(&mut buffer), feature_count)
                    .map_err(|e| crate::errors::SinkError::ShapefileWriter(e.to_string()))?;
                shx_out
                    .write(bytes::Bytes::from(buffer))
                    .map_err(|e| crate::errors::SinkError::ShapefileWriter(e.to_string()))?;
            } else {
                // write .prj file if this type has geometry
                let repo = ProjectionRepository::new();
                let prj_out = output
                    .join(&format!("{key_stem}.prj"))
                    .map_err(|e| crate::errors::SinkError::ShapefileWriter(e.to_string()))?;
                let mut buffer = Vec::new();
                crs::write_prj(BufWriter::new(&mut buffer), &repo, epsg)
                    .map_err(|e| crate::errors::SinkError::ShapefileWriter(e.to_string()))?;
                prj_out
                    .write(bytes::Bytes::from(buffer))
                    .map_err(|e| crate::errors::SinkError::ShapefileWriter(e.to_string()))?;
            }
            Ok::<(), crate::errors::SinkError>(())
        },
    );

    match ra {
        Ok(_) => {}
        Err(err) => {
            ctx.event_hub.error_log(
                None,
                format!("Failed to write shapefile with: {:?}", err.to_string()),
            );
        }
    }
    match rb {
        Ok(_) => {}
        Err(err) => {
            ctx.event_hub.error_log(
                None,
                format!("Failed to write shapefile with: {:?}", err.to_string()),
            );
        }
    }
    Ok(())
}
