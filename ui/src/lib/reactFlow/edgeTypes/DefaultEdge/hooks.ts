import { useNodes } from "@xyflow/react";
import { MouseEvent, useCallback, useEffect, useMemo, useState } from "react";

import { config } from "@flow/config";
import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import { useT } from "@flow/lib/i18n";
import { useIndexedDB } from "@flow/lib/indexedDB";
import {
  DebugRunState,
  SelectedIntermediateData,
  useCurrentProject,
} from "@flow/stores";
import { NodeCustomizations } from "@flow/types";

export default ({
  id,
  currentWorkflowId,
  source,
  sourceHandleId,
  target,
  selected,
}: {
  id: string;
  currentWorkflowId?: string;
  source: string;
  sourceHandleId?: string | null;
  target: string;
  selected?: boolean;
}) => {
  const t = useT();
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

  const jobStatus = useMemo(
    () => debugJobState?.status,
    [debugJobState?.status],
  );

  const [hasIntermediateData, setHasIntermediateData] = useState(false);

  const intermediateDataIsSet = useMemo(
    () =>
      debugJobState?.selectedIntermediateData?.find((sid) => sid.edgeId === id),
    [debugJobState?.selectedIntermediateData, id],
  );

  const intermediateDataUrl = useMemo(() => {
    if (!api || !debugJobState?.jobId) {
      return undefined;
    }
    if (!currentWorkflowId || currentWorkflowId === DEFAULT_ENTRY_GRAPH_ID) {
      return `${api}/artifacts/${debugJobState.jobId}/feature-store/${id}.jsonl.zst`;
    }
    return `${api}/artifacts/${debugJobState.jobId}/feature-store/${currentWorkflowId}.${id}.jsonl.zst`;
  }, [api, debugJobState?.jobId, id, currentWorkflowId]);

  useEffect(() => {
    if (intermediateDataUrl) {
      if (
        !hasIntermediateData &&
        debugJobState?.jobId &&
        debugJobState.status === "completed"
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
    async (autoSelect?: boolean) => {
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

            let newFocusedURL: string | undefined = undefined;

            if (isCurrentlySelected) {
              // Remove the item
              const filtered = currentData.filter((sid) => sid.edgeId !== id);
              // Keep as empty array (don't set to undefined) - user has interacted
              newSelectedIntermediateData = filtered;

              const removedIndex = currentData.findIndex(
                (sid) => sid.edgeId === id,
              );

              // Try to focus on the next URL, or previous if last was removed
              if (
                removedIndex !== undefined &&
                removedIndex >= 0 &&
                filtered &&
                filtered.length > 0
              ) {
                if (removedIndex < filtered.length) {
                  newFocusedURL = filtered[removedIndex].url;
                } else if (removedIndex - 1 >= 0) {
                  newFocusedURL = filtered[removedIndex - 1].url;
                }
              }
            } else {
              const sourceCustomizations = sourceNode?.data.customizations as
                | NodeCustomizations
                | undefined;
              const targetCustomizations = targetNode?.data.customizations as
                | NodeCustomizations
                | undefined;
              const sourceName = (sourceCustomizations?.customName ||
                sourceNode?.data?.officialName ||
                sourceNode?.type ||
                `Node ${source}`) as string;
              const targetName = (targetCustomizations?.customName ||
                targetNode?.data?.officialName ||
                targetNode?.type ||
                `Node ${target}`) as string;
              const edgeDisplayName = `${sourceName} → ${targetName} (${sourceHandleId ?? t("default")} ${t("port")})`;

              // Add the item (initialize array if undefined)
              newSelectedIntermediateData = [
                ...currentData,
                {
                  edgeId: id,
                  url: intermediateDataUrl,
                  displayName: edgeDisplayName,
                  sourceName: sourceName,
                  targetName: targetName,
                },
              ];
            }

            newFocusedURL = newFocusedURL ?? intermediateDataUrl;

            return {
              ...job,
              focusedIntermediateData: newFocusedURL,
              selectedIntermediateData: newSelectedIntermediateData,
            };
          }) ?? [],
      };
      await updateValue(newDebugRunState);
    },
    [
      t,
      selected,
      intermediateDataUrl,
      debugRunState,
      currentProject,
      id,
      updateValue,
      sourceNode,
      targetNode,
      source,
      sourceHandleId,
      target,
    ],
  );

  const handleDoubleClick = useCallback(
    (e?: MouseEvent) => {
      e?.preventDefault();
      handleIntermediateDataSet();
    },
    [handleIntermediateDataSet],
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
    intermediateDataIsSet,
    hasIntermediateData,
    handleDoubleClick,
  };
};
