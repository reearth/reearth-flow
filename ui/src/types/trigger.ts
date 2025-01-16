// Types are subject to change or edit
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
