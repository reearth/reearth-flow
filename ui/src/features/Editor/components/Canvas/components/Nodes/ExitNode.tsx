import { Position } from "@xyflow/react";
import { memo } from "react";

import { GeneralNodeProps } from "./GeneralNode";
import CustomHandle from "./GeneralNode/components/CustomHandle/CustomHandle";

type Props = GeneralNodeProps;

const ExitNode: React.FC<Props> = ({ data }) => {
  return (
    <div className="rounded-l-3xl rounded-r-sm border border-node-exit bg-node-exit/40 px-4 py-6">
      <CustomHandle
        id={data.inputs?.[0]}
        className="left-[10px] z-[1001] h-3/4 rounded-l-full rounded-r-sm px-2"
        type="target"
        position={Position.Left}
      />
    </div>
  );
};

export default memo(ExitNode);
