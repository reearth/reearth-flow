//! The Shapefile Reader action.

use std::{collections::HashMap, sync::Arc};

use bytes::Bytes;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::NodeContext,
    node::{IngestionMessage, Port, Source, SourceFactory, FEATURES_PORT},
};
use reearth_flow_types::Feature;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc::Sender;

use super::archive;
use super::geometry::ShapeConverter;
use super::record;
use crate::{
    errors::{ShapefileError, SourceError},
    file::reader::runner::{get_content, FileReaderCommonParam, FileReaderCompiledParam},
};

/// Builds the Shapefile Reader source.
#[derive(Debug, Clone, Default)]
pub(crate) struct ShapefileReaderFactory;

impl SourceFactory for ShapefileReaderFactory {
    fn name(&self) -> &str {
        "Shapefile Reader"
    }

    fn description(&self) -> &str {
        "Reads features from a shapefile packaged in a ZIP archive. The archive must hold a \
         .shp and a .dbf sharing one name."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(ShapefileReaderParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Input"]
    }

    fn tags(&self) -> &[&'static str] {
        &["shapefile", "vector"]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
        _state: Option<Vec<u8>>,
    ) -> Result<Box<dyn Source>, BoxedError> {
        let params: ShapefileReaderParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                SourceError::ShapefileReaderFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SourceError::ShapefileReaderFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(SourceError::ShapefileReaderFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let compiled_params = ShapefileReaderCompiledParam {
            common: params.common_property.compile(&ctx).map_err(|e| {
                SourceError::ShapefileReaderFactory(format!("Failed to compile params: {e:?}"))
            })?,
            encoding: params.encoding,
            force_2d: params.force_2d,
            allow_empty_path: params.allow_empty_path,
        };
        Ok(Box::new(ShapefileReader {
            params: compiled_params,
        }))
    }
}

/// [`ShapefileReaderParam`] with its expressions compiled.
#[derive(Debug, Clone)]
struct ShapefileReaderCompiledParam {
    /// Where the archive is read from.
    common: FileReaderCompiledParam,
    /// The encoding to read the attribute table in, overriding the archive's.
    encoding: Option<String>,
    /// Whether to drop elevations.
    force_2d: bool,
    /// Whether a null dataset path yields no features rather than an error.
    allow_empty_path: bool,
}

/// The Shapefile Reader source.
#[derive(Debug, Clone)]
pub(super) struct ShapefileReader {
    /// The reader's compiled parameters.
    params: ShapefileReaderCompiledParam,
}

/// # Shapefile Reader Parameters
///
/// Sets which archive is read, the encoding of its attribute table, and whether elevations are
/// kept.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct ShapefileReaderParam {
    #[serde(flatten)]
    pub(super) common_property: FileReaderCommonParam,
    /// # Character Encoding
    /// Character encoding for attribute data in the DBF file, such as "UTF-8", "Shift-JIS", or "Windows-1252"; labels are case-insensitive. When omitted, the encoding is taken from the .cpg file if present, else from the code page the .dbf header declares, otherwise UTF-8 (UTF-16 is not supported).
    pub(super) encoding: Option<String>,
    /// # Force 2D
    /// If true, drops elevations and reads every geometry as 2D. The read fails on a multipatch,
    /// which describes a surface in space and has no 2D form.
    #[serde(default, rename = "force2D", alias = "force2d")]
    pub(super) force_2d: bool,
    /// # Allow Empty Path
    /// If true, a dataset path that is empty or null yields no features instead of failing,
    /// allowing an optional shapefile input.
    #[serde(default)]
    pub(super) allow_empty_path: bool,
}

#[async_trait::async_trait]
impl Source for ShapefileReader {
    async fn initialize(&self, _ctx: NodeContext) {}

    fn name(&self) -> &str {
        "Shapefile Reader"
    }

    async fn serialize_state(&self) -> Result<Vec<u8>, BoxedError> {
        Ok(vec![])
    }

