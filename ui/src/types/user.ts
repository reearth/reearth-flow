import { ApiResponse } from "./api";

export type GetMe = {
  me: User | undefined | null;
  isLoading: boolean;
} & ApiResponse;

export type User = {
  id: string;
  name: string;
  email: string;
  myWorkspaceId: string;
  lang?: string;
  theme?: string;
  // workspace?: string;
  // auth?: string[];
};
