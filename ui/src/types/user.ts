import { ApiResponse } from "./api";
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
