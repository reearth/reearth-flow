import { useMemo, useState } from "react";
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from "recharts";
import type { AnalyzerReport, MemoryDataPoint } from "../types";

interface UnifiedMemoryChartProps {
  report: AnalyzerReport;
  visibleNodes: Set<string>;
}

type MemoryViewTab = "current" | "peak";

const formatBytes = (bytes: number): string => {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`;
  if (bytes < 1024 * 1024 * 1024)
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
};

// Color palette for different nodes
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

interface UnifiedDataPoint {
  relativeTime: number;
  timestamp_ms: number;
  [key: string]: number | string | null;
}

/** Data point for peak memory horizontal line segments */
interface PeakSegmentPoint {
  relativeTime: number;
  [key: string]: number | null;
}

interface CustomTooltipProps {
  active?: boolean;
  payload?: Array<{
    dataKey: string;
    value: number;
    color: string;
    name: string;
  }>;
  label?: number;
}

const CustomTooltip = ({
  active,
  payload,
  label,
}: CustomTooltipProps) => {
  if (active && payload && payload.length) {
    // Filter out entries with null/undefined values and sort by value descending
    const validEntries = payload
      .filter((entry) => entry.value != null)
      .sort((a, b) => b.value - a.value);

    if (validEntries.length === 0) return null;

    return (
      <div className="bg-white/95 backdrop-blur-sm p-3 border border-gray-200 rounded-lg shadow-xl text-sm max-h-80 overflow-y-auto min-w-48">
        <p className="font-semibold text-gray-700 mb-2 pb-2 border-b border-gray-200">
          Time: {label?.toFixed(2)}s
        </p>
        <div className="space-y-1">
          {validEntries.map((entry, index) => (
            <div key={index} className="flex items-center justify-between gap-3">
              <div className="flex items-center gap-2 min-w-0">
                <span
                  className="w-2.5 h-2.5 rounded-full flex-shrink-0"
                  style={{ backgroundColor: entry.color }}
                />
                <span className="truncate text-gray-600">{entry.name}</span>
              </div>
              <span className="font-medium text-gray-900 flex-shrink-0">
                {formatBytes(entry.value)}
              </span>
            </div>
          ))}
        </div>
      </div>
    );
  }
  return null;
};

export const UnifiedMemoryChart: React.FC<UnifiedMemoryChartProps> = ({
  report,
  visibleNodes,
}) => {
  const [activeTab, setActiveTab] = useState<MemoryViewTab>("current");
  const workflowStartTime = report.start_time_ms;

  // Get all visible nodes with memory data (use quantized data for efficiency)
  const nodesWithData = useMemo(() => {
    return Object.entries(report.memory_reports)
      .filter(([nodeId]) => visibleNodes.has(nodeId))
      .map(([nodeId, memReport]) => ({
        nodeId,
        nodeName: memReport.info.node_name || nodeId,
        // Use quantized data if available, fall back to raw data for backwards compatibility
        dataPoints:
          memReport.quantized_data_points?.length > 0
            ? memReport.quantized_data_points
            : memReport.data_points,
      }));
  }, [report.memory_reports, visibleNodes]);

  // Calculate the time domain (start and end times relative to workflow start)
  const timeDomain = useMemo(() => {
    const startTime = 0; // Always start at 0
    const endTime = (report.end_time_ms - workflowStartTime) / 1000;
    return [startTime, Math.max(endTime, 1)]; // Ensure at least 1 second range
  }, [report.end_time_ms, workflowStartTime]);

  // Create unified time-series data for current memory (step graph)
  const currentMemoryData = useMemo(() => {
    if (nodesWithData.length === 0) return [];

    // Collect all unique timestamps across all nodes
    const allTimestamps = new Set<number>();
    nodesWithData.forEach(({ dataPoints }) => {
      dataPoints.forEach((point) => {
        allTimestamps.add(point.timestamp_ms);
      });
    });

    // Sort timestamps
    const sortedTimestamps = Array.from(allTimestamps).sort((a, b) => a - b);

    // Create data points for each timestamp
    const dataMap = new Map<number, UnifiedDataPoint>();

    sortedTimestamps.forEach((timestamp) => {
      dataMap.set(timestamp, {
        relativeTime: (timestamp - workflowStartTime) / 1000,
        timestamp_ms: timestamp,
      });
    });

    // Fill in data for each node
    nodesWithData.forEach(({ nodeId, dataPoints }) => {
      // Create a map of timestamp to data point for this node
      const nodeDataMap = new Map<number, MemoryDataPoint>();
      dataPoints.forEach((point) => {
        nodeDataMap.set(point.timestamp_ms, point);
      });

      // For each timestamp, find the closest preceding data point
      let lastValue: MemoryDataPoint | null = null;
      sortedTimestamps.forEach((timestamp) => {
        const exactPoint = nodeDataMap.get(timestamp);
        if (exactPoint) {
          lastValue = exactPoint;
        }
        const dataPoint = dataMap.get(timestamp);
        if (dataPoint && lastValue) {
          dataPoint[`${nodeId}_current`] = lastValue.current_memory_bytes;
        }
      });
    });

    return Array.from(dataMap.values());
  }, [nodesWithData, workflowStartTime]);

  // Create horizontal line segments for peak memory
  // Each segment spans from start_timestamp_ms to timestamp_ms (end)
  const peakMemoryData = useMemo(() => {
    if (nodesWithData.length === 0) return [];

    // Collect all segment boundaries (start and end points) across all nodes
    const allTimes = new Set<number>();
    nodesWithData.forEach(({ dataPoints }) => {
      dataPoints.forEach((point) => {
        // Use start_timestamp_ms if available, otherwise fall back to timestamp_ms
        const startTime = point.start_timestamp_ms || point.timestamp_ms;
        allTimes.add(startTime);
        allTimes.add(point.timestamp_ms);
      });
    });

    // Sort times
    const sortedTimes = Array.from(allTimes).sort((a, b) => a - b);

    // Create data points
    const result: PeakSegmentPoint[] = [];

    sortedTimes.forEach((time) => {
      const point: PeakSegmentPoint = {
        relativeTime: (time - workflowStartTime) / 1000,
      };

      // For each node, check if this time falls within any segment
      nodesWithData.forEach(({ nodeId, dataPoints }) => {
        let peakValue: number | null = null;

        // Find if this time is within any segment for this node
        for (const dp of dataPoints) {
          const segmentStart = dp.start_timestamp_ms || dp.timestamp_ms;
          const segmentEnd = dp.timestamp_ms;

          // Check if time is within this segment (inclusive on both ends)
          if (time >= segmentStart && time <= segmentEnd) {
            // Use the peak value from this segment
            peakValue = dp.peak_memory_bytes;
            break;
          }
        }

        point[`${nodeId}_peak`] = peakValue;
      });

      result.push(point);
    });

    return result;
  }, [nodesWithData, workflowStartTime]);

  if (nodesWithData.length === 0) {
    return (
      <div className="w-full h-96 flex items-center justify-center text-gray-500">
        No memory data available. Select nodes to display.
      </div>
    );
  }

  // Choose the appropriate data and suffix based on active tab
  const chartData = activeTab === "current" ? currentMemoryData : peakMemoryData;
  const dataKeySuffix = activeTab === "current" ? "_current" : "_peak";

  return (
    <div className="w-full">
      {/* Tab buttons */}
      <div className="flex items-center gap-4 mb-4">
        <h3 className="text-lg font-semibold">Memory Usage</h3>
        <div className="flex border border-gray-300 rounded-lg overflow-hidden">
          <button
            onClick={() => setActiveTab("current")}
            className={`px-4 py-1.5 text-sm font-medium transition-colors ${
              activeTab === "current"
                ? "bg-blue-600 text-white"
                : "bg-white text-gray-700 hover:bg-gray-100"
            }`}
          >
            Current Memory
          </button>
          <button
            onClick={() => setActiveTab("peak")}
            className={`px-4 py-1.5 text-sm font-medium transition-colors ${
              activeTab === "peak"
                ? "bg-blue-600 text-white"
                : "bg-white text-gray-700 hover:bg-gray-100"
            }`}
          >
            Peak Memory
          </button>
        </div>
      </div>

      {/* Chart */}
      <div className="w-full h-80">
        <ResponsiveContainer width="100%" height="100%">
          <LineChart data={chartData}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis
              dataKey="relativeTime"
              type="number"
              domain={timeDomain}
              label={{ value: "Time (s)", position: "bottom", offset: -5 }}
              tickFormatter={(value) => value.toFixed(0)}
            />
            <YAxis
              tickFormatter={formatBytes}
              label={{ value: "Memory", angle: -90, position: "insideLeft" }}
            />
            <Tooltip content={<CustomTooltip />} />
            {nodesWithData.map(({ nodeId, nodeName }, index) => {
              const color = COLORS[index % COLORS.length];
              return (
                <Line
                  key={`${nodeId}${dataKeySuffix}`}
                  type={activeTab === "peak" ? "linear" : "monotone"}
                  dataKey={`${nodeId}${dataKeySuffix}`}
                  name={nodeName}
                  stroke={color}
                  dot={false}
                  strokeWidth={2}
                  connectNulls={false}
                />
              );
            })}
          </LineChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
};
