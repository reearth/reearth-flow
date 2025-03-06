import { ApiResponse } from "./api";
import { Deployment } from "./deployment";
import { Job } from "./job";
import { Workflow } from "./workflow";

export type Project = {
  id: string;
  name: string;
  createdAt: string;
  updatedAt: string;
  description: string;
  workspaceId: string;
  workflows?: Workflow[];
  sharedToken?: string;
  deployment?: Deployment;
};

export type ProjectToImport = {
  name: string;
  description: string;
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
  job?: Job;
} & ApiResponse;

export type ShareProject = {
  projectId?: string;
  sharingUrl?: string;
} & ApiResponse;

export type UnshareProject = {
  projectId?: string;
} & ApiResponse;
