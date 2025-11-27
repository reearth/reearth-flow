import React, { useMemo } from "react";
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from "recharts";
import type { QueueDataPoint } from "../types";

interface QueueChartProps {
  data: QueueDataPoint[];
  nodeName: string;
}

interface TooltipPayload {
  payload: QueueDataPoint & { relativeTime: number; total: number };
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
    const total = data.features_waiting + data.features_processing;
    return (
      <div className="bg-white p-3 border rounded shadow-lg text-sm">
        <p className="font-semibold">
          Time: {((data.timestamp_ms - startTime) / 1000).toFixed(2)}s
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
  const startTime = useMemo(
    () => (data.length > 0 ? data[0].timestamp_ms : 0),
    [data]
  );

  const chartData = useMemo(
    () =>
      data.map((point) => ({
        ...point,
        relativeTime: (point.timestamp_ms - startTime) / 1000,
        total: point.features_waiting + point.features_processing,
      })),
    [data, startTime]
  );

  if (data.length === 0) {
    return (
      <div className="w-full h-96 flex items-center justify-center text-gray-500">
        No queue data available
      </div>
    );
  }

  return (
    <div className="w-full h-96">
      <h3 className="text-lg font-semibold mb-2">Feature Queue: {nodeName}</h3>
      <ResponsiveContainer width="100%" height="100%">
        <AreaChart data={chartData}>
          <CartesianGrid strokeDasharray="3 3" />
          <XAxis
            dataKey="relativeTime"
            label={{ value: "Time (s)", position: "bottom", offset: -5 }}
          />
          <YAxis
            label={{ value: "Features", angle: -90, position: "insideLeft" }}
          />
          <Tooltip
            content={<CustomTooltip startTime={startTime} />}
          />
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
