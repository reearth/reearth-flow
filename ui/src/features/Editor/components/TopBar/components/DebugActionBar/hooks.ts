import { useEffect, useMemo, useState } from "react";

import { useJob } from "@flow/lib/gql/job";
import { useSubscription } from "@flow/lib/gql/subscriptions/useSubscription";
import { useIndexedDB } from "@flow/lib/indexedDB";
import { useCurrentProject } from "@flow/stores";

export default ({
  onDebugRunStart,
}: {
  onDebugRunStart: () => Promise<void>;
}) => {
  const [currentProject] = useCurrentProject();

  const [showDialog, setShowDialog] = useState<
    "deploy" | "share" | "debugStop" | undefined
  >(undefined);

  const handleShowDebugStopDialog = () => setShowDialog("debugStop");
  const handleDialogClose = () => setShowDialog(undefined);

  const { useGetJob } = useJob();

  const { value: debugRunState, updateValue: updateDebugRunState } =
    useIndexedDB("debugRun");

  const debugJobId = useMemo(
    () =>
      debugRunState?.jobs?.find((job) => job.projectId === currentProject?.id)
        ?.jobId,
    [debugRunState, currentProject],
  );

  const debugJob = useGetJob(debugJobId).job;

  const { data: realTimeJobStatus } = useSubscription(
    "GetSubscribedJobStatus",
    debugJobId,
    !debugJob ||
      debugJob?.status === "completed" ||
      debugJob?.status === "failed" ||
      debugJob?.status === "cancelled",
  );

  const jobStatus = useMemo(
    () => realTimeJobStatus ?? debugJob?.status ?? undefined,
    [realTimeJobStatus, debugJob],
  );

  const [debugRunStarted, setDebugRunStarted] = useState(false);

  useEffect(() => {
    if (
      debugRunStarted &&
      (jobStatus === "completed" ||
        jobStatus === "failed" ||
        jobStatus === "cancelled")
    ) {
      setDebugRunStarted(false);
    }
  }, [debugJob, jobStatus, debugRunStarted]);

  const handleDebugRunStart = async () => {
    setDebugRunStarted(true);
    await onDebugRunStart();
  };

  const handleDebugRunReset = async () => {
    const jobState = debugRunState?.jobs?.filter(
      (job) => job.projectId !== currentProject?.id,
    );
    if (!jobState) return;
    await updateDebugRunState({ jobs: jobState });
  };

  return {
    showDialog,
    debugRunStarted,
    jobStatus,
    debugJob,
    handleDebugRunStart,
    handleShowDebugStopDialog,
    handleDialogClose,
    handleDebugRunReset,
  };
};
