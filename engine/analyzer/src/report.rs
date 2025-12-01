use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

use crate::events::AnalyzerEvent;

/// Memory data point for time-series graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDataPoint {
    /// Unix timestamp in milliseconds when tracking ended (after process()/finish()).
    pub timestamp_ms: u64,
    /// Unix timestamp in milliseconds when tracking started (before process()/finish()).
    #[serde(default)]
    pub start_timestamp_ms: u64,
    /// Name of the thread that processed the feature.
    pub thread_name: String,
    /// Current memory usage in bytes at end of process().
    pub current_memory_bytes: usize,
    /// Peak memory usage in bytes during process().
    pub peak_memory_bytes: usize,
    /// Time spent in process() in milliseconds.
    pub processing_time_ms: u64,
}

/// Feature queue data point for time-series graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueDataPoint {
    /// Unix timestamp in milliseconds.
    pub timestamp_ms: u64,
    /// Number of features waiting in the input queue.
    pub features_waiting: u64,
    /// Number of features currently being processed.
    pub features_processing: u64,
}

/// Node information for the report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    /// Node's unique identifier.
    pub node_id: String,
    /// Human-readable node name.
    pub node_name: String,
}

/// Edge information for the report.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EdgeInfo {
    /// Edge's unique identifier.
    pub edge_id: String,
    /// Source node's unique identifier.
    pub source_node_id: String,
    /// Total number of features that passed through this edge.
    pub total_features: u64,
    /// Total bytes transferred through this edge.
    pub total_bytes: usize,
    /// Average feature size in bytes.
    pub avg_feature_size: usize,
    /// Minimum feature size in bytes.
    pub min_feature_size: usize,
    /// Maximum feature size in bytes.
    pub max_feature_size: usize,
}

/// Default quantization resolution (number of discrete levels).
pub const DEFAULT_QUANTIZATION_RESOLUTION: usize = 100;

/// Per-node memory report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMemoryReport {
    /// Node information.
    pub info: NodeInfo,
    /// Time-series data points (raw, all collected events).
    pub data_points: Vec<MemoryDataPoint>,
    /// Quantized data points for efficient graphing.
    /// Only includes points where the quantized memory value changed.
    #[serde(default)]
    pub quantized_data_points: Vec<MemoryDataPoint>,
    /// Peak memory usage across all data points.
    pub total_peak_memory: usize,
    /// Average memory usage across all data points.
    pub avg_memory: usize,
    /// Total processing time in milliseconds.
    pub total_processing_time_ms: u64,
    /// Number of features processed.
    pub features_processed: u64,
}

impl NodeMemoryReport {
    fn new(node_id: String, node_name: String) -> Self {
        Self {
            info: NodeInfo { node_id, node_name },
            data_points: Vec::new(),
            quantized_data_points: Vec::new(),
            total_peak_memory: 0,
            avg_memory: 0,
            total_processing_time_ms: 0,
            features_processed: 0,
        }
    }

    /// Quantize data points to reduce the number of points for efficient graphing.
    ///
    /// This method:
    /// 1. Finds the maximum memory value across all data points
    /// 2. Divides the range into `resolution` discrete levels
    /// 3. Keeps only data points where the quantized value changes from the previous point
    ///
    /// This dramatically reduces the number of points while preserving the shape of the graph.
    pub fn quantize(&mut self, resolution: usize) {
        if self.data_points.is_empty() || resolution == 0 {
            self.quantized_data_points = Vec::new();
            return;
        }

        // Find max memory value
        let max_memory = self
            .data_points
            .iter()
            .map(|p| p.current_memory_bytes)
            .max()
            .unwrap_or(0);

        // Handle edge case where all values are 0
        if max_memory == 0 {
            // Just keep first and last points
            self.quantized_data_points = vec![self.data_points[0].clone()];
            if self.data_points.len() > 1 {
                self.quantized_data_points
                    .push(self.data_points.last().unwrap().clone());
            }
            return;
        }

        // Calculate quantum size (each level represents this many bytes)
        let quantum_size = max_memory.div_ceil(resolution);

        let mut result = Vec::new();
        let mut last_quantized_value: Option<usize> = None;

        for point in &self.data_points {
            let quantized_value = point.current_memory_bytes / quantum_size;

            // Keep this point if the quantized value changed
            if last_quantized_value.is_none() || last_quantized_value != Some(quantized_value) {
                result.push(point.clone());
                last_quantized_value = Some(quantized_value);
            }
        }

        // Always include the last point to ensure the graph ends at the correct position
        if let Some(last_point) = self.data_points.last() {
            if result.last().map(|p| p.timestamp_ms) != Some(last_point.timestamp_ms) {
                result.push(last_point.clone());
            }
        }

        self.quantized_data_points = result;
    }
}

