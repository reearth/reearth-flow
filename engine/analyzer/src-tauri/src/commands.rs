use std::path::PathBuf;
use std::sync::Mutex;

use reearth_flow_analyzer::{AnalyzerReport, NodeMemoryReport, NodeQueueReport};
use tauri::State;

use crate::error::Error;

/// Application state containing the reports directory.
pub struct AppState {
    pub reports_dir: Mutex<PathBuf>,
}

impl AppState {
    pub fn new(reports_dir: PathBuf) -> Self {
        Self {
            reports_dir: Mutex::new(reports_dir),
        }
    }

    pub fn get_reports_dir(&self) -> PathBuf {
        self.reports_dir.lock().unwrap().clone()
    }

    pub fn set_reports_dir(&self, path: PathBuf) {
        *self.reports_dir.lock().unwrap() = path;
    }
}

/// List all available analyzer reports.
#[tauri::command]
pub fn list_reports(state: State<AppState>) -> Result<Vec<ReportInfo>, Error> {
    let reports_dir = state.get_reports_dir();

    if !reports_dir.exists() {
        return Ok(Vec::new());
    }

    let mut reports = Vec::new();

    for entry in std::fs::read_dir(&reports_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map(|e| e == "json").unwrap_or(false) {
            if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                // Try to extract metadata from the report
                if let Ok(report) = AnalyzerReport::load_from_file(&path) {
                    reports.push(ReportInfo {
                        filename: filename.to_string(),
                        workflow_name: report.workflow_name,
                        workflow_id: report.workflow_id.map(|id| id.to_string()),
                        start_time_ms: report.start_time_ms,
                        duration_ms: report.duration_ms,
                        success: report.success,
                    });
                } else {
                    // Include file even if we can't parse it
                    reports.push(ReportInfo {
                        filename: filename.to_string(),
                        workflow_name: None,
                        workflow_id: None,
                        start_time_ms: 0,
                        duration_ms: 0,
                        success: false,
                    });
                }
            }
        }
    }

    // Sort by start time, most recent first
    reports.sort_by(|a, b| b.start_time_ms.cmp(&a.start_time_ms));

    Ok(reports)
}

/// Load a specific analyzer report.
#[tauri::command]
pub fn load_report(state: State<AppState>, filename: String) -> Result<AnalyzerReport, Error> {
    let reports_dir = state.get_reports_dir();
    let path = reports_dir.join(&filename);

    if !path.exists() {
        return Err(Error::ReportNotFound(filename));
    }

    AnalyzerReport::load_from_file(&path).map_err(Error::Io)
}

/// Get memory data points for a specific node.
#[tauri::command]
pub fn get_node_memory_data(
    state: State<AppState>,
    filename: String,
    node_id: String,
) -> Result<NodeMemoryReport, Error> {
    let report = load_report(state, filename)?;

    report
        .memory_reports
        .get(&node_id)
        .cloned()
        .ok_or(Error::NodeNotFound(node_id))
}

/// Get queue data points for a specific node.
#[tauri::command]
pub fn get_node_queue_data(
    state: State<AppState>,
    filename: String,
    node_id: String,
) -> Result<NodeQueueReport, Error> {
    let report = load_report(state, filename)?;

    report
        .queue_reports
        .get(&node_id)
        .cloned()
        .ok_or(Error::NodeNotFound(node_id))
}

/// Set the reports directory.
#[tauri::command]
pub fn set_reports_directory(state: State<AppState>, path: String) -> Result<(), Error> {
    let path = PathBuf::from(&path);

    if !path.exists() {
        return Err(Error::InvalidPath(format!(
            "Path does not exist: {}",
            path.display()
        )));
    }

    if !path.is_dir() {
        return Err(Error::InvalidPath(format!(
            "Path is not a directory: {}",
            path.display()
        )));
    }

    state.set_reports_dir(path);
    Ok(())
}

/// Get the current reports directory.
#[tauri::command]
pub fn get_reports_directory(state: State<AppState>) -> String {
    state.get_reports_dir().display().to_string()
}

/// Get a list of all nodes in a report.
#[tauri::command]
pub fn get_report_nodes(
    state: State<AppState>,
    filename: String,
) -> Result<Vec<NodeSummary>, Error> {
    let report = load_report(state, filename)?;

    let mut nodes = Vec::new();

    for (node_id, mem_report) in &report.memory_reports {
        nodes.push(NodeSummary {
            node_id: node_id.clone(),
            node_name: mem_report.info.node_name.clone(),
            has_memory_data: !mem_report.quantized_data_points.is_empty(),
            has_queue_data: report.queue_reports.contains_key(node_id),
            features_processed: mem_report.features_processed,
            total_peak_memory: mem_report.total_peak_memory,
            total_processing_time_ms: mem_report.total_processing_time_ms,
        });
    }

    // Add nodes that only have queue data
    for (node_id, queue_report) in &report.queue_reports {
        if !report.memory_reports.contains_key(node_id) {
            nodes.push(NodeSummary {
                node_id: node_id.clone(),
                node_name: queue_report.info.node_name.clone(),
                has_memory_data: false,
                has_queue_data: true,
                features_processed: 0,
                total_peak_memory: 0,
                total_processing_time_ms: 0,
            });
        }
    }

    Ok(nodes)
}

/// Report information for listing.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReportInfo {
    pub filename: String,
    pub workflow_name: Option<String>,
    pub workflow_id: Option<String>,
    pub start_time_ms: u64,
    pub duration_ms: u64,
    pub success: bool,
}

/// Node summary information.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NodeSummary {
    pub node_id: String,
    pub node_name: String,
    pub has_memory_data: bool,
    pub has_queue_data: bool,
    pub features_processed: u64,
    pub total_peak_memory: usize,
    pub total_processing_time_ms: u64,
}
