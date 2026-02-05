use reearth_flow_runtime::{
    cache::executor_cache_subdir,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BinaryHeap, HashMap};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

use super::errors::FeatureProcessorError;
use crate::ACCUMULATOR_BUFFER_BYTE_THRESHOLD;

const MERGE_FAN_IN: usize = 64;

struct HeapEntry {
    key: AttributeValue,
    key_json: String,
    feature_json: String,
    chunk_idx: usize,
    descending: bool,
}

impl PartialEq for HeapEntry {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}
impl Eq for HeapEntry {}
impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.descending {
            self.key.cmp(&other.key)
        } else {
            other.key.cmp(&self.key)
        }
    }
}

fn read_entry(
    reader: &mut BufReader<File>,
    chunk_idx: usize,
    descending: bool,
) -> Option<HeapEntry> {
    let mut line = String::new();
    match reader.read_line(&mut line) {
        Ok(0) => None,
        Ok(_) => {
            let line = line.trim_end_matches('\n');
            let tab_pos = line.find('\t')?;
            let key_json = &line[..tab_pos];
            let feature_json = &line[tab_pos + 1..];
            let key: AttributeValue = serde_json::from_str(key_json).ok()?;
            Some(HeapEntry {
                key,
                key_json: key_json.to_string(),
                feature_json: feature_json.to_string(),
                chunk_idx,
                descending,
            })
        }
        Err(_) => None,
    }
}

fn merge_chunks_to_writer(
    chunk_paths: &[PathBuf],
    writer: &mut BufWriter<File>,
    descending: bool,
) -> Result<(), BoxedError> {
    let mut readers: Vec<BufReader<File>> = chunk_paths
        .iter()
        .map(|p| {
            let file = File::open(p).expect("failed to open chunk file");
            BufReader::new(file)
        })
        .collect();

    let mut heap = BinaryHeap::new();
    for (i, reader) in readers.iter_mut().enumerate() {
        if let Some(entry) = read_entry(reader, i, descending) {
            heap.push(entry);
        }
    }

    while let Some(entry) = heap.pop() {
        writer.write_all(entry.key_json.as_bytes())?;
        writer.write_all(b"\t")?;
        writer.write_all(entry.feature_json.as_bytes())?;
        writer.write_all(b"\n")?;
        if let Some(next) = read_entry(&mut readers[entry.chunk_idx], entry.chunk_idx, descending) {
            heap.push(next);
        }
    }
    writer.flush()?;
    Ok(())
}

#[derive(Debug, Clone, Default)]
pub(super) struct FeatureSorterFactory;

impl ProcessorFactory for FeatureSorterFactory {
    fn name(&self) -> &str {
        "FeatureSorter"
    }

