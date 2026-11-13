import { useMemo } from "react";

import { useWorkspaceUsers } from "@flow/hooks";
import { useT } from "@flow/lib/i18n";
import type { Job, User } from "@flow/types";

export type JobRunner = Pick<User, "id" | "name" | "photoURL">;

// What started a job: the user who ran it, and the trigger it ran from. Runs
// no user started, such as scheduled and API trigger runs, have no user, and
// jobs from before either was recorded have neither.
export default () => {
  const t = useT();
  const users = useWorkspaceUsers();

  return useMemo(
    () => ({
      runBy: (job: Pick<Job, "runById">): JobRunner | undefined =>
        job.runById
          ? // A user who has left the workspace is still shown, by a placeholder.
            (users.get(job.runById) ?? {
              id: job.runById,
              name: t("Unknown user"),
            })
          : undefined,
      triggerName: (
        job: Pick<Job, "triggerId" | "triggerDescription">,
      ): string | undefined =>
        job.triggerId
          ? (job.triggerDescription ?? t("Deleted trigger"))
          : undefined,
    }),
    [users, t],
  );
};
