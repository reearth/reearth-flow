import { useCallback, useEffect, useMemo, useState } from "react";

import { config } from "@flow/config";
import { useIndexedDB } from "@flow/lib/indexedDB";
import {
  DebugRunState,
  SelectedIntermediateData,
  useCurrentProject,
} from "@flow/stores";
import { NodeData } from "@flow/types";

export default ({
  nodeId,
  nodeData,
  portName,
  nodeType,
}: {
  nodeId: string;
  nodeData: NodeData;
  portName: string;
  nodeType?: string;
}) => {
  const [currentProject] = useCurrentProject();
  const { api } = config();
  const { value: debugRunState, updateValue } = useIndexedDB("debugRun");

  const debugJobState = useMemo(
    () =>
      debugRunState?.jobs?.find((job) => job.projectId === currentProject?.id),
    [debugRunState, currentProject],
  );
  const jobStatus = useMemo(
    () => debugJobState?.status,
    [debugJobState?.status],
  );

  const dataUrl = useMemo(() => {
    if (!api || !debugJobState?.jobId) return undefined;
    return `${api}/artifacts/${debugJobState.jobId}/feature-store/${nodeData.workflowPath ? `${nodeData.workflowPath}.` : ""}${nodeId}/${portName}.jsonl.zst`;
  }, [api, nodeData.workflowPath, debugJobState?.jobId, nodeId, portName]);

  const [hasIntermediateData, setHasIntermediateData] = useState(false);

  const isSelected = useMemo(
    () =>
      !!debugJobState?.selectedIntermediateData?.find(
        (sid) => sid.nodeId === nodeId && sid.portName === portName,
      ),
    [debugJobState?.selectedIntermediateData, nodeId, portName],
  );

  useEffect(() => {
    if (
      debugJobState?.status !== "completed" ||
      !debugJobState?.jobId ||
      !dataUrl
    ) {
      setHasIntermediateData(false);
      return;
    }
    if (hasIntermediateData) return;

    (async () => {
      const response = await fetch(dataUrl, { method: "HEAD" });
      setHasIntermediateData(response.ok);
    })();
  }, [
    debugJobState?.status,
    debugJobState?.jobId,
    dataUrl,
    hasIntermediateData,
  ]);

  const handleClick = useCallback(async () => {
    if (!dataUrl) return;

    const newDebugRunState: DebugRunState = {
      ...debugRunState,
      jobs:
        debugRunState?.jobs?.map((job) => {
          if (job.projectId !== currentProject?.id) return job;

          const currentData = job.selectedIntermediateData ?? [];
          const isCurrentlySelected = currentData.find(
            (sid) => sid.nodeId === nodeId && sid.portName === portName,
          );

          let newSelectedIntermediateData:
            | SelectedIntermediateData[]
            | undefined;
          let newFocusedURL: string | undefined;

          if (isCurrentlySelected) {
            const filtered = currentData.filter(
              (sid) => !(sid.nodeId === nodeId && sid.portName === portName),
            );
            newSelectedIntermediateData = filtered;

            const removedIndex = currentData.findIndex(
              (sid) => sid.nodeId === nodeId && sid.portName === portName,
            );
            if (removedIndex >= 0 && filtered.length > 0) {
              newFocusedURL =
                removedIndex < filtered.length
                  ? filtered[removedIndex].url
                  : filtered[removedIndex - 1]?.url;
            }
          } else {
            const nodeName =
              nodeData.customizations?.customName ||
              nodeData.officialName ||
              nodeId;
            newSelectedIntermediateData = [
              ...currentData,
              {
                nodeId,
                url: dataUrl,
                portName,
                displayName: `${nodeName} (${portName})`,
              },
            ];
          }

          return {
            ...job,
            focusedIntermediateData: newFocusedURL ?? dataUrl,
            selectedIntermediateData: newSelectedIntermediateData,
          };
        }) ?? [],
    };
    await updateValue(newDebugRunState);
  }, [
    dataUrl,
    debugRunState,
    currentProject,
    nodeId,
    portName,
    nodeData,
    updateValue,
  ]);

  // Auto-select for writer nodes on first job completion
  useEffect(() => {
    const hasNeverBeenTouched =
      debugJobState?.selectedIntermediateData === undefined;

    if (
      nodeType === "writer" &&
      hasIntermediateData &&
      !isSelected &&
      hasNeverBeenTouched &&
      debugJobState?.status === "completed"
    ) {
      handleClick();
    }
  }, [
    nodeType,
    hasIntermediateData,
    isSelected,
    debugJobState?.selectedIntermediateData,
    debugJobState?.status,
    handleClick,
  ]);

  return { hasIntermediateData, isSelected, jobStatus, handleClick };
};
