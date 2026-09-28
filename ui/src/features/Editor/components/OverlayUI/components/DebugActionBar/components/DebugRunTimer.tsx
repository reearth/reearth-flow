import { memo, useEffect, useState } from "react";

import type { JobStatus } from "@flow/types";
import { formatElapsedTime } from "@flow/utils";

type Props = {
  startedAt?: string;
  completedAt?: string;
  jobStatus?: JobStatus;
};

// startedAt is set when the job is created, so the elapsed time includes any
// time spent queued. Key this component by startedAt so each run starts fresh.
const DebugRunTimer: React.FC<Props> = ({
  startedAt,
  completedAt,
  jobStatus,
}) => {
  const isActive = jobStatus === "queued" || jobStatus === "running";
  const startTime = startedAt ? Date.parse(startedAt) : undefined;

  // Only advanced while the run is active. A run that has just finished keeps
  // its last value until completedAt arrives with the refetched job.
  const [now, setNow] = useState<number | undefined>(() =>
    isActive ? Date.now() : undefined,
  );

  useEffect(() => {
    if (!isActive || startTime === undefined) return;

    let timeout: ReturnType<typeof setTimeout>;
    // Schedule each tick on the next whole second since startedAt, so the
    // display changes exactly when the elapsed second does.
    const tick = () => {
      const current = Date.now();
      setNow(current);
      const intoSecond = (((current - startTime) % 1000) + 1000) % 1000;
      timeout = setTimeout(tick, 1000 - intoSecond);
    };
    timeout = setTimeout(tick, 0);

    return () => clearTimeout(timeout);
  }, [isActive, startTime]);

  if (startTime === undefined || Number.isNaN(startTime)) return null;

  const endTime =
    !isActive && completedAt ? Date.parse(completedAt) : (now ?? NaN);
  if (Number.isNaN(endTime)) return null;

  return (
    <span className="text-xs tabular-nums">
      {formatElapsedTime(endTime - startTime)}
    </span>
  );
};

export default memo(DebugRunTimer);
