import { useMemo } from "react";

import { useEditorContext } from "@flow/features/Editor/editorContext";
import { NodeData } from "@flow/types";
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

  const { onNodesChange } = useEditorContext();

  const handleNodeDelete = () => {
    onNodesChange?.([{ id, type: "remove" }]);
  };

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
  };
};
