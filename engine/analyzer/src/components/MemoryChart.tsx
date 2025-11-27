import React, { useMemo } from "react";
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from "recharts";
import type { MemoryDataPoint } from "../types";

interface MemoryChartProps {
  data: MemoryDataPoint[];
  nodeName: string;
}

const formatBytes = (bytes: number): string => {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`;
  if (bytes < 1024 * 1024 * 1024)
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
};

interface TooltipPayload {
  payload: MemoryDataPoint & { relativeTime: number };
}

interface CustomTooltipProps {
  active?: boolean;
  payload?: TooltipPayload[];
  startTime: number;
}

const CustomTooltip: React.FC<CustomTooltipProps> = ({
  active,
  payload,
  startTime,
}) => {
  if (active && payload && payload.length) {
    const data = payload[0].payload;
    return (
      <div className="bg-white p-3 border rounded shadow-lg text-sm">
        <p className="font-semibold">
          Time: {((data.timestamp_ms - startTime) / 1000).toFixed(2)}s
        </p>
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
  const startTime = useMemo(
    () => (data.length > 0 ? data[0].timestamp_ms : 0),
    [data]
  );

  const chartData = useMemo(
    () =>
      data.map((point) => ({
        ...point,
        relativeTime: (point.timestamp_ms - startTime) / 1000,
      })),
    [data, startTime]
  );

  if (data.length === 0) {
    return (
      <div className="w-full h-96 flex items-center justify-center text-gray-500">
        No memory data available
      </div>
    );
  }

  return (
    <div className="w-full h-96">
      <h3 className="text-lg font-semibold mb-2">Memory Usage: {nodeName}</h3>
      <ResponsiveContainer width="100%" height="100%">
        <LineChart data={chartData}>
          <CartesianGrid strokeDasharray="3 3" />
          <XAxis
            dataKey="relativeTime"
            label={{ value: "Time (s)", position: "bottom", offset: -5 }}
          />
          <YAxis
            tickFormatter={formatBytes}
            label={{ value: "Memory", angle: -90, position: "insideLeft" }}
          />
          <Tooltip
            content={<CustomTooltip startTime={startTime} />}
          />
          <Legend />
          <Line
            type="monotone"
            dataKey="current_memory_bytes"
            name="Current Memory"
            stroke="#8884d8"
            dot={false}
            strokeWidth={2}
          />
          <Line
            type="monotone"
            dataKey="peak_memory_bytes"
            name="Peak Memory"
            stroke="#82ca9d"
            dot={false}
            strokeWidth={2}
          />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
};
