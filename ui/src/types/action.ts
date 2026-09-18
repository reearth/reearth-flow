import type { FlowSchema } from "@flow/lib/schemaForm";

import type { ApiResponse } from "./api";

export type Action = {
  name: string;
  description: string;
  type: string;
  categories: string[];
  tags: string[];
  inputPorts: string[];
  outputPorts: string[];
  parameter?: FlowSchema;
  customizations?: FlowSchema;
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
