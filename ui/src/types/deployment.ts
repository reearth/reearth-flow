import { ApiResponse } from "./api";
import { Job } from "./job";

export type Deployment = {
  id: string;
  projectId?: string | null;
  projectName?: string;
  workspaceId: string;
  workflowUrl: string;
  description?: string;
  version: string;
  createdAt: string;
  updatedAt: string;
  // workspace: Workspace;
};

export type GetDeployments = {
  pages?: ({ deployments?: Deployment[] } | undefined)[];
  hasNextPage: boolean;
  fetchNextPage: () => void;
  isLoading: boolean;
  isFetching: boolean;
} & ApiResponse;

export type CreateDeployment = {
  deployment?: Deployment;
} & ApiResponse;

export type UpdateDeployment = {
  deployment?: Deployment;
} & ApiResponse;

export type DeleteDeployment = {
  deploymentId?: string;
} & ApiResponse;

export type ExecuteDeployment = {
  job?: Job;
} & ApiResponse;
