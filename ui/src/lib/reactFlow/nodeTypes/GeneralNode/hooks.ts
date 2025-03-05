import { useMemo } from "react";

import type { NodeData } from "@flow/types";
import { isDefined } from "@flow/utils";

import { getPropsFrom } from "../utils";

import { getNodeColors } from "./nodeColors";
import useNodeStatus from "./useNodeStatus";

export default ({ data, type }: { data: NodeData; type: string }) => {
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
  };
};
