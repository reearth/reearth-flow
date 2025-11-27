# Engine Analyzer Design Document

## Overview

The analyzer is an optional feature for the Re:Earth Flow engine runtime that provides detailed memory and performance profiling capabilities. When enabled, it collects real-time data about memory usage per action and feature flow through edges, then generates a comprehensive report viewable in a Tauri desktop application.

## Architecture

### High-Level Components

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Engine Runtime                               │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                    Analyzer Feature Flag                       │   │
│  │  ┌─────────────────┐    ┌─────────────────┐                   │   │
│  │  │ Thread-Local    │    │ Feature Size    │                   │   │
│  │  │ Allocator Stats │    │ Measurement     │                   │   │
│  │  │ (Global Alloc)  │    │ (datasize)      │                   │   │
│  │  └────────┬────────┘    └────────┬────────┘                   │   │
│  │           │                      │                             │   │
│  │           ▼                      ▼                             │   │
│  │  ┌────────────────────────────────────────┐                   │   │
│  │  │         AnalyzerEvent Channel          │                   │   │
│  │  │     (crossbeam mpsc, bounded)          │                   │   │
│  │  └────────────────────┬───────────────────┘                   │   │
│  │                       │                                        │   │
│  │                       ▼                                        │   │
│  │  ┌────────────────────────────────────────┐                   │   │
│  │  │         Analyzer Sink (Async)          │                   │   │
│  │  │  - Collects all events                 │                   │   │
│  │  │  - Aggregates data by node/edge        │                   │   │
│  │  │  - Generates final report              │                   │   │
│  │  └────────────────────┬───────────────────┘                   │   │
│  └───────────────────────│──────────────────────────────────────┘   │
│                          │                                           │
│                          ▼                                           │
│  ┌──────────────────────────────────────────┐                       │
│  │           Analyzer Report (JSON)          │                       │
│  │  Saved to: <working_dir>/analyzer/        │                       │
│  └────────────────────────┬─────────────────┘                       │
└───────────────────────────│─────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     Tauri Analyzer App                               │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  React Frontend (TypeScript)                                  │   │
│  │  ┌─────────────────────┐    ┌─────────────────────┐         │   │
│  │  │  Memory Timeline    │    │  Feature Queue      │         │   │
│  │  │  Chart (per action) │    │  Chart (per node)   │         │   │
│  │  │  - Time vs Memory   │    │  - Time vs Count    │         │   │
│  │  │  - Hover: details   │    │  - Waiting + Active │         │   │
│  │  └─────────────────────┘    └─────────────────────┘         │   │
│  └──────────────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  Rust Backend (Tauri)                                         │   │
│  │  - Load analyzer reports                                      │   │
│  │  - Provide data to frontend                                   │   │
│  └──────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Part 1: Memory Usage Per Action

### 1.1 Thread-Local Allocator Statistics

Create a custom global allocator that tracks allocations per thread. This uses a wrapper around the system allocator with thread-local counters.

**Location**: `engine/analyzer/src/allocator.rs`

