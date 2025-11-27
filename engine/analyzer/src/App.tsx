import React, { useState } from "react";
import {
  useReportsList,
  useAnalyzerReport,
  useReportNodes,
} from "./hooks/useAnalyzerReport";
import { ReportsList } from "./components/ReportsList";
import { ReportSummary } from "./components/ReportSummary";
import { NodeSelector } from "./components/NodeSelector";
import { MemoryChart } from "./components/MemoryChart";
import { QueueChart } from "./components/QueueChart";

function App() {
  const [selectedFilename, setSelectedFilename] = useState<string | null>(null);
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);

  const { reports, loading: reportsLoading, error: reportsError, refresh } = useReportsList();
  const { report, loading: reportLoading, error: reportError } = useAnalyzerReport(selectedFilename);
  const { nodes, loading: nodesLoading, error: nodesError } = useReportNodes(selectedFilename);

  const handleSelectReport = (filename: string) => {
    setSelectedFilename(filename);
    setSelectedNodeId(null);
  };

  const handleSelectNode = (nodeId: string) => {
    setSelectedNodeId(nodeId);
  };

  const selectedNode = nodes.find((n) => n.node_id === selectedNodeId);
  const selectedMemoryReport = report?.memory_reports[selectedNodeId ?? ""];
  const selectedQueueReport = report?.queue_reports[selectedNodeId ?? ""];

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

                {/* Node selection and charts */}
                <div className="grid grid-cols-12 gap-6">
                  {/* Node selector */}
                  <div className="col-span-4">
                    <NodeSelector
                      nodes={nodes}
                      selectedNodeId={selectedNodeId}
                      onSelectNode={handleSelectNode}
                    />
                  </div>

                  {/* Charts */}
                  <div className="col-span-8 space-y-6">
                    {!selectedNodeId ? (
                      <div className="bg-white rounded-lg shadow p-8 text-center text-gray-500">
                        Select a node to view memory and queue charts
                      </div>
                    ) : (
                      <>
                        {/* Memory chart */}
                        {selectedNode?.has_memory_data && selectedMemoryReport && (
                          <div className="bg-white rounded-lg shadow p-6">
                            <MemoryChart
                              data={selectedMemoryReport.data_points}
                              nodeName={selectedNode.node_name || selectedNodeId}
                            />
                          </div>
                        )}

                        {/* Queue chart */}
                        {selectedNode?.has_queue_data && selectedQueueReport && (
                          <div className="bg-white rounded-lg shadow p-6">
                            <QueueChart
                              data={selectedQueueReport.data_points}
                              nodeName={selectedNode.node_name || selectedNodeId}
                            />
                          </div>
                        )}

                        {/* No data message */}
                        {!selectedNode?.has_memory_data && !selectedNode?.has_queue_data && (
                          <div className="bg-white rounded-lg shadow p-8 text-center text-gray-500">
                            No chart data available for this node
                          </div>
                        )}
                      </>
                    )}
                  </div>
                </div>
              </div>
            ) : null}
          </div>
        </div>
      </main>
    </div>
  );
}

export default App;
