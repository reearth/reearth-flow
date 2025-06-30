use std::sync::Arc;

use bytes::Bytes;
use reearth_flow_common::{csv::Delimiter, uri::Uri};
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::{AttributeValue, Feature};

use super::FeatureProcessorError;

pub(super) fn write_csv(
    output: &Uri,
    delimiter: Delimiter,
    storage_resolver: &Arc<StorageResolver>,
    features: &[Feature],
) -> Result<(), FeatureProcessorError> {
    if features.is_empty() {
        return Ok(());
    }
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(delimiter.into())
        .quote_style(csv::QuoteStyle::NonNumeric)
        .from_writer(vec![]);
    let rows: Vec<AttributeValue> = features.iter().map(|f| f.clone().into()).collect();
    let mut fields = if let Some(first_row) = rows.first() {
        get_fields(first_row)
    } else {
        return Ok(());
    };

    if let Some(ref mut fields) = fields {
        // Remove _id field
        fields.retain(|field| field != "_id");
        // Write header
        if !fields.is_empty() {
            wtr.write_record(fields.clone())
                .map_err(|e| FeatureProcessorError::FeatureWriter(format!("{e:?}")))?;
        }
    }

    for row in rows {
        match fields {
            Some(ref fields) if !fields.is_empty() => {
                let values = get_row_values(&row, &fields.clone())?;
                wtr.write_record(values)
                    .map_err(|e| FeatureProcessorError::FeatureWriter(format!("{e:?}")))?;
            }
            _ => match row {
                AttributeValue::String(s) => wtr
                    .write_record(vec![s])
                    .map_err(|e| FeatureProcessorError::FeatureWriter(format!("{e:?}")))?,
                AttributeValue::Array(s) => {
                    let values = s
                        .into_iter()
                        .map(|v| match v {
                            AttributeValue::String(s) => s,
                            _ => "".to_string(),
                        })
                        .collect::<Vec<_>>();
                    wtr.write_record(values)
                        .map_err(|e| FeatureProcessorError::FeatureWriter(format!("{e:?}")))?
                }
                _ => {
                    return Err(FeatureProcessorError::FeatureWriter(
                        "Unsupported input".to_string(),
                    ))
                }
            },
        }
    }
    wtr.flush()
        .map_err(|e| FeatureProcessorError::FeatureWriter(format!("{e:?}")))?;
    let data = String::from_utf8(
        wtr.into_inner()
            .map_err(|e| FeatureProcessorError::FeatureWriter(format!("{e:?}")))?,
    )
    .map_err(|e| FeatureProcessorError::FeatureWriter(format!("{e:?}")))?;
    let storage = storage_resolver
        .resolve(output)
        .map_err(|e| FeatureProcessorError::FeatureWriter(format!("{e:?}")))?;
    storage
        .put_sync(output.path().as_path(), Bytes::from(data))
        .map_err(|e| FeatureProcessorError::FeatureWriter(format!("{e:?}")))?;
    Ok(())
}

fn get_fields(row: &AttributeValue) -> Option<Vec<String>> {
    match row {
        AttributeValue::Map(row) => Some(row.keys().cloned().collect::<Vec<_>>()),
        _ => None,
    }
}

fn get_row_values(
    row: &AttributeValue,
    fields: &[String],
) -> Result<Vec<String>, FeatureProcessorError> {
    fields
        .iter()
        .map(|field| match row {
            AttributeValue::Map(row) => row.get(field).map(|v| v.to_string()).ok_or_else(|| {
                FeatureProcessorError::FeatureWriter(format!("Field not found: {field}"))
            }),
            _ => Err(FeatureProcessorError::FeatureWriter(
                "Unsupported input".to_string(),
            )),
        })
        .collect()
}
