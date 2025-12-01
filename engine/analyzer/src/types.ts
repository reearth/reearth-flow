// Types matching the Rust backend

export interface MemoryDataPoint {
  /** Unix timestamp in milliseconds when tracking ended (after process()/finish()). */
  timestamp_ms: number;
  /** Unix timestamp in milliseconds when tracking started (before process()/finish()). */
  start_timestamp_ms: number;
  thread_name: string;
  current_memory_bytes: number;
  peak_memory_bytes: number;
  processing_time_ms: number;
}

export interface QueueDataPoint {
  timestamp_ms: number;
  features_waiting: number;
  features_processing: number;
}

export interface NodeInfo {
  node_id: string;
  node_name: string;
}

export interface EdgeInfo {
  edge_id: string;
  source_node_id: string;
  total_features: number;
  total_bytes: number;
  avg_feature_size: number;
  min_feature_size: number;
  max_feature_size: number;
}

export interface NodeMemoryReport {
  info: NodeInfo;
  data_points: MemoryDataPoint[];
  /** Quantized data points for efficient graphing. Only includes points where the quantized memory value changed. */
  quantized_data_points: MemoryDataPoint[];
  total_peak_memory: number;
  avg_memory: number;
  total_processing_time_ms: number;
  features_processed: number;
}

export interface NodeQueueReport {
  info: NodeInfo;
  data_points: QueueDataPoint[];
  /** Quantized data points for efficient graphing. Only includes points where the quantized queue depth changed. */
  quantized_data_points: QueueDataPoint[];
  max_queue_depth: number;
  avg_queue_depth: number;
}

export interface AnalyzerSummary {
  total_features_processed: number;
  total_bytes_transferred: number;
  peak_memory_usage: number;
  slowest_node: string | null;
  slowest_avg_time_ms: number | null;
  highest_memory_node: string | null;
}

export interface AnalyzerReport {
  version: string;
  workflow_id: string | null;
  workflow_name: string | null;
  start_time_ms: number;
  end_time_ms: number;
  duration_ms: number;
  success: boolean;
  memory_reports: Record<string, NodeMemoryReport>;
  queue_reports: Record<string, NodeQueueReport>;
  edge_reports: Record<string, EdgeInfo>;
  summary: AnalyzerSummary;
}

export interface ReportInfo {
  filename: string;
  workflow_name: string | null;
  workflow_id: string | null;
  start_time_ms: number;
  duration_ms: number;
  success: boolean;
}

export interface NodeSummary {
  node_id: string;
  node_name: string;
  has_memory_data: boolean;
  has_queue_data: boolean;
  features_processed: number;
  total_peak_memory: number;
  total_processing_time_ms: number;
}
