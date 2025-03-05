use std::{collections::HashMap, io::Cursor, str::FromStr, sync::Arc};

use reearth_flow_common::{csv::Delimiter, uri::Uri};
use reearth_flow_runtime::{
    executor_operation::ExecutorContext, forwarder::ProcessorChannelForwarder, node::DEFAULT_PORT,
};
use reearth_flow_types::{Attribute, AttributeValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::CompiledCommonReaderParam;

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CsvReaderParam {
    /// The offset of the first row to read
    offset: Option<usize>,
}

pub(crate) fn read_csv(
    delimiter: Delimiter,
    global_params: &Option<HashMap<String, serde_json::Value>>,
    params: &CompiledCommonReaderParam,
    csv_params: &CsvReaderParam,
    ctx: ExecutorContext,
    fw: &ProcessorChannelForwarder,
) -> Result<(), super::errors::FeatureProcessorError> {
    let feature = &ctx.feature;
    let expr_engine = Arc::clone(&ctx.expr_engine);
    let storage_resolver = &ctx.storage_resolver;
    let scope = feature.new_scope(expr_engine.clone(), global_params);
    let csv_path = scope.eval_ast::<String>(&params.expr).map_err(|e| {
        super::errors::FeatureProcessorError::FileCsvReader(format!(
            "Failed to evaluate expr: {}",
            e
        ))
    })?;
    let input_path = Uri::from_str(csv_path.as_str())
        .map_err(|e| super::errors::FeatureProcessorError::FileCsvReader(format!("{:?}", e)))?;
    let storage = storage_resolver
        .resolve(&input_path)
        .map_err(|e| super::errors::FeatureProcessorError::FileCsvReader(format!("{:?}", e)))?;
    let byte = storage
        .get_sync(input_path.path().as_path())
        .map_err(|e| super::errors::FeatureProcessorError::FileCsvReader(format!("{:?}", e)))?;
    let cursor = Cursor::new(byte);
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .has_headers(false)
        .delimiter(delimiter.into())
        .from_reader(cursor);
    let offset = csv_params.offset.unwrap_or(0);
    let header = rdr
        .deserialize()
        .nth(offset)
        .unwrap_or(Ok(Vec::<String>::new()))
        .map_err(|e| super::errors::FeatureProcessorError::FileCsvReader(format!("{:?}", e)))?;
    for rd in rdr.deserialize() {
        let record: Vec<String> = rd
            .map_err(|e| super::errors::FeatureProcessorError::FileCsvReader(format!("{:?}", e)))?;
        let row = record
            .iter()
            .enumerate()
            .map(|(i, value)| {
                (
                    Attribute::new(header[i].clone()),
                    AttributeValue::String(value.clone()),
                )
            })
            .collect::<HashMap<Attribute, AttributeValue>>();
        let mut feature = feature.clone();
        feature.attributes.extend(row);
        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
    }
    Ok(())
}
