import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { useEchoDropdownSync } from "@flow/features/Editor/editorContext";
import { useJob } from "@flow/lib/gql/job";
import { ECHO_KEYS } from "@flow/lib/yjs";
import { useSubscription } from "@flow/lib/gql/subscriptions/useSubscription";
import { useIndexedDB } from "@flow/lib/indexedDB";
import { useCurrentProject } from "@flow/stores";
import { AnyWorkflowVariable } from "@flow/types";

export default ({
  onDebugRunStart,
  onDebugRunStop,
  onResetDebugRunWorkflowVariables,
  refetchWorkflowVariables,
  onUserFocusedElement,
  customDebugRunWorkflowVariables,
}: {
  onDebugRunStart: () => Promise<void>;
  onDebugRunStop: () => Promise<void>;
  onResetDebugRunWorkflowVariables: () => void;
  onUserFocusedElement?: (isOpen: boolean) => void;
  refetchWorkflowVariables: () => void;
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

  const handleShowDebugStartPopover = () => {
    onUserFocusedElement?.(true);
    setshowOverlayElement("debugStart");
  };
  const handleShowDebugStopPopover = () => {
    onUserFocusedElement?.(true);
    setshowOverlayElement("debugStop");
  };
  const handleShowDebugActiveRunsPopover = () => {
    onUserFocusedElement?.(true);
    setshowOverlayElement("debugRuns");
  };
  const handleShowDebugWorkflowVariablesDialog = () => {
    refetchWorkflowVariables();
    setshowOverlayElement("debugWorkflowVariables");
  };
  const handlePopoverClose = () => {
    onUserFocusedElement?.(false);
    setshowOverlayElement(undefined);
  };

  // Each popover echoes under its own key and opens when the spotlighted user
  // opens it. debugStop is left out on purpose: it is a confirmation for
  // cancelling a run, and prompting a follower to stop a job they do not own is
  // not passive observation.
  const setDebugStartOpen = useCallback((open: boolean) => {
    setshowOverlayElement(open ? "debugStart" : undefined);
  }, []);
  const setDebugRunsOpen = useCallback((open: boolean) => {
    setshowOverlayElement(open ? "debugRuns" : undefined);
  }, []);
  const setDebugVariablesOpen = useCallback((open: boolean) => {
    setshowOverlayElement(open ? "debugWorkflowVariables" : undefined);
  }, []);

  useEchoDropdownSync(
    ECHO_KEYS.debugStartPopover,
    showOverlayElement === "debugStart",
    setDebugStartOpen,
  );
  useEchoDropdownSync(
    ECHO_KEYS.debugRunsPopover,
    showOverlayElement === "debugRuns",
    setDebugRunsOpen,
  );
  useEchoDropdownSync(
    ECHO_KEYS.debugVariablesDialog,
    showOverlayElement === "debugWorkflowVariables",
    setDebugVariablesOpen,
  );
  const [debugRunStarted, setDebugRunStarted] = useState(false);
  const isStartingRef = useRef(false);

  const { useGetJob } = useJob();

  const { value: debugRunState, updateValue: updateDebugRunState } =
    useIndexedDB("debugRun");

  const debugJobState = useMemo(
    () =>
      debugRunState?.jobs?.find((job) => job.projectId === currentProject?.id),
    [debugRunState, currentProject],
  );

  const { job, refetch } = useGetJob(debugJobState?.jobId);

  const debugJob = job;

  const { data: realTimeJobStatus } = useSubscription(
    "GetSubscribedJobStatus",
    debugJobState?.jobId,
    !debugJob ||
      debugJob?.status === "completed" ||
      debugJob?.status === "failed" ||
      debugJob?.status === "cancelled",
  );

  const jobStatus = useMemo(
    () => realTimeJobStatus ?? debugJob?.status ?? undefined,
    [realTimeJobStatus, debugJob],
  );

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
    if (isStartingRef.current) return;
    isStartingRef.current = true;
    try {
      if (
        customDebugRunWorkflowVariables &&
        customDebugRunWorkflowVariables.length > 0
      ) {
        handleShowDebugWorkflowVariablesDialog();
      } else {
        setDebugRunStarted(true);
        handlePopoverClose();
        try {
          await onDebugRunStart();
        } catch (e) {
          setDebugRunStarted(false);
          throw e;
        }
      }
    } finally {
      isStartingRef.current = false;
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
    onResetDebugRunWorkflowVariables();
  };

  useEffect(() => {
    if (
      (realTimeJobStatus === "completed" ||
        realTimeJobStatus === "failed" ||
        realTimeJobStatus === "cancelled") &&
      debugJobState?.status !== realTimeJobStatus
    ) {
      refetch();
    }
  }, [realTimeJobStatus, debugJobState, refetch]);

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
