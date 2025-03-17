import { useEffect, useMemo, useState } from "react";

import { useProjectExport } from "@flow/hooks";
import { useJob } from "@flow/lib/gql/job";
import { useIndexedDB } from "@flow/lib/indexedDB";
import { useCurrentProject } from "@flow/stores";

export default ({
  onDebugRunStart,
}: {
  onDebugRunStart: () => Promise<void>;
}) => {
  const [currentProject] = useCurrentProject();

  const { handleProjectExport } = useProjectExport(currentProject);

  const handleShowDeployDialog = () => setShowDialog("deploy");
  const handleShowShareDialog = () => setShowDialog("share");
  const handleShowDebugStopDialog = () => setShowDialog("debugStop");
  const handleDialogClose = () => setShowDialog(undefined);

  const { useGetJob } = useJob();

  const { value: debugRunState, updateValue: updateDebugRunState } =
    useIndexedDB("debugRun");

  const debugJobId = useMemo(
    () =>
      debugRunState?.jobs?.find((job) => job.projectId === currentProject?.id)
        ?.jobId ?? "",
    [debugRunState, currentProject],
  );

  const debugJob = useGetJob(debugJobId).job;

  const [showDialog, setShowDialog] = useState<
    "deploy" | "share" | "debugStop" | undefined
  >(undefined);
  const [debugRunStarted, setDebugRunStarted] = useState(false);

  useEffect(() => {
    if (
      debugRunStarted &&
      debugJob &&
      (debugJob.status === "completed" ||
        debugJob.status === "failed" ||
        debugJob.status === "cancelled")
    ) {
      setDebugRunStarted(false);
    }
  }, [debugJob, debugRunStarted]);

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
    debugJob,
    handleDebugRunStart,
    handleShowDeployDialog,
    handleShowShareDialog,
    handleShowDebugStopDialog,
    handleDialogClose,
    handleDebugRunReset,
    handleProjectExport,
  };
};
