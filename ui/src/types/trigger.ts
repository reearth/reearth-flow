import type { ApiResponse } from "./api";
import type { Deployment } from "./deployment";
import type { AnyWorkflowVariable, VarType } from "./workflowVariables";

export type EventSourceType = "TIME_DRIVEN" | "API_DRIVEN";
export enum TriggerOrderBy {
  CreatedAt = "createdAt",
  UpdatedAt = "updatedAt",
  LastTriggered = "lastTriggered",
  Description = "description",
}

export enum TimeIntervalEnum {
  EVERY_DAY = "EVERY_DAY",
  EVERY_HOUR = "EVERY_HOUR",
  EVERY_MONTH = "EVERY_MONTH",
  EVERY_WEEK = "EVERY_WEEK",
}

export type TimeInterval = keyof typeof TimeIntervalEnum;

export type Variable = {
  key: string;
  type: VarType;
  value: any;
};

export type TriggerVariableConfig = {
  name: string;
  type: VarType;
  defaultValue: any;
  deploymentDefault: any;
  // Set only when the deployment saved its variables' definitions.
  required?: boolean;
  config?: AnyWorkflowVariable["config"];
  // The trigger has a value for a variable the deployment no longer has.
  stale?: boolean;
};

export type Trigger = {
  id: string;
  createdAt: string;
  updatedAt: string;
  lastTriggered?: string;
  workspaceId: string;
  deployment: Deployment;
  deploymentId: string;
  eventSource: EventSourceType;
  // The last characters of the auth token. Unset for a token set by hand
  // before tokens were generated.
  authTokenHint?: string;
  timeInterval?: TimeInterval;
  description?: string;
  variables: Variable[];
  enabled: boolean;
};

export type GetTriggers = {
  triggers?: Trigger[];
} & ApiResponse;

// authToken is set only when the server just issued a new token. It is never
// returned again, so it has to be shown to the user there and then.
export type CreateTrigger = {
  trigger?: Trigger;
  authToken?: string;
} & ApiResponse;

export type UpdateTrigger = {
  trigger?: Trigger;
  authToken?: string;
} & ApiResponse;

export type RegenerateTriggerToken = {
  trigger?: Trigger;
  authToken?: string;
} & ApiResponse;

export type DeleteTrigger = {
  success?: boolean;
} & ApiResponse;
