use bytes::Bytes;
use indexmap::IndexMap;
use reearth_flow_common::csv::{
    auto_generate_header, build_csv_reader, read_merged_header, Delimiter,
};
#[cfg(feature = "new-geometry")]
use reearth_flow_diagnostics::ErrorCode;
#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::Geometry;
#[cfg(feature = "new-geometry")]
use reearth_flow_runtime::executor_operation::NodeContext;
#[cfg(feature = "new-geometry")]
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::node::{IngestionMessage, Port, FEATURES_PORT};
use reearth_flow_types::{AttributeValue, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;

use super::csv_geometry::GeometryConfig;
#[cfg(all(test, feature = "new-geometry"))]
use super::csv_geometry::GeometryMode;

/// Attribute holding the parse error on a rejected row. Fixed rather than
/// configurable: a configurable name means a new parameter, and that would move
/// the action's parameter schema. Follows the `_http_error` convention in
/// `action-processor/src/http/processor.rs`.
#[cfg(feature = "new-geometry")]
pub(crate) const GEOMETRY_ERROR_ATTRIBUTE: &str = "_csv_geometry_error";

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CsvReaderParam {
    /// # Header Row Offset
    /// Skip this many rows from the beginning to find the header row (0 = first row is header)
    pub(crate) offset: Option<usize>,
    /// # Header Row Count
    /// Number of consecutive rows that make up the header (default: 1). When 0, column names are auto-generated as "column1", "column2", and so on; when greater than 1, names are formed by joining values from each header row with "_".
    pub(crate) header_rows: Option<usize>,
    /// # Geometry Configuration
    /// Optional configuration for parsing geometry from CSV columns
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) geometry: Option<GeometryConfig>,
}

#[cfg(not(feature = "new-geometry"))]
pub(crate) async fn read_csv(
    delimiter: Delimiter,
    content: &Bytes,
    props: &CsvReaderParam,
    encoding: Option<&str>,
    sender: Sender<(Port, IngestionMessage)>,
) -> Result<(), crate::errors::SourceError> {
    let offset = props.offset.unwrap_or(0);
    let mut rdr = build_csv_reader(content.as_ref(), encoding, delimiter, offset)
        .map_err(crate::errors::SourceError::CsvFileReader)?;

    let header_rows = props.header_rows.unwrap_or(1);
    let mut header = read_merged_header(&mut rdr, header_rows)
        .map_err(crate::errors::SourceError::CsvFileReader)?;

    for rd in rdr.deserialize() {
        let record: Vec<String> =
            rd.map_err(|e| crate::errors::SourceError::CsvFileReader(format!("{e:?}")))?;

        if header_rows == 0 && header.is_empty() {
            header = auto_generate_header(record.len());
        }

        // Build a map of column name -> value for geometry parsing
        let row_map: IndexMap<String, String> = record
            .iter()
            .enumerate()
            .filter_map(|(i, value)| header.get(i).map(|h| (h.clone(), value.clone())))
            .collect();

        // Parse geometry if config is provided and get column names to exclude
        let (geometry, excluded_columns) = if let Some(geom_config) = &props.geometry {
            let geom = super::csv_geometry::parse_geometry(&row_map, geom_config)?;
            let excluded = super::csv_geometry::get_geometry_column_names(geom_config);
            (geom, excluded)
        } else {
            (reearth_flow_types::Geometry::default(), vec![])
        };

        // Convert to attributes, excluding geometry columns
        let attributes = row_map
            .into_iter()
            .filter(|(k, _)| !excluded_columns.contains(k))
            .map(|(k, v)| (k, AttributeValue::String(v)))
            .collect::<IndexMap<String, AttributeValue>>();

        // Create feature with geometry
        let mut feature = Feature::from(attributes);
        feature.geometry = std::sync::Arc::new(geometry);

        sender
            .send((
                FEATURES_PORT.clone(),
                IngestionMessage::OperationEvent { feature },
            ))
            .await
            .map_err(|e| crate::errors::SourceError::CsvFileReader(format!("{e:?}")))?;
    }
    Ok(())
}