    fn description(&self) -> &str {
        "Sorts features based on specified attributes in ascending or descending order"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FeatureSorterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Feature"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: FeatureSorterParam = if let Some(with) = with.clone() {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::SorterFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::SorterFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(FeatureProcessorError::SorterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let process = FeatureSorter {
            params,
            buffer: Vec::new(),
            buffer_bytes: 0,
            temp_dir: None,
            chunk_count: 0,
            executor_id: None,
        };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
struct FeatureSorter {
    params: FeatureSorterParam,
    buffer: Vec<(AttributeValue, String, String)>, // (key, key_json, feature_json)
    buffer_bytes: usize,
    temp_dir: Option<PathBuf>,
    chunk_count: usize,
    /// Executor ID for cache isolation, set on first process() call
    executor_id: Option<uuid::Uuid>,
}

/// # FeatureSorter Parameters
///
/// Configuration for sorting features based on attribute values.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct FeatureSorterParam {
    /// Attributes to use for sorting features (sort order based on attribute order)
    attributes: Vec<Attribute>,
    /// Sorting order (ascending or descending)
    order: Order,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
enum Order {
    #[serde(rename = "ascending")]
    Asc,
    #[serde(rename = "descending")]
    Desc,
}

/// Executor-specific engine cache folder for accumulating processors
fn engine_cache_dir(executor_id: uuid::Uuid) -> PathBuf {
    executor_cache_subdir(executor_id, "processors")
}

impl FeatureSorter {
    fn ensure_temp_dir(&mut self) -> Result<&PathBuf, BoxedError> {
        if self.temp_dir.is_none() {
            let executor_id = self.executor_id.unwrap_or_else(uuid::Uuid::nil);
            let dir = engine_cache_dir(executor_id).join(format!("sorter-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&dir)?;
            self.temp_dir = Some(dir);
        }
        Ok(self.temp_dir.as_ref().unwrap())
    }

    fn flush_buffer(&mut self) -> Result<(), BoxedError> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        if self.params.order == Order::Desc {
            self.buffer.sort_by(|a, b| b.0.cmp(&a.0));
        } else {
            self.buffer.sort_by(|a, b| a.0.cmp(&b.0));
        }

        let dir = self.ensure_temp_dir()?.clone();
        let chunk_path = dir.join(format!("chunk_{:06}.tsv", self.chunk_count));
        let file = File::create(&chunk_path)?;
        let mut writer = BufWriter::new(file);

        for (_, key_json, feature_json) in &self.buffer {
            writer.write_all(key_json.as_bytes())?;
            writer.write_all(b"\t")?;
            writer.write_all(feature_json.as_bytes())?;
            writer.write_all(b"\n")?;
        }
        writer.flush()?;

        self.chunk_count += 1;
        self.buffer.clear();
        self.buffer_bytes = 0;
        Ok(())
    }
}

impl Drop for FeatureSorter {
    fn drop(&mut self) {
        if let Some(ref dir) = self.temp_dir {
            let _ = std::fs::remove_dir_all(dir);
        }
    }
}

impl Processor for FeatureSorter {
    fn is_accumulating(&self) -> bool {
        true
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        // Capture executor_id on first process call for cache isolation
        if self.executor_id.is_none() {
            self.executor_id = Some(fw.executor_id());
        }

        let feature = ctx.feature;
        let key: Vec<AttributeValue> = self
            .params
            .attributes
            .iter()
            .flat_map(|attribute| feature.get(attribute))
            .cloned()
            .collect();
        let key = AttributeValue::Array(key);
        let key_json = serde_json::to_string(&key)?;
        let feature_json = serde_json::to_string(&feature)?;

        self.buffer_bytes += key_json.len() + feature_json.len();
        self.buffer.push((key, key_json, feature_json));

        if self.buffer_bytes >= ACCUMULATOR_BUFFER_BYTE_THRESHOLD {
            self.flush_buffer()?;
        }
        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        self.flush_buffer()?;
        // Reclaim buffer memory before the merge phase
        self.buffer = Vec::new();

        if self.chunk_count == 0 {
            return Ok(());
        }

        let dir = self.temp_dir.as_ref().unwrap().clone();
        let descending = self.params.order == Order::Desc;

        // Collect initial chunk paths
        let mut chunk_paths: Vec<PathBuf> = (0..self.chunk_count)
            .map(|i| dir.join(format!("chunk_{:06}.tsv", i)))
            .collect();

        // Multi-pass merge: merge groups of MERGE_FAN_IN until few enough remain
        let mut pass: usize = 0;
        while chunk_paths.len() > MERGE_FAN_IN {
            pass += 1;
            let mut next_paths = Vec::new();

            for (group_idx, group) in chunk_paths.chunks(MERGE_FAN_IN).enumerate() {
                let out_path = dir.join(format!("pass_{pass:03}_chunk_{group_idx:06}.tsv"));
                let file = File::create(&out_path)?;
                let mut writer = BufWriter::new(file);
                merge_chunks_to_writer(group, &mut writer, descending)?;
                next_paths.push(out_path);
            }

            // Delete old pass files
            for p in &chunk_paths {
                let _ = std::fs::remove_file(p);
            }

            chunk_paths = next_paths;
        }

        // Final merge: write to output JSONL file for file-backed sending
        let output_path = dir.join("output.jsonl");
        {
            let mut readers: Vec<BufReader<File>> = chunk_paths
                .iter()
                .map(|p| {
                    let file = File::open(p).expect("failed to open chunk file");
                    BufReader::new(file)
                })
                .collect();

            let mut heap = BinaryHeap::new();
            for (i, reader) in readers.iter_mut().enumerate() {
                if let Some(entry) = read_entry(reader, i, descending) {
                    heap.push(entry);
                }
            }

            let out_file = File::create(&output_path)?;
            let mut writer = BufWriter::new(out_file);
            while let Some(entry) = heap.pop() {
                writer.write_all(entry.feature_json.as_bytes())?;
                writer.write_all(b"\n")?;
                if let Some(next) =
                    read_entry(&mut readers[entry.chunk_idx], entry.chunk_idx, descending)
                {
                    heap.push(next);
                }
            }
            writer.flush()?;
        }

        fw.send_file(output_path, DEFAULT_PORT.clone(), ctx.as_context());

        Ok(())
    }

    fn name(&self) -> &str {
        "FeatureSorter"
    }
}
