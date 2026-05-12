import {
  JobFragment,
  UserFacingLogFragment,
} from "@flow/lib/gql/__gen__/graphql";

export const mockJobs: JobFragment[] = [
  {
    id: "job-1",
    deployment: {
      id: "deployment-1",
      description: "Initial deployment of data processing pipeline",
    },
    workspaceId: "workspace-1",
    status: "COMPLETED",
    debug: false,
    startedAt: "2024-01-15T10:00:00Z",
    completedAt: "2024-01-15T10:05:30Z",
    outputURLs: [
      "https://output.reearth.io/job-1/result.json",
      "https://output.reearth.io/job-1/processed_data.geojson",
    ],
    userFacingLogsURL: null,
  },
  {
    id: "job-2",
    deployment: {
      id: "deployment-2",
      description: "Real-time analytics deployment with improved performance",
    },
    workspaceId: "workspace-2",
    status: "RUNNING",
    debug: true,
    startedAt: "2024-01-28T14:20:00Z",
    completedAt: null,
    outputURLs: [],
    userFacingLogsURL: null,
  },
  {
    id: "job-3",
    deployment: {
      id: "deployment-3",
      description: "Failed ML workflow deployment",
    },
    workspaceId: "workspace-2",
    status: "FAILED",
    debug: false,
    startedAt: "2024-01-25T09:15:00Z",
    completedAt: "2024-01-25T09:18:45Z",
    outputURLs: [],
    userFacingLogsURL: null,
  },
  {
    id: "job-4",
    deployment: {
      id: "deployment-4",
      description: "Dashboard deployment in progress",
    },
    workspaceId: "workspace-3",
    status: "PENDING",
    debug: false,
    startedAt: "2024-01-28T16:00:00Z",
    completedAt: null,
    outputURLs: [],
    userFacingLogsURL: null,
  },
  {
    id: "job-5",
    deployment: {
      id: "deployment-5",
      description: "Legacy migration deployment",
    },
    workspaceId: "workspace-1",
    status: "CANCELLED",
    debug: false,
    startedAt: "2024-01-20T11:30:00Z",
    completedAt: "2024-01-20T11:35:15Z",
    outputURLs: [],
    userFacingLogsURL: null,
  },
];

export const mockLogs: UserFacingLogFragment[] = [
  {
    jobId: "job-1",
    timestamp: "2024-01-15T10:00:10Z",
    nodeId: null,
    nodeName: null,
    level: "INFO",
    message: "Job started successfully",
  },
];
