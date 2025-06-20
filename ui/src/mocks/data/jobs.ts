export type MockJob = {
  id: string;
  deploymentId: string;
  workspaceId: string;
  status: "PENDING" | "RUNNING" | "COMPLETED" | "FAILED" | "CANCELLED";
  debug: boolean;
  startedAt: string;
  completedAt?: string;
  logsURL?: string;
  workerLogsURL?: string;
  outputURLs: string[];
};

export type MockLog = {
  jobId: string;
  nodeId?: string;
  timestamp: string;
  logLevel: "ERROR" | "WARN" | "INFO" | "DEBUG" | "TRACE";
  message: string;
};

export const mockJobs: MockJob[] = [
  {
    id: "job-1",
    deploymentId: "deployment-1",
    workspaceId: "workspace-1",
    status: "COMPLETED",
    debug: false,
    startedAt: "2024-01-15T10:00:00Z",
    completedAt: "2024-01-15T10:05:30Z",
    logsURL: "https://logs.reearth.io/job-1",
    workerLogsURL: "https://logs.reearth.io/worker/job-1",
    outputURLs: [
      "https://output.reearth.io/job-1/result.json",
      "https://output.reearth.io/job-1/processed_data.geojson",
    ],
  },
  {
    id: "job-2",
    deploymentId: "deployment-2",
    workspaceId: "workspace-2",
    status: "RUNNING",
    debug: true,
    startedAt: "2024-01-28T14:20:00Z",
    logsURL: "https://logs.reearth.io/job-2",
    workerLogsURL: "https://logs.reearth.io/worker/job-2",
    outputURLs: [],
  },
  {
    id: "job-3",
    deploymentId: "deployment-3",
    workspaceId: "workspace-2",
    status: "FAILED",
    debug: false,
    startedAt: "2024-01-25T09:15:00Z",
    completedAt: "2024-01-25T09:18:45Z",
    logsURL: "https://logs.reearth.io/job-3",
    workerLogsURL: "https://logs.reearth.io/worker/job-3",
    outputURLs: [],
  },
  {
    id: "job-4",
    deploymentId: "deployment-4",
    workspaceId: "workspace-3",
    status: "PENDING",
    debug: false,
    startedAt: "2024-01-28T16:00:00Z",
    outputURLs: [],
  },
  {
    id: "job-5",
    deploymentId: "deployment-5",
    workspaceId: "workspace-1",
    status: "CANCELLED",
    debug: false,
    startedAt: "2024-01-20T11:30:00Z",
    completedAt: "2024-01-20T11:35:15Z",
    outputURLs: [],
  },
];

export const mockLogs: MockLog[] = [
  {
    jobId: "job-1",
    timestamp: "2024-01-15T10:00:10Z",
    logLevel: "INFO",
    message: "Job started successfully",
  },
  {
    jobId: "job-1",
    timestamp: "2024-01-15T10:01:00Z",
    logLevel: "INFO",
    message: "Processing input data",
  },
  {
    jobId: "job-1",
    timestamp: "2024-01-15T10:03:20Z",
    logLevel: "INFO",
    message: "Data transformation completed",
  },
  {
    jobId: "job-1",
    timestamp: "2024-01-15T10:05:30Z",
    logLevel: "INFO",
    message: "Job completed successfully",
  },
  {
    jobId: "job-2",
    timestamp: "2024-01-28T14:20:05Z",
    logLevel: "INFO",
    message: "Job started in debug mode",
  },
  {
    jobId: "job-2",
    timestamp: "2024-01-28T14:20:30Z",
    logLevel: "DEBUG",
    message: "Initializing ML model",
  },
  {
    jobId: "job-2",
    timestamp: "2024-01-28T14:22:15Z",
    logLevel: "INFO",
    message: "Loading training data",
  },
  {
    jobId: "job-2",
    timestamp: "2024-01-28T14:25:45Z",
    logLevel: "INFO",
    message: "Model training in progress",
  },
  {
    jobId: "job-3",
    timestamp: "2024-01-25T09:15:10Z",
    logLevel: "INFO",
    message: "Job started",
  },
  {
    jobId: "job-3",
    timestamp: "2024-01-25T09:16:00Z",
    logLevel: "WARN",
    message: "Input validation warnings detected",
  },
  {
    jobId: "job-3",
    timestamp: "2024-01-25T09:18:30Z",
    logLevel: "ERROR",
    message: "Failed to connect to external API",
  },
  {
    jobId: "job-3",
    timestamp: "2024-01-25T09:18:45Z",
    logLevel: "ERROR",
    message: "Job failed due to external dependency error",
  },
  {
    jobId: "job-4",
    timestamp: "2024-01-28T16:00:00Z",
    logLevel: "INFO",
    message: "Job queued for execution",
  },
  {
    jobId: "job-5",
    timestamp: "2024-01-20T11:30:05Z",
    logLevel: "INFO",
    message: "Job started",
  },
  {
    jobId: "job-5",
    timestamp: "2024-01-20T11:32:00Z",
    logLevel: "INFO",
    message: "Processing data batch 1 of 10",
  },
  {
    jobId: "job-5",
    timestamp: "2024-01-20T11:35:15Z",
    logLevel: "WARN",
    message: "Job cancelled by user request",
  },
];
