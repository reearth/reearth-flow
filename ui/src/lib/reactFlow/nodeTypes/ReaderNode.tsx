import { memo } from "react";

import GeneralNode, { type GeneralNodeProps } from "./GeneralNode";

const ReaderNode: React.FC<GeneralNodeProps> = (props) => {
  return <GeneralNode className="bg-node-reader/60" {...props} />;
};

export default memo(ReaderNode);
