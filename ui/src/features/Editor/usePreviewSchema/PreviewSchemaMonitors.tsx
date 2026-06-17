import { memo, useEffect, useRef } from "react";

import { useJob } from "@flow/lib/gql/job";
import { useSubscription } from "@flow/lib/gql/subscriptions/useSubscription";
import type { ReaderSchemaProbe } from "@flow/stores";
import type { Job } from "@flow/types";

type MonitorProps = {
  probe: ReaderSchemaProbe;
  onComplete: (nodeId: string, job: Job) => void;
  onError: (nodeId: string) => void;
};

/**
 * Headless monitor for a single in-flight preview-schema job. Mirrors the
 * debug-run status watch: it polls the job and listens to the job-status
 * subscription, refetching when a terminal status arrives, then reports the
 * completed job (with its `outputURLs`) or a failure exactly once.
 */
const PreviewSchemaJobMonitor: React.FC<MonitorProps> = ({
  probe,
  onComplete,
  onError,
}) => {
  const { nodeId, jobId } = probe;
  const { useGetJob } = useJob();
  const { job, refetch } = useGetJob(jobId);

  const isTerminal =
    job?.status === "completed" ||
    job?.status === "failed" ||
    job?.status === "cancelled";

  const { data: realTimeJobStatus } = useSubscription(
    "GetSubscribedJobStatus",
    jobId,
    !jobId || isTerminal,
  );

  // Refetch the job once the subscription reports a terminal status so we pick
  // up the populated `outputURLs`.
  useEffect(() => {
    if (
      realTimeJobStatus === "completed" ||
      realTimeJobStatus === "failed" ||
      realTimeJobStatus === "cancelled"
    ) {
      refetch();
    }
  }, [realTimeJobStatus, refetch]);

  const settledRef = useRef(false);

  useEffect(() => {
    if (settledRef.current || !job) return;
    if (job.status === "completed") {
      settledRef.current = true;
      onComplete(nodeId, job);
    } else if (job.status === "failed" || job.status === "cancelled") {
      settledRef.current = true;
      onError(nodeId);
    }
  }, [job, nodeId, onComplete, onError]);

  return null;
};

type Props = {
  probes: ReaderSchemaProbe[];
  onComplete: (nodeId: string, job: Job) => void;
  onError: (nodeId: string) => void;
};

const PreviewSchemaMonitors: React.FC<Props> = ({
  probes,
  onComplete,
  onError,
}) => (
  <>
    {probes
      .filter((probe) => probe.jobId && probe.status === "running")
      .map((probe) => (
        <PreviewSchemaJobMonitor
          key={probe.jobId}
          probe={probe}
          onComplete={onComplete}
          onError={onError}
        />
      ))}
  </>
);

export default memo(PreviewSchemaMonitors);
