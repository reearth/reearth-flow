import { useCallback } from "react";

import { useT } from "@flow/lib/i18n";

import { toPlanLimit } from "./errors";

// usePlanLimitMessage explains an error the server returned because of a plan
// limit, or returns undefined for any other error so the caller keeps its own
// message.
export const usePlanLimitMessage = () => {
  const t = useT();

  return useCallback(
    (err: unknown): string | undefined => {
      const planLimit = toPlanLimit(err);
      switch (planLimit?.kind) {
        // Only debug runs count toward the runs at a time; deployment and
        // trigger runs do not.
        case "jobLimitReached":
          return t(
            "This workspace is already running as many debug runs as its plan allows. Wait for one to finish, or cancel one, and try again.",
          );
        case "debugRunsPerHourLimit":
          return t(
            "This workspace has used all the debug runs its plan allows this hour. Try again later.",
          );
        case "deploymentRunsPerDayLimit":
          return t(
            "This workspace has used all the deployment runs its plan allows today. Try again later.",
          );
        case "hourlyScheduleNotAllowed":
          return t(
            "This workspace's plan doesn't include hourly schedules. Choose a daily, weekly or monthly schedule instead.",
          );
        case "enabledTriggerLimit":
          return t(
            "This workspace already has as many triggers enabled as its plan allows. Disable another trigger first.",
          );
        case "assetTooLarge":
          return planLimit.limit
            ? t(
                "This file is larger than this workspace's plan allows. The limit is {{limit}}.",
                { limit: planLimit.limit },
              )
            : t("This file is larger than this workspace's plan allows.");
        default:
          return undefined;
      }
    },
    [t],
  );
};