```rust
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};

// Thread-local storage for allocation tracking
thread_local! {
    static CURRENT_ALLOC: Cell<usize> = Cell::new(0);
    static PEAK_ALLOC: Cell<usize> = Cell::new(0);
    static TRACKING_ENABLED: Cell<bool> = Cell::new(false);
}

// Global flag to enable/disable tracking (compile-time feature + runtime check)
static ANALYZER_ENABLED: AtomicBool = AtomicBool::new(false);

pub struct AnalyzerAllocator<A: GlobalAlloc = System> {
    inner: A,
}

impl<A: GlobalAlloc> AnalyzerAllocator<A> {
    pub const fn new(inner: A) -> Self {
        Self { inner }
    }
}

unsafe impl<A: GlobalAlloc> GlobalAlloc for AnalyzerAllocator<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.inner.alloc(layout);
        if !ptr.is_null() && ANALYZER_ENABLED.load(Ordering::Relaxed) {
            TRACKING_ENABLED.with(|enabled| {
                if enabled.get() {
                    CURRENT_ALLOC.with(|current| {
                        let new_val = current.get() + layout.size();
                        current.set(new_val);
                        PEAK_ALLOC.with(|peak| {
                            if new_val > peak.get() {
                                peak.set(new_val);
                            }
                        });
                    });
                }
            });
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if ANALYZER_ENABLED.load(Ordering::Relaxed) {
            TRACKING_ENABLED.with(|enabled| {
                if enabled.get() {
                    CURRENT_ALLOC.with(|current| {
                        current.set(current.get().saturating_sub(layout.size()));
                    });
                }
            });
        }
        self.inner.dealloc(ptr, layout)
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = self.inner.realloc(ptr, layout, new_size);
        if !new_ptr.is_null() && ANALYZER_ENABLED.load(Ordering::Relaxed) {
            TRACKING_ENABLED.with(|enabled| {
                if enabled.get() {
                    CURRENT_ALLOC.with(|current| {
                        let old_size = layout.size();
                        let diff = new_size as isize - old_size as isize;
                        let new_val = (current.get() as isize + diff).max(0) as usize;
                        current.set(new_val);
                        PEAK_ALLOC.with(|peak| {
                            if new_val > peak.get() {
                                peak.set(new_val);
                            }
                        });
                    });
                }
            });
        }
        new_ptr
    }
}

// Helper functions for tracking
pub fn enable_analyzer() {
    ANALYZER_ENABLED.store(true, Ordering::SeqCst);
}

pub fn start_tracking() {
    TRACKING_ENABLED.with(|enabled| enabled.set(true));
    CURRENT_ALLOC.with(|c| c.set(0));
    PEAK_ALLOC.with(|p| p.set(0));
}

pub fn stop_tracking() -> (usize, usize) {
    TRACKING_ENABLED.with(|enabled| enabled.set(false));
    let current = CURRENT_ALLOC.with(|c| c.get());
    let peak = PEAK_ALLOC.with(|p| p.get());
    (current, peak)
}

pub fn get_current_stats() -> (usize, usize) {
    let current = CURRENT_ALLOC.with(|c| c.get());
    let peak = PEAK_ALLOC.with(|p| p.get());
    (current, peak)
}
```

### 1.2 Integration with process() Function

Modify `processor_node.rs` to wrap the `process()` call with memory tracking when the analyzer feature is enabled:

```rust
// In processor_node.rs, modify the process() function

#[cfg(feature = "analyzer")]
use crate::analyzer::{AnalyzerEvent, AnalyzerEventSender, allocator};

fn process(
    ctx: ExecutorContext,
    node_handle: NodeHandle,
    node_name: String,
    span: Span,
    event_hub: EventHub,
    channel_manager: Arc<parking_lot::RwLock<ProcessorChannelForwarder>>,
    processor: Arc<parking_lot::RwLock<Box<dyn Processor>>>,
    has_failed: Arc<std::sync::atomic::AtomicBool>,
    features_processed: Arc<AtomicU64>,
    #[cfg(feature = "analyzer")] analyzer_sender: Option<AnalyzerEventSender>,
) {
    let feature_id = ctx.feature.id;
    let channel_manager_guard = channel_manager.read();
    let mut processor_guard = processor.write();
    let channel_manager: &ProcessorChannelForwarder = &channel_manager_guard;
    let processor: &mut Box<dyn Processor> = &mut processor_guard;

    // Start memory tracking
    #[cfg(feature = "analyzer")]
    let tracking_start = if analyzer_sender.is_some() {
        allocator::start_tracking();
        Some(std::time::Instant::now())
    } else {
        None
    };

    let now = time::Instant::now();
    let result = processor.process(ctx, channel_manager);
    let elapsed = now.elapsed();
    let name = processor.name();

    // Stop tracking and report
    #[cfg(feature = "analyzer")]
    if let (Some(sender), Some(start_time)) = (&analyzer_sender, tracking_start) {
        let (current_mem, peak_mem) = allocator::stop_tracking();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let _ = sender.send(AnalyzerEvent::ActionMemory {
            timestamp,
            node_handle: node_handle.clone(),
            node_name: node_name.clone(),
            thread_name: std::thread::current().name().unwrap_or("unknown").to_string(),
            current_memory_bytes: current_mem,
            peak_memory_bytes: peak_mem,
            processing_time_ms: elapsed.as_millis() as u64,
        });
    }

    // ... rest of existing logic ...
}
```

---

## Part 2: Feature Flow Through Edges

### 2.1 Feature Size Measurement

Use the `datasize` crate to estimate the memory footprint of features as they pass through edges.

