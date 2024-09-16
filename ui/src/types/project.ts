import { ApiResponse } from "./api";
import { Workflow } from "./workflow";

export type Project = {
  id: string;
  name: string;
  createdAt: Date;
  updatedAt: Date;
  description: string;
  workspaceId: string;
  workflows?: Workflow[];
  // workspace: Workspace;
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