/// Per-node queue report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeQueueReport {
    /// Node information.
    pub info: NodeInfo,
    /// Time-series data points (raw, cleared after quantization).
    pub data_points: Vec<QueueDataPoint>,
    /// Quantized data points for efficient graphing.
    #[serde(default)]
    pub quantized_data_points: Vec<QueueDataPoint>,
    /// Maximum queue depth observed.
    pub max_queue_depth: u64,
    /// Average queue depth.
    pub avg_queue_depth: f64,
}

impl NodeQueueReport {
    fn new(node_id: String) -> Self {
        Self {
            info: NodeInfo {
                node_id,
                node_name: String::new(),
            },
            data_points: Vec::new(),
            quantized_data_points: Vec::new(),
            max_queue_depth: 0,
            avg_queue_depth: 0.0,
        }
    }

    /// Quantize queue data points to reduce the number of points for efficient graphing.
    ///
    /// Uses the total queue depth (features_waiting + features_processing) for quantization.
    pub fn quantize(&mut self, resolution: usize) {
        if self.data_points.is_empty() || resolution == 0 {
            self.quantized_data_points = Vec::new();
            return;
        }

        // Find max queue depth
        let max_depth = self
            .data_points
            .iter()
            .map(|p| p.features_waiting + p.features_processing)
            .max()
            .unwrap_or(0);

        // Handle edge case where all values are 0
        if max_depth == 0 {
            self.quantized_data_points = vec![self.data_points[0].clone()];
            if self.data_points.len() > 1 {
                self.quantized_data_points
                    .push(self.data_points.last().unwrap().clone());
            }
            return;
        }

        // Calculate quantum size
        let quantum_size = max_depth.div_ceil(resolution as u64);

        let mut result = Vec::new();
        let mut last_quantized_value: Option<u64> = None;

        for point in &self.data_points {
            let total_depth = point.features_waiting + point.features_processing;
            let quantized_value = total_depth / quantum_size;

            if last_quantized_value.is_none() || last_quantized_value != Some(quantized_value) {
                result.push(point.clone());
                last_quantized_value = Some(quantized_value);
            }
        }

        // Always include the last point
        if let Some(last_point) = self.data_points.last() {
            if result.last().map(|p| p.timestamp_ms) != Some(last_point.timestamp_ms) {
                result.push(last_point.clone());
            }
        }

        self.quantized_data_points = result;
    }
}

/// Summary statistics for the analyzer report.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalyzerSummary {
    /// Total number of features processed.
    pub total_features_processed: u64,
    /// Total bytes transferred across all edges.
    pub total_bytes_transferred: usize,
    /// Peak memory usage across all nodes.
    pub peak_memory_usage: usize,
    /// Node with the slowest average processing time.
    pub slowest_node: Option<String>,
    /// Slowest average processing time in milliseconds.
    pub slowest_avg_time_ms: Option<u64>,
    /// Node with the highest peak memory usage.
    pub highest_memory_node: Option<String>,
}

/// Complete analyzer report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzerReport {
    /// Report format version.
    pub version: String,
    /// Workflow's unique identifier.
    pub workflow_id: Option<Uuid>,
    /// Human-readable workflow name.
    pub workflow_name: Option<String>,
    /// Workflow start time in milliseconds.
    pub start_time_ms: u64,
    /// Workflow end time in milliseconds.
    pub end_time_ms: u64,
    /// Total workflow duration in milliseconds.
    pub duration_ms: u64,
    /// Whether the workflow completed successfully.
    pub success: bool,

    /// Memory reports keyed by node_id.
    pub memory_reports: HashMap<String, NodeMemoryReport>,

    /// Queue reports keyed by node_id.
    pub queue_reports: HashMap<String, NodeQueueReport>,

    /// Edge statistics keyed by edge_id.
    pub edge_reports: HashMap<String, EdgeInfo>,

    /// Summary statistics.
    pub summary: AnalyzerSummary,
}

