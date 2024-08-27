import { ApiResponse } from "./api";

export type Action = {
  name: string;
  description: string;
  type: string;
  categories: string[];
  inputPorts: string[];
  outputPorts: string[];
  parameters: any; // eslint-disable-line @typescript-eslint/no-explicit-any
  builtin: boolean;
};

export type ActionsSegregated = Record<string, Action[] | undefined>;

export type Segregated = Record<string, ActionsSegregated>;

export type GetActions = {
  actions?: Action[];
  isLoading: boolean;
} & ApiResponse;

export type GetAction = {
  action?: Action;
  isLoading: boolean;
} & ApiResponse;

export type GetActionsSegregated = {
  actions?: Segregated;
  isLoading: boolean;
} & ApiResponse;
