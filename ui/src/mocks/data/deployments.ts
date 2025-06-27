export type MockDeployment = {
  id: string;
  projectId?: string;
  workspaceId: string;
  version: string;
  description: string;
  isHead: boolean;
  headId?: string;
  workflowUrl: string;
  createdAt: string;
  updatedAt: string;
};

export const mockDeployments: MockDeployment[] = [
  {
    id: "deployment-1",
    projectId: "project-1",
    workspaceId: "workspace-1",
    version: "1.0.0",
    description: "Initial deployment of data processing pipeline",
    isHead: true,
    workflowUrl: "https://workflow-1.reearth-flow.com",
    createdAt: "2024-01-15T09:45:00Z",
    updatedAt: "2024-01-15T09:55:00Z",
  },
  {
    id: "deployment-2",
    projectId: "project-2",
    workspaceId: "workspace-2",
    version: "2.1.0",
    description: "Real-time analytics deployment with improved performance",
    isHead: true,
    workflowUrl: "https://workflow-2.reearth-flow.com",
    createdAt: "2024-01-28T14:00:00Z",
    updatedAt: "2024-01-28T14:10:00Z",
  },
  {
    id: "deployment-3",
    projectId: "project-3",
    workspaceId: "workspace-2",
    version: "1.0.0",
    description: "Failed ML workflow deployment",
    isHead: false,
    workflowUrl: "https://workflow-3.reearth-flow.com",
    createdAt: "2024-01-25T09:00:00Z",
    updatedAt: "2024-01-25T09:20:00Z",
  },
  {
    id: "deployment-4",
    projectId: "project-4",
    workspaceId: "workspace-3",
    version: "3.0.0",
    description: "Dashboard deployment in progress",
    isHead: true,
    workflowUrl: "https://workflow-4.reearth-flow.com",
    createdAt: "2024-01-28T15:50:00Z",
    updatedAt: "2024-01-28T15:50:00Z",
  },
  {
    id: "deployment-5",
    projectId: "project-5",
    workspaceId: "workspace-1",
    version: "1.0.0",
    description: "Legacy migration deployment",
    isHead: true,
    workflowUrl: "https://workflow-5.reearth-flow.com",
    createdAt: "2023-12-15T16:30:00Z",
    updatedAt: "2023-12-15T16:45:00Z",
  },
  {
    id: "deployment-6",
    projectId: "project-6",
    workspaceId: "workspace-4",
    version: "1.2.0",
    description: "Design system components deployment",
    isHead: true,
    workflowUrl: "https://workflow-6.reearth-flow.com",
    createdAt: "2024-01-22T14:30:00Z",
    updatedAt: "2024-01-22T14:40:00Z",
  },
];
