import { useMemo } from "react";

import { useIndexedDB } from "@flow/lib/indexedDB";
import { useCurrentProject } from "@flow/stores";

export default ({
  currentWorkflowId: _currentWorkflowId,
  sourceNodeId,
  sourcePortName,
}: {
  currentWorkflowId?: string;
  sourceNodeId: string;
  sourcePortName?: string | null;
}) => {
  const [currentProject] = useCurrentProject();
  const { value: debugRunState } = useIndexedDB("debugRun");

  const debugJobState = useMemo(
    () =>
      debugRunState?.jobs?.find((job) => job.projectId === currentProject?.id),
    [debugRunState, currentProject],
  );

  const jobStatus = useMemo(
    () => debugJobState?.status,
    [debugJobState?.status],
  );

  const hasIntermediateData = useMemo(
    () =>
      !!sourcePortName &&
      !!debugJobState?.availableIntermediateData?.some(
        (e) => e.nodeId === sourceNodeId && e.portName === sourcePortName,
      ),
    [debugJobState?.availableIntermediateData, sourceNodeId, sourcePortName],
  );

  return { jobStatus, hasIntermediateData };
};
