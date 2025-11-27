import React from "react";
import type { ReportInfo } from "../types";

interface ReportsListProps {
  reports: ReportInfo[];
  selectedFilename: string | null;
  onSelectReport: (filename: string) => void;
  loading: boolean;
  error: string | null;
  onRefresh: () => void;
}

const formatDate = (ms: number): string => {
  if (ms === 0) return "Unknown";
  return new Date(ms).toLocaleString();
};

const formatDuration = (ms: number): string => {
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
  return `${(ms / 60000).toFixed(1)}m`;
};

export const ReportsList: React.FC<ReportsListProps> = ({
  reports,
  selectedFilename,
  onSelectReport,
  loading,
  error,
  onRefresh,
}) => {
  if (loading) {
    return (
      <div className="bg-white rounded-lg shadow p-4 text-center text-gray-500">
        Loading reports...
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-white rounded-lg shadow p-4">
        <div className="text-red-500 mb-2">Error loading reports: {error}</div>
        <button
          onClick={onRefresh}
          className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
        >
          Retry
        </button>
      </div>
    );
  }

  if (reports.length === 0) {
    return (
      <div className="bg-white rounded-lg shadow p-4">
        <div className="text-gray-500 mb-2">No reports found</div>
        <button
          onClick={onRefresh}
          className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
        >
          Refresh
        </button>
      </div>
    );
  }

  return (
    <div className="bg-white rounded-lg shadow">
      <div className="p-4 border-b flex justify-between items-center">
        <h3 className="text-lg font-semibold">Reports</h3>
        <button
          onClick={onRefresh}
          className="px-3 py-1 text-sm bg-gray-100 text-gray-700 rounded hover:bg-gray-200"
        >
          Refresh
        </button>
      </div>
      <div className="max-h-96 overflow-y-auto">
        {reports.map((report) => (
          <button
            key={report.filename}
            onClick={() => onSelectReport(report.filename)}
            className={`w-full p-4 text-left border-b hover:bg-gray-50 transition-colors ${
              selectedFilename === report.filename
                ? "bg-blue-50 border-l-4 border-l-blue-500"
                : ""
            }`}
          >
            <div className="font-medium">
              {report.workflow_name || report.filename}
            </div>
            <div className="text-sm text-gray-500 mt-1">
              {formatDate(report.start_time_ms)}
            </div>
            <div className="flex items-center gap-2 mt-2">
              <span
                className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${
                  report.success
                    ? "bg-green-100 text-green-800"
                    : "bg-red-100 text-red-800"
                }`}
              >
                {report.success ? "Success" : "Failed"}
              </span>
              {report.duration_ms > 0 && (
                <span className="text-xs text-gray-500">
                  {formatDuration(report.duration_ms)}
                </span>
              )}
            </div>
          </button>
        ))}
      </div>
    </div>
  );
};
