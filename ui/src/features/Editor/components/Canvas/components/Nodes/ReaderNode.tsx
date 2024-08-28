import { memo } from "react";

import GeneralNode, { type GeneralNodeProps } from "./GeneralNode";

const ReaderNode: React.FC<GeneralNodeProps> = (props) => {
  // const onChange = useCallback(
  //   (evt: any) => {
  //     console.log("EVT", evt.target.value);
  //     console.log("data", data);
  //   },
  //   [data],
  // );
  return <GeneralNode className="bg-node-reader/60" {...props} />;
};

export default memo(ReaderNode);
