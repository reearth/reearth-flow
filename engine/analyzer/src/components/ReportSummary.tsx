import React from "react";
import type { AnalyzerReport } from "../types";

interface ReportSummaryProps {
  report: AnalyzerReport;
}

const formatBytes = (bytes: number): string => {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`;
  if (bytes < 1024 * 1024 * 1024)
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
};

const formatDuration = (ms: number): string => {
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(2)}s`;
  return `${(ms / 60000).toFixed(2)}m`;
};

const formatDate = (ms: number): string => {
  return new Date(ms).toLocaleString();
};

export const ReportSummary: React.FC<ReportSummaryProps> = ({ report }) => {
  return (
    <div className="bg-white rounded-lg shadow p-6">
      <h2 className="text-xl font-bold mb-4">Report Summary</h2>

      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
        <div className="bg-gray-50 rounded p-4">
          <div className="text-sm text-gray-500">Workflow</div>
          <div className="text-lg font-semibold">
            {report.workflow_name || "Unknown"}
          </div>
        </div>

        <div className="bg-gray-50 rounded p-4">
          <div className="text-sm text-gray-500">Status</div>
          <div
            className={`text-lg font-semibold ${
              report.success ? "text-green-600" : "text-red-600"
            }`}
          >
            {report.success ? "Success" : "Failed"}
          </div>
        </div>

        <div className="bg-gray-50 rounded p-4">
          <div className="text-sm text-gray-500">Duration</div>
          <div className="text-lg font-semibold">
            {formatDuration(report.duration_ms)}
          </div>
        </div>

        <div className="bg-gray-50 rounded p-4">
          <div className="text-sm text-gray-500">Started</div>
          <div className="text-lg font-semibold">
            {formatDate(report.start_time_ms)}
          </div>
        </div>
      </div>

      <h3 className="text-lg font-semibold mb-3">Statistics</h3>
      <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
        <div className="border rounded p-4">
          <div className="text-sm text-gray-500">Features Processed</div>
          <div className="text-2xl font-bold text-blue-600">
            {report.summary.total_features_processed.toLocaleString()}
          </div>
        </div>

        <div className="border rounded p-4">
          <div className="text-sm text-gray-500">Data Transferred</div>
          <div className="text-2xl font-bold text-green-600">
            {formatBytes(report.summary.total_bytes_transferred)}
          </div>
        </div>

        <div className="border rounded p-4">
          <div className="text-sm text-gray-500">Peak Memory</div>
          <div className="text-2xl font-bold text-purple-600">
            {formatBytes(report.summary.peak_memory_usage)}
          </div>
        </div>

        {report.summary.slowest_node && (
          <div className="border rounded p-4">
            <div className="text-sm text-gray-500">Slowest Node</div>
            <div className="text-lg font-semibold text-orange-600">
              {report.summary.slowest_node}
            </div>
            {report.summary.slowest_avg_time_ms && (
              <div className="text-sm text-gray-500">
                Avg: {formatDuration(report.summary.slowest_avg_time_ms)}
              </div>
            )}
          </div>
        )}

        {report.summary.highest_memory_node && (
          <div className="border rounded p-4">
            <div className="text-sm text-gray-500">Highest Memory Node</div>
            <div className="text-lg font-semibold text-red-600">
              {report.summary.highest_memory_node}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
