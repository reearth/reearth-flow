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

export enum MachineTypeOption {
  E2_STANDARD_2 = "e2-standard-2",
  E2_STANDARD_4 = "e2-standard-4",
  E2_STANDARD_8 = "e2-standard-8",
  E2_STANDARD_16 = "e2-standard-16",
  E2_HIGH_MEM_2 = "e2-highmem-2",
  E2_HIGH_MEM_4 = "e2-highmem-4",
  E2_HIGH_MEM_8 = "e2-highmem-8",
  E2_HIGH_MEM_16 = "e2-highmem-16",
  E2_HIGH_CPU_2 = "e2-highcpu-2",
  E2_HIGH_CPU_4 = "e2-highcpu-4",
  E2_HIGH_CPU_8 = "e2-highcpu-8",
  E2_HIGH_CPU_16 = "e2-highcpu-16",
  N2_STANDARD_2 = "n2-standard-2",
  N2_STANDARD_4 = "n2-standard-4",
  N2_STANDARD_8 = "n2-standard-8",
  N2_STANDARD_16 = "n2-standard-16",
}

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