    async fn start(
        &mut self,
        ctx: NodeContext,
        sender: Sender<(Port, IngestionMessage)>,
    ) -> Result<(), BoxedError> {
        let storage_resolver = Arc::clone(&ctx.storage_resolver);

        if self.params.allow_empty_path
            && self.params.common.dataset.is_none()
            && self.params.common.inline.is_none()
        {
            return Ok(());
        }

        let content = get_content(&self.params.common, storage_resolver).await?;
        read_shapefile(&content, &self.params, sender)
            .await
            .map_err(Into::<BoxedError>::into)
    }
}

/// Read the shapefile in the ZIP archive `content`, sending one feature per
/// record as it is read.
async fn read_shapefile(
    content: &Bytes,
    params: &ShapefileReaderCompiledParam,
    sender: Sender<(Port, IngestionMessage)>,
) -> Result<(), SourceError> {
    if !archive::is_zip(content) {
        return Err(ShapefileError::DirectBytesNotSupported.into());
    }
    let mut archive = archive::open(content, &params.encoding)?;
    let converter = ShapeConverter::new(archive.epsg, params.force_2d);

    let fields = std::mem::take(&mut archive.fields);
    for record in archive.records() {
        let (shape, record) = record.map_err(|e| {
            SourceError::shapefile_reader(format!("Failed to read shape and record: {e}"))
        })?;
        let geometry = converter.convert(shape)?;
        let feature = Feature::new_with_attributes_and_geometry(
            record::to_attributes(record, &fields),
            geometry,
        );
        sender
            .send((
                FEATURES_PORT.clone(),
                IngestionMessage::OperationEvent { feature },
            ))
            .await
            .map_err(|e| SourceError::shapefile_reader(format!("Failed to send feature: {e}")))?;
    }
    converter.report_discarded_measures();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use reearth_flow_common::datetime::{DateTime, NaiveDate};
    use reearth_flow_types::{Attribute, AttributeValue};
    use shapefile::dbase::{FieldName, FieldValue, TableWriterBuilder};
    use std::io::{Cursor, Write as _};

    /// A one-record shapefile whose table declares every field type.
    fn every_field_type_zipped() -> Bytes {
        let name = |name: &str| FieldName::try_from(name).unwrap();
        let mut shp = Vec::new();
        let mut dbf = Vec::new();
        {
            let shapes = shapefile::ShapeWriter::new(Cursor::new(&mut shp));
            let table = TableWriterBuilder::new()
                .add_numeric_field(name("count"), 10, 0)
                .add_numeric_field(name("ratio"), 10, 3)
                .add_float_field(name("fcount"), 10, 0)
                .add_integer_field(name("icount"))
                .add_logical_field(name("flag"))
                .add_date_field(name("day"))
                .add_character_field(name("text"), 10)
                .build_with_dest(Cursor::new(&mut dbf));
            let mut writer = shapefile::Writer::new(shapes, table);
            let mut record = shapefile::dbase::Record::default();
            record.insert("count".into(), FieldValue::Numeric(Some(42.0)));
            record.insert("ratio".into(), FieldValue::Numeric(Some(0.125)));
            record.insert("fcount".into(), FieldValue::Float(Some(7.0)));
            record.insert("icount".into(), FieldValue::Integer(9));
            record.insert("flag".into(), FieldValue::Logical(Some(true)));
            record.insert(
                "day".into(),
                FieldValue::Date(Some(shapefile::dbase::Date::new(17, 7, 2025))),
            );
            record.insert("text".into(), FieldValue::Character(Some("abc".into())));
            writer
                .write_shape_and_record(&shapefile::Point::new(1.0, 2.0), &record)
                .expect("the record is expected to write");
        }
        let mut buffer = Vec::new();
        {
            let mut zip = zip::ZipWriter::new(Cursor::new(&mut buffer));
            for (entry, bytes) in [("data.shp", &shp), ("data.dbf", &dbf)] {
                zip.start_file(entry, zip::write::SimpleFileOptions::default())
                    .unwrap();
                zip.write_all(bytes).unwrap();
            }
            zip.finish().unwrap();
        }
        Bytes::from(buffer)
    }

    // A field's declared type decides the attribute type its values read as,
    // whatever the values happen to be.
    #[test]
    fn each_field_type_reads_as_its_attribute_type() {
        let mut archive = archive::open(&every_field_type_zipped(), &None)
            .expect("the archive is expected to open");
        let fields = std::mem::take(&mut archive.fields);
        let (_, record) = archive
            .records()
            .next()
            .expect("one record is expected")
            .expect("the record is expected to read");
        let attributes = record::to_attributes(record, &fields);
        let get = |name: &str| attributes[&Attribute::new(name)].clone();

        assert_eq!(get("count"), AttributeValue::Number(42.into()));
        assert_eq!(
            get("ratio"),
            AttributeValue::Number(serde_json::Number::from_f64(0.125).unwrap())
        );
        assert_eq!(get("fcount"), AttributeValue::Number(7.into()));
        assert_eq!(get("icount"), AttributeValue::Number(9.into()));
        assert_eq!(get("flag"), AttributeValue::Bool(true));
        assert_eq!(
            get("day"),
            AttributeValue::DateTime(DateTime::NaiveDate(
                NaiveDate::from_ymd_opt(2025, 7, 17).unwrap()
            ))
        );
        assert_eq!(get("text"), AttributeValue::String("abc".into()));
    }
}
