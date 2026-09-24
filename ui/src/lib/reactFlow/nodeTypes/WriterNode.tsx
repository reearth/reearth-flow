import { memo } from "react";

import GeneralNode from "./GeneralNode";
import type { GeneralNodeProps } from "./GeneralNode";

const WriterNode: React.FC<GeneralNodeProps> = (props) => {
  return <GeneralNode {...props} />;
};

export default memo(WriterNode);
