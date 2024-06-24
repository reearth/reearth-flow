import { ApiResponse } from "./api";
import { Workflow } from "./workflow";

export type Project = {
  id: string;
  createdAt: Date;
  updatedAt: Date;
  name: string;
  description: string;
  workspaceId: string;
  workflows?: Workflow[];
  // workspace: Workspace;
};

export type GetWorkspaceProjects = {
  projects?: Project[];
  isLoading: boolean;
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
