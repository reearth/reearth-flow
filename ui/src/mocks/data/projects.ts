import { ProjectFragment } from "@flow/lib/gql/__gen__/graphql";

export const mockProjects: ProjectFragment[] = [
  {
    id: "project-1",
    name: "Data Processing Pipeline",
    description: "A comprehensive data processing pipeline for geospatial data",
    workspaceId: "workspace-1",
    sharedToken: "shared-token-1",
    createdAt: "2024-01-01T10:00:00Z",
    updatedAt: "2024-01-15T10:00:00Z",
  },
  {
    id: "project-2",
    name: "Real-time Analytics",
    description: "Real-time data analytics and visualization workflows",
    workspaceId: "workspace-2",
    createdAt: "2024-01-05T14:30:00Z",
    updatedAt: "2024-01-20T09:15:00Z",
  },
  {
    id: "project-3",
    name: "Machine Learning Workflow",
    description: "Automated machine learning pipeline for prediction models",
    workspaceId: "workspace-2",
    sharedToken: "shared-token-3",
    createdAt: "2024-01-10T16:45:00Z",
    updatedAt: "2024-01-25T11:20:00Z",
  },
  {
    id: "project-4",
    name: "Data Visualization Dashboard",
    description: "Interactive dashboard for business intelligence",
    workspaceId: "workspace-3",
    createdAt: "2024-01-12T09:00:00Z",
    updatedAt: "2024-01-28T15:30:00Z",
  },
  {
    id: "project-5",
    name: "Legacy Data Migration",
    description: "Migration workflow for legacy data systems",
    workspaceId: "workspace-1",
    createdAt: "2023-12-01T08:00:00Z",
    updatedAt: "2023-12-15T17:00:00Z",
  },
  {
    id: "project-6",
    name: "Design System Components",
    description: "Automated component generation and documentation",
    workspaceId: "workspace-4",
    createdAt: "2024-01-08T12:00:00Z",
    updatedAt: "2024-01-22T14:45:00Z",
  },
];
