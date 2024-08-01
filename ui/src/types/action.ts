import { ApiResponse } from "./api";

export type Action = {
  name: string;
  description: string;
  type: string;
  categories: string[];
};

export type ActionsSegregated = {
  [bySegregated: string]: Action[] | undefined;
};

export type Segregated = {
  [key: string]: ActionsSegregated;
};

export type GetActions = {
  actions?: Action[];
  isLoading: boolean;
} & ApiResponse;

export type GetAction = {
  action?: Action;
  isLoading: boolean;
} & ApiResponse;

export type GetActionSegregated = {
  actions?: Segregated;
  isLoading: boolean;
} & ApiResponse;
