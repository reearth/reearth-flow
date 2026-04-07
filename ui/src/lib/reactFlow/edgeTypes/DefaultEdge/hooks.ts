import { useMemo } from "react";

import { useIndexedDB } from "@flow/lib/indexedDB";
import { useCurrentProject } from "@flow/stores";

export default ({
  currentWorkflowId: _currentWorkflowId,
}: {
  currentWorkflowId?: string;
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

  return { jobStatus };
};
