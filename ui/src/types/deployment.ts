import type { ApiResponse } from "./api";
import type { Job } from "./job";
import type { AnyWorkflowVariable } from "./workflowVariables";

export enum DeploymentOrderBy {
  Version = "version",
  UpdatedAt = "updatedAt",
  Description = "description",
}

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
  // The project's workflow variables as they were when this version was
  // deployed. Empty for a deployment uploaded as a file, whose variables are
  // only in the workflow file.
  parameters: AnyWorkflowVariable[];
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
