import { ApiResponse } from "./api";

export type Me = {
  myWorkspaceId: string;
  lang?: string;
  theme?: string;
  workspaces?: Workspace[];
} & User;

export type GetMe = {
  me: Me | undefined;
  isLoading: boolean;
} & ApiResponse;

export type GetMeAndWorkspaces = {
  me: Me | undefined;
  isLoading: boolean;
} & ApiResponse;

export type User = {
  id: string;
  name: string;
  email: string;
};

export type Workspace = {
  id: string;
  name: string;
  personal: boolean;
};

export type SearchUser = {
  user?: User;
};

export type UpdateMe = {
  me?: User;
} & ApiResponse;
