import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { config } from "@flow/config";
import useDoubleClick from "@flow/hooks/useDoubleClick";
import { useIndexedDB } from "@flow/lib/indexedDB";
import {
  AvailableIntermediateData,
  DebugRunState,
  SelectedIntermediateData,
  useCurrentProject,
} from "@flow/stores";
import { NodeData } from "@flow/types";

export default ({
  nodeId,
  nodeData,
  portName,
  readonly,
}: {
  nodeId: string;
  nodeData: NodeData;
  portName: string;
  readonly: boolean;
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
    return `${api}/artifacts/${debugJobState.jobId}/feature-store/${nodeData.workflowPath ? `${nodeData.workflowPath}.` : ""}${nodeId}.${portName}.jsonl.zst`;
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
      readonly ||
      debugJobState?.status !== "completed" ||
      !debugJobState?.jobId ||
      !dataUrl
    ) {
      setHasIntermediateData(false);
      return;
    }
    if (hasIntermediateData) return;
    const controller = new AbortController();
    const { signal } = controller;
    let isCancelled = false;
    (async () => {
      try {
        const response = await fetch(dataUrl, {
          method: "HEAD",
          signal,
        });
        if (!isCancelled && !signal.aborted) {
          setHasIntermediateData(response.ok);
        }
      } catch (_error) {
        if (!isCancelled && !signal.aborted) {
          setHasIntermediateData(false);
        }
      }
    })();
    return () => {
      isCancelled = true;
      controller.abort();
    };
  }, [
    readonly,
    debugJobState?.status,
    debugJobState?.jobId,
    dataUrl,
    hasIntermediateData,
  ]);

  const writtenForJobRef = useRef<string | null>(null);

  useEffect(() => {
    const jobId = debugJobState?.jobId;
    if (!hasIntermediateData || !jobId || !currentProject?.id) return;
    const writeKey = `${nodeId}:${portName}:${jobId}`;
    if (writtenForJobRef.current === writeKey) return;
    writtenForJobRef.current = writeKey;

    updateValue((prev) => ({
      ...prev,
      jobs: (prev.jobs ?? []).map((job) => {
        if (job.projectId !== currentProject.id) return job;
        const existing: AvailableIntermediateData[] =
          job.availableIntermediateData ?? [];
        if (
          existing.some((e) => e.nodeId === nodeId && e.portName === portName)
        )
          return job;
        return {
          ...job,
          availableIntermediateData: [...existing, { nodeId, portName }],
        };
      }),
    }));
  }, [
    hasIntermediateData,
    debugJobState?.jobId,
    currentProject?.id,
    nodeId,
    portName,
    updateValue,
  ]);

  const selectIntermediateData = useCallback(async () => {
    if (!dataUrl) return;

    const newDebugRunState: DebugRunState = {
      ...debugRunState,
      jobs:
        debugRunState?.jobs?.map((job) => {
          if (job.projectId !== currentProject?.id) return job;

          const currentData = job.selectedIntermediateData ?? [];
          const isCurrentlySelected = currentData.some(
            (sid) => sid.nodeId === nodeId && sid.portName === portName,
          );
          let newSelectedIntermediateData: SelectedIntermediateData[];
          if (isCurrentlySelected) {
            newSelectedIntermediateData = currentData;
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
            focusedIntermediateData: dataUrl,
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

  const removeIntermediateData = useCallback(async () => {
    if (!dataUrl) return;

    const newDebugRunState: DebugRunState = {
      ...debugRunState,
      jobs:
        debugRunState?.jobs?.map((job) => {
          if (job.projectId !== currentProject?.id) return job;

          const currentData = job.selectedIntermediateData ?? [];
          const isCurrentlySelected = currentData.some(
            (sid) => sid.nodeId === nodeId && sid.portName === portName,
          );

          if (!isCurrentlySelected) return job;

          const filtered = currentData.filter(
            (sid) => !(sid.nodeId === nodeId && sid.portName === portName),
          );

          const removedIndex = currentData.findIndex(
            (sid) => sid.nodeId === nodeId && sid.portName === portName,
          );
          let newFocusedURL: string | undefined;
          if (filtered.length > 0) {
            newFocusedURL =
              removedIndex < filtered.length
                ? filtered[removedIndex].url
                : filtered[removedIndex - 1]?.url;
          }

          return {
            ...job,
            focusedIntermediateData: newFocusedURL,
            selectedIntermediateData: filtered,
          };
        }) ?? [],
    };
    await updateValue(newDebugRunState);
  }, [dataUrl, debugRunState, currentProject, nodeId, portName, updateValue]);

  const [handleSingleClick, handleDoubleClick] = useDoubleClick(
    selectIntermediateData,
    removeIntermediateData,
  );

  return {
    hasIntermediateData,
    handleDoubleClick,
    isSelected,
    jobStatus,
    handleSingleClick,
  };
};
