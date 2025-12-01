import { useState, useEffect, useCallback } from "react";
import {
  useReportsList,
  useAnalyzerReport,
  useReportNodes,
} from "./hooks/useAnalyzerReport";
import { ReportsList } from "./components/ReportsList";
import { ReportSummary } from "./components/ReportSummary";
import { ActionSelector } from "./components/ActionSelector";
import { UnifiedMemoryChart } from "./components/UnifiedMemoryChart";

function App() {
  const [selectedFilename, setSelectedFilename] = useState<string | null>(null);
  const [visibleNodes, setVisibleNodes] = useState<Set<string>>(new Set());

  const { reports, loading: reportsLoading, error: reportsError, refresh } = useReportsList();
  const { report, loading: reportLoading, error: reportError } = useAnalyzerReport(selectedFilename);
  const { nodes } = useReportNodes(selectedFilename);

  // Initialize visible nodes when nodes are loaded - only show nodes with memory data
  useEffect(() => {
    if (nodes.length > 0) {
      const nodesWithMemory = nodes
        .filter((n) => n.has_memory_data)
        .map((n) => n.node_id);
      setVisibleNodes(new Set(nodesWithMemory));
    }
  }, [nodes]);

  const handleSelectReport = (filename: string) => {
    setSelectedFilename(filename);
    setVisibleNodes(new Set());
  };

  const handleToggleNode = useCallback((nodeId: string) => {
    setVisibleNodes((prev) => {
      const next = new Set(prev);
      if (next.has(nodeId)) {
        next.delete(nodeId);
      } else {
        next.add(nodeId);
      }
      return next;
    });
  }, []);

  const handleToggleAll = useCallback((visible: boolean) => {
    if (visible) {
      // Only show nodes with memory data
      const nodesWithMemory = nodes
        .filter((n) => n.has_memory_data)
        .map((n) => n.node_id);
      setVisibleNodes(new Set(nodesWithMemory));
    } else {
      setVisibleNodes(new Set());
    }
  }, [nodes]);

  return (
    <div className="min-h-screen bg-gray-100">
      {/* Header */}
      <header className="bg-white shadow">
        <div className="max-w-7xl mx-auto py-4 px-4">
          <h1 className="text-2xl font-bold text-gray-900">
            Re:Earth Flow Analyzer
          </h1>
        </div>
      </header>

      {/* Main content */}
      <main className="max-w-7xl mx-auto py-6 px-4">
        <div className="grid grid-cols-12 gap-6">
          {/* Left sidebar - Reports list */}
          <div className="col-span-3">
            <ReportsList
              reports={reports}
              selectedFilename={selectedFilename}
              onSelectReport={handleSelectReport}
              loading={reportsLoading}
              error={reportsError}
              onRefresh={refresh}
            />
          </div>

          {/* Main content area */}
          <div className="col-span-9">
            {!selectedFilename ? (
              <div className="bg-white rounded-lg shadow p-8 text-center text-gray-500">
                <p className="text-lg">Select a report to view details</p>
                <p className="text-sm mt-2">
                  Reports are stored in the analyzer reports directory
                </p>
              </div>
            ) : reportLoading ? (
              <div className="bg-white rounded-lg shadow p-8 text-center text-gray-500">
                Loading report...
              </div>
            ) : reportError ? (
              <div className="bg-white rounded-lg shadow p-8 text-center text-red-500">
                Error loading report: {reportError}
              </div>
            ) : report ? (
              <div className="space-y-6">
                {/* Report summary */}
                <ReportSummary report={report} />

                {/* Unified memory chart */}
                <div className="bg-white rounded-lg shadow p-6">
                  <UnifiedMemoryChart
                    report={report}
                    visibleNodes={visibleNodes}
                  />
                </div>

                {/* Action selector with search and toggles */}
                <ActionSelector
                  nodes={nodes}
                  visibleNodes={visibleNodes}
                  onToggleNode={handleToggleNode}
                  onToggleAll={handleToggleAll}
                />
              </div>
            ) : null}
          </div>
        </div>
      </main>
    </div>
  );
}

export default App;
