import { useEffect, useRef } from "react";

import { ToastAction } from "@flow/components";
import { toast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import { UserDebug } from "@flow/types";

type Props = {
  activeDebugRuns: UserDebug[];
  onJoin: (jobId: string, userName: string) => void;
};

const DebugRunNotification: React.FC<Props> = ({ activeDebugRuns, onJoin }) => {
  const dismissedRef = useRef<Set<string>>(new Set());
  const t = useT();
  useEffect(() => {
    activeDebugRuns.forEach((run) => {
      if (dismissedRef.current.has(run.jobId)) return;

      dismissedRef.current.add(run.jobId);

      const duration = Date.now() - run.startedAt;
      const timeAgo =
        duration < 60000 ? "just now" : `${Math.floor(duration / 60000)}m ago`;

      toast({
        title: `${run.userName} ${t("started a debug run")}`,
        description: `${t("Started ")} ${timeAgo}`,
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
  }, [t, activeDebugRuns, onJoin]);

  return null;
};

export default DebugRunNotification;
