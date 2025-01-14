import { ApiResponse } from "./api";
import { Job } from "./job";

export type Trigger = {
  id: string;
  authToken: string;
  createdAt: string;
  updatedAt: string;
  deployment: string;
  projectId: string | null;
  timeInterval: string;
  lastTriggered: string;
  eventSource: "api" | "cms" | "manual";
};

export type GetTriggers = {
  pages?: ({ triggers?: Trigger[] } | undefined)[];
  hasNextPage: boolean;
  fetchNextPage: () => void;
  isLoading: boolean;
  isFetching: boolean;
} & ApiResponse;

export type CreateTrigger = {
  trigger?: Trigger;
} & ApiResponse;

export type UpdateTrigger = {
  trigger?: Trigger;
} & ApiResponse;

export type DeleteTrigger = {
  triggerId?: string;
} & ApiResponse;

export type ExecuteTrigger = {
  job?: Job;
} & ApiResponse;
