//! Re:Earth Flow Analyzer
//!
//! This crate provides memory and performance analysis tools for the Re:Earth Flow engine.
//! When enabled, it tracks:
//!
//! - Memory usage per action during `process()` calls
//! - Feature sizes as they flow through edges
//! - Queue depths at each node
//!
//! The analyzer generates JSON reports that can be viewed in the companion Tauri desktop app.
//!
//! # Usage
//!
//! The analyzer is typically enabled via the `analyzer` feature flag on the runtime crate.
//! When enabled, the engine will emit events to a dedicated channel, which are collected
//! by an async sink and aggregated into a report.
//!
//! ```rust,ignore
//! use reearth_flow_analyzer::{
//!     allocator,
//!     events::{create_channel, AnalyzerEvent, DEFAULT_CHANNEL_CAPACITY},
//!     sink::AnalyzerSinkBuilder,
//! };
//! use std::sync::Arc;
//! use tokio::sync::Notify;
//!
//! // Enable the allocator
//! allocator::enable_analyzer();
//!
//! // Create event channel
//! let (sender, receiver) = create_channel(DEFAULT_CHANNEL_CAPACITY);
//! let shutdown = Arc::new(Notify::new());
//!
//! // Run the sink in a background task
//! let sink_handle = tokio::spawn({
//!     let shutdown = shutdown.clone();
//!     async move {
//!         AnalyzerSinkBuilder::new(receiver, shutdown)
//!             .with_output_path("./analyzer_report.json")
//!             .run()
//!             .await
//!     }
//! });
//!
//! // Send events during workflow execution...
//! sender.send(AnalyzerEvent::WorkflowStart {
//!     timestamp_ms: AnalyzerEvent::now_ms(),
//!     workflow_id: uuid::Uuid::new_v4(),
//!     workflow_name: "my_workflow".to_string(),
//! }).ok();
//!
//! // When done, signal shutdown
//! shutdown.notify_one();
//! let report = sink_handle.await.unwrap().unwrap();
//! ```

pub mod allocator;
pub mod events;
pub mod report;
pub mod sink;
pub mod size;

pub use allocator::{
    disable_analyzer, enable_analyzer, get_current_memory, get_current_stats, is_analyzer_enabled,
    is_tracking, start_tracking, stop_tracking, AnalyzerAllocator, TrackingGuard,
};
pub use events::{
    create_channel, AnalyzerEvent, AnalyzerEventReceiver, AnalyzerEventSender, EdgeId, NodeHandle,
    DEFAULT_CHANNEL_CAPACITY,
};
pub use report::{
    AnalyzerReport, AnalyzerSummary, EdgeInfo, MemoryDataPoint, NodeInfo, NodeMemoryReport,
    NodeQueueReport, QueueDataPoint, DEFAULT_QUANTIZATION_RESOLUTION,
};
pub use sink::{default_reports_dir, generate_report_filename, AnalyzerSink, AnalyzerSinkBuilder};
pub use size::estimate_size;
