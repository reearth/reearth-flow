import React from "react";
import type { NodeSummary } from "../types";

interface NodeSelectorProps {
  nodes: NodeSummary[];
  selectedNodeId: string | null;
  onSelectNode: (nodeId: string) => void;
}

const formatBytes = (bytes: number): string => {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024)
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
};

const formatDuration = (ms: number): string => {
  if (ms < 1000) return `${ms}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
};

export const NodeSelector: React.FC<NodeSelectorProps> = ({
  nodes,
  selectedNodeId,
  onSelectNode,
}) => {
  if (nodes.length === 0) {
    return (
      <div className="bg-white rounded-lg shadow p-4 text-gray-500">
        No nodes found in this report
      </div>
    );
  }

  return (
    <div className="bg-white rounded-lg shadow">
      <div className="p-4 border-b">
        <h3 className="text-lg font-semibold">Nodes</h3>
      </div>
      <div className="max-h-96 overflow-y-auto">
        {nodes.map((node) => (
          <button
            key={node.node_id}
            onClick={() => onSelectNode(node.node_id)}
            className={`w-full p-4 text-left border-b hover:bg-gray-50 transition-colors ${
              selectedNodeId === node.node_id
                ? "bg-blue-50 border-l-4 border-l-blue-500"
                : ""
            }`}
          >
            <div className="font-medium">{node.node_name || node.node_id}</div>
            <div className="text-sm text-gray-500 mt-1 flex flex-wrap gap-2">
              {node.has_memory_data && (
                <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-purple-100 text-purple-800">
                  Memory
                </span>
              )}
              {node.has_queue_data && (
                <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800">
                  Queue
                </span>
              )}
            </div>
            {node.features_processed > 0 && (
              <div className="text-sm text-gray-500 mt-2">
                <span>{node.features_processed.toLocaleString()} features</span>
                <span className="mx-2">•</span>
                <span>{formatBytes(node.total_peak_memory)} peak</span>
                <span className="mx-2">•</span>
                <span>{formatDuration(node.total_processing_time_ms)} total</span>
              </div>
            )}
          </button>
        ))}
      </div>
    </div>
  );
};
