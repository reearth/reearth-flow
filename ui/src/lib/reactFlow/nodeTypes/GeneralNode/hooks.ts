import { useReactFlow } from "@xyflow/react";
import { useCallback, useMemo } from "react";

import { useEditorContext } from "@flow/features/Editor/editorContext";
import { Node, NodeData } from "@flow/types";
import { isDefined } from "@flow/utils";

import { getPropsFrom } from "../utils";

import { getNodeColors } from "./nodeColors";
import useNodeStatus from "./useNodeStatus";

export default ({
  id,
  data,
  type,
}: {
  id: string;
  data: NodeData;
  type: string;
}) => {
  const { getNode } = useReactFlow<Node>();
  const {
    officialName,
    customName,
    inputs: defaultInputs,
    outputs: defaultOutputs,
  } = data;

  const { nodeExecution } = useNodeStatus();

  const { status } = useMemo(() => nodeExecution, [nodeExecution]) ?? {};

  const inputs: string[] = useMemo(() => {
    if (data.params?.conditions) {
      const i = data.params.conditions
        .map((condition: any) => condition.inputPort)
        .filter(isDefined);
      return i.length ? i : defaultInputs;
    }
    return defaultInputs;
  }, [data.params?.conditions, defaultInputs]);

  const outputs: string[] = useMemo(() => {
    if (data.params?.conditions) {
      const i = data.params.conditions
        .map((condition: any) => condition.outputPort)
        .filter(isDefined);
      return i.length ? i : defaultOutputs;
    }
    return defaultOutputs;
  }, [data.params?.conditions, defaultOutputs]);

  const metaProps = getPropsFrom(status);

  const [borderColor, selectedColor, selectedBackgroundColor] =
    getNodeColors(type);

  const { onNodesChange, onParamsEditorOpen } = useEditorContext();

  const handleNodeDelete = useCallback(() => {
    onNodesChange?.([{ id, type: "remove" }]);
  }, [id, onNodesChange]);

  const handleParamsEditorOpen = useCallback(() => {
    const node = getNode(id);
    if (!node) return;
    onParamsEditorOpen?.(undefined, node);
  }, [id, onParamsEditorOpen, getNode]);

  return {
    officialName,
    customName,
    inputs,
    outputs,
    status,
    metaProps,
    borderColor,
    selectedColor,
    selectedBackgroundColor,
    handleNodeDelete,
    handleParamsEditorOpen,
  };
};
