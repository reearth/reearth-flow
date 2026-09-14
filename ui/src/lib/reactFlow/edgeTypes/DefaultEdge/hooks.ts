import { ReactFlowState, useStore } from "@xyflow/react";
import { useCallback, useMemo } from "react";

import { useIndexedDB } from "@flow/lib/indexedDB";
import { useCurrentProject } from "@flow/stores";
import type { NodeData } from "@flow/types";

export default ({
  sourceNodeId,
  sourcePortName,
}: {
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

  /**
   * Intermediate data is keyed by workflow as well as by node and port, so
   * matching on `(nodeId, portName)` alone conflates ports that share those
   * across subworkflows. Take the path off the source node — the same field the
   * port that recorded the availability used — rather than deriving it again
   * from the open workflow, so the two can't drift apart.
   */
  const sourceWorkflowPath = useStore(
    useCallback(
      (s: ReactFlowState) =>
        (s.nodeLookup.get(sourceNodeId)?.data as NodeData | undefined)
          ?.workflowPath ?? "",
      [sourceNodeId],
    ),
  );

  const hasIntermediateData = useMemo(
    () =>
      !!sourcePortName &&
      !!debugJobState?.availableIntermediateData?.some(
        (e) =>
          e.nodeId === sourceNodeId &&
          e.portName === sourcePortName &&
          (e.workflowPath ?? "") === sourceWorkflowPath,
      ),
    [
      debugJobState?.availableIntermediateData,
      sourceNodeId,
      sourcePortName,
      sourceWorkflowPath,
    ],
  );

  return { jobStatus, hasIntermediateData };
};
