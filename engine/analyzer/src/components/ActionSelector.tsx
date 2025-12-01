import React, { useMemo, useState } from "react";
import type { NodeSummary } from "../types";

interface ActionSelectorProps {
  nodes: NodeSummary[];
  visibleNodes: Set<string>;
  onToggleNode: (nodeId: string) => void;
  onToggleAll: (visible: boolean) => void;
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

// Color palette matching UnifiedMemoryChart
const COLORS = [
  "#8884d8",
  "#82ca9d",
  "#ffc658",
  "#ff7300",
  "#00C49F",
  "#FFBB28",
  "#FF8042",
  "#0088FE",
  "#00C49F",
  "#d0ed57",
  "#a4de6c",
  "#8dd1e1",
  "#83a6ed",
  "#8e4585",
  "#ff6b6b",
  "#4ecdc4",
];

export const ActionSelector: React.FC<ActionSelectorProps> = ({
  nodes,
  visibleNodes,
  onToggleNode,
  onToggleAll,
}) => {
  const [searchQuery, setSearchQuery] = useState("");

  // Filter nodes based on search query
  const filteredNodes = useMemo(() => {
    if (!searchQuery.trim()) return nodes;
    const query = searchQuery.toLowerCase();
    return nodes.filter(
      (node) =>
        node.node_name.toLowerCase().includes(query) ||
        node.node_id.toLowerCase().includes(query)
    );
  }, [nodes, searchQuery]);

  // Get color for a node based on its index in the full nodes list
  const getNodeColor = (nodeId: string) => {
    const index = nodes.findIndex((n) => n.node_id === nodeId);
    return COLORS[index % COLORS.length];
  };

  const allVisible = nodes.length > 0 && nodes.every((n) => visibleNodes.has(n.node_id));
  const noneVisible = nodes.length > 0 && nodes.every((n) => !visibleNodes.has(n.node_id));

  if (nodes.length === 0) {
    return (
      <div className="bg-white rounded-lg shadow p-4 text-gray-500">
        No actions found in this report
      </div>
    );
  }

  return (
    <div className="bg-white rounded-lg shadow">
      <div className="p-4 border-b">
        <h3 className="text-lg font-semibold mb-3">Actions</h3>
        {/* Search bar */}
        <div className="relative">
          <input
            type="text"
            placeholder="Search actions..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
          {searchQuery && (
            <button
              onClick={() => setSearchQuery("")}
              className="absolute right-2 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600"
            >
              ✕
            </button>
          )}
        </div>
        {/* Toggle all buttons */}
        <div className="flex gap-2 mt-3">
          <button
            onClick={() => onToggleAll(true)}
            disabled={allVisible}
            className={`flex-1 px-3 py-1.5 text-sm rounded border ${
              allVisible
                ? "bg-gray-100 text-gray-400 cursor-not-allowed"
                : "bg-white text-blue-600 border-blue-600 hover:bg-blue-50"
            }`}
          >
            Show All
          </button>
          <button
            onClick={() => onToggleAll(false)}
            disabled={noneVisible}
            className={`flex-1 px-3 py-1.5 text-sm rounded border ${
              noneVisible
                ? "bg-gray-100 text-gray-400 cursor-not-allowed"
                : "bg-white text-gray-600 border-gray-600 hover:bg-gray-50"
            }`}
          >
            Hide All
          </button>
        </div>
      </div>
      <div className="max-h-[500px] overflow-y-auto">
        {filteredNodes.length === 0 ? (
          <div className="p-4 text-center text-gray-500">
            No actions match "{searchQuery}"
          </div>
        ) : (
          filteredNodes.map((node) => {
            const isVisible = visibleNodes.has(node.node_id);
            const color = getNodeColor(node.node_id);
            return (
              <div
                key={node.node_id}
                className={`p-4 border-b hover:bg-gray-50 transition-colors ${
                  isVisible ? "" : "opacity-50"
                }`}
              >
                <div className="flex items-center gap-3">
                  {/* Toggle switch */}
                  <button
                    onClick={() => onToggleNode(node.node_id)}
                    className={`relative w-11 h-6 rounded-full transition-colors ${
                      isVisible ? "bg-blue-600" : "bg-gray-300"
                    }`}
                  >
                    <span
                      className={`absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform ${
                        isVisible ? "translate-x-5" : "translate-x-0"
                      }`}
                    />
                  </button>
                  {/* Color indicator */}
                  <div
                    className="w-3 h-3 rounded-full flex-shrink-0"
                    style={{ backgroundColor: color }}
                  />
                  {/* Node info */}
                  <div className="flex-1 min-w-0">
                    <div className="font-medium truncate">
                      {node.node_name || node.node_id}
                    </div>
                    <div className="text-sm text-gray-500 flex flex-wrap gap-2 mt-1">
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
                      <div className="text-xs text-gray-400 mt-1">
                        {node.features_processed.toLocaleString()} features •{" "}
                        {formatBytes(node.total_peak_memory)} peak •{" "}
                        {formatDuration(node.total_processing_time_ms)}
                      </div>
                    )}
                  </div>
                </div>
              </div>
            );
          })
        )}
      </div>
      {searchQuery && filteredNodes.length > 0 && (
        <div className="p-2 text-center text-sm text-gray-500 border-t">
          Showing {filteredNodes.length} of {nodes.length} actions
        </div>
      )}
    </div>
  );
};
