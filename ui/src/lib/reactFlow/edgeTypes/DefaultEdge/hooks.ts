import { useNodes } from "@xyflow/react";
import { useCallback, useEffect, useMemo, useState } from "react";

import { config } from "@flow/config";
import { useIndexedDB } from "@flow/lib/indexedDB";
import { DebugRunState, useCurrentProject } from "@flow/stores";

export default ({
  id,
  source,
  selected,
}: {
  id: string;
  source: string;
  selected?: boolean;
}) => {
  const [currentProject] = useCurrentProject();
  const { api } = config();

  const { value: debugRunState, updateValue } = useIndexedDB("debugRun");

  const debugJobState = useMemo(
    () =>
      debugRunState?.jobs?.find((job) => job.projectId === currentProject?.id),
    [debugRunState, currentProject],
  );

  const [hasIntermediateData, setHasIntermediateData] = useState(false);

  const intermediateDataIsSet = useMemo(
    () => debugJobState?.selectedIntermediateData?.edgeId === id,
    [debugJobState?.selectedIntermediateData?.edgeId, id],
  );

  const intermediateDataUrl = useMemo(() => {
    if (api && debugJobState?.jobId) {
      return `${api}/artifacts/${debugJobState.jobId}/feature-store/${id}.jsonl`;
    }
    return undefined;
  }, [api, debugJobState?.jobId, id]);

  useEffect(() => {
    if (intermediateDataUrl) {
      if (
        !hasIntermediateData &&
        debugJobState?.jobId &&
        (debugJobState.status === "completed" ||
          debugJobState?.status === "cancelled" ||
          debugJobState?.status === "failed")
      ) {
        (async () => {
          const response = await fetch(intermediateDataUrl, { method: "HEAD" });
          if (response.ok) {
            setHasIntermediateData(true);
          } else {
            setHasIntermediateData(false);
          }
        })();
      }
    } else {
      setHasIntermediateData(false);
    }
  }, [
    hasIntermediateData,
    debugJobState?.jobId,
    debugJobState?.status,
    intermediateDataUrl,
    id,
  ]);

  const handleIntermediateDataSet = useCallback(async () => {
    if (!selected || !intermediateDataUrl) return;
    const newDebugRunState: DebugRunState = {
      ...debugRunState,
      jobs:
        debugRunState?.jobs?.map((job) =>
          job.projectId === currentProject?.id
            ? {
                ...job,
                selectedIntermediateData: {
                  edgeId: id,
                  url: intermediateDataUrl,
                },
              }
            : job,
        ) ?? [],
    };
    await updateValue(newDebugRunState);
  }, [
    selected,
    intermediateDataUrl,
    debugRunState,
    currentProject,
    id,
    updateValue,
  ]);

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
    intermediateDataIsSet,
    hasIntermediateData,
    handleIntermediateDataSet,
  };
};
