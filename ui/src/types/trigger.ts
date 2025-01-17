import { Deployment } from "./deployment";
import { Workspace } from "./workspace";

// Enums
export enum EventSourceType {
  API_DRIVEN = "API_DRIVEN",
  TIME_DRIVEN = "TIME_DRIVEN",
}

export enum TimeInterval {
  EVERY_DAY = "EVERY_DAY",
  EVERY_HOUR = "EVERY_HOUR",
  EVERY_MONTH = "EVERY_MONTH",
  EVERY_WEEK = "EVERY_WEEK",
}

// Trigger Type
export type Trigger = {
  id: string;
  createdAt: string;
  updatedAt: string;
  lastTriggered?: string | null;
  workspaceId: string;
  workspace?: Workspace;
  deployment: Deployment;
  deploymentId: string;
  eventSource: EventSourceType;
  authToken?: string | null;
  timeInterval?: TimeInterval | null;
};
