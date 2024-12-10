import { memo, useMemo } from "react";

import GeneralNode, { GeneralNodeProps } from "./GeneralNode";

type Props = GeneralNodeProps;

const SubworkflowNode: React.FC<Props> = (props) => {
  console.log("SubworkflowNode", props);
  const { data } = props;
  const uiInputs = useMemo(
    () => data.pseudoInputs || data.inputs || [],
    [data.pseudoInputs, data.inputs],
  );

  const uiOutputs = useMemo(
    () => data.pseudoOutputs || data.outputs || [],
    [data.pseudoOutputs, data.outputs],
  );
  return (
    <GeneralNode
      className="bg-node-entrance/60"
      {...props}
      data={{ ...data, inputs: uiInputs, outputs: uiOutputs }}
    />
  );
};

export default memo(SubworkflowNode);
