import { ApiResponse } from "./api";
import { Deployment } from "./deployment";
import { Workflow } from "./workflow";

export type Project = {
  id: string;
  name: string;
  createdAt: string;
  updatedAt: string;
  description: string;
  workspaceId: string;
  workflows?: Workflow[];
  deployment?: Deployment;
};

export type GetWorkspaceProjects = {
  pages?: ({ projects?: Project[] } | undefined)[];
  hasNextPage: boolean;
  fetchNextPage: () => void;
  isLoading: boolean;
  isFetching: boolean;
} & ApiResponse;

export type GetProject = {
  project?: Project;
  isLoading: boolean;
} & ApiResponse;

export type CreateProject = {
  project?: Project;
} & ApiResponse;

export type UpdateProject = {
  project?: Project;
} & ApiResponse;

export type DeleteProject = {
  projectId?: string;
} & ApiResponse;

export type RunProject = {
  projectId?: string;
  started?: boolean;
} & ApiResponse;
