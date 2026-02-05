import { useEffect, useMemo, useState } from "react";

import { useJob } from "@flow/lib/gql/job";
import { useSubscription } from "@flow/lib/gql/subscriptions/useSubscription";
import { useIndexedDB } from "@flow/lib/indexedDB";
import { useCurrentProject } from "@flow/stores";
import { AnyWorkflowVariable } from "@flow/types";

export default ({
  onDebugRunStart,
  onDebugRunStop,
  customDebugRunWorkflowVariables,
}: {
  onDebugRunStart: () => Promise<void>;
  onDebugRunStop: () => Promise<void>;
  customDebugRunWorkflowVariables: AnyWorkflowVariable[] | undefined;
}) => {
  const [currentProject] = useCurrentProject();
  const [showOverlayElement, setshowOverlayElement] = useState<
    | "debugStart"
    | "debugStop"
    | "debugRuns"
    | "debugWorkflowVariables"
    | undefined
  >(undefined);

  const handleShowDebugStartPopover = () => setshowOverlayElement("debugStart");
  const handleShowDebugStopPopover = () => setshowOverlayElement("debugStop");
  const handleShowDebugActiveRunsPopover = () =>
    setshowOverlayElement("debugRuns");
  const handleShowDebugWorkflowVariablesDialog = () =>
    setshowOverlayElement("debugWorkflowVariables");
  const handlePopoverClose = () => setshowOverlayElement(undefined);
  const [debugRunStarted, setDebugRunStarted] = useState(false);

  const { useGetJob } = useJob();

  const { value: debugRunState, updateValue: updateDebugRunState } =
    useIndexedDB("debugRun");

  const debugJobId = useMemo(
    () =>
      debugRunState?.jobs?.find((job) => job.projectId === currentProject?.id)
        ?.jobId,
    [debugRunState, currentProject],
  );

  const { job, refetch } = useGetJob(debugJobId);

  const debugJob = job;

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

  console.log("realTimeJobStatus", jobStatus);
  useEffect(() => {
    if (
      debugRunStarted &&
      (jobStatus === "completed" ||
        jobStatus === "failed" ||
        jobStatus === "cancelled")
    ) {
      setDebugRunStarted(false);
    } else if (!debugRunStarted && jobStatus === "running") {
      setDebugRunStarted(true);
    }
  }, [debugJob, jobStatus, debugRunStarted]);

  const handleDebugRunStart = async () => {
    if (
      customDebugRunWorkflowVariables &&
      customDebugRunWorkflowVariables.length > 0
    ) {
      handleShowDebugWorkflowVariablesDialog();
    } else {
      setDebugRunStarted(true);
      await onDebugRunStart();
      handlePopoverClose();
    }
  };

  const handleDebugRunStop = async () => {
    await onDebugRunStop();
    setDebugRunStarted(false);
    handlePopoverClose();
  };

  const handleDebugRunReset = async () => {
    const jobState = debugRunState?.jobs?.filter(
      (job) => job.projectId !== currentProject?.id,
    );
    if (!jobState) return;
    await updateDebugRunState({ jobs: jobState });
  };

  useEffect(() => {
    if (
      realTimeJobStatus === "completed" ||
      realTimeJobStatus === "failed" ||
      realTimeJobStatus === "cancelled"
    ) {
      console.log("Refetching job data after status change");
      refetch();
    }
  }, [realTimeJobStatus, refetch]);

  return {
    showOverlayElement,
    debugRunStarted,
    jobStatus,
    debugJob,
    handleDebugRunStart,
    handleDebugRunStop,
    handleShowDebugStartPopover,
    handleShowDebugStopPopover,
    handleShowDebugActiveRunsPopover,
    handleShowDebugWorkflowVariablesDialog,
    handlePopoverClose,
    handleDebugRunReset,
  };
};
