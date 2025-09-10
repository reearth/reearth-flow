import { useNodes } from "@xyflow/react";
import { useCallback, useEffect, useMemo, useState } from "react";

import { config } from "@flow/config";
import { useIndexedDB } from "@flow/lib/indexedDB";
import {
  DebugRunState,
  SelectedIntermediateData,
  useCurrentProject,
} from "@flow/stores";
import { NodeCustomizations } from "@flow/types";

export default ({
  id,
  source,
  target,
  selected,
}: {
  id: string;
  source: string;
  target: string;
  selected?: boolean;
}) => {
  const [currentProject] = useCurrentProject();
  const { api } = config();

  const { value: debugRunState, updateValue } = useIndexedDB("debugRun");
  const nodes = useNodes();

  const debugJobState = useMemo(
    () =>
      debugRunState?.jobs?.find((job) => job.projectId === currentProject?.id),
    [debugRunState, currentProject],
  );

  // Get source and target node names
  const sourceNode = useMemo(
    () => nodes.find((node) => node.id === source),
    [nodes, source],
  );

  const targetNode = useMemo(
    () => nodes.find((node) => node.id === target),
    [nodes, target],
  );

  const edgeDisplayName = useMemo(() => {
    const sourceCustomizations = sourceNode?.data.customizations as
      | NodeCustomizations
      | undefined;
    const targetCustomizations = targetNode?.data.customizations as
      | NodeCustomizations
      | undefined;
    const sourceName =
      sourceCustomizations?.customName ||
      sourceNode?.data?.officialName ||
      sourceNode?.type ||
      `Node ${source}`;
    const targetName =
      targetCustomizations?.customName ||
      targetNode?.data?.officialName ||
      targetNode?.type ||
      `Node ${target}`;
    return `${sourceName} → ${targetName}`;
  }, [sourceNode, targetNode, source, target]);

  const jobStatus = useMemo(
    () => debugJobState?.status,
    [debugJobState?.status],
  );

  const tempWorkflowHasPossibleIssuesFlag = useMemo(
    () => debugJobState?.tempWorkflowHasPossibleIssuesFlag,
    [debugJobState?.tempWorkflowHasPossibleIssuesFlag],
  );

  const [hasIntermediateData, setHasIntermediateData] = useState(false);

  const intermediateDataIsSet = useMemo(
    () =>
      debugJobState?.selectedIntermediateData?.find((sid) => sid.edgeId === id),
    [debugJobState?.selectedIntermediateData, id],
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

  const handleIntermediateDataSet = useCallback(
    async (autoSelect = false) => {
      if ((!selected && !autoSelect) || !intermediateDataUrl) return;

      const newDebugRunState: DebugRunState = {
        ...debugRunState,
        jobs:
          debugRunState?.jobs?.map((job) => {
            if (job.projectId !== currentProject?.id) return job;

            const currentData = job.selectedIntermediateData ?? [];
            const isCurrentlySelected = currentData.find(
              (sid) => sid.edgeId === id,
            );

            let newSelectedIntermediateData:
              | SelectedIntermediateData[]
              | undefined;

            if (isCurrentlySelected) {
              // Remove the item
              const filtered = currentData.filter((sid) => sid.edgeId !== id);
              // Keep as empty array (don't set to undefined) - user has interacted
              newSelectedIntermediateData = filtered;
            } else {
              // Add the item (initialize array if undefined)
              newSelectedIntermediateData = [
                ...currentData,
                {
                  edgeId: id,
                  url: intermediateDataUrl,
                  displayName: edgeDisplayName,
                  sourceName: (sourceNode?.data?.name ||
                    sourceNode?.data?.title ||
                    sourceNode?.type ||
                    `Node ${source}`) as string,
                  targetName: (targetNode?.data?.name ||
                    targetNode?.data?.title ||
                    targetNode?.type ||
                    `Node ${target}`) as string,
                },
              ];
            }

            return {
              ...job,
              selectedIntermediateData: newSelectedIntermediateData,
            };
          }) ?? [],
      };
      await updateValue(newDebugRunState);
    },
    [
      selected,
      intermediateDataUrl,
      debugRunState,
      currentProject,
      id,
      updateValue,
      edgeDisplayName,
      sourceNode,
      targetNode,
      source,
      target,
    ],
  );

  // Auto-select intermediate data for writer target nodes
  useEffect(() => {
    const hasNeverBeenTouched =
      debugJobState?.selectedIntermediateData === undefined;

    if (
      hasIntermediateData &&
      targetNode?.type === "writer" &&
      !intermediateDataIsSet &&
      hasNeverBeenTouched && // Only auto-select if user has never interacted with selections
      debugJobState?.status === "completed"
    ) {
      handleIntermediateDataSet(true); // Pass autoSelect=true
    }
  }, [
    hasIntermediateData,
    targetNode?.type,
    intermediateDataIsSet,
    debugJobState?.selectedIntermediateData,
    debugJobState?.status,
    handleIntermediateDataSet,
  ]);

  // Optional: Add source node status if needed later
  // const sourceNodeStatus = useMemo(() => {
  //   if (!debugJobState?.nodeExecutions) return undefined;
  //   return debugJobState?.nodeExecutions?.find(
  //     (nodeExecution) => nodeExecution.nodeId === source,
  //   )?.status;
  // }, [debugJobState?.nodeExecutions, source]);

  return {
    // sourceNodeStatus,
    jobStatus,
    tempWorkflowHasPossibleIssuesFlag,
    intermediateDataIsSet,
    hasIntermediateData,
    handleIntermediateDataSet,
  };
};
