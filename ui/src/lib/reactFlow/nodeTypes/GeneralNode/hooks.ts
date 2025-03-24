import { useMemo } from "react";

import type { NodeData } from "@flow/types";
import { isDefined } from "@flow/utils";

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

  const { nodeStatus } = useNodeStatus({ id });

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

  const [borderColor, selectedColor, selectedBackgroundColor] = getNodeColors(
    type,
    nodeStatus,
  );

  return {
    officialName,
    customName,
    inputs,
    outputs,
    status: nodeStatus,
    borderColor,
    selectedColor,
    selectedBackgroundColor,
  };
};
