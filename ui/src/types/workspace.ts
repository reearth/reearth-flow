import type { Member, Project, ApiResponse } from "@flow/types";

export type Workspace = {
  id: string;
  name: string;
  alias: string;
  personal: boolean;
  members: Member[];
  projects?: Project[];
};

export type WorkspaceMutation = {
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

export type WorkspacePolicy = {
  workspaceId: string;
  // False on free plans (personal workspaces and free teams).
  canCreatePrivateProject: boolean;
  // Whether triggers may run every hour.
  canScheduleHourly: boolean;
};

export type GetWorkspacePolicy = {
  policy?: WorkspacePolicy;
  isLoading: boolean;
} & ApiResponse;

export type GetWorkspacePolicies = {
  // Keyed by workspace ID; undefined while loading or when the query failed.
  policies: Record<string, WorkspacePolicy | undefined>;
  isLoading: boolean;
};
