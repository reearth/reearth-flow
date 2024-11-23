import { ApiResponse } from "./api";

export type Me = {
  myWorkspaceId: string;
  lang?: string;
  theme?: string;
} & User;

export type GetMe = {
  me: Me | undefined;
  isLoading: boolean;
} & ApiResponse;

export type User = {
  id: string;
  name: string;
  email: string;
  tenant_id?: string;
};

export type SearchUser = {
  user?: User;
};

export type UpdateMe = {
  me?: User;
} & ApiResponse;
