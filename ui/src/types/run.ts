export type Run = {
  id: string;
  status: "running" | "queued" | "complete" | "failed";
  startedAt: string;
  completedAt?: string;
};
