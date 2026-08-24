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

/// Why a row was routed to `rejected` instead of `features`. Distinct error
/// codes, not just distinct messages: diagnostic messages are discarded in
/// production, so the code is the only signal a user gets, and "bad geometry"
/// and "wrong field count" point at very different fixes.
#[cfg(feature = "new-geometry")]
enum RowFailure {
    Shape,
    Geometry(String),
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
    // Counted locally and reported once after the loop -- see the comment
    // above the `report_drop` calls below for why per-row reporting would be
    // wrong here.
    let mut shape_rejected = 0usize;
    let mut geometry_rejected = 0usize;

    for rd in rdr.deserialize() {
        // A record the csv crate itself cannot make sense of (invalid UTF-8
        // in a field, an IO failure) still fails the whole read: that is not
        // "one bad row", it means the byte stream itself is unreadable. Note
        // this is narrower than it might sound -- with `flexible(true)`
        // above, a merely unusual record like an unterminated quoted field is
        // *not* one of these; the csv crate absorbs it as a valid (if odd)
        // field running to EOF rather than erroring.
        let record: Vec<String> =
            rd.map_err(|e| crate::errors::SourceError::CsvFileReader(format!("{e:?}")))?;
        data_rows += 1;

        if header_rows == 0 && header.is_empty() {
            header = auto_generate_header(record.len());
        }

        let row_map: IndexMap<String, String> = record
            .iter()
            .enumerate()
            .filter_map(|(i, value)| header.get(i).map(|h| (h.clone(), value.clone())))
            .collect();

        // `build_csv_reader` sets `flexible(true)`, so the csv crate never
        // enforces a consistent field count on its own (that check applies to
        // every deserialize target, not just fixed-shape ones, and flexible
        // mode disables it outright). A ragged row -- one whose field count
        // does not match the header -- is treated the same as any other bad
        // row: it costs its own row, not the file. The tempting alternative,
        // aborting the whole read, buys nothing: the scenario it would catch
        // (a wrong delimiter, a truncated file) makes *every* row ragged, so
        // every row rejects and the problem is still glaringly obvious. And
        // aborting would cost the 900,000-row case everything for one bad
        // line.
        //
        // When the shape is wrong the columns may be misaligned, so geometry
        // is not parsed at all -- parsing out of misaligned columns could
        // silently produce plausible-but-wrong coordinates. Shape rejection
        // takes precedence over a geometry failure.
        let shape_mismatch = record.len() != header.len();

        let (geometry, excluded_columns, failure) = if shape_mismatch {
            (Geometry::default(), Vec::new(), Some(RowFailure::Shape))
        } else {
            match &props.geometry {
                Some(config) => {
                    let excluded = super::csv_geometry::get_geometry_column_names(config);
                    match super::csv_geometry::parse_geometry(&row_map, config) {
                        Ok(geometry) => (geometry, excluded, None),
                        // Keep the geometry column(s) in the rejected row's
                        // attributes instead of excluding them: the error
                        // message may carry only a truncated copy of the
                        // offending value (see `annotate_wkt_parse_error` in
                        // `csv_geometry_next.rs`), so the full original value
                        // has to be recoverable from the row itself. This
                        // also brings geometry-failure rejections in line
                        // with shape-mismatch ones just above, which already
                        // keep every column. A row that parses successfully
                        // (the `Ok` arm) still excludes them exactly as
                        // before, so a good row's attributes never duplicate
                        // its own geometry column.
                        Err(why) => (
                            Geometry::default(),
                            Vec::new(),
                            Some(RowFailure::Geometry(why.to_string())),
                        ),
                    }
                }
                None => (Geometry::default(), Vec::new(), None),
            }
        };

        let record_len = record.len();
        let header_len = header.len();
        let attributes = row_map
            .into_iter()
            .filter(|(k, _)| !excluded_columns.contains(k))
            .map(|(k, v)| (k, AttributeValue::String(v)))
            .collect::<IndexMap<String, AttributeValue>>();

        let mut feature = Feature::from(attributes);
        feature.set_geometry(geometry);

        let port = match failure {
            None => FEATURES_PORT.clone(),
            Some(RowFailure::Shape) => {
                feature.insert(
                    GEOMETRY_ERROR_ATTRIBUTE,
                    AttributeValue::String(format!(
                        "record has {record_len} fields but the header has {header_len}"
                    )),
                );
                shape_rejected += 1;
                REJECTED_PORT.clone()
            }
            Some(RowFailure::Geometry(message)) => {
                feature.insert(GEOMETRY_ERROR_ATTRIBUTE, AttributeValue::String(message));
                geometry_rejected += 1;
                REJECTED_PORT.clone()
            }
        };

        sender
            .send((port, IngestionMessage::OperationEvent { feature }))
            .await
            .map_err(|e| crate::errors::SourceError::CsvFileReader(format!("{e:?}")))?;
    }

