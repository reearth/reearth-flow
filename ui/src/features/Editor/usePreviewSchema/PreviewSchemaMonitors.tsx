import { memo, useCallback, useEffect, useMemo, useRef, useState } from "react";

import { useAuth } from "@flow/lib/auth";
import { OnJobStatusChangeSubscription } from "@flow/lib/gql/__gen__/graphql";
import { toJobStatus } from "@flow/lib/gql/convert";
import { useJob } from "@flow/lib/gql/job";
import { useSubscription } from "@flow/lib/gql/subscriptions/useSubscription";
import { useSubscriptionSetup } from "@flow/lib/gql/subscriptions/useSubscriptionSetup";
import type { ReaderSchemaProbe } from "@flow/stores";
import type { Job } from "@flow/types";

type MonitorProps = {
  probe: ReaderSchemaProbe;
  accessToken?: string;
  onComplete: (nodeId: string, job: Job) => void;
  onError: (nodeId: string) => void;
};

// Watches one in-flight probe job via the jobStatus subscription; on a terminal
// status, fetches the job once for its outputURLs and reports it exactly once.
const PreviewSchemaJobMonitor: React.FC<MonitorProps> = ({
  probe,
  accessToken,
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

  const variables = useMemo(() => ({ jobId }), [jobId]);
  const jobStatusDataFormatter = useCallback(
    (data: OnJobStatusChangeSubscription) => toJobStatus(data.jobStatus),
    [],
  );

  const disabled = !jobId || !accessToken || isTerminal;

  useSubscriptionSetup<OnJobStatusChangeSubscription>(
    "GetSubscribedJobStatus",
    accessToken,
    variables,
    jobId,
    jobStatusDataFormatter,
    disabled,
  );

  const { data: realTimeJobStatus } = useSubscription(
    "GetSubscribedJobStatus",
    jobId,
    disabled,
  );

  // The subscription only carries status; refetch once on a terminal event to
  // pick up the populated `outputURLs`.
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
}) => {
  const { getAccessToken } = useAuth();
  const [accessToken, setAccessToken] = useState<string | undefined>(undefined);

  useEffect(() => {
    if (accessToken) return;
    (async () => {
      const token = await getAccessToken();
      setAccessToken(token);
    })();
  }, [accessToken, getAccessToken]);

  return (
    <>
      {probes
        .filter((probe) => probe.jobId && probe.status === "running")
        .map((probe) => (
          <PreviewSchemaJobMonitor
            key={probe.jobId}
            probe={probe}
            accessToken={accessToken}
            onComplete={onComplete}
            onError={onError}
          />
        ))}
    </>
  );
};

export default memo(PreviewSchemaMonitors);