#[cfg(feature = "new-geometry")]
pub(crate) async fn read_csv(
    delimiter: Delimiter,
    content: &Bytes,
    props: &CsvReaderParam,
    encoding: Option<&str>,
    sender: Sender<(Port, IngestionMessage)>,
    ctx: &NodeContext,
) -> Result<(), crate::errors::SourceError> {
    let offset = props.offset.unwrap_or(0);
    let mut rdr = build_csv_reader(content.as_ref(), encoding, delimiter, offset)
        .map_err(crate::errors::SourceError::CsvFileReader)?;

    let header_rows = props.header_rows.unwrap_or(1);
    let mut header = read_merged_header(&mut rdr, header_rows)
        .map_err(crate::errors::SourceError::CsvFileReader)?;

    let mut data_rows = 0usize;

    for rd in rdr.deserialize() {
        // A structurally broken record means the file is wrong, not one cell.
        // Still fatal, deliberately.
        let record: Vec<String> =
            rd.map_err(|e| crate::errors::SourceError::CsvFileReader(format!("{e:?}")))?;
        data_rows += 1;

        if header_rows == 0 && header.is_empty() {
            header = auto_generate_header(record.len());
        }

        // `build_csv_reader` sets `flexible(true)` so the csv crate never
        // enforces a consistent field count on its own (that check applies
        // regardless of the deserialize target, but flexible mode disables
        // it outright). Enforce it here instead: a record whose shape does
        // not match the header means the file itself is wrong, not one
        // cell, so it must fail the whole read rather than being routed to
        // `rejected` alongside a geometry failure.
        if record.len() != header.len() {
            return Err(crate::errors::SourceError::CsvFileReader(format!(
                "record has {} fields but the header has {}",
                record.len(),
                header.len()
            )));
        }

        let row_map: IndexMap<String, String> = record
            .iter()
            .enumerate()
            .filter_map(|(i, value)| header.get(i).map(|h| (h.clone(), value.clone())))
            .collect();

        // A geometry that will not parse costs its own row and nothing more.
        let (geometry, excluded_columns, failure) = match &props.geometry {
            Some(config) => {
                let excluded = super::csv_geometry::get_geometry_column_names(config);
                match super::csv_geometry::parse_geometry(&row_map, config) {
                    Ok(geometry) => (geometry, excluded, None),
                    Err(why) => (Geometry::default(), excluded, Some(why.to_string())),
                }
            }
            None => (Geometry::default(), Vec::new(), None),
        };

        let attributes = row_map
            .into_iter()
            .filter(|(k, _)| !excluded_columns.contains(k))
            .map(|(k, v)| (k, AttributeValue::String(v)))
            .collect::<IndexMap<String, AttributeValue>>();

        let mut feature = Feature::from(attributes);
        feature.set_geometry(geometry);

        let port = match failure {
            None => FEATURES_PORT.clone(),
            Some(message) => {
                feature.insert(GEOMETRY_ERROR_ATTRIBUTE, AttributeValue::String(message));
                // One call per rejected row, so the aggregator's count is the
                // number of rows. This is the backstop for an unwired port:
                // without it, a disconnected `rejected` port loses rows in
                // silence.
                ctx.report_drop(
                    ErrorCode::CsvGeometryRejected,
                    Some(feature.id),
                    Some(false),
                );
                REJECTED_PORT.clone()
            }
        };

        sender
            .send((port, IngestionMessage::OperationEvent { feature }))
            .await
            .map_err(|e| crate::errors::SourceError::CsvFileReader(format!("{e:?}")))?;
    }

    if data_rows == 0 {
        ctx.report_drop(ErrorCode::CsvNoDataRows, None, None);
    }
    Ok(())
}

#[cfg(all(test, feature = "new-geometry"))]
mod tests {
    use super::*;
    use reearth_flow_runtime::node::REJECTED_PORT;
    use tokio::sync::mpsc;

    fn param(geometry: Option<GeometryConfig>) -> CsvReaderParam {
        CsvReaderParam {
            offset: None,
            header_rows: None,
            geometry,
        }
    }

    fn coords_geometry() -> GeometryConfig {
        GeometryConfig {
            mode: GeometryMode::Coordinates {
                x_column: "lon".to_string(),
                y_column: "lat".to_string(),
                z_column: None,
            },
            epsg: None,
        }
    }

    /// Drain everything the reader sent, grouped by port.
    async fn run(csv: &str, props: &CsvReaderParam) -> Vec<(Port, Feature)> {
        let (tx, mut rx) = mpsc::channel(64);
        let content = Bytes::from(csv.to_string());
        read_csv(
            Delimiter::Comma,
            &content,
            props,
            None,
            tx,
            &NodeContext::default(),
        )
        .await
        .expect("the read should succeed");
        let mut out = Vec::new();
        while let Ok((port, msg)) = rx.try_recv() {
            let IngestionMessage::OperationEvent { feature } = msg;
            out.push((port, feature));
        }
        out
    }

