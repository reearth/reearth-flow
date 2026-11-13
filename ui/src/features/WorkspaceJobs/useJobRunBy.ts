import { useCallback } from "react";

import { useWorkspaceUsers } from "@flow/hooks";
import { useT } from "@flow/lib/i18n";
import type { Job } from "@flow/types";

// Names who or what started a job: the user, or for runs no user started,
// such as scheduled and API trigger runs, the trigger. Jobs from before either
// was recorded have neither.
export default () => {
  const t = useT();
  const users = useWorkspaceUsers();

  return useCallback(
    (job: Pick<Job, "runById" | "triggerId" | "triggerDescription">) => {
      if (job.runById) {
        // A user who has left the workspace is still named, by a placeholder.
        return users.get(job.runById)?.name ?? t("Unknown user");
      }
      if (job.triggerId) {
        if (job.triggerDescription === undefined) return t("Deleted trigger");
        return job.triggerDescription
          ? t("Trigger: {{name}}", { name: job.triggerDescription })
          : t("Trigger");
      }
      return t("N/A");
    },
    [users, t],
  );
};
