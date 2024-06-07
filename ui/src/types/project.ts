import { ApiResponse } from "./api";
import { Workflow } from "./workflow";

export type Project = {
  id: string;
  isArchived: boolean;
  createdAt: Date;
  updatedAt: Date;
  name: string;
  description: string;
  workspaceId: string;
  workflow?: Workflow;
  // workspace: Workspace;
};

export type GetProjects = {
  projects?: Project[];
  isLoading: boolean;
} & ApiResponse;

export type CreateProject = {
  project?: Project;
} & ApiResponse;

export type DeleteProject = {
  projectId?: string;
} & ApiResponse;
