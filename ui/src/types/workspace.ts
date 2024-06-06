import type { Member, Project, ApiResponse } from "@flow/types";

export type Workspace = {
  id: string;
  name: string;
  personal: boolean;
  members?: Member[];
  projects?: Project[];
};

export type CreateWorkspace = {
  workspace: Workspace | undefined;
} & ApiResponse;

export type GetWorkspace = {
  workspaces: Workspace[] | undefined;
  isLoading: boolean;
} & ApiResponse;

export type DeleteWorkspace = {
  workspaceId: string | undefined;
} & ApiResponse;