    // Sources get no diagnostics aggregator: `NodeContext::new` (what
    // `source_node.rs` builds this `ctx` from) always sets `diagnostics:
    // None`, so `report_drop` always takes the `None` branch -- there is no
    // batching, no disposition resolution, and no `node_id` on the resulting
    // event, unlike a sink or processor whose `NodeDiagnosticsHandle`
    // aggregates and later summarises. Each call here is one raw diagnostic
    // event, published immediately.
    //
    // Calling it per row would mean one event per rejected row -- a
    // 900,000-row file with every row ragged would emit 900,000 events. That
    // is not what the diagnostic is for: it exists only as a backstop so a
    // disconnected `rejected` port doesn't lose rows in total silence, which
    // needs exactly one signal per run, not one per row. So rejections are
    // counted locally in the loop above and reported at most once per
    // failure kind here, after it.
    if shape_rejected > 0 {
        ctx.report_drop(ErrorCode::CsvRowShapeRejected, None, None);
    }
    if geometry_rejected > 0 {
        ctx.report_drop(ErrorCode::CsvGeometryRejected, None, None);
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

    fn wkt_geometry(column: &str) -> GeometryConfig {
        GeometryConfig {
            mode: GeometryMode::Wkt {
                column: column.to_string(),
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

    /// Geometry columns are consumed on a successful row, exactly as before.
    /// A rejected row now keeps them instead (see the next test): this used
    /// to assert exclusion "on either port", which stopped being true once a
    /// geometry-failure rejection started keeping its geometry column so the
    /// offending value is recoverable from the row itself, not just from a
    /// (possibly truncated) copy inside the error message.
    #[tokio::test]
    async fn geometry_columns_are_excluded_from_a_successful_rows_attributes() {
        let csv = "name,lon,lat\na,1.0,2.0\n";
        let sent = run(csv, &param(Some(coords_geometry()))).await;
        assert_eq!(on_port(&sent, &FEATURES_PORT).len(), 1);
        for (_, feature) in &sent {
            assert!(feature.get("lon").is_none(), "lon leaked into attributes");
            assert!(feature.get("lat").is_none(), "lat leaked into attributes");
            assert!(feature.get("name").is_some(), "name should survive");
        }
    }

    /// A geometry-failure rejection keeps its geometry column(s), unlike a
    /// successful row. This is what makes the offending value recoverable
    /// even when the error message's own copy is truncated, and it brings
    /// geometry-failure rejections in line with shape-mismatch ones, which
    /// already kept every column.
    #[tokio::test]
    async fn a_geometry_failure_rejection_keeps_its_geometry_columns() {
        let csv = "name,lon,lat\na,1.0,2.0\nb,oops,2.0\n";
        let sent = run(csv, &param(Some(coords_geometry()))).await;

        let good = on_port(&sent, &FEATURES_PORT);
        assert_eq!(good.len(), 1);
        assert!(
            good[0].get("lon").is_none(),
            "a successful row must still exclude its geometry column"
        );
        assert!(good[0].get("lat").is_none());

        let rejected = on_port(&sent, &REJECTED_PORT);
        assert_eq!(rejected.len(), 1);
        assert_eq!(
            rejected[0].get("lon").map(|v| v.to_string()),
            Some("oops".to_string()),
            "a rejected row must keep its geometry column"
        );
        assert_eq!(
            rejected[0].get("lat").map(|v| v.to_string()),
            Some("2.0".to_string())
        );
    }

    #[tokio::test]
    async fn a_row_with_no_geometry_config_goes_to_features() {
        let csv = "name,value\na,1\n";
        let sent = run(csv, &param(None)).await;
        assert_eq!(on_port(&sent, &FEATURES_PORT).len(), 1);
        assert!(on_port(&sent, &REJECTED_PORT).is_empty());
    }

    /// A ragged row -- one whose field count doesn't match the header -- is a
    /// bad row like any other: it costs its own row, not the file. The
    /// columns may be misaligned, so its error names the shape mismatch
    /// rather than attempting (and possibly silently mis-parsing) geometry.
    #[tokio::test]
    async fn a_ragged_row_is_rejected_and_the_others_survive() {
        // A row with more fields than the header.
        let csv = "name,lon,lat\na,1.0,2.0\nb,1.0,2.0,3.0,4.0\nc,3.0,4.0\n";
        let sent = run(csv, &param(Some(coords_geometry()))).await;

        assert_eq!(on_port(&sent, &FEATURES_PORT).len(), 2);
        let rejected = on_port(&sent, &REJECTED_PORT);
        assert_eq!(rejected.len(), 1);
        let error = rejected[0]
            .get(GEOMETRY_ERROR_ATTRIBUTE)
            .expect("the error attribute must be present")
            .to_string();
        assert!(error.contains('5'), "{error}"); // 5 fields sent
        assert!(error.contains('3'), "{error}"); // header has 3
    }

    /// The wrong-delimiter (or badly-configured header) case: every row is
    /// ragged. Recovery still applies per row rather than aborting -- the
    /// problem is glaringly obvious from an all-rejected result, and nothing
    /// about it justifies losing the whole file.
    #[tokio::test]
    async fn a_file_where_every_row_is_ragged_rejects_every_row_and_still_succeeds() {
        // Every data row has a different field count than the 3-column header.
        let csv = "name,lon,lat\na,1.0\nb,2.0,3.0,4.0\nc,5.0\n";
        let sent = run(csv, &param(Some(coords_geometry()))).await;

        assert!(on_port(&sent, &FEATURES_PORT).is_empty());
        assert_eq!(on_port(&sent, &REJECTED_PORT).len(), 3);
    }

    /// A WKT parse failure has to name the offending column and repeat the
    /// offending text: `WktParsing` on its own only ever carries the `wkt`
    /// crate's parser message, which names neither.
    #[tokio::test]
    async fn a_wkt_parse_failure_names_the_column_and_the_value() {
        let csv = "name,geom\nb,NOT WKT AT ALL\n";
        let sent = run(csv, &param(Some(wkt_geometry("geom")))).await;
        let rejected = on_port(&sent, &REJECTED_PORT);
        assert_eq!(rejected.len(), 1);
        let error = rejected[0]
            .get(GEOMETRY_ERROR_ATTRIBUTE)
            .expect("the error attribute must be present")
            .to_string();
        assert!(error.contains("geom"), "{error}");
        assert!(error.contains("NOT WKT AT ALL"), "{error}");
    }

    /// A long WKT value must not blow up the error message: it is capped and
    /// marked with a trailing "...". The untruncated original still has to
    /// survive, in the row's own geometry column.
    #[tokio::test]
    async fn a_long_wkt_value_is_truncated_in_the_error_but_intact_in_its_column() {
        let bad_wkt = format!("POINT({}", "1".repeat(200));
        let csv = format!("name,geom\nb,{bad_wkt}\n");
        let sent = run(&csv, &param(Some(wkt_geometry("geom")))).await;
        let rejected = on_port(&sent, &REJECTED_PORT);
        assert_eq!(rejected.len(), 1);
        let feature = rejected[0];

        let error = feature
            .get(GEOMETRY_ERROR_ATTRIBUTE)
            .expect("the error attribute must be present")
            .to_string();
        assert!(
            error.len() < bad_wkt.len(),
            "the error must truncate the value: {error}"
        );
        assert!(error.contains("..."), "{error}");

        assert_eq!(
            feature.get("geom").map(|v| v.to_string()),
            Some(bad_wkt.clone()),
            "the untruncated value must survive in its own column"
        );
    }

    /// A rejected WKT row keeps its geometry column; a successful WKT row
    /// still excludes it, exactly as the coordinates-mode case does.
    #[tokio::test]
    async fn a_rejected_wkt_row_keeps_its_geometry_column_but_a_successful_row_still_excludes_it() {
        let csv = "name,geom\na,POINT(1 2)\nb,NOT WKT AT ALL\n";
        let sent = run(csv, &param(Some(wkt_geometry("geom")))).await;

        let good = on_port(&sent, &FEATURES_PORT);
        assert_eq!(good.len(), 1);
        assert!(
            good[0].get("geom").is_none(),
            "a successful row must still exclude its geometry column"
        );

        let rejected = on_port(&sent, &REJECTED_PORT);
        assert_eq!(rejected.len(), 1);
        assert_eq!(
            rejected[0].get("geom").map(|v| v.to_string()),
            Some("NOT WKT AT ALL".to_string()),
            "a rejected row must keep its geometry column"
        );
    }

    /// Genuine csv-crate errors are not "one bad row": the crate itself
    /// cannot make sense of the byte stream, so the whole read still fails.
    /// Only a field-count mismatch is recoverable.
    ///
    /// An unclosed quote is not actually one of these: with this reader's
    /// settings (`flexible(true)`, default quoting) an unterminated quoted
    /// field is not an error -- the csv crate treats everything up to EOF as
    /// that field's content and returns it successfully. Invalid UTF-8 inside
    /// a field is a genuine, reliably-fatal case instead: `deserialize`
    /// requires `String` fields to be valid UTF-8, so a non-UTF-8 byte fails
    /// the record with a `csv::Error`, independent of `flexible`.
    #[tokio::test]
    async fn invalid_utf8_in_a_field_still_fails_the_whole_read() {
        let mut csv: Vec<u8> = b"name,lon,lat\na,1.0,2.0\nb,".to_vec();
        csv.push(0xFF); // not valid UTF-8 on its own
        csv.extend_from_slice(b",2.0\n");

        let (tx, _rx) = mpsc::channel(64);
        let content = Bytes::from(csv);
        let result = read_csv(
            Delimiter::Comma,
            &content,
            &param(Some(coords_geometry())),
            None,
            tx,
            &NodeContext::default(),
        )
        .await;
        assert!(
            result.is_err(),
            "invalid UTF-8 in a field must fail the read"
        );
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
