import { useNodes } from "@xyflow/react";
import { useMemo } from "react";

import { useIndexedDB } from "@flow/lib/indexedDB";
import { useCurrentProject } from "@flow/stores";

export default ({ source }: { source: string }) => {
  const [currentProject] = useCurrentProject();

  const { value: debugRunState } = useIndexedDB("debugRun");

  const debugJobState = useMemo(
    () =>
      debugRunState?.jobs?.find((job) => job.projectId === currentProject?.id),
    [debugRunState, currentProject],
  );

  const sourceNodeStatus = useMemo(() => {
    if (!debugJobState?.nodeExecutions) return undefined;
    const nodes = useNodes();
    const sourceNode = nodes.find((node) => node.id === source);

    console.log("sourceNode", sourceNode); // TODO: delete
    return debugJobState?.nodeExecutions?.find(
      (nodeExecution) => nodeExecution.nodeId === sourceNode?.id,
    )?.status;
  }, [debugJobState?.nodeExecutions, source]);

  return {
    sourceNodeStatus,
  };
};
