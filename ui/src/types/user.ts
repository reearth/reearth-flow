import { ApiResponse } from "./api";
import { JobStatus } from "./job";
import type { Workspace } from "./workspace";

export type Me = {
  myWorkspaceId: string;
  lang?: string;
  theme?: string;
} & User;

export type GetMe = {
  me: Me | undefined;
  isLoading: boolean;
} & ApiResponse;

export type GetMeAndWorkspaces = {
  me: Me | undefined;
  workspaces: Workspace[] | undefined;
  isLoading: boolean;
} & ApiResponse;

export type User = {
  id: string;
  name: string;
  email: string;
};

export type SearchUser = {
  user?: User;
};

export type UpdateMe = {
  me?: User;
} & ApiResponse;

export type AwarenessUser = {
  clientId: number;
  cursor?: {
    x: number;
    y: number;
  };
  viewport?: {
    x: number;
    y: number;
    zoom: number;
  };
  color: string;
  userName: string;
  currentWorkflowId?: string;
  openWorkflowIds?: string[];
  debugRun?: UserDebugRun;
};

export type UserDebugRun = {
  projectId: string;
  jobId: string;
  startedAt: number;
  status: JobStatus;
};
