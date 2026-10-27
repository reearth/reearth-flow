import { useWorkspace } from "@flow/lib/gql";
import type { Trigger } from "@flow/types";

// The workspace's plan decides whether triggers may run every hour; the server
// refuses one otherwise. This greys the option out before the user saves. A
// trigger that is already hourly keeps its schedule, so the option stays
// available for it, matching the server.
export const useHourlySchedule = (
  workspaceId?: string,
  savedTrigger?: Trigger,
) => {
  const { useGetWorkspacePolicy } = useWorkspace();
  const { policy } = useGetWorkspacePolicy(workspaceId);
  // Unknown until the policy loads, and treated as not allowed meanwhile.
  const canScheduleHourly = policy?.canScheduleHourly;
  const wasHourly =
    savedTrigger?.eventSource === "TIME_DRIVEN" &&
    savedTrigger.timeInterval === "EVERY_HOUR";

  return {
    hourlyDisabled: !canScheduleHourly && !wasHourly,
    hourlyNotInPlan: canScheduleHourly === false && !wasHourly,
  };
};
