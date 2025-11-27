use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Notify;

use crate::events::{AnalyzerEvent, AnalyzerEventReceiver};
use crate::report::AnalyzerReport;

/// The analyzer sink collects events from the engine and builds a report.
pub struct AnalyzerSink {
    receiver: AnalyzerEventReceiver,
    shutdown: Arc<Notify>,
    report: AnalyzerReport,
}

impl AnalyzerSink {
    /// Create a new analyzer sink.
    pub fn new(receiver: AnalyzerEventReceiver, shutdown: Arc<Notify>) -> Self {
        Self {
            receiver,
            shutdown,
            report: AnalyzerReport::new(),
        }
    }

    /// Run the sink, collecting events until shutdown is signaled.
    /// Returns the finalized report.
    pub async fn run(mut self) -> AnalyzerReport {
        loop {
            tokio::select! {
                biased;

                _ = self.shutdown.notified() => {
                    // Drain remaining events
                    tracing::info!("Analyzer sink received shutdown signal, draining remaining events");
                    self.drain_remaining();
                    break;
                }

                result = self.recv_event() => {
                    match result {
                        Ok(Some(event)) => self.report.process_event(event),
                        Ok(None) => {
                            // Timeout, no event available, continue waiting
                            continue;
                        }
                        Err(()) => {
                            // Channel disconnected
                            tracing::info!("Analyzer sink channel disconnected");
                            break;
                        }
                    }
                }
            }
        }

        self.report.finalize();
        self.report
    }

    /// Run the sink synchronously (blocking).
    /// Returns the finalized report.
    pub fn run_sync(mut self) -> AnalyzerReport {
        loop {
            match self.receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(event) => self.report.process_event(event),
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                    // Check if we should shutdown
                    // In sync mode, we rely on the channel being closed
                    continue;
                }
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                    // Channel closed, drain and exit
                    self.drain_remaining();
                    break;
                }
            }
        }

        self.report.finalize();
        self.report
    }

    /// Receive a single event asynchronously.
    /// Returns Ok(Some(event)) if an event was received.
    /// Returns Ok(None) if timeout occurred (no event available).
    /// Returns Err(()) if channel is disconnected.
    async fn recv_event(&self) -> Result<Option<AnalyzerEvent>, ()> {
        let receiver = self.receiver.clone();
        match tokio::task::spawn_blocking(move || receiver.recv_timeout(Duration::from_millis(100)))
            .await
        {
            Ok(Ok(event)) => Ok(Some(event)),
            Ok(Err(crossbeam_channel::RecvTimeoutError::Timeout)) => Ok(None),
            Ok(Err(crossbeam_channel::RecvTimeoutError::Disconnected)) => Err(()),
            Err(_) => Err(()), // spawn_blocking panicked
        }
    }

    /// Drain all remaining events from the channel.
    fn drain_remaining(&mut self) {
        while let Ok(event) = self.receiver.try_recv() {
            self.report.process_event(event);
        }
    }

    /// Get a reference to the current report (not finalized).
    pub fn current_report(&self) -> &AnalyzerReport {
        &self.report
    }
}

/// Builder for creating and running an analyzer sink with auto-save functionality.
pub struct AnalyzerSinkBuilder {
    receiver: AnalyzerEventReceiver,
    shutdown: Arc<Notify>,
    output_path: Option<std::path::PathBuf>,
}

impl AnalyzerSinkBuilder {
    /// Create a new builder.
    pub fn new(receiver: AnalyzerEventReceiver, shutdown: Arc<Notify>) -> Self {
        Self {
            receiver,
            shutdown,
            output_path: None,
        }
    }

    /// Set the output path for auto-saving the report.
    pub fn with_output_path(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.output_path = Some(path.into());
        self
    }

    /// Build and run the sink asynchronously, returning the report.
    pub async fn run(self) -> std::io::Result<AnalyzerReport> {
        let sink = AnalyzerSink::new(self.receiver, self.shutdown);
        let report = sink.run().await;

        if let Some(path) = &self.output_path {
            // Ensure parent directory exists
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            report.save_to_file(path)?;
        }

        Ok(report)
    }

    /// Build and run the sink synchronously, returning the report.
    pub fn run_sync(self) -> std::io::Result<AnalyzerReport> {
        let sink = AnalyzerSink::new(self.receiver, self.shutdown);
        let report = sink.run_sync();

        if let Some(path) = &self.output_path {
            // Ensure parent directory exists
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            report.save_to_file(path)?;
        }

        Ok(report)
    }
}

/// Generate a report filename with timestamp.
pub fn generate_report_filename() -> String {
    let now = chrono::Utc::now();
    format!("analyzer_report_{}.json", now.format("%Y%m%d_%H%M%S"))
}

/// Get the default analyzer reports directory.
pub fn default_reports_dir() -> std::path::PathBuf {
    directories::ProjectDirs::from("com", "eukarya", "reearth-flow")
        .map(|dirs| dirs.data_dir().join("analyzer"))
        .unwrap_or_else(|| std::path::PathBuf::from("analyzer"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{create_channel, EdgeId};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_sink_collects_events() {
        let (sender, receiver) = create_channel(100);
        let shutdown = Arc::new(Notify::new());
        let shutdown_clone = shutdown.clone();

        // Spawn the sink
        let handle = tokio::spawn(async move {
            let sink = AnalyzerSink::new(receiver, shutdown_clone);
            sink.run().await
        });

        // Send some events
        let workflow_id = Uuid::new_v4();
        sender
            .send(AnalyzerEvent::WorkflowStart {
                timestamp_ms: 1000,
                workflow_id,
                workflow_name: "test".to_string(),
            })
            .unwrap();

        sender
            .send(AnalyzerEvent::EdgeFeature {
                timestamp_ms: 1100,
                edge_id: EdgeId::new("edge1"),
                feature_id: Uuid::new_v4(),
                feature_size_bytes: 512,
                source_node_id: Uuid::new_v4(),
            })
            .unwrap();

        sender
            .send(AnalyzerEvent::WorkflowEnd {
                timestamp_ms: 2000,
                workflow_id,
                success: true,
            })
            .unwrap();

        // Give sink time to process
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Signal shutdown
        shutdown.notify_one();

        let report = handle.await.unwrap();
        assert_eq!(report.workflow_name, Some("test".to_string()));
        assert_eq!(report.edge_reports.len(), 1);
        assert!(report.success);
    }

    #[test]
    fn test_sink_sync() {
        let (sender, receiver) = create_channel(100);
        let shutdown = Arc::new(Notify::new());

        // Send events before running sink
        let workflow_id = Uuid::new_v4();
        sender
            .send(AnalyzerEvent::WorkflowStart {
                timestamp_ms: 1000,
                workflow_id,
                workflow_name: "test_sync".to_string(),
            })
            .unwrap();

        sender
            .send(AnalyzerEvent::WorkflowEnd {
                timestamp_ms: 2000,
                workflow_id,
                success: true,
            })
            .unwrap();

        // Drop sender to close channel
        drop(sender);

        let sink = AnalyzerSink::new(receiver, shutdown);
        let report = sink.run_sync();

        assert_eq!(report.workflow_name, Some("test_sync".to_string()));
        assert!(report.success);
    }

    #[test]
    fn test_generate_report_filename() {
        let filename = generate_report_filename();
        assert!(filename.starts_with("analyzer_report_"));
        assert!(filename.ends_with(".json"));
    }
}