impl Default for AnalyzerReport {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalyzerReport {
    /// Create a new empty analyzer report.
    pub fn new() -> Self {
        Self {
            version: "1.0.0".to_string(),
            workflow_id: None,
            workflow_name: None,
            start_time_ms: 0,
            end_time_ms: 0,
            duration_ms: 0,
            success: false,
            memory_reports: HashMap::new(),
            queue_reports: HashMap::new(),
            edge_reports: HashMap::new(),
            summary: AnalyzerSummary::default(),
        }
    }

    /// Process a single analyzer event.
    pub fn process_event(&mut self, event: AnalyzerEvent) {
        match event {
            AnalyzerEvent::ActionMemory {
                timestamp_ms,
                node_id,
                node_name,
                thread_name,
                current_memory_bytes,
                peak_memory_bytes,
                processing_time_ms,
                start_timestamp_ms,
            } => {
                let node_id_str = node_id.to_string();
                let report = self
                    .memory_reports
                    .entry(node_id_str.clone())
                    .or_insert_with(|| NodeMemoryReport::new(node_id_str, node_name));

                report.data_points.push(MemoryDataPoint {
                    timestamp_ms,
                    start_timestamp_ms,
                    thread_name,
                    current_memory_bytes,
                    peak_memory_bytes,
                    processing_time_ms,
                });
            }

            AnalyzerEvent::EdgeFeature {
                timestamp_ms: _,
                edge_id,
                feature_id: _,
                feature_size_bytes,
                source_node_id,
            } => {
                let edge_id_str = edge_id.to_string();
                let report = self
                    .edge_reports
                    .entry(edge_id_str.clone())
                    .or_insert_with(|| EdgeInfo {
                        edge_id: edge_id_str,
                        source_node_id: source_node_id.to_string(),
                        total_features: 0,
                        total_bytes: 0,
                        avg_feature_size: 0,
                        min_feature_size: usize::MAX,
                        max_feature_size: 0,
                    });

                report.total_features += 1;
                report.total_bytes += feature_size_bytes;
                report.min_feature_size = report.min_feature_size.min(feature_size_bytes);
                report.max_feature_size = report.max_feature_size.max(feature_size_bytes);
            }

            AnalyzerEvent::NodeProcessingState {
                timestamp_ms,
                node_id,
                features_waiting,
                features_processing,
            } => {
                let node_id_str = node_id.to_string();
                let report = self
                    .queue_reports
                    .entry(node_id_str.clone())
                    .or_insert_with(|| NodeQueueReport::new(node_id_str));

                report.data_points.push(QueueDataPoint {
                    timestamp_ms,
                    features_waiting,
                    features_processing,
                });
            }

            AnalyzerEvent::WorkflowStart {
                timestamp_ms,
                workflow_id,
                workflow_name,
            } => {
                self.start_time_ms = timestamp_ms;
                self.workflow_id = Some(workflow_id);
                self.workflow_name = Some(workflow_name);
            }

            AnalyzerEvent::WorkflowEnd {
                timestamp_ms,
                workflow_id,
                success,
            } => {
                tracing::info!(
                    "AnalyzerReport::process_event received WorkflowEnd: workflow_id={}, success={}",
                    workflow_id,
                    success
                );
                self.end_time_ms = timestamp_ms;
                self.duration_ms = self.end_time_ms.saturating_sub(self.start_time_ms);
                self.success = success;
            }
        }
    }

    /// Finalize the report by calculating aggregated statistics.
    pub fn finalize(&mut self) {
        self.finalize_with_quantization(DEFAULT_QUANTIZATION_RESOLUTION);
    }

