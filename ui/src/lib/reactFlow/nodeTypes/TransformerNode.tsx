import { memo } from "react";

import GeneralNode, { type GeneralNodeProps } from "./GeneralNode";

const TransformerNode: React.FC<GeneralNodeProps> = (props) => {
  return <GeneralNode {...props} />;
};

export default memo(TransformerNode);
