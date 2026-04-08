import { useParams } from "@tanstack/react-router";
import { useMemo } from "react";

import { useIndexedDB } from "@flow/lib/indexedDB";

export default ({
  currentWorkflowId: _currentWorkflowId,
}: {
  currentWorkflowId?: string;
}) => {
  const { value: debugRunState } = useIndexedDB("debugRun");
  const { debugId } = useParams({ strict: false }) as { debugId?: string };
  const debugJobState = useMemo(
    () => debugRunState?.jobs?.find((job) => job.jobId === debugId),
    [debugRunState, debugId],
  );

  const jobStatus = useMemo(
    () => debugJobState?.status,
    [debugJobState?.status],
  );

  return { jobStatus };
};
