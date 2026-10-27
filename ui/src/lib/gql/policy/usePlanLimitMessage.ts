import { useCallback } from "react";

import { useT } from "@flow/lib/i18n";

import { toPlanLimitKind } from "./errors";

// usePlanLimitMessage explains an error the server returned because of a plan
// limit, or returns undefined for any other error so the caller keeps its own
// message.
export const usePlanLimitMessage = () => {
  const t = useT();

  return useCallback(
    (err: unknown): string | undefined => {
      switch (toPlanLimitKind(err)) {
        case "jobLimitReached":
          return t(
            "This workspace is already running as many jobs as its plan allows. Wait for a job to finish, or cancel one, and try again.",
          );
        case "hourlyScheduleNotAllowed":
          return t(
            "This workspace's plan doesn't include hourly schedules. Choose a daily, weekly or monthly schedule instead.",
          );
        default:
          return undefined;
      }
    },
    [t],
  );
};
