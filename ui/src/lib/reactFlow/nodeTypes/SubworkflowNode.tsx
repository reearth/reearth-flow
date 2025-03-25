import { memo, useMemo } from "react";

import GeneralNode, { GeneralNodeProps } from "./GeneralNode";

type Props = GeneralNodeProps;

const SubworkflowNode: React.FC<Props> = ({ data, ...props }) => {
  const uiInputs = useMemo(
    () => data.pseudoInputs?.map((pi) => pi.portName) || data.inputs || [],
    [data.pseudoInputs, data.inputs],
  );

  const uiOutputs = useMemo(
    () => data.pseudoOutputs?.map((po) => po.portName) || data.outputs || [],
    [data.pseudoOutputs, data.outputs],
  );
  return (
    <GeneralNode
      className="bg-node-subworkflow/60"
      {...props}
      data={{ ...data, inputs: uiInputs, outputs: uiOutputs }}
    />
  );
};

export default memo(SubworkflowNode);