**Cargo.toml addition**:
```toml
[dependencies]
datasize = { version = "0.2", optional = true }

[features]
analyzer = ["datasize"]
```

**Feature struct modification** (in `runtime/types/src/feature.rs`):
```rust
#[cfg(feature = "analyzer")]
use datasize::DataSize;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "analyzer", derive(DataSize))]
pub struct Feature {
    pub id: uuid::Uuid,
    pub attributes: IndexMap<Attribute, AttributeValue>,
    pub metadata: Metadata,
    pub geometry: Geometry,
}
```

### 2.2 Edge Pass-Through Events

Modify the `ChannelManager::send_op()` in `forwarder.rs` to emit feature size events:

```rust
// In forwarder.rs

#[cfg(feature = "analyzer")]
use datasize::DataSize;
#[cfg(feature = "analyzer")]
use crate::analyzer::{AnalyzerEvent, AnalyzerEventSender};

impl ChannelManager {
    pub fn send_op(
        &self,
        mut ctx: ExecutorContext,
        #[cfg(feature = "analyzer")] analyzer_sender: Option<&AnalyzerEventSender>,
    ) -> Result<(), ExecutionError> {
        // ... existing logic ...

        #[cfg(feature = "analyzer")]
        if let Some(sender) = analyzer_sender {
            let feature_size = ctx.feature.estimate_heap_size();
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;

            for (port, edge_id) in &edges_to_send {
                let _ = sender.send(AnalyzerEvent::EdgeFeature {
                    timestamp,
                    edge_id: edge_id.clone(),
                    feature_id: ctx.feature.id,
                    feature_size_bytes: feature_size,
                    source_node: self.node_handle.clone(),
                });
            }
        }

        // ... rest of existing logic ...
    }
}
```

---

## Part 3: Analyzer Event System

### 3.1 Event Types

**Location**: `engine/analyzer/src/events.rs`

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalyzerEvent {
    /// Memory usage report from an action's process() call
    ActionMemory {
        timestamp: u64,           // Unix timestamp in milliseconds
        node_handle: NodeHandle,
        node_name: String,
        thread_name: String,
        current_memory_bytes: usize,
        peak_memory_bytes: usize,
        processing_time_ms: u64,
    },

    /// Feature passing through an edge
    EdgeFeature {
        timestamp: u64,
        edge_id: EdgeId,
        feature_id: Uuid,
        feature_size_bytes: usize,
        source_node: NodeHandle,
    },

    /// Node processing state change (for queue tracking)
    NodeProcessingState {
        timestamp: u64,
        node_handle: NodeHandle,
        features_waiting: u64,    // In channel queue
        features_processing: u64, // Currently being processed
    },

    /// Workflow start marker
    WorkflowStart {
        timestamp: u64,
        workflow_id: Uuid,
        workflow_name: String,
    },

    /// Workflow end marker
    WorkflowEnd {
        timestamp: u64,
        workflow_id: Uuid,
    },
}

pub type AnalyzerEventSender = crossbeam::channel::Sender<AnalyzerEvent>;
pub type AnalyzerEventReceiver = crossbeam::channel::Receiver<AnalyzerEvent>;
```

### 3.2 Analyzer Sink

**Location**: `engine/analyzer/src/sink.rs`

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Notify;
use crate::events::{AnalyzerEvent, AnalyzerEventReceiver};
use crate::report::AnalyzerReport;

pub struct AnalyzerSink {
    receiver: AnalyzerEventReceiver,
    shutdown: Arc<Notify>,
}

impl AnalyzerSink {
    pub fn new(receiver: AnalyzerEventReceiver, shutdown: Arc<Notify>) -> Self {
        Self { receiver, shutdown }
    }

    pub async fn run(self) -> AnalyzerReport {
        let mut report = AnalyzerReport::new();

        loop {
            tokio::select! {
                _ = self.shutdown.notified() => {
                    // Drain remaining events
                    while let Ok(event) = self.receiver.try_recv() {
                        report.process_event(event);
                    }
                    break;
                }
                result = tokio::task::spawn_blocking({
                    let receiver = self.receiver.clone();
                    move || receiver.recv_timeout(std::time::Duration::from_millis(100))
                }) => {
                    if let Ok(Ok(event)) = result {
                        report.process_event(event);
                    }
                }
            }
        }

        report.finalize();
        report
    }
}
```

