use std::{
    collections::HashMap,
    io::{Cursor, Read},
    path::Path,
    str::FromStr,
    sync::Arc,
};

use once_cell::sync::Lazy;
use reearth_flow_common::{dir::project_temp_dir, uri::Uri};
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_sevenz::{decompress_with_extract_fn, default_entry_extract_fn};
use reearth_flow_storage::storage::Storage;
use reearth_flow_types::{AttributeValue, Expr, Feature, FilePath};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::FeatureProcessorError;

pub static UNFILTERED_PORT: Lazy<Port> = Lazy::new(|| Port::new("unfiltered"));

#[derive(Debug, Clone, Default)]
pub struct FeatureFilePathExtractorFactory;

impl ProcessorFactory for FeatureFilePathExtractorFactory {
    fn name(&self) -> &str {
        "FeatureFilePathExtractor"
    }

    fn description(&self) -> &str {
        "Extracts features by file path"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FeatureFilePathExtractorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Feature"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone(), UNFILTERED_PORT.clone()]
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let param: FeatureFilePathExtractorParam = if let Some(with) = with.clone() {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::FilePathExtractorFactory(format!(
                    "Failed to serialize `with` parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::FilePathExtractorFactory(format!(
                    "Failed to deserialize `with` parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(FeatureProcessorError::FilePathExtractorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let source_dataset = expr_engine
            .compile(param.source_dataset.as_ref())
            .map_err(|e| {
                FeatureProcessorError::FilePathExtractorFactory(format!(
                    "Failed to compile `source_dataset` expression: {}",
                    e
                ))
            })?;
        let process = FeatureFilePathExtractor {
            params: FeatureFilePathExtractorCompiledParam {
                source_dataset,
                extract_archive: param.extract_archive,
            },
            with,
        };
        Ok(Box::new(process))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeatureFilePathExtractorParam {
    source_dataset: Expr,
    extract_archive: bool,
}

#[derive(Debug, Clone)]
pub struct FeatureFilePathExtractorCompiledParam {
    source_dataset: rhai::AST,
    extract_archive: bool,
}

#[derive(Debug, Clone)]
pub struct FeatureFilePathExtractor {
    params: FeatureFilePathExtractorCompiledParam,
    with: Option<HashMap<String, Value>>,
}

impl Processor for FeatureFilePathExtractor {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let scope = feature.new_scope(expr_engine, &self.with);
        let source_dataset = scope
            .eval_ast::<String>(&self.params.source_dataset)
            .map_err(|e| {
                FeatureProcessorError::FilePathExtractor(format!(
                    "Failed to evaluate `source_dataset` expression: {}",
                    e
                ))
            })?;
        let source_dataset = Uri::from_str(source_dataset.as_str())
            .map_err(|_| FeatureProcessorError::FilePathExtractor("Invalid path".to_string()))?;
        let storage = ctx.storage_resolver.resolve(&source_dataset).map_err(|e| {
            FeatureProcessorError::FilePathExtractor(format!(
                "Failed to resolve `source_dataset` path: {}",
                e
            ))
        })?;

        if self.is_extractable_archive(&source_dataset) {
            let root_output_path = project_temp_dir(uuid::Uuid::new_v4().to_string().as_str())?;
            let root_output_path = Uri::from_str(root_output_path.to_str().ok_or(
                FeatureProcessorError::FilePathExtractor("Invalid path".to_string()),
            )?)
            .map_err(|e| {
                FeatureProcessorError::FilePathExtractor(format!(
                    "Failed to convert `root_output_path` to URI: {}",
                    e
                ))
            })?;
            let bytes = storage
                .get_sync(source_dataset.path().as_path())
                .map_err(|e| {
                    FeatureProcessorError::FilePathExtractor(format!(
                        "Failed to get `source_dataset` content: {}",
                        e
                    ))
                })?;
            let root_output_storage = ctx
                .storage_resolver
                .resolve(&root_output_path)
                .map_err(|e| FeatureProcessorError::FilePathExtractor(format!("{:?}", e)))?;
            root_output_storage
                .create_dir_sync(root_output_path.path().as_path())
                .map_err(|e| FeatureProcessorError::FilePathExtractor(format!("{:?}", e)))?;
            let features = if let Some(ext) = source_dataset.path().as_path().extension() {
                if let Some(ext) = ext.to_str() {
                    if ["7z", "7zip"].contains(&ext) {
                        extract_sevenz(bytes, root_output_path)?
                    } else {
                        extract_zip(bytes, root_output_path, root_output_storage)?
                    }
                } else {
                    extract_zip(bytes, root_output_path, root_output_storage)?
                }
            } else {
                extract_zip(bytes, root_output_path, root_output_storage)?
            };
            for feature in features {
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
        } else if source_dataset.is_dir() {
            let entries = storage
                .list_sync(Some(source_dataset.path().as_path()), true)
                .map_err(|e| FeatureProcessorError::FilePathExtractor(format!("{:?}", e)))?;
            for entry in entries {
                let attribute_value =
                    AttributeValue::try_from(FilePath::try_from(entry).unwrap_or_default())?;
                fw.send(ctx.new_with_feature_and_port(
                    Feature::from(attribute_value),
                    DEFAULT_PORT.clone(),
                ));
            }
        } else {
            let attribute_value = AttributeValue::try_from(FilePath::try_from(source_dataset)?)?;
            fw.send(
                ctx.new_with_feature_and_port(Feature::from(attribute_value), DEFAULT_PORT.clone()),
            );
        }
        Ok(())
    }

    fn finish(
        &self,
        _ctx: NodeContext,
        _fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "FeatureFilePathExtractor"
    }
}

impl FeatureFilePathExtractor {
    fn is_extractable_archive(&self, path: &Uri) -> bool {
        self.params.extract_archive
            && !path.is_dir()
            && path.extension().is_some()
            && matches!(path.extension().unwrap(), "zip" | "7z" | "7zip")
    }
}

fn extract_zip(
    bytes: bytes::Bytes,
    root_output_path: Uri,
    storage: Arc<Storage>,
) -> super::errors::Result<Vec<Feature>> {
    let mut zip_archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|e| {
        FeatureProcessorError::FilePathExtractor(format!(
            "Failed to open `source_dataset` as zip archive: {}",
            e
        ))
    })?;
    let mut features = Vec::<Feature>::new();
    for i in 0..zip_archive.len() {
        let mut entry = zip_archive.by_index(i).map_err(|e| {
            FeatureProcessorError::FilePathExtractor(format!(
                "Failed to get `source_dataset` entry: {}",
                e
            ))
        })?;
        let filename = entry.name();
        let outpath = root_output_path.join(filename).map_err(|e| {
            FeatureProcessorError::FilePathExtractor(format!(
                "Output path join error with: error = {:?}",
                e
            ))
        })?;
        let filepath = Path::new(filename);
        if filepath
            .file_name()
            .take_if(|s| s.to_string_lossy().starts_with("."))
            .is_some()
        {
            continue;
        }
        if entry.is_dir() {
            if storage.exists_sync(outpath.path().as_path()).map_err(|e| {
                FeatureProcessorError::FilePathExtractor(format!(
                    "Storage exists error with: error = {:?}",
                    e
                ))
            })? {
                continue;
            }
            storage
                .create_dir_sync(outpath.path().as_path())
                .map_err(|e| {
                    FeatureProcessorError::FilePathExtractor(format!(
                        "Failed to create directory: error = {:?}",
                        e
                    ))
                })?;
            continue;
        }
        if let Some(p) = outpath.parent() {
            if !storage.exists_sync(p.path().as_path()).map_err(|e| {
                FeatureProcessorError::FilePathExtractor(format!(
                    "Storage exists error with: error = {:?}",
                    e
                ))
            })? {
                storage.create_dir_sync(p.path().as_path()).map_err(|e| {
                    FeatureProcessorError::FilePathExtractor(format!(
                        "Create dir error with: error = {:?}",
                        e
                    ))
                })?;
            }
        }
        let mut buf = Vec::<u8>::new();
        entry.read_to_end(&mut buf).map_err(|e| {
            FeatureProcessorError::FilePathExtractor(format!(
                "Failed to read `source_dataset` entry: {}",
                e
            ))
        })?;
        let file_path = FilePath::try_from(outpath.clone()).map_err(|e| {
            FeatureProcessorError::FilePathExtractor(format!(
                "Filepath convert error with: error = {:?}",
                e
            ))
        })?;
        let attribute_value = AttributeValue::try_from(file_path).map_err(|e| {
            FeatureProcessorError::FilePathExtractor(format!(
                "Attribute Value convert error with: error = {:?}",
                e
            ))
        })?;
        storage
            .put_sync(outpath.path().as_path(), bytes::Bytes::from(buf))
            .map_err(|e| {
                FeatureProcessorError::FilePathExtractor(format!(
                    "Storage put error with: error = {:?}",
                    e
                ))
            })?;
        let feature = Feature::from(attribute_value);
        features.push(feature);
    }
    Ok(features)
}

fn extract_sevenz(
    bytes: bytes::Bytes,
    root_output_path: Uri,
) -> super::errors::Result<Vec<Feature>> {
    let mut entries = Vec::<Uri>::new();
    let cursor = Cursor::new(bytes);
    decompress_with_extract_fn(cursor, root_output_path.as_path(), |entry, reader, dest| {
        if !entry.is_directory {
            let dest_uri = Uri::try_from(dest.clone()).map_err(|e| {
                reearth_flow_sevenz::Error::Unknown(format!(
                    "Failed to convert `dest` to URI: {}",
                    e
                ))
            });
            if let Ok(dest_uri) = dest_uri {
                entries.push(dest_uri);
            }
        }
        default_entry_extract_fn(entry, reader, dest)
    })
    .map_err(|e| {
        FeatureProcessorError::FilePathExtractor(format!(
            "Failed to extract `source_dataset` archive: {}",
            e
        ))
    })?;
    let features = entries
        .iter()
        .flat_map(|entry| {
            let file_path = FilePath::try_from(entry.clone())
                .map_err(|e| {
                    FeatureProcessorError::FilePathExtractor(format!(
                        "Filepath convert error with: error = {:?}",
                        e
                    ))
                })
                .ok()?;
            let attribute_value = AttributeValue::try_from(file_path)
                .map_err(|e| {
                    FeatureProcessorError::FilePathExtractor(format!(
                        "Attribute Value convert error with: error = {:?}",
                        e
                    ))
                })
                .ok()?;
            Some(Feature::from(attribute_value))
        })
        .collect::<Vec<_>>();
    Ok(features)
}
