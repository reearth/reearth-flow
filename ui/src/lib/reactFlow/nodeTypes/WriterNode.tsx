import { memo } from "react";

import GeneralNode, { type GeneralNodeProps } from "./GeneralNode";

const WriterNode: React.FC<GeneralNodeProps> = (props) => {
  return <GeneralNode className="bg-node-writer/60" {...props} />;
};

export default memo(WriterNode);
