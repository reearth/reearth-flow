import { useCallback, useMemo } from "react";
import * as Y from "yjs";

import { useEditorContext } from "@flow/features/Editor/editorContext";
import { useIndexedDB } from "@flow/lib/indexedDB";
import type { YNodesMap, YNodeValue } from "@flow/lib/yjs/types";
import { useCurrentProject } from "@flow/stores";
import type { NodeData } from "@flow/types";
import { isDefined } from "@flow/utils";

import { getNodeColors } from "./nodeColors";

export default ({
  data,
  type,
  nodeId,
}: {
  data: NodeData;
  type: string;
  nodeId: string;
}) => {
  const { officialName, inputs: defaultInputs, outputs: defaultOutputs } = data;
  const { currentYWorkflow, undoTrackerActionWrapper } = useEditorContext();

  const [currentProject] = useCurrentProject();
  const { value: debugRunState } = useIndexedDB("debugRun");
  const isNodeStale = useMemo(() => {
    const job = debugRunState?.jobs?.find(
      (j) => j.projectId === currentProject?.id,
    );
    return !!job?.isRunStale && !!job.staleNodeIds?.includes(nodeId);
  }, [debugRunState, currentProject?.id, nodeId]);

  const inputs: string[] = useMemo(() => {
    if (data.params?.conditions) {
      const conditionalInputs = data.params.conditions
        .map((condition: any) => condition.inputPort)
        .filter(isDefined);
      return conditionalInputs.length ? conditionalInputs : defaultInputs;
    }
    return defaultInputs;
  }, [data.params?.conditions, defaultInputs]);

  const outputs: string[] = useMemo(() => {
    if (data.params?.conditions) {
      const availableOutputs: string[] = [];

      if (defaultOutputs) {
        availableOutputs.push(...defaultOutputs);
      }

      const conditionalOutputs = data.params.conditions
        .map((condition: any) => condition.outputPort)
        .filter(isDefined);

      return conditionalOutputs.length
        ? [...availableOutputs, ...conditionalOutputs]
        : availableOutputs;
    }
    return defaultOutputs || [];
  }, [data.params?.conditions, defaultOutputs]);

  const nodeType = data.isDisabled ? "disabled" : type;
  const [borderColor, backgroundColor, selectedColor, selectedBackgroundColor] =
    getNodeColors(nodeType);

  const handleCollapsedToggle = useCallback(
    (collapsed: boolean) => {
      undoTrackerActionWrapper?.(() => {
        const yNodes = currentYWorkflow?.get("nodes") as YNodesMap | undefined;
        const yNode = yNodes?.get(nodeId);
        if (!yNode) return;
        const yData = yNode?.get("data") as Y.Map<YNodeValue>;
        yData?.set("isCollapsed", collapsed);
      });
    },
    [currentYWorkflow, nodeId, undoTrackerActionWrapper],
  );

  return {
    officialName,
    inputs,
    outputs,
    // status: nodeStatus,
    backgroundColor,
    borderColor,
    selectedColor,
    selectedBackgroundColor,
    isNodeStale,
    handleCollapsedToggle,
  };
};
