use std::{collections::HashMap, io::Cursor, sync::Arc};

use reearth_flow_common::{csv::Delimiter, uri::Uri};
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder, executor_operation::ExecutorContext, node::DEFAULT_PORT,
};
use reearth_flow_types::{Attribute, AttributeValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::CompiledCommonPropertySchema;

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CsvPropertySchema {
    pub(super) offset: Option<usize>,
}

pub(crate) fn read_csv(
    delimiter: Delimiter,
    params: &CompiledCommonPropertySchema,
    csv_params: &CsvPropertySchema,
    ctx: ExecutorContext,
    fw: &mut dyn ProcessorChannelForwarder,
) -> Result<(), super::errors::FeatureProcessorError> {
    let feature = &ctx.feature;
    let expr_engine = Arc::clone(&ctx.expr_engine);
    let storage_resolver = &ctx.storage_resolver;
    let scope = expr_engine.new_scope();
    for (k, v) in &feature.attributes {
        scope.set(k.inner().as_str(), v.clone().into());
    }
    let csv_path = scope.eval_ast::<String>(&params.expr).map_err(|e| {
        super::errors::FeatureProcessorError::FileCsvReader(format!(
            "Failed to evaluate expr: {}",
            e
        ))
    })?;
    let input_path = Uri::for_test(csv_path.as_str());
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
