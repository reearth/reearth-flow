import { ApiResponse } from "./api";

export type WorkerConfig = {
  id: string;
  machineType?: string;
  computeCpuMilli?: number;
  computeMemoryMib?: number;
  bootDiskSizeGB?: number;
  taskCount?: number;
  maxConcurrency?: number;
  threadPoolSize?: number;
  channelBufferSize?: number;
  featureFlushThreshold?: number;
  nodeStatusPropagationDelayMilli?: number;
  createdAt: string;
  updatedAt: string;
};

export type WorkerConfigMutation = {
  config?: WorkerConfig;
} & ApiResponse;

export type GetWorkerConfig = {
  workerConfig?: WorkerConfig;
  isLoading: boolean;
} & ApiResponse;

export type DeleteWorkerConfig = {
  id: string;
} & ApiResponse;
