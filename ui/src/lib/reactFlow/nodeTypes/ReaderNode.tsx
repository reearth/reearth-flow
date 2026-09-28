import { memo } from "react";

import GeneralNode from "./GeneralNode";
import type { GeneralNodeProps } from "./GeneralNode";

const ReaderNode: React.FC<GeneralNodeProps> = (props) => {
  return <GeneralNode {...props} />;
};

export default memo(ReaderNode);