---

## Part 4: Analyzer Report Format

### 4.1 Report Structure

**Location**: `engine/analyzer/src/report.rs`

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Memory data point for time-series graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDataPoint {
    pub timestamp: u64,
    pub thread_name: String,
    pub current_memory_bytes: usize,
    pub peak_memory_bytes: usize,
    pub processing_time_ms: u64,
}

/// Feature queue data point for time-series graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueDataPoint {
    pub timestamp: u64,
    pub features_waiting: u64,
    pub features_processing: u64,
}

/// Node information for the report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub node_id: String,
    pub node_name: String,
    pub action_type: String,
}

/// Edge information for the report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeInfo {
    pub edge_id: String,
    pub source_node_id: String,
    pub target_node_id: String,
    pub total_features: u64,
    pub total_bytes: usize,
    pub avg_feature_size: usize,
}

/// Per-node memory report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMemoryReport {
    pub info: NodeInfo,
    pub data_points: Vec<MemoryDataPoint>,
    pub total_peak_memory: usize,
    pub avg_memory: usize,
    pub total_processing_time_ms: u64,
}

/// Per-node queue report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeQueueReport {
    pub info: NodeInfo,
    pub data_points: Vec<QueueDataPoint>,
    pub max_queue_depth: u64,
    pub avg_queue_depth: f64,
}

/// Complete analyzer report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzerReport {
    pub version: String,
    pub workflow_id: Option<Uuid>,
    pub workflow_name: Option<String>,
    pub start_time: u64,
    pub end_time: u64,
    pub duration_ms: u64,

    /// Memory reports keyed by node_id
    pub memory_reports: HashMap<String, NodeMemoryReport>,

    /// Queue reports keyed by node_id
    pub queue_reports: HashMap<String, NodeQueueReport>,

    /// Edge statistics
    pub edge_reports: HashMap<String, EdgeInfo>,

    /// Summary statistics
    pub summary: AnalyzerSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzerSummary {
    pub total_features_processed: u64,
    pub total_bytes_transferred: usize,
    pub peak_memory_usage: usize,
    pub slowest_action: Option<String>,
    pub highest_memory_action: Option<String>,
}

