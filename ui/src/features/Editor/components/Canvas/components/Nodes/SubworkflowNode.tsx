import { memo } from "react";

import GeneralNode, { GeneralNodeProps } from "./GeneralNode";

type Props = GeneralNodeProps;

const SubworkflowNode: React.FC<Props> = (props) => {
  return <GeneralNode className="bg-[#a21caf]/60" {...props} />;
};

export default memo(SubworkflowNode);
