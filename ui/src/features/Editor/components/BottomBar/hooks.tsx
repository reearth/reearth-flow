import { useMemo } from "react";

import { useJob } from "@flow/lib/gql/job";
import { useIndexedDB } from "@flow/lib/indexedDB";
import { useCurrentProject } from "@flow/stores";

export default () => {
  const [currentProject] = useCurrentProject();
  const { useGetJob } = useJob();

  const { value: debugRunState } = useIndexedDB("debugRun");

  const debugJobId = useMemo(
    () =>
      debugRunState?.jobs?.find((job) => job.projectId === currentProject?.id)
        ?.jobId ?? "",
    [debugRunState, currentProject],
  );

  const debugJob = useGetJob(debugJobId).job;

  return {
    debugJob,
  };
};
