import { useMemo } from "react";

import { useIndexedDB } from "@flow/lib/indexedDB";
import { useCurrentProject } from "@flow/stores";

export default () => {
  const [currentProject] = useCurrentProject();

  const { value: debugRunState } = useIndexedDB("debugRun");

  const debugJobId = useMemo(
    () =>
      debugRunState?.jobs?.find((job) => job.projectId === currentProject?.id)
        ?.jobId,
    [debugRunState, currentProject],
  );

  return {
    debugJobId,
  };
};
