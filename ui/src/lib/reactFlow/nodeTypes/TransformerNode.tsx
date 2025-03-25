import { memo } from "react";

import GeneralNode, { type GeneralNodeProps } from "./GeneralNode";

const TransformerNode: React.FC<GeneralNodeProps> = (props) => {
  return <GeneralNode className="bg-node-transformer" {...props} />;
};

export default memo(TransformerNode);