impl AnalyzerReport {
    pub fn new() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            workflow_id: None,
            workflow_name: None,
            start_time: 0,
            end_time: 0,
            duration_ms: 0,
            memory_reports: HashMap::new(),
            queue_reports: HashMap::new(),
            edge_reports: HashMap::new(),
            summary: AnalyzerSummary::default(),
        }
    }

    pub fn process_event(&mut self, event: AnalyzerEvent) {
        match event {
            AnalyzerEvent::ActionMemory {
                timestamp, node_handle, node_name, thread_name,
                current_memory_bytes, peak_memory_bytes, processing_time_ms
            } => {
                let report = self.memory_reports
                    .entry(node_handle.id.to_string())
                    .or_insert_with(|| NodeMemoryReport {
                        info: NodeInfo {
                            node_id: node_handle.id.to_string(),
                            node_name: node_name.clone(),
                            action_type: String::new(), // Set from processor name
                        },
                        data_points: Vec::new(),
                        total_peak_memory: 0,
                        avg_memory: 0,
                        total_processing_time_ms: 0,
                    });

                report.data_points.push(MemoryDataPoint {
                    timestamp,
                    thread_name,
                    current_memory_bytes,
                    peak_memory_bytes,
                    processing_time_ms,
                });
            }

            AnalyzerEvent::EdgeFeature {
                timestamp, edge_id, feature_id, feature_size_bytes, source_node
            } => {
                let report = self.edge_reports
                    .entry(edge_id.to_string())
                    .or_insert_with(|| EdgeInfo {
                        edge_id: edge_id.to_string(),
                        source_node_id: source_node.id.to_string(),
                        target_node_id: String::new(),
                        total_features: 0,
                        total_bytes: 0,
                        avg_feature_size: 0,
                    });

                report.total_features += 1;
                report.total_bytes += feature_size_bytes;
            }

            AnalyzerEvent::NodeProcessingState {
                timestamp, node_handle, features_waiting, features_processing
            } => {
                let report = self.queue_reports
                    .entry(node_handle.id.to_string())
                    .or_insert_with(|| NodeQueueReport {
                        info: NodeInfo {
                            node_id: node_handle.id.to_string(),
                            node_name: String::new(),
                            action_type: String::new(),
                        },
                        data_points: Vec::new(),
                        max_queue_depth: 0,
                        avg_queue_depth: 0.0,
                    });

                report.data_points.push(QueueDataPoint {
                    timestamp,
                    features_waiting,
                    features_processing,
                });
            }

            AnalyzerEvent::WorkflowStart { timestamp, workflow_id, workflow_name } => {
                self.start_time = timestamp;
                self.workflow_id = Some(workflow_id);
                self.workflow_name = Some(workflow_name);
            }

            AnalyzerEvent::WorkflowEnd { timestamp, workflow_id } => {
                self.end_time = timestamp;
                self.duration_ms = self.end_time.saturating_sub(self.start_time);
            }
        }
    }

    pub fn finalize(&mut self) {
        // Calculate averages and summaries
        for (_, report) in &mut self.memory_reports {
            if !report.data_points.is_empty() {
                report.total_peak_memory = report.data_points
                    .iter()
                    .map(|p| p.peak_memory_bytes)
                    .max()
                    .unwrap_or(0);

                let sum: usize = report.data_points
                    .iter()
                    .map(|p| p.current_memory_bytes)
                    .sum();
                report.avg_memory = sum / report.data_points.len();

                report.total_processing_time_ms = report.data_points
                    .iter()
                    .map(|p| p.processing_time_ms)
                    .sum();
            }
        }

        for (_, report) in &mut self.edge_reports {
            if report.total_features > 0 {
                report.avg_feature_size = report.total_bytes / report.total_features as usize;
            }
        }

        for (_, report) in &mut self.queue_reports {
            if !report.data_points.is_empty() {
                report.max_queue_depth = report.data_points
                    .iter()
                    .map(|p| p.features_waiting + p.features_processing)
                    .max()
                    .unwrap_or(0);

                let sum: u64 = report.data_points
                    .iter()
                    .map(|p| p.features_waiting + p.features_processing)
                    .sum();
                report.avg_queue_depth = sum as f64 / report.data_points.len() as f64;
            }
        }

        // Compute summary
        self.summary.total_features_processed = self.edge_reports
            .values()
            .map(|e| e.total_features)
            .sum();

        self.summary.total_bytes_transferred = self.edge_reports
            .values()
            .map(|e| e.total_bytes)
            .sum();

        self.summary.peak_memory_usage = self.memory_reports
            .values()
            .map(|r| r.total_peak_memory)
            .max()
            .unwrap_or(0);
    }

    pub fn save_to_file(&self, path: &std::path::Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)
    }
}
```

---

## Part 5: Tauri Desktop Application

### 5.1 Project Structure

```
engine/analyzer/
├── Cargo.toml                    # Rust library crate
├── src/
│   ├── lib.rs                    # Library exports
│   ├── allocator.rs              # Thread-local allocator
│   ├── events.rs                 # Event types
│   ├── sink.rs                   # Event sink
│   └── report.rs                 # Report structure
├── src-tauri/                    # Tauri backend
│   ├── Cargo.toml
│   ├── build.rs
│   ├── tauri.conf.json
│   ├── icons/
│   └── src/
│       ├── main.rs               # Tauri app entry
│       └── commands.rs           # Tauri commands
├── src/                          # React frontend
│   ├── App.tsx
│   ├── main.tsx
│   ├── index.css
│   ├── components/
│   │   ├── MemoryChart.tsx       # Memory time-series chart
│   │   ├── QueueChart.tsx        # Queue depth chart
│   │   ├── ReportSummary.tsx     # Summary stats
│   │   ├── NodeSelector.tsx      # Node selection
│   │   └── Tooltip.tsx           # Hover tooltip
│   └── hooks/
│       └── useAnalyzerReport.ts  # Report loading hook
├── package.json
├── tsconfig.json
├── vite.config.ts
└── tailwind.config.ts
```

### 5.2 Tauri Backend Commands

**Location**: `engine/analyzer/src-tauri/src/commands.rs`

```rust
use std::path::PathBuf;
use tauri::State;
use reearth_flow_analyzer::report::AnalyzerReport;

pub struct AppState {
    pub reports_dir: PathBuf,
}

