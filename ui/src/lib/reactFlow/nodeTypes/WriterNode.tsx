import { memo } from "react";

import GeneralNode, { type GeneralNodeProps } from "./GeneralNode";

const WriterNode: React.FC<GeneralNodeProps> = (props) => {
  return <GeneralNode {...props} />;
};

export default memo(WriterNode);
