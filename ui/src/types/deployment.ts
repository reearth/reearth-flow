import { ApiResponse } from "./api";
import { Job } from "./job";

export type Deployment = {
  id: string;
  projectId: string;
  workspaceId: string;
  workflowUrl: string;
  version: string;
  createdAt: string;
  updatedAt: string;
  // project: Project;
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

export type ExecuteDeployment = {
  job?: Job;
} & ApiResponse;