#[tauri::command]
pub fn list_reports(state: State<AppState>) -> Result<Vec<String>, String> {
    let mut reports = Vec::new();

    let entries = std::fs::read_dir(&state.reports_dir)
        .map_err(|e| format!("Failed to read reports directory: {}", e))?;

    for entry in entries.flatten() {
        if let Some(name) = entry.file_name().to_str() {
            if name.ends_with(".json") {
                reports.push(name.to_string());
            }
        }
    }

    reports.sort_by(|a, b| b.cmp(a)); // Most recent first
    Ok(reports)
}

#[tauri::command]
pub fn load_report(state: State<AppState>, filename: String) -> Result<AnalyzerReport, String> {
    let path = state.reports_dir.join(&filename);
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read report: {}", e))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse report: {}", e))
}

#[tauri::command]
pub fn get_node_memory_data(
    state: State<AppState>,
    filename: String,
    node_id: String,
) -> Result<Vec<MemoryDataPoint>, String> {
    let report = load_report(state, filename)?;

    report.memory_reports
        .get(&node_id)
        .map(|r| r.data_points.clone())
        .ok_or_else(|| "Node not found".to_string())
}

#[tauri::command]
pub fn get_node_queue_data(
    state: State<AppState>,
    filename: String,
    node_id: String,
) -> Result<Vec<QueueDataPoint>, String> {
    let report = load_report(state, filename)?;

    report.queue_reports
        .get(&node_id)
        .map(|r| r.data_points.clone())
        .ok_or_else(|| "Node not found".to_string())
}
```

### 5.3 React Frontend Components

**Memory Chart Component** (`engine/analyzer/src/components/MemoryChart.tsx`):

```tsx
import React, { useMemo, useState } from 'react';
import {
  LineChart, Line, XAxis, YAxis, CartesianGrid,
  Tooltip, Legend, ResponsiveContainer
} from 'recharts';
import { MemoryDataPoint } from '../types';

interface MemoryChartProps {
  data: MemoryDataPoint[];
  nodeName: string;
}

const formatBytes = (bytes: number): string => {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
};

const formatTime = (timestamp: number, startTime: number): string => {
  const elapsed = timestamp - startTime;
  return `${(elapsed / 1000).toFixed(2)}s`;
};

const CustomTooltip = ({ active, payload, label, startTime }: any) => {
  if (active && payload && payload.length) {
    const data = payload[0].payload;
    return (
      <div className="bg-white p-3 border rounded shadow-lg">
        <p className="font-semibold">Time: {formatTime(data.timestamp, startTime)}</p>
        <p>Thread: {data.thread_name}</p>
        <p>Current Memory: {formatBytes(data.current_memory_bytes)}</p>
        <p>Peak Memory: {formatBytes(data.peak_memory_bytes)}</p>
        <p>Processing Time: {data.processing_time_ms}ms</p>
      </div>
    );
  }
  return null;
};

