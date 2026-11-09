import type { TriggerRefusal } from "@flow/types";

// Whether the scheduler is still retrying a refused run. Once its retries end,
// the run was skipped.
export const isRetrying = (refusal: TriggerRefusal, now: Date = new Date()) =>
  !!refusal.retryUntil && new Date(refusal.retryUntil) > now;