    /// Finalize the report with a custom quantization resolution.
    pub fn finalize_with_quantization(&mut self, quantization_resolution: usize) {
        // Calculate memory report statistics and apply quantization
        for report in self.memory_reports.values_mut() {
            if !report.data_points.is_empty() {
                report.features_processed = report.data_points.len() as u64;

                report.total_peak_memory = report
                    .data_points
                    .iter()
                    .map(|p| p.peak_memory_bytes)
                    .max()
                    .unwrap_or(0);

                let sum: usize = report
                    .data_points
                    .iter()
                    .map(|p| p.current_memory_bytes)
                    .sum();
                report.avg_memory = sum / report.data_points.len();

                report.total_processing_time_ms = report
                    .data_points
                    .iter()
                    .map(|p| p.processing_time_ms)
                    .sum();

                // Apply quantization for efficient graphing
                report.quantize(quantization_resolution);

                // Clear raw data points to reduce file size - we only need quantized for graphing
                report.data_points.clear();
                report.data_points.shrink_to_fit();
            }
        }

        // Calculate edge report statistics
        for report in self.edge_reports.values_mut() {
            if report.total_features > 0 {
                report.avg_feature_size = report.total_bytes / report.total_features as usize;
            }
            // Fix min_feature_size if no features were recorded
            if report.min_feature_size == usize::MAX {
                report.min_feature_size = 0;
            }
        }

        // Calculate queue report statistics and apply quantization
        for report in self.queue_reports.values_mut() {
            if !report.data_points.is_empty() {
                report.max_queue_depth = report
                    .data_points
                    .iter()
                    .map(|p| p.features_waiting + p.features_processing)
                    .max()
                    .unwrap_or(0);

                let sum: u64 = report
                    .data_points
                    .iter()
                    .map(|p| p.features_waiting + p.features_processing)
                    .sum();
                report.avg_queue_depth = sum as f64 / report.data_points.len() as f64;

                // Apply quantization for efficient graphing
                report.quantize(quantization_resolution);

                // Clear raw data points to reduce file size
                report.data_points.clear();
                report.data_points.shrink_to_fit();
            }
        }

        // Calculate summary statistics
        self.summary.total_features_processed =
            self.edge_reports.values().map(|e| e.total_features).sum();

        self.summary.total_bytes_transferred =
            self.edge_reports.values().map(|e| e.total_bytes).sum();

        self.summary.peak_memory_usage = self
            .memory_reports
            .values()
            .map(|r| r.total_peak_memory)
            .max()
            .unwrap_or(0);

        // Find slowest node
        let slowest = self
            .memory_reports
            .iter()
            .filter(|(_, r)| r.features_processed > 0)
            .max_by_key(|(_, r)| r.total_processing_time_ms / r.features_processed);

        if let Some((node_id, report)) = slowest {
            self.summary.slowest_node = Some(node_id.clone());
            self.summary.slowest_avg_time_ms =
                Some(report.total_processing_time_ms / report.features_processed);
        }

        // Find highest memory node
        let highest_mem = self
            .memory_reports
            .iter()
            .max_by_key(|(_, r)| r.total_peak_memory);

        if let Some((node_id, _)) = highest_mem {
            self.summary.highest_memory_node = Some(node_id.clone());
        }
    }

    /// Save the report to a JSON file.
    pub fn save_to_file(&self, path: &Path) -> std::io::Result<()> {
        let json =
            serde_json::to_string_pretty(self).map_err(|e| std::io::Error::other(e.to_string()))?;
        std::fs::write(path, json)
    }