    fn on_port<'a>(sent: &'a [(Port, Feature)], port: &Port) -> Vec<&'a Feature> {
        sent.iter()
            .filter(|(p, _)| p == port)
            .map(|(_, f)| f)
            .collect()
    }

    /// The whole point of decision 3: one bad row costs one row.
    #[tokio::test]
    async fn a_bad_row_is_rejected_and_the_others_survive() {
        let csv = "name,lon,lat\na,1.0,2.0\nb,oops,2.0\nc,3.0,4.0\n";
        let sent = run(csv, &param(Some(coords_geometry()))).await;

        assert_eq!(on_port(&sent, &FEATURES_PORT).len(), 2);
        assert_eq!(on_port(&sent, &REJECTED_PORT).len(), 1);
    }

    /// The rejected row has to be actionable: its own columns, plus why.
    #[tokio::test]
    async fn a_rejected_row_carries_its_columns_and_the_reason() {
        let csv = "name,lon,lat\nb,oops,2.0\n";
        let sent = run(csv, &param(Some(coords_geometry()))).await;
        let rejected = on_port(&sent, &REJECTED_PORT);
        assert_eq!(rejected.len(), 1);
        let feature = rejected[0];

        assert_eq!(
            feature.get("name").map(|v| v.to_string()),
            Some("b".to_string()),
            "the original columns must survive"
        );
        let error = feature
            .get(GEOMETRY_ERROR_ATTRIBUTE)
            .expect("the error attribute must be present")
            .to_string();
        assert!(error.contains("lon"), "{error}");
        assert!(error.contains("oops"), "{error}");
    }

    /// Ten bad rows surface in one pass. This is the needle-in-a-haystack case.
    #[tokio::test]
    async fn ten_bad_rows_all_surface_in_a_single_pass() {
        let mut csv = String::from("name,lon,lat\n");
        for i in 0..10 {
            csv.push_str(&format!("bad{i},oops,2.0\n"));
        }
        for i in 0..5 {
            csv.push_str(&format!("good{i},1.0,2.0\n"));
        }
        let sent = run(&csv, &param(Some(coords_geometry()))).await;

        assert_eq!(on_port(&sent, &REJECTED_PORT).len(), 10);
        assert_eq!(on_port(&sent, &FEATURES_PORT).len(), 5);
    }

    /// Geometry columns are consumed, not emitted, on either port.
    #[tokio::test]
    async fn geometry_columns_are_excluded_from_attributes_on_both_ports() {
        let csv = "name,lon,lat\na,1.0,2.0\nb,oops,2.0\n";
        let sent = run(csv, &param(Some(coords_geometry()))).await;
        for (_, feature) in &sent {
            assert!(feature.get("lon").is_none(), "lon leaked into attributes");
            assert!(feature.get("lat").is_none(), "lat leaked into attributes");
            assert!(feature.get("name").is_some(), "name should survive");
        }
    }

    #[tokio::test]
    async fn a_row_with_no_geometry_config_goes_to_features() {
        let csv = "name,value\na,1\n";
        let sent = run(csv, &param(None)).await;
        assert_eq!(on_port(&sent, &FEATURES_PORT).len(), 1);
        assert!(on_port(&sent, &REJECTED_PORT).is_empty());
    }

    /// Recovery is scoped to geometry and must not widen to everything. A
    /// structurally broken record means the file is wrong, not one cell, so it
    /// still fails the whole read. Without this test, someone "improving"
    /// resilience later could make a wrong-delimiter file silently return a
    /// partial result.
    #[tokio::test]
    async fn a_structurally_broken_record_still_fails_the_whole_read() {
        // A row with more fields than the header, which the csv crate rejects
        // when deserializing into a fixed shape.
        let csv = "name,lon,lat\na,1.0,2.0\nb,1.0,2.0,3.0,4.0\n";
        let (tx, _rx) = mpsc::channel(64);
        let content = Bytes::from(csv.to_string());
        let result = read_csv(
            Delimiter::Comma,
            &content,
            &param(Some(coords_geometry())),
            None,
            tx,
            &NodeContext::default(),
        )
        .await;
        assert!(result.is_err(), "a broken record must fail the read");
    }

    #[tokio::test]
    async fn an_empty_file_yields_nothing_and_still_succeeds() {
        assert!(run("", &param(None)).await.is_empty());
    }

    #[tokio::test]
    async fn a_header_only_file_yields_nothing_and_still_succeeds() {
        assert!(run("name,lon,lat\n", &param(Some(coords_geometry())))
            .await
            .is_empty());
    }
}
