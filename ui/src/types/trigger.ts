import type { ApiResponse } from "./api";
import type { Deployment } from "./deployment";

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

export type Trigger = {
  id: string;
  createdAt: string;
  updatedAt: string;
  lastTriggered?: string;
  workspaceId: string;
  deployment: Deployment;
  deploymentId: string;
  eventSource: EventSourceType;
  authToken?: string;
  timeInterval?: TimeInterval;
  description?: string;
};

export type GetTriggers = {
  triggers?: Trigger[];
} & ApiResponse;

export type CreateTrigger = {
  trigger?: Trigger;
} & ApiResponse;

export type UpdateTrigger = {
  trigger?: Trigger;
} & ApiResponse;

export type DeleteTrigger = {
  success?: boolean;
} & ApiResponse;