export const MemoryChart: React.FC<MemoryChartProps> = ({ data, nodeName }) => {
  const startTime = useMemo(() =>
    data.length > 0 ? data[0].timestamp : 0,
    [data]
  );

  const chartData = useMemo(() =>
    data.map(point => ({
      ...point,
      relativeTime: (point.timestamp - startTime) / 1000,
    })),
    [data, startTime]
  );

  return (
    <div className="w-full h-96">
      <h3 className="text-lg font-semibold mb-2">
        Memory Usage: {nodeName}
      </h3>
      <ResponsiveContainer width="100%" height="100%">
        <LineChart data={chartData}>
          <CartesianGrid strokeDasharray="3 3" />
          <XAxis
            dataKey="relativeTime"
            label={{ value: 'Time (s)', position: 'bottom' }}
          />
          <YAxis
            tickFormatter={formatBytes}
            label={{ value: 'Memory', angle: -90, position: 'left' }}
          />
          <Tooltip content={<CustomTooltip startTime={startTime} />} />
          <Legend />
          <Line
            type="monotone"
            dataKey="current_memory_bytes"
            name="Current Memory"
            stroke="#8884d8"
            dot={false}
          />
          <Line
            type="monotone"
            dataKey="peak_memory_bytes"
            name="Peak Memory"
            stroke="#82ca9d"
            dot={false}
          />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
};
```

**Queue Chart Component** (`engine/analyzer/src/components/QueueChart.tsx`):

```tsx
import React, { useMemo } from 'react';
import {
  AreaChart, Area, XAxis, YAxis, CartesianGrid,
  Tooltip, Legend, ResponsiveContainer
} from 'recharts';
import { QueueDataPoint } from '../types';

interface QueueChartProps {
  data: QueueDataPoint[];
  nodeName: string;
}

const CustomTooltip = ({ active, payload, startTime }: any) => {
  if (active && payload && payload.length) {
    const data = payload[0].payload;
    const total = data.features_waiting + data.features_processing;
    return (
      <div className="bg-white p-3 border rounded shadow-lg">
        <p className="font-semibold">
          Time: {((data.timestamp - startTime) / 1000).toFixed(2)}s
        </p>
        <p>Waiting: {data.features_waiting}</p>
        <p>Processing: {data.features_processing}</p>
        <p className="font-semibold">Total: {total}</p>
      </div>
    );
  }
  return null;
};

export const QueueChart: React.FC<QueueChartProps> = ({ data, nodeName }) => {
  const startTime = useMemo(() =>
    data.length > 0 ? data[0].timestamp : 0,
    [data]
  );

  const chartData = useMemo(() =>
    data.map(point => ({
      ...point,
      relativeTime: (point.timestamp - startTime) / 1000,
      total: point.features_waiting + point.features_processing,
    })),
    [data, startTime]
  );

  return (
    <div className="w-full h-96">
      <h3 className="text-lg font-semibold mb-2">
        Feature Queue: {nodeName}
      </h3>
      <ResponsiveContainer width="100%" height="100%">
        <AreaChart data={chartData}>
          <CartesianGrid strokeDasharray="3 3" />
          <XAxis
            dataKey="relativeTime"
            label={{ value: 'Time (s)', position: 'bottom' }}
          />
          <YAxis
            label={{ value: 'Features', angle: -90, position: 'left' }}
          />
          <Tooltip content={<CustomTooltip startTime={startTime} />} />
          <Legend />
          <Area
            type="monotone"
            dataKey="features_processing"
            name="Processing"
            stackId="1"
            stroke="#82ca9d"
            fill="#82ca9d"
          />
          <Area
            type="monotone"
            dataKey="features_waiting"
            name="Waiting"
            stackId="1"
            stroke="#8884d8"
            fill="#8884d8"
          />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  );
};
```

---

## Part 6: Integration with Runtime

### 6.1 Feature Flag Integration

**In `runtime/runtime/Cargo.toml`**:
```toml
[features]
default = []
analyzer = ["reearth-flow-analyzer"]

[dependencies]
reearth-flow-analyzer = { path = "../../analyzer", optional = true }
```

### 6.2 Runtime Integration Points

1. **DAG Executor** (`dag_executor.rs`):
   - Create analyzer channel and sink when feature enabled
   - Pass analyzer sender to nodes
   - Collect report at shutdown

2. **Processor Node** (`processor_node.rs`):
   - Wrap process() calls with memory tracking
   - Report queue state periodically

3. **Channel Manager** (`forwarder.rs`):
   - Measure feature size on send
   - Emit edge feature events

4. **Source Node** (`source_node.rs`):
   - Track initial feature emissions

5. **Sink Node** (`sink_node.rs`):
   - Track final feature consumption

### 6.3 Report Storage Location

Reports are saved to:
```
<working_directory>/projects/<project_key>/jobs/<job_id>/analyzer/
    report_<timestamp>.json
```

This integrates with the existing working directory structure.

---

## Summary

This design provides:

1. **Memory Tracking**: Thread-local allocator statistics wrapped around process() calls
2. **Feature Flow Tracking**: datasize-based estimation at edge crossings
3. **Async Event Collection**: Bounded channel + dedicated sink thread
4. **Comprehensive Reports**: JSON format with time-series data for visualization
5. **Desktop Viewer**: Tauri app with React frontend using recharts for visualization
6. **Optional Feature**: Compile-time flag to avoid overhead when not needed

The analyzer is designed to:
- Have minimal overhead when disabled (compile-time feature)
- Use efficient bounded channels for event collection
- Provide accurate timing with millisecond resolution
- Generate reports compatible with the existing storage structure
- Offer interactive visualization with hover tooltips showing detailed information
