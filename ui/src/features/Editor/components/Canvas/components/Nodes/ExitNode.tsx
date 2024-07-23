import { Position } from "@xyflow/react";
import { memo } from "react";

import { GeneralNodeProps } from "./GeneralNode";
import CustomHandle from "./GeneralNode/components/CustomHandle/CustomHandle";

type Props = GeneralNodeProps;

const ExitNode: React.FC<Props> = ({ data }) => {
  return (
    <div className="rounded-l-3xl rounded-r-sm border border-[#7e22ce] bg-[#7e22ce]/40 px-3 py-4">
      <CustomHandle
        id={data.inputs?.[0]}
        className="left-3 z-[1001] rounded-l-3xl rounded-r-sm px-3 py-4"
        type="target"
        position={Position.Left}
      />
    </div>
  );
};

export default memo(ExitNode);
