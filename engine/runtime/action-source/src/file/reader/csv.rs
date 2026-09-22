#[cfg(feature = "new-geometry")]
use crate::errors::GeometryParsingError;
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
use reearth_flow_runtime::node::{IngestionMessage, Port, FEATURES_PORT};
use reearth_flow_types::{AttributeValue, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;

use super::csv_geometry::GeometryConfig;
#[cfg(all(test, feature = "new-geometry"))]
use super::csv_geometry::GeometryMode;

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
    /// Names the columns that hold geometry, either as Well-Known Text or as separate coordinate columns. Those columns are read as the feature's geometry instead of as attributes. Rows become attribute-only features when omitted.
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

/// Fails the whole read when a configured geometry column is not in `header`.
/// A missing *column* is a configuration error, not a row error -- every row
/// would fail identically, so this belongs up front as one clear error rather
/// than as a rejection repeated once per row. The message lists the available
/// columns alongside the missing one, since that is the actionable half.
#[cfg(feature = "new-geometry")]
fn validate_geometry_columns(
    header: &[String],
    config: &GeometryConfig,
) -> Result<(), crate::errors::SourceError> {
    for column in super::csv_geometry::get_geometry_column_names(config) {
        if !header.contains(&column) {
            return Err(GeometryParsingError::ConfiguredColumnMissing {
                column,
                available: header.join(", "),
            }
            .into());
        }
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

    // With a real header (`header_rows >= 1`) it is known before any row is
    // read, so a misconfigured geometry column is caught right here: the
    // whole read fails with one clear error instead of every row rejecting
    // for the same reason. With `header_rows == 0` there is no header yet --
    // `read_merged_header` returns an empty list, and column names are only
    // auto-generated from the first data row inside the loop below -- so that
    // path validates there instead, right after the header is synthesised.
    if header_rows != 0 {
        if let Some(config) = &props.geometry {
            validate_geometry_columns(&header, config)?;
        }
    }

    let mut data_rows = 0usize;

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
            if let Some(config) = &props.geometry {
                validate_geometry_columns(&header, config)?;
            }
        }

        let row_map: IndexMap<String, String> = record
            .iter()
            .enumerate()
            .filter_map(|(i, value)| header.get(i).map(|h| (h.clone(), value.clone())))
            .collect();

        // `build_csv_reader` sets `flexible(true)`, so the csv crate never
        // enforces a consistent field count on its own (that check applies to
        // every deserialize target, not just fixed-shape ones, and flexible
        // mode disables it outright). That is fine: varying field counts are
        // normal in real CSV files. A long row's extra fields have no header
        // name and are silently dropped by the `header.get(i)` lookup above.
        // A short row simply has no value for whichever columns it didn't
        // reach -- the same as any other blank cell -- which is handled below
        // as the `ColumnNotFound` arm.
        let (geometry, excluded_columns) = match &props.geometry {
            Some(config) => {
                let excluded = super::csv_geometry::get_geometry_column_names(config);
                match super::csv_geometry::parse_geometry(&row_map, config) {
                    Ok(geometry) => (geometry, excluded),
                    // A geometry value that is present but will not parse is
                    // corrupt input, and corrupt input fails the read rather
                    // than being routed somewhere. Note how narrow this is
                    // after `parse_geometry`'s own handling: a row too short
                    // to reach the column, and a blank cell, both come back
                    // as `Ok(Geometry::None)`, so neither reaches here. What
                    // is left is a cell with actual content in it that is not
                    // the geometry it claims to be.
                    //
                    // The row number is part of the message because it is the
                    // only way back to the offending line in a large file --
                    // the error names the column and the value, but a file
                    // can repeat both.
                    Err(why) => {
                        return Err(crate::errors::SourceError::CsvFileReader(format!(
                            "row {data_rows}: {why}"
                        )))
                    }
                }
            }
            None => (Geometry::default(), Vec::new()),
        };

        let attributes = row_map
            .into_iter()
            .filter(|(k, _)| !excluded_columns.contains(k))
            .map(|(k, v)| (k, AttributeValue::String(v)))
            .collect::<IndexMap<String, AttributeValue>>();

        let mut feature = Feature::from(attributes);
        feature.set_geometry(geometry);

        sender
            .send((
                FEATURES_PORT.clone(),
                IngestionMessage::OperationEvent { feature },
            ))
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
        try_run(csv, props).await.expect("the read should succeed")
    }

    /// Same, but hands back the read's own result so a test can assert that a
    /// file fails rather than that it succeeds.
    async fn try_run(
        csv: &str,
        props: &CsvReaderParam,
    ) -> Result<Vec<(Port, Feature)>, crate::errors::SourceError> {
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
        .await?;
        let mut out = Vec::new();
        while let Ok((port, msg)) = rx.try_recv() {
            let IngestionMessage::OperationEvent { feature } = msg;
            out.push((port, feature));
        }
        Ok(out)
    }

    fn on_port<'a>(sent: &'a [(Port, Feature)], port: &Port) -> Vec<&'a Feature> {
        sent.iter()
            .filter(|(p, _)| p == port)
            .map(|(_, f)| f)
            .collect()
    }

    /// A geometry value with content in it that is not the geometry it claims
    /// to be is corrupt input, and corrupt input fails the read. There is no
    /// second port for it to go to.
    #[tokio::test]
    async fn a_value_that_will_not_parse_fails_the_read() {
        let csv = "name,lon,lat\na,1.0,2.0\nb,oops,2.0\nc,3.0,4.0\n";
        let error = try_run(csv, &param(Some(coords_geometry())))
            .await
            .expect_err("an unparseable geometry value must fail the read");

        let error = error.to_string();
        assert!(error.contains("lon"), "must name the column: {error}");
        assert!(error.contains("oops"), "must quote the value: {error}");
        assert!(
            error.contains("row 2"),
            "must locate the row, since a file can repeat a column and value: {error}"
        );
    }

    /// Only the reader's own output port exists. Asserted directly rather than
    /// through the factory so that removing `rejected` from `get_output_ports`
    /// cannot silently disagree with what the reader actually sends to.
    #[tokio::test]
    async fn every_row_leaves_by_the_features_port() {
        let csv = "name,lon,lat\na,1.0,2.0\nb,,\nc,3.0,4.0\n";
        let sent = run(csv, &param(Some(coords_geometry()))).await;

        assert_eq!(sent.len(), 3);
        assert!(
            sent.iter().all(|(port, _)| *port == *FEATURES_PORT),
            "the reader has one output port"
        );
    }

    /// Geometry columns are consumed rather than duplicated into attributes.
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

    #[tokio::test]
    async fn a_row_with_no_geometry_config_goes_to_features() {
        let csv = "name,value\na,1\n";
        let sent = run(csv, &param(None)).await;
        assert_eq!(on_port(&sent, &FEATURES_PORT).len(), 1);
        assert!(on_port(&sent, &REJECTED_PORT).is_empty());
    }

    /// A long row -- more fields than the header -- is not a bad row at all.
    /// Its extra fields simply have no header name and are dropped, same as
    /// before; the row still gets its geometry and goes out `features`.
    #[tokio::test]
    async fn a_long_row_still_has_its_extra_fields_dropped_and_is_not_rejected() {
        let csv = "name,lon,lat\na,1.0,2.0\nb,3.0,4.0,extra,fields\nc,5.0,6.0\n";
        let sent = run(csv, &param(Some(coords_geometry()))).await;

        assert_eq!(on_port(&sent, &FEATURES_PORT).len(), 3);
        assert!(on_port(&sent, &REJECTED_PORT).is_empty());
    }

    /// A short row -- fewer fields than the header -- leaves a configured
    /// geometry column with no value on that row. That is a blank, not a
    /// broken value: the feature goes out `features` with no geometry,
    /// exactly like an explicit blank cell, rather than being rejected.
    #[tokio::test]
    async fn a_short_row_missing_its_geometry_column_yields_no_geometry_on_the_normal_port() {
        let csv = "name,lon,lat\na,1.0,2.0\nb,3.0\nc,5.0,6.0\n";
        let sent = run(csv, &param(Some(coords_geometry()))).await;

        assert!(on_port(&sent, &REJECTED_PORT).is_empty());
        let features = on_port(&sent, &FEATURES_PORT);
        assert_eq!(features.len(), 3);

        let short_row = features
            .iter()
            .find(|f| f.get("name").map(|v| v.to_string()) == Some("b".to_string()))
            .expect("the short row must still produce a feature");
        assert_eq!(*short_row.geometry, Geometry::None);
        assert!(
            short_row.get("lon").is_none(),
            "the geometry column is still excluded from attributes, as for any other row"
        );
    }

    /// The same short-row case for `zColumn`: `xColumn` reaches the row,
    /// `zColumn` falls off the end. No special-casing by which column is
    /// missing -- it is still just "no geometry".
    #[tokio::test]
    async fn a_short_row_missing_only_its_z_column_still_yields_no_geometry() {
        let config = GeometryConfig {
            mode: GeometryMode::Coordinates {
                x_column: "lon".to_string(),
                y_column: "lat".to_string(),
                z_column: Some("h".to_string()),
            },
            epsg: None,
        };
        let csv = "name,lon,lat,h\na,1.0,2.0\n";
        let sent = run(csv, &param(Some(config))).await;

        assert!(on_port(&sent, &REJECTED_PORT).is_empty());
        let features = on_port(&sent, &FEATURES_PORT);
        assert_eq!(features.len(), 1);
        assert_eq!(*features[0].geometry, Geometry::None);
    }

    /// The safety net a shape check used to provide: a wrong delimiter (or a
    /// truncated header) collapses every field on the header line into one,
    /// so the configured geometry column can never be found in it. That is a
    /// configuration error, so it must still fail the whole read loudly,
    /// rather than degrading into a per-row "no geometry" for every single
    /// row.
    #[tokio::test]
    async fn a_wrong_delimiter_still_fails_the_read_via_the_missing_column_check() {
        // Comma-delimited content read with a tab-delimited reader: the
        // entire header line becomes a single field.
        let csv = "name,lon,lat\na,1.0,2.0\n";
        let content = Bytes::from(csv.to_string());
        let (tx, _rx) = mpsc::channel(64);
        let result = read_csv(
            Delimiter::Tab,
            &content,
            &param(Some(coords_geometry())),
            None,
            tx,
            &NodeContext::default(),
        )
        .await;

        let error = result
            .expect_err("a wrong delimiter must fail the read")
            .to_string();
        assert!(error.contains("lon"), "{error}");
        assert!(error.contains("name,lon,lat"), "{error}");
    }

    /// A configured geometry column that isn't in the header at all is a
    /// configuration error, not a row error -- every row would fail
    /// identically, so the whole read fails up front with one message naming
    /// the missing column and listing what is actually available, rather than
    /// rejecting every row for the same reason.
    #[tokio::test]
    async fn a_missing_configured_column_fails_the_read_naming_it_and_the_available_columns() {
        let csv = "name,longitude,latitude\na,1.0,2.0\n";
        let content = Bytes::from(csv.to_string());
        let (tx, _rx) = mpsc::channel(64);
        let result = read_csv(
            Delimiter::Comma,
            &content,
            &param(Some(coords_geometry())), // configured for "lon"/"lat"
            None,
            tx,
            &NodeContext::default(),
        )
        .await;

        let error = result
            .expect_err("a missing configured column must fail the read")
            .to_string();
        assert!(error.contains("lon"), "{error}");
        assert!(error.contains("longitude"), "{error}");
        assert!(error.contains("latitude"), "{error}");
        assert!(error.contains("name"), "{error}");
    }

    /// The same check in `headerRows: 0` mode, where column names don't exist
    /// until they are auto-generated from the first data row. The failure
    /// still has to land on that first row, not silently pass through to
    /// every later one.
    #[tokio::test]
    async fn a_missing_configured_column_fails_the_read_with_auto_generated_headers() {
        let params = CsvReaderParam {
            offset: None,
            header_rows: Some(0),
            geometry: Some(wkt_geometry("geom")),
        };
        let csv = "a,1.0\nb,2.0\n";
        let content = Bytes::from(csv.to_string());
        let (tx, _rx) = mpsc::channel(64);
        let result = read_csv(
            Delimiter::Comma,
            &content,
            &params,
            None,
            tx,
            &NodeContext::default(),
        )
        .await;

        let error = result
            .expect_err(
                "a missing configured column must fail the read even with auto-generated headers",
            )
            .to_string();
        assert!(error.contains("geom"), "{error}");
        assert!(error.contains("column1"), "{error}");
        assert!(error.contains("column2"), "{error}");
    }

    /// A WKT parse failure has to name the offending column and repeat the
    /// offending text: `WktParsing` on its own only ever carries the `wkt`
    /// crate's parser message, which names neither.
    #[tokio::test]
    async fn a_wkt_parse_failure_names_the_column_and_the_value() {
        let csv = "name,geom\nb,NOT WKT AT ALL\n";
        let error = try_run(csv, &param(Some(wkt_geometry("geom"))))
            .await
            .expect_err("unparseable WKT must fail the read")
            .to_string();
        assert!(error.contains("geom"), "{error}");
        assert!(error.contains("NOT WKT AT ALL"), "{error}");
        assert!(error.contains("row 1"), "{error}");
    }

    /// A long WKT value must not blow up the error message: it is capped and
    /// marked with a trailing "...". The row number is what leads the user
    /// back to the untruncated original in the file.
    #[tokio::test]
    async fn a_long_wkt_value_is_truncated_in_the_error() {
        let bad_wkt = format!("POINT({}", "1".repeat(200));
        let csv = format!("name,geom\na,POINT(1 2)\nb,{bad_wkt}\n");
        let error = try_run(&csv, &param(Some(wkt_geometry("geom"))))
            .await
            .expect_err("unparseable WKT must fail the read")
            .to_string();

        assert!(
            !error.contains(&bad_wkt),
            "the error must not repeat the whole value: {error}"
        );
        assert!(error.contains("..."), "{error}");
        assert!(
            error.contains("row 2"),
            "the row number is the way back to the full value: {error}"
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