    /// Load a report from a JSON file.
    pub fn load_from_file(path: &Path) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content).map_err(|e| std::io::Error::other(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EdgeId;

    #[test]
    fn test_new_report() {
        let report = AnalyzerReport::new();
        assert_eq!(report.version, "1.0.0");
        assert!(report.memory_reports.is_empty());
        assert!(report.queue_reports.is_empty());
        assert!(report.edge_reports.is_empty());
    }

    #[test]
    fn test_process_action_memory_event() {
        let mut report = AnalyzerReport::new();
        let node_id = Uuid::new_v4();

        report.process_event(AnalyzerEvent::ActionMemory {
            timestamp_ms: 1000,
            node_id,
            node_name: "TestNode".to_string(),
            thread_name: "main".to_string(),
            current_memory_bytes: 1024,
            peak_memory_bytes: 2048,
            processing_time_ms: 50,
            start_timestamp_ms: 950,
        });

        assert_eq!(report.memory_reports.len(), 1);
        let mem_report = report.memory_reports.get(&node_id.to_string()).unwrap();
        assert_eq!(mem_report.data_points.len(), 1);
        assert_eq!(mem_report.data_points[0].peak_memory_bytes, 2048);
        assert_eq!(mem_report.data_points[0].start_timestamp_ms, 950);
    }

    #[test]
    fn test_process_edge_feature_event() {
        let mut report = AnalyzerReport::new();

        report.process_event(AnalyzerEvent::EdgeFeature {
            timestamp_ms: 1000,
            edge_id: EdgeId::new("edge1"),
            feature_id: Uuid::new_v4(),
            feature_size_bytes: 512,
            source_node_id: Uuid::new_v4(),
        });

        report.process_event(AnalyzerEvent::EdgeFeature {
            timestamp_ms: 1100,
            edge_id: EdgeId::new("edge1"),
            feature_id: Uuid::new_v4(),
            feature_size_bytes: 1024,
            source_node_id: Uuid::new_v4(),
        });

        assert_eq!(report.edge_reports.len(), 1);
        let edge_report = report.edge_reports.get("edge1").unwrap();
        assert_eq!(edge_report.total_features, 2);
        assert_eq!(edge_report.total_bytes, 1536);
    }

    #[test]
    fn test_finalize() {
        let mut report = AnalyzerReport::new();
        let node_id = Uuid::new_v4();

        // Add some memory events
        for i in 0..3 {
            report.process_event(AnalyzerEvent::ActionMemory {
                timestamp_ms: 1000 + i * 100,
                node_id,
                node_name: "TestNode".to_string(),
                thread_name: "main".to_string(),
                current_memory_bytes: 1000 + i as usize * 100,
                peak_memory_bytes: 2000 + i as usize * 100,
                processing_time_ms: 50,
                start_timestamp_ms: 950 + i * 100,
            });
        }

        // Add workflow events
        report.process_event(AnalyzerEvent::WorkflowStart {
            timestamp_ms: 1000,
            workflow_id: Uuid::new_v4(),
            workflow_name: "test".to_string(),
        });
        report.process_event(AnalyzerEvent::WorkflowEnd {
            timestamp_ms: 2000,
            workflow_id: Uuid::new_v4(),
            success: true,
        });

        report.finalize();

        let mem_report = report.memory_reports.get(&node_id.to_string()).unwrap();
        assert_eq!(mem_report.features_processed, 3);
        assert_eq!(mem_report.total_peak_memory, 2200);
        assert_eq!(mem_report.total_processing_time_ms, 150);
        assert_eq!(report.duration_ms, 1000);
        assert!(report.success);
    }

    #[test]
    fn test_save_and_load() {
        let mut report = AnalyzerReport::new();
        report.workflow_name = Some("test_workflow".to_string());

        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("report.json");

        report.save_to_file(&file_path).unwrap();
        let loaded = AnalyzerReport::load_from_file(&file_path).unwrap();

        assert_eq!(loaded.workflow_name, Some("test_workflow".to_string()));
    }

    #[test]
    fn test_quantization_basic() {
        let mut mem_report = NodeMemoryReport::new("test_node".to_string(), "TestNode".to_string());

        // Add 1000 data points with memory values that span from 0 to 10000
        // With resolution 100, quantum size will be 100 (10000/100)
        // So each step of 10 bytes won't trigger a new level, only every 100 bytes will
        for i in 0..1000 {
            mem_report.data_points.push(MemoryDataPoint {
                timestamp_ms: 1000 + i * 10,
                start_timestamp_ms: 999 + i * 10,
                thread_name: "main".to_string(),
                current_memory_bytes: i as usize * 10, // 0, 10, 20, ..., 9990
                peak_memory_bytes: i as usize * 10,
                processing_time_ms: 1,
            });
        }

        mem_report.quantize(100);

        // With max 9990 and resolution 100, quantum size = ceil(9990/100) = 100
        // So we keep points where floor(memory/100) changes
        // That's about 100 unique quantized levels out of 1000 points
        assert!(
            mem_report.quantized_data_points.len() < mem_report.data_points.len(),
            "quantized points ({}) should be less than raw points ({})",
            mem_report.quantized_data_points.len(),
            mem_report.data_points.len()
        );
        // Should be around 100 quantized points (one per level) + last point
        assert!(
            mem_report.quantized_data_points.len() <= 150,
            "Expected around 100 quantized points, got {}",
            mem_report.quantized_data_points.len()
        );
    }

    #[test]
    fn test_quantization_reduces_similar_values() {
        let mut mem_report = NodeMemoryReport::new("test_node".to_string(), "TestNode".to_string());

        // Add many data points with the same memory value
        // These should all be collapsed into just first and last
        for i in 0..1000 {
            mem_report.data_points.push(MemoryDataPoint {
                timestamp_ms: 1000 + i,
                start_timestamp_ms: 999 + i,
                thread_name: "main".to_string(),
                current_memory_bytes: 500, // All same value
                peak_memory_bytes: 500,
                processing_time_ms: 1,
            });
        }

        mem_report.quantize(100);

        // With all same values, should keep only first + last = 2 points
        assert_eq!(
            mem_report.quantized_data_points.len(),
            2,
            "Should keep only first and last point when all values are identical"
        );
        assert_eq!(mem_report.quantized_data_points[0].timestamp_ms, 1000);
        assert_eq!(mem_report.quantized_data_points[1].timestamp_ms, 1999);
    }

    #[test]
    fn test_quantization_preserves_changes() {
        let mut mem_report = NodeMemoryReport::new("test_node".to_string(), "TestNode".to_string());

        // Add data points with a clear step change
        // First half at 100, second half at 1000
        for i in 0..100 {
            let memory = if i < 50 { 100 } else { 1000 };
            mem_report.data_points.push(MemoryDataPoint {
                timestamp_ms: 1000 + i * 10,
                start_timestamp_ms: 999 + i * 10,
                thread_name: "main".to_string(),
                current_memory_bytes: memory,
                peak_memory_bytes: memory,
                processing_time_ms: 1,
            });
        }

        mem_report.quantize(100);

        // Should capture the transition point
        // First point at 100, then point at 1000, then last point
        assert!(
            mem_report.quantized_data_points.len() >= 2,
            "Should keep at least the change points"
        );

        // First point should be at 100
        assert_eq!(
            mem_report.quantized_data_points[0].current_memory_bytes,
            100
        );

        // Should have a point at 1000
        let has_high_point = mem_report
            .quantized_data_points
            .iter()
            .any(|p| p.current_memory_bytes == 1000);
        assert!(has_high_point, "Should preserve the high memory point");
    }

    #[test]
    fn test_quantization_empty_data() {
        let mut mem_report = NodeMemoryReport::new("test_node".to_string(), "TestNode".to_string());
        mem_report.quantize(100);
        assert!(mem_report.quantized_data_points.is_empty());
    }

    #[test]
    fn test_quantization_single_point() {
        let mut mem_report = NodeMemoryReport::new("test_node".to_string(), "TestNode".to_string());
        mem_report.data_points.push(MemoryDataPoint {
            timestamp_ms: 1000,
            start_timestamp_ms: 999,
            thread_name: "main".to_string(),
            current_memory_bytes: 500,
            peak_memory_bytes: 500,
            processing_time_ms: 1,
        });

        mem_report.quantize(100);

        assert_eq!(mem_report.quantized_data_points.len(), 1);
        assert_eq!(
            mem_report.quantized_data_points[0].current_memory_bytes,
            500
        );
    }

    #[test]
    fn test_finalize_applies_quantization() {
        let mut report = AnalyzerReport::new();
        let node_id = Uuid::new_v4();

        // Add many memory events with gradually increasing memory
        for i in 0..1000 {
            report.process_event(AnalyzerEvent::ActionMemory {
                timestamp_ms: 1000 + i,
                node_id,
                node_name: "TestNode".to_string(),
                thread_name: "main".to_string(),
                current_memory_bytes: i as usize * 10,
                peak_memory_bytes: i as usize * 10,
                processing_time_ms: 1,
                start_timestamp_ms: 999 + i,
            });
        }

        report.finalize();

        let mem_report = report.memory_reports.get(&node_id.to_string()).unwrap();

        // Raw data should be cleared after finalization to save space
        assert_eq!(
            mem_report.data_points.len(),
            0,
            "Raw data points should be cleared after finalization"
        );

        // Quantized data should have the reduced points
        assert!(
            !mem_report.quantized_data_points.is_empty(),
            "Quantized data points should not be empty"
        );

        // With default resolution of 100, we expect roughly 100 quantized points
        assert!(
            mem_report.quantized_data_points.len() <= 150,
            "Expected roughly 100 quantized points with resolution 100, got {}",
            mem_report.quantized_data_points.len()
        );
    }
}
