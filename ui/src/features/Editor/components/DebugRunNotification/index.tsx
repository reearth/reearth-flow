import { useEffect, useRef } from "react";

import { ToastAction } from "@flow/components";
import { toast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import { UserDebug } from "@flow/types";

type Props = {
  activeDebugRuns: UserDebug[];
  currentUserJobIds: string[];
  onJoin: (jobId: string, userName: string) => void;
};

const DebugRunNotification: React.FC<Props> = ({
  activeDebugRuns,
  currentUserJobIds,
  onJoin,
}) => {
  const dismissedRef = useRef<Set<string>>(new Set());
  const t = useT();
  useEffect(() => {
    const activeJobIds = new Set(activeDebugRuns.map((run) => run.jobId));
    dismissedRef.current.forEach((jobId) => {
      if (!activeJobIds.has(jobId)) {
        dismissedRef.current.delete(jobId);
      }
    });

    activeDebugRuns.forEach((run) => {
      if (
        dismissedRef.current.has(run.jobId) ||
        currentUserJobIds.includes(run.jobId)
      )
        return;

      dismissedRef.current.add(run.jobId);

      const duration = Math.max(0, Date.now() - run.startedAt);
      let timeAgo;
      if (duration < 60000) {
        timeAgo = t("just now");
      } else if (duration < 3600000) {
        timeAgo = t("{{minutes}}m ago", {
          minutes: Math.floor(duration / 60000),
        });
      } else {
        timeAgo = t("{{hours}}h ago", {
          hours: Math.floor(duration / 3600000),
        });
      }

      toast({
        title: `${run.userName} ${t("started a debug run")}`,
        description: t("Started {{time}}", { time: timeAgo }),
        action: (
          <ToastAction
            altText="View debug run"
            onClick={() => {
              onJoin(run.jobId, run.userName);
            }}>
            {t("View")}
          </ToastAction>
        ),
      });
    });
  }, [t, activeDebugRuns, currentUserJobIds, onJoin]);

  return null;
};

export default DebugRunNotification;
