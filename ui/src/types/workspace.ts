import type { Member, Project, ApiResponse } from "@flow/types";

export type Workspace = {
  id: string;
  name: string;
  personal: boolean;
  members: Member[];
  projects?: Project[];
};

export type CreateWorkspace = {
  workspace?: Workspace;
} & ApiResponse;

export type UpdateWorkspace = {
  workspace?: Workspace;
} & ApiResponse;

export type GetWorkspaces = {
  workspaces?: Workspace[];
  isLoading: boolean;
} & ApiResponse;

export type GetWorkspace = {
  workspace?: Workspace;
  isLoading: boolean;
} & ApiResponse;

export type DeleteWorkspace = {
  workspaceId?: string;
} & ApiResponse;
